#!/usr/bin/env python3
"""DEM -> 2D SWE solver -> thematic flood map.

What changed and why
--------------------
The previous version of this bridge was named for DEMNAS and advertised as
"menjalankan solver fisika 2D SWE di Rust", but it never invoked the solver: it
downloaded a DEM, coarsened it, and rendered an elevation map. Everything after
step 3 was cartography. It also:

  * pinned the computational grid at 100x100 regardless of AOI size, so an 8 m
    DEM over a 5 km buffer was resampled to 100 m cells and the terrain detail
    that actually routes flow was discarded;
  * resampled with bilinear interpolation, which smooths embankments and channel
    banks - the features that most control inundation extent;
  * replaced nodata with elevation 0 m, creating artificial sea-level sinks that
    a shallow-water solver will preferentially fill;
  * placed the inflow at a fixed geometric position (mid-left edge) unrelated to
    where water actually enters;
  * asked for 5 snapshots via `output_interval_s` and then discarded them.

This version runs the solver, sizes the grid from the DEM and AOI, masks nodata,
derives the inflow location from the terrain, and exports the timestep history.

DEM accuracy in Indonesia (Susetyo 2023, DOI 10.3846/gac.2023.18168, vs GPS):
    SRTM-1      RMSE 5.529 m
    DEMNAS      RMSE 8.172 m
    ASTER GDEM  RMSE 13.632 m
DEMNAS wins on horizontal resolution (8 m vs 30 m), not vertical accuracy, and
it is a DSM: buildings and canopy are included. For flood modelling a bare-earth
DEM is preferred (Hawker et al. 2022, DOI 10.1088/1748-9326/ac4d4f: MAE 1.61 ->
1.12 m built-up, 5.15 -> 2.88 m forest; Hawker et al. 2024,
DOI 10.5194/nhess-24-539-2024, validated in tropical SE Asia).

Depth bias: without measured river cross-sections, depths are expected to be
biased low. Sahid 2024 (DOI 10.23917/forgeo.v38i1.1839, Ciberes/Cirebon) reports
+11.67% depth accuracy from DEM filtering alone versus +24.98% with cross-
sections fused in. van Rutten et al. 2025 (DOI 10.3390/rs17132171) found the
same underestimation direction for Sentinel-1 + FABDEM in Vietnam.
"""
import sys, os, json, math, subprocess, tempfile, warnings
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, SCRIPT_DIR)
sys.path.insert(0, os.path.join(SCRIPT_DIR, '..', 'datasources'))

try:
    import rasterio
    from rasterio.transform import from_bounds
except ImportError:
    print("ERROR: rasterio tidak tersedia")
    sys.exit(1)

from cartography import generate_sni_map
import telegram_delivery

# Published vertical RMSE for DEMs commonly used over Indonesia.
DEM_VERTICAL_RMSE_M = {
    'SRTM-1 (30 m)': 5.529,
    'DEMNAS (8 m, DSM)': 8.172,
    'ASTER GDEM (30 m)': 13.632,
}
DEM_RMSE_SOURCE = "Susetyo 2023, Geodesy and Cartography 49(4):209-215, DOI 10.3846/gac.2023.18168"

# Upper bound on solver cells per axis. Above this the explicit CFL-limited
# solver becomes impractically slow for interactive use.
MAX_GRID_CELLS_PER_AXIS = 400
# Below this the discretisation is too coarse to say anything about routing.
MIN_GRID_CELLS_PER_AXIS = 20


def dem_accuracy_note(dem_label):
    """Text block stating the vertical accuracy bound for the DEM in use."""
    lines = ["Batas ketelitian vertikal DEM (Indonesia, terhadap GPS):"]
    for name, rmse in sorted(DEM_VERTICAL_RMSE_M.items(), key=lambda kv: kv[1]):
        mark = "  <-- dipakai" if name.split()[0].lower() in dem_label.lower() else ""
        lines.append(f"  {name:<22} RMSE {rmse:.3f} m{mark}")
    lines.append(f"  Sumber: {DEM_RMSE_SOURCE}")
    lines.append("")
    lines.append("  DEMNAS unggul pada resolusi horizontal (8 m), bukan akurasi vertikal,")
    lines.append("  dan berstatus DSM (bangunan + tajuk ikut terhitung). Untuk pemodelan")
    lines.append("  banjir, DEM bare-earth lebih tepat: Hawker et al. 2022,")
    lines.append("  DOI 10.1088/1748-9326/ac4d4f.")
    return "\n".join(lines)


def choose_grid(dem_shape, aoi_span_m, native_res_m):
    """Pick a solver grid that respects the DEM's own resolution.

    Returns (nx, ny, dx_m, note). The grid is not fixed: it follows the AOI and
    the DEM resolution, capped for runtime. Upsampling beyond the native
    resolution is refused - it would invent detail the DEM does not carry.
    """
    rows, cols = dem_shape
    # Cells the native resolution supports across the AOI.
    native_cells = max(1, int(round(aoi_span_m / max(native_res_m, 1e-6))))
    target = min(native_cells, MAX_GRID_CELLS_PER_AXIS, max(rows, cols))
    target = max(target, MIN_GRID_CELLS_PER_AXIS)
    dx = aoi_span_m / target
    if dx < native_res_m:
        # Never finer than the source data.
        target = max(MIN_GRID_CELLS_PER_AXIS, int(aoi_span_m // native_res_m))
        dx = aoi_span_m / target
    note = (
        f"Grid {target}x{target}, dx={dx:.1f} m. Resolusi DEM asli {native_res_m:.1f} m, "
        f"AOI {aoi_span_m/1000:.1f} km. "
    )
    if dx > native_res_m * 1.5:
        note += (
            f"Diperkasar {dx/native_res_m:.1f}x dari resolusi asli demi waktu komputasi; "
            f"detail sub-{dx:.0f} m tidak terwakili."
        )
    else:
        note += "Mendekati resolusi asli DEM."
    return target, target, dx, note


def aggregate_dem(dem_data, nodata, target_ny, target_nx):
    """Block-aggregate a DEM to the solver grid, keeping nodata as a mask.

    Uses block means over valid cells only. Nodata stays nodata (NaN) rather than
    becoming 0 m: substituting zero manufactures sea-level sinks that a
    shallow-water solver will fill preferentially, which is worse than an
    explicit hole.
    """
    arr = dem_data.astype('float64')
    invalid = ~np.isfinite(arr)
    if nodata is not None:
        invalid |= (arr == nodata)
    # DEMNAS/SRTM sentinel values for ocean and voids.
    invalid |= (arr <= -9000)
    arr = np.where(invalid, np.nan, arr)

    rows, cols = arr.shape
    # Trim to a whole multiple so reshape-based block aggregation is exact.
    by = max(1, rows // target_ny)
    bx = max(1, cols // target_nx)
    use_rows = (rows // by) * by
    use_cols = (cols // bx) * bx
    trimmed = arr[:use_rows, :use_cols]
    blocks = trimmed.reshape(use_rows // by, by, use_cols // bx, bx)

    with warnings.catch_warnings():
        # All-nodata blocks are expected and become NaN, which is the intent.
        warnings.simplefilter("ignore", category=RuntimeWarning)
        coarse = np.nanmean(blocks, axis=(1, 3))

    valid_frac = float(np.isfinite(coarse).sum()) / coarse.size if coarse.size else 0.0
    return coarse, valid_frac


def fill_nodata_for_solver(coarse):
    """Give the solver a finite DEM while reporting what was filled.

    Holes are filled with the maximum valid elevation, not zero: a high value
    behaves as a wall (water does not accumulate there), whereas zero behaves as
    a sink that attracts flow. Returns (dem_for_solver, mask_filled, n_filled).
    """
    mask_filled = ~np.isfinite(coarse)
    n_filled = int(mask_filled.sum())
    if n_filled == 0:
        return coarse.copy(), mask_filled, 0
    if np.isfinite(coarse).sum() == 0:
        raise ValueError("DEM tidak punya satu pun sel valid setelah masking nodata")
    wall = float(np.nanmax(coarse))
    filled = np.where(mask_filled, wall, coarse)
    return filled, mask_filled, n_filled


def pick_inflow_cell(dem_for_solver, mask_filled):
    """Place the inflow at the lowest valid interior cell.

    The previous version hardcoded (x=5, y=ny//2) - mid-left edge - with no
    relation to where water enters the domain. Absent a gauged inflow point,
    the lowest interior cell is at least hydrologically defensible: it is where
    the terrain itself directs water. This is a screening assumption, and it is
    reported as such.
    """
    ny, nx = dem_for_solver.shape
    interior = dem_for_solver[1:ny - 1, 1:nx - 1].copy()
    interior_mask = mask_filled[1:ny - 1, 1:nx - 1]
    interior = np.where(interior_mask, np.inf, interior)
    if not np.isfinite(interior).any():
        return 1, 1, "Semua sel interior nodata; inflow ditempatkan di (1,1)."
    flat = int(np.argmin(interior))
    j, i = np.unravel_index(flat, interior.shape)
    y, x = int(j) + 1, int(i) + 1
    note = (
        f"Inflow di sel terendah interior (x={x}, y={y}, elev="
        f"{dem_for_solver[y, x]:.1f} m). Asumsi screening: tanpa titik inflow "
        f"terukur, terrain dipakai sebagai penentu."
    )
    return x, y, note


def run_swe(dem_matrix, dx, duration_s, discharge_m3s, inflow_x, inflow_y,
            manning_n, snapshots, binary):
    """Invoke the Rust solver via `--test-tool integrated_environment_study`."""
    request = {
        "aoi_geojson": json.dumps({"type": "Point", "coordinates": [0.0, 0.0]}),
        "domains": ["flood"],
        "satellite_fallback": False,
        "flood": {
            "dem": dem_matrix,
            "dx_m": dx,
            "manning_n": manning_n,
            "duration_s": duration_s,
            "dt_max_s": max(0.1, min(2.0, dx / 20.0)),
            "second_order": True,
            "inflow_discharge_m3s": discharge_m3s,
            "inflow_x": inflow_x,
            "inflow_y": inflow_y,
            "inflow_width": 1,
            "history_interval_s": duration_s / snapshots if snapshots > 0 else None,
            "synthetic": False,
        },
    }
    proc = subprocess.run(
        [binary, "--test-tool", "integrated_environment_study", json.dumps(request)],
        capture_output=True, text=True, timeout=1800,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"solver exit {proc.returncode}: {proc.stderr.strip()[:500]}"
        )
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as e:
        raise RuntimeError(f"solver output bukan JSON: {e}. stdout: {proc.stdout[:400]}")


def find_binary():
    for build in ("release", "debug"):
        cand = os.path.abspath(
            os.path.join(SCRIPT_DIR, '..', '..', '..', 'target', build, 'env-indonesia-mcp')
        )
        if os.path.exists(cand):
            return cand
    return None


def run_swe_with_dem(lat, lon, buffer_km, discharge_m3s, duration_hours, output_path,
                     manning_n=0.035, snapshots=10):
    """DEM -> 2D SWE -> depth map. Returns a dict describing the run."""
    print("=== SIMULASI BANJIR 2D SWE ===")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Debit: {discharge_m3s} m3/s | Durasi: {duration_hours} jam")

    # --- 1. DEM ---
    print("\n1. Mengambil DEM...")
    dem_tif, dem_label, native_res_m = None, None, None
    try:
        import demnas_engine
        cand = os.path.join(tempfile.gettempdir(), f"demnas_swe_{lat}_{lon}.tif")
        demnas_engine.download_demnas(lat, lon, buffer_km, cand)
        if os.path.exists(cand):
            dem_tif, dem_label, native_res_m = cand, "DEMNAS 8 m (BIG, DSM)", 8.0
            print(f"   DEMNAS: {cand}")
    except Exception as e:
        print(f"   DEMNAS tidak tersedia: {e}")

    if dem_tif is None:
        dem_tif = _download_srtm(lat, lon, buffer_km)
        if dem_tif:
            dem_label, native_res_m = "SRTM-1 30 m (GEE)", 30.0
            print(f"   SRTM-1: {dem_tif}")

    if not dem_tif or not os.path.exists(dem_tif):
        print("ERROR: Tidak ada DEM tersedia.")
        return None

    print()
    print(dem_accuracy_note(dem_label))

    # --- 2. Grid + aggregation ---
    print("\n2. Menyusun grid komputasi...")
    aoi_span_m = buffer_km * 2 * 1000.0
    with rasterio.open(dem_tif) as src:
        dem_data = src.read(1)
        nodata = src.nodata

    nx, ny, dx, grid_note = choose_grid(dem_data.shape, aoi_span_m, native_res_m)
    print(f"   {grid_note}")

    coarse, valid_frac = aggregate_dem(dem_data, nodata, ny, nx)
    print(f"   Sel valid setelah masking nodata: {valid_frac*100:.1f}%")
    if valid_frac < 0.5:
        print("   PERINGATAN: lebih dari separuh domain nodata. Hasil tidak dapat dipakai.")
        return None

    dem_for_solver, mask_filled, n_filled = fill_nodata_for_solver(coarse)
    if n_filled:
        print(f"   {n_filled} sel nodata diisi elevasi maksimum (berperan sebagai dinding,")
        print(f"   BUKAN 0 m yang akan menarik air seperti cekungan).")

    ny_eff, nx_eff = dem_for_solver.shape
    print(f"   Elevasi valid: min={np.nanmin(coarse):.1f} m, max={np.nanmax(coarse):.1f} m")

    inflow_x, inflow_y, inflow_note = pick_inflow_cell(dem_for_solver, mask_filled)
    print(f"   {inflow_note}")

    # --- 3. Solver ---
    binary = find_binary()
    if binary is None:
        print("\nERROR: binary solver tidak ditemukan. Jalankan `cargo build --release`.")
        return None

    print(f"\n3. Menjalankan solver 2D SWE (HLLC + MUSCL)...")
    print(f"   Binary: {binary}")
    duration_s = duration_hours * 3600.0
    try:
        report = run_swe(
            dem_for_solver.tolist(), dx, duration_s, discharge_m3s,
            inflow_x, inflow_y, manning_n, snapshots, binary,
        )
    except Exception as e:
        print(f"ERROR: solver gagal: {e}")
        return None

    flood = next(
        (d for d in report.get('domain_results', []) if d.get('domain') == 'urban_flood'),
        None,
    )
    if flood is None or flood.get('status') == 'insufficient_data':
        print(f"ERROR: solver menolak input: {flood}")
        return None

    s = flood.get('summary', {})
    depth_flat = s.get('depth_grid_m') or []
    history = s.get('timestep_history') or []
    print(f"   Max depth: {s.get('max_depth_m', float('nan')):.3f} m")
    print(f"   Volume: {s.get('total_volume_m3', float('nan')):.1f} m3")
    print(f"   Sel tergenang: {s.get('flooded_cells')} / {s.get('total_cells')}")
    print(f"   Snapshot history: {len(history)}")

    if not depth_flat:
        print("ERROR: solver tidak mengembalikan depth grid.")
        return None

    # Solver flattens as x * ny + y (see swe_solver.rs), so reshape (nx, ny)
    # then transpose to get (row=y, col=x) for rasterio.
    depth = np.asarray(depth_flat, dtype='float64').reshape(nx_eff, ny_eff).T
    # Do not claim water where the DEM had no data.
    depth = np.where(mask_filled, np.nan, depth)

    # --- 4. Map ---
    print("\n4. Merender peta kedalaman banjir...")
    from pyproj import Transformer
    transformer = Transformer.from_crs("EPSG:4326", "EPSG:3857", always_xy=True)
    d = buffer_km / 111.0
    dlon = d / math.cos(math.radians(lat))
    x_min, y_min = transformer.transform(lon - dlon, lat - d)
    x_max, y_max = transformer.transform(lon + dlon, lat + d)

    depth_tif = os.path.join(tempfile.gettempdir(), f"swe_depth_{lat}_{lon}.tif")
    transform = from_bounds(x_min, y_min, x_max, y_max, nx_eff, ny_eff)
    with rasterio.open(depth_tif, 'w', driver='GTiff', height=ny_eff, width=nx_eff,
                       count=1, dtype='float32', crs='EPSG:3857',
                       transform=transform, nodata=np.nan) as dst:
        dst.write(depth.astype('float32'), 1)

    geojson_data = {
        "type": "FeatureCollection",
        "features": [{
            "type": "Feature", "properties": {"name": "Area Simulasi"},
            "geometry": {"type": "Polygon", "coordinates": [[
                [lon - dlon, lat - d], [lon + dlon, lat - d],
                [lon + dlon, lat + d], [lon - dlon, lat + d], [lon - dlon, lat - d],
            ]]},
        }],
    }

    stats = {
        'Sumber DEM': dem_label,
        'RMSE vertikal DEM': f"{DEM_VERTICAL_RMSE_M.get(dem_label, float('nan')):.2f} m",
        'Grid Komputasi': f'{nx_eff}x{ny_eff}',
        'Resolusi Sel': f'{dx:.0f} m',
        'Sel nodata diisi': str(n_filled),
        'Max Kedalaman': f"{s.get('max_depth_m', 0):.2f} m",
        'Volume Total': f"{s.get('total_volume_m3', 0):.0f} m3",
        'Snapshot temporal': str(len(history)),
        'Status': 'screening_only',
    }

    kesimpulan = (
        f"- Solver: 2D SWE HLLC + MUSCL (dijalankan, bukan estimasi)\n"
        f"- Max kedalaman: {s.get('max_depth_m', 0):.2f} m\n"
        f"- Volume: {s.get('total_volume_m3', 0):.0f} m3\n"
        f"- Snapshot temporal: {len(history)}\n"
        f"- KEDALAMAN BIAS RENDAH: tanpa penampang sungai terukur.\n"
        f"  Sahid 2024: +11.67% (filter DEM) vs +24.98% (dengan cross-section).\n"
        f"- Status: screening_only, belum divalidasi terhadap genangan teramati."
    )

    result_msg = generate_sni_map(
        json.dumps(geojson_data), output_path,
        title="PETA KEDALAMAN BANJIR — SOLVER 2D SWE",
        realtime=False, author="Rizki Agustiawan x ZeroClaw AI",
        overlay_raster=depth_tif, analysis_type='continuous', cmap='Blues',
        vmin=0.0, vmax=max(0.1, float(s.get('max_depth_m', 0.1))),
        analysis_stats=stats, colorbar_label="Kedalaman Banjir (m)",
        conclusion_text=kesimpulan,
    )
    print(f"\n5. {result_msg}")

    history_path = None
    if history:
        history_path = os.path.splitext(output_path)[0] + "_history.json"
        with open(history_path, 'w') as f:
            json.dump({
                "grid": {"nx": nx_eff, "ny": ny_eff, "dx_m": dx,
                         "flatten_order": "x * ny + y"},
                "dem_source": dem_label,
                "inflow": {"x": inflow_x, "y": inflow_y, "note": inflow_note},
                "snapshots": history,
                "note": "Riwayat timestep asli dari solver, bukan state akhir yang diskalakan.",
            }, f)
        print(f"   History timestep: {history_path}")

    if os.path.exists(depth_tif):
        os.remove(depth_tif)

    return {
        "output_path": output_path,
        "history_path": history_path,
        "max_depth_m": s.get('max_depth_m'),
        "total_volume_m3": s.get('total_volume_m3'),
        "flooded_cells": s.get('flooded_cells'),
        "grid": {"nx": nx_eff, "ny": ny_eff, "dx_m": dx},
        "dem_source": dem_label,
        "snapshots": len(history),
        "limitations": flood.get('limitations', []),
    }


def _download_srtm(lat, lon, buffer_km):
    """SRTM-1 30 m via GEE. Best vertical accuracy of the open DEMs in Indonesia."""
    try:
        import ee, requests
        ee.Initialize()
        roi = ee.Geometry.Point([lon, lat]).buffer(buffer_km * 1000).bounds()
        srtm = ee.Image('USGS/SRTMGL1_003').clip(roi)
        tif_path = os.path.join(tempfile.gettempdir(), f"srtm_{lat}_{lon}.tif")
        url = srtm.getDownloadURL({
            'scale': 30, 'crs': 'EPSG:4326', 'region': roi, 'format': 'GEO_TIFF'
        })
        r = requests.get(url, timeout=120)
        with open(tif_path, 'wb') as f:
            f.write(r.content)
        return tif_path
    except Exception as e:
        print(f"   SRTM gagal: {e}")
        return None


# Backwards-compatible alias.
run_swe_with_demnas = run_swe_with_dem


if __name__ == '__main__':
    if len(sys.argv) < 6:
        print("Usage: swe_demnas_bridge.py lat lon buffer_km discharge_m3s duration_hours [output.png]")
        sys.exit(1)

    lat = float(sys.argv[1])
    lon = float(sys.argv[2])
    buffer_km = float(sys.argv[3])
    discharge = float(sys.argv[4])
    duration = float(sys.argv[5])
    output = sys.argv[6] if len(sys.argv) > 6 else "/tmp/opencode/swe_flood_map.png"

    res = run_swe_with_dem(lat, lon, buffer_km, discharge, duration, output)

    if res and os.path.exists(res["output_path"]):
        print(f"\nSUCCESS: {res['output_path']}")
        msg = (
            f"PETA KEDALAMAN BANJIR (2D SWE)\n"
            f"Lokasi: {lat}, {lon} | Buffer: {buffer_km} km\n"
            f"DEM: {res['dem_source']}\n"
            f"Max kedalaman: {res['max_depth_m']:.2f} m | "
            f"Volume: {res['total_volume_m3']:.0f} m3\n"
            f"Grid: {res['grid']['nx']}x{res['grid']['ny']} @ {res['grid']['dx_m']:.0f} m\n"
            f"Snapshot: {res['snapshots']}\n"
            f"Status: screening_only, kedalaman bias rendah tanpa cross-section sungai"
        )
        telegram_delivery.send_to_telegram(res["output_path"], msg)
    else:
        print("FAILED: Tidak ada output dihasilkan.")
