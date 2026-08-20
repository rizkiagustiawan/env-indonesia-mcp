"""Flood screening orchestrator — target bebas untuk seluruh Indonesia.

Merangkai: nama lokasi -> resolusi geometri (GADM) -> DEM (DEMNAS/SRTM) ->
solver 2D SWE -> peta kedalaman + JSON hasil.

Keputusan jujur:
- Tanpa debit (`--discharge-m3s`), hasil = `insufficient_data` (tidak mengarang
  debit). Debit eksplisit adalah satu-satunya jalur untuk menjalankan solver.
- Bahkan dengan debit, hasil tetap `screening_only`: belum divalidasi terhadap
  genangan teramati, dan tanpa penampang sungai terukur kedalamannya bias.
- Titik inflow dipilih dari sel terendah interior (bukan posisi hidrologis),
  karena jaringan sungai Indonesia belum tersedia — lihat `swe_demnas_bridge`.
"""

import argparse
import json
import os
import sys

_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.abspath(os.path.join(_SCRIPT_DIR, "..", "..", "..")))

from src.tools.gis import location_resolver as resolver  # noqa: E402


def plan_screening(location, discharge_m3s, buffer_km=None, duration_hours=6,
                   manning_n=0.035, snapshots=10):
    """Rencana screening tanpa menjalankan solver (murni, bisa diuji)."""
    r = resolver.resolve_location(location)
    lon = r["representative_point"]["lon"]
    lat = r["representative_point"]["lat"]
    buf = buffer_km if buffer_km is not None else r["buffer_km_suggested"]

    if discharge_m3s is None:
        return {
            "status": "insufficient_data",
            "synthetic": False,
            "resolution": r,
            "missing": ["inflow_discharge_m3s"],
            "limitations": [
                "Debit inflow (m³/s) tidak diberikan. Tanpa debit, tidak ada "
                "forcing yang bisa dijalankan ke solver 2D SWE; tool menolak "
                "mengarang angka. Berikan --discharge-m3s, atau hitung debit "
                "rancangan dari seri debit terukur (flow_duration_curve).",
                "Hujan satelit (GPM IMERG) TIDAK dipakai sebagai pengganti "
                "debit: korelasi jamannya terlalu rendah untuk banjir "
                "real-time (r≈0.10).",
            ],
            "run": None,
        }

    return {
        "status": "screening_only",
        "synthetic": False,
        "resolution": r,
        "missing": [],
        "limitations": [
            "Output screening_only: belum divalidasi terhadap genangan "
            "teramati, dan tanpa penampang sungai terukur kedalamannya bias "
            "(Sahid 2024: +11.67% filter DEM vs +24.98% dengan cross-section).",
            "Inflow ditempatkan di sel terendah interior karena jaringan "
            "drainase sungai Indonesia belum tersedia; ini screening "
            "terrain-based, bukan routing sungai.",
        ],
        "run": {
            "lat": lat,
            "lon": lon,
            "buffer_km": float(buf),
            "discharge_m3s": float(discharge_m3s),
            "duration_hours": float(duration_hours),
            "manning_n": manning_n,
            "snapshots": snapshots,
        },
    }


def _run_swe(run_params, output_path):
    from src.tools.gis import swe_demnas_bridge as bridge
    return bridge.run_swe_with_dem(
        run_params["lat"], run_params["lon"], run_params["buffer_km"],
        run_params["discharge_m3s"], run_params["duration_hours"],
        output_path,
        manning_n=run_params["manning_n"],
        snapshots=run_params["snapshots"],
    )


def main(argv=None):
    ap = argparse.ArgumentParser(
        description="Flood screening untuk target bebas di Indonesia (GADM -> DEM -> 2D SWE)."
    )
    ap.add_argument("--location", required=True,
                    help="Nama kota/kabupaten/provinsi (mis. 'kota semarang'), "
                         "atau 'lat,lon'. Nama ambigu (mis. 'bima') ditolak.")
    ap.add_argument("--discharge-m3s", type=float, default=None,
                    help="Debit inflow m³/s. WAJIB untuk menjalankan solver; "
                         "tanpa ini hasil = insufficient_data.")
    ap.add_argument("--buffer-km", type=float, default=None,
                    help="Radius AOI (km). Default: saran dari ukuran wilayah.")
    ap.add_argument("--duration-hours", type=float, default=6.0)
    ap.add_argument("--manning-n", type=float, default=0.035)
    ap.add_argument("--snapshots", type=int, default=10)
    ap.add_argument("--output", default=None,
                    help="Path PNG output. Default: /tmp/flood_screening_<name>.png")
    args = ap.parse_args(argv)

    try:
        plan = plan_screening(
            args.location, args.discharge_m3s, args.buffer_km,
            args.duration_hours, args.manning_n, args.snapshots,
        )
    except resolver.LocationError as e:
        print(json.dumps({"error": e.message}, ensure_ascii=False))
        return 1

    if plan["run"] is None:
        print(json.dumps(plan, ensure_ascii=False))
        return 0

    output_path = args.output or os.path.join(
        "/tmp", f"flood_screening_{plan['resolution']['name'].replace(' ', '_')}.png"
    )
    result = _run_swe(plan["run"], output_path)
    if result is None:
        print(json.dumps({
            "error": "solver/DEM gagal. Lihat log untuk detail.",
            "plan": plan,
        }, ensure_ascii=False))
        return 2

    result["resolution"] = plan["resolution"]
    result["status"] = "screening_only"
    result["missing"] = []
    result["limitations"] = plan["limitations"]
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
