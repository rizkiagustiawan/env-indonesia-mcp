#!/usr/bin/env python3
"""Build Wflow SBM staticmaps and forcing for Citarum Hulu screening.

Inputs:
  - DEM conditioned (burn-before-fill, 5m burn depth) at 30m EPSG:32748
  - Open-Meteo daily forcing 2016-03-10/2016-03-16
  - RBI river network (for reference, not required by pyflwdir)
  - Literature soil/landcover defaults (no local survey)

Output:
  - data/benchmarks/citarum_hulu/wflow/staticmaps.nc
  - data/benchmarks/citarum_hulu/wflow/forcing_2016.nc
  - data/benchmarks/citarum_hulu/wflow/citarum_sbm.toml
  - data/benchmarks/citarum_hulu/wflow/build_receipt.json

Run from repo root:
  source tools/wflow_env/python/venv/bin/activate
  python tools/wflow_env/build_citarum_model.py
"""
import json, hashlib, os, sys, time
from pathlib import Path

import numpy as np
import rasterio
from rasterio.enums import Resampling
from rasterio.warp import reproject, calculate_default_transform, Resampling as WarpResampling, transform_bounds
from rasterio.transform import from_origin
import xarray as xr
import pyflwdir
from scipy import ndimage

REPO = Path(__file__).resolve().parents[2]
BENCH = REPO / "data/benchmarks/citarum_hulu"
DEM_SRC = BENCH / "conditioning/dem_citarum_hulu_conditioned_streamburn5m.tif"
FORCING_SRC = BENCH / "forcing_open_meteo_2016-03-10_2016-03-16.json"
OUT_DIR = BENCH / "wflow"
RES_DEG = 0.005  # ~557m at equator

OUT_DIR.mkdir(parents=True, exist_ok=True)

def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()

# ---------------------------------------------------------------------------
# 1. Resample DEM to target grid in EPSG:4326
# ---------------------------------------------------------------------------
print("1. Resampling DEM ...")
dem_fine = DEM_SRC
with rasterio.open(dem_fine) as src:
    src_crs = src.crs  # EPSG:32748
    src_bounds = src.bounds
    dst_crs = "EPSG:4326"

    # Compute output grid in EPSG:4326 manually for guaranteed resolution
    bounds_wgs84 = transform_bounds(src_crs, dst_crs, *src.bounds)
    xmin, ymin, xmax, ymax = bounds_wgs84
    width = int(np.ceil((xmax - xmin) / RES_DEG))
    height = int(np.ceil((ymax - ymin) / RES_DEG))
    transform_4326 = from_origin(xmin, ymax, RES_DEG, RES_DEG)
    print(f"  Bounds 4326: ({xmin:.4f}, {ymin:.4f}, {xmax:.4f}, {ymax:.4f})")
    print(f"  Output grid 4326: {width} x {height}")
    print(f"  Resolution: {transform_4326.a} x {abs(transform_4326.e)} deg")

    dem_4326 = np.full((height, width), -9999.0, dtype=np.float32)
    reproject(
        source=src.read(1),
        destination=dem_4326,
        src_transform=src.transform,
        src_crs=src_crs,
        src_nodata=-9999.0,
        dst_transform=transform_4326,
        dst_crs=dst_crs,
        dst_nodata=-9999.0,
        resampling=Resampling.bilinear,
    )
    dem_mask = dem_4326 > -9998.0
    valid = dem_mask.sum()
    print(f"  Valid cells: {valid:,} / {width * height:,}")

# ---------------------------------------------------------------------------
# 2. Derive LDD, upstream area, streams via pyflwdir
# ---------------------------------------------------------------------------
print("2. Computing drainage directions ...")
# pyflwdir needs a 2D float array and a transform
flw = pyflwdir.from_dem(
    data=dem_4326,
    nodata=-9999.0,
    transform=transform_4326,
    latlon=True,
)
print(f"  LDD shape: {flw.shape}")
_pit_count = len(flw.idxs_pit) if flw.idxs_pit is not None and hasattr(flw.idxs_pit, '__len__') else 0
print(f"  Pit cells: {_pit_count}")

# Upstream area in number-of-cells
uparea = flw.upstream_area("cell")
# Subcatchments (basin IDs from pit cells)
subcatch = flw.basins()
print(f"  Subcatchments: {subcatch.max()}")

# River mask: > 500 cells upstream area (~500 * 0.25 km2 = 125 km2 at 0.005 deg)
# Actually at lat -7, each cell is ~557m x 554m = 0.309 km2
# 50 km2 threshold -> ~162 cells
# 100 km2 threshold -> ~324 cells
# Let's use a low threshold for screening: ~108 cells = ~33 km2
river_threshold_cells = 108
river_mask = uparea >= river_threshold_cells
print(f"  River cells: {river_mask.sum()} (threshold {river_threshold_cells} cells)")

# ---- Reclassify subcatchments: only create subcatchments for rivers above threshold ----
# Map each subcatchment ID to its outlet's upstream area
subcatch_ids = np.unique(subcatch[subcatch > 0])
subcatch_outlet_area = {}
for sid in subcatch_ids:
    mask = subcatch == sid
    # Find outlet: the cell with max upstream area in this subcatch
    sub_upa = np.where(mask, uparea, 0)
    max_uparea = sub_upa.max()
    subcatch_outlet_area[sid] = max_uparea

# Assign new sequential IDs to subcatchments that have outlet_uparea >= river_threshold_cells
new_id = 0
subcatch_map = {}
for sid in sorted(subcatch_ids):
    if subcatch_outlet_area.get(sid, 0) >= river_threshold_cells:
        new_id += 1
        subcatch_map[sid] = new_id

subcatch2 = np.zeros_like(subcatch)
for sid, nid in subcatch_map.items():
    subcatch2[subcatch == sid] = nid
print(f"  Active subcatchments: {new_id}")

# ---------------------------------------------------------------------------
# 3. Compute river properties
# ---------------------------------------------------------------------------
print("3. Computing river properties ...")

# River order (Strahler) - but we just use the mask
# River length: each cell contributes sqrt(dx^2 + dy^2) depending on flow direction
# Use pyflwdir's cell_area and dx/dy
cell_dx = abs(transform_4326[0]) * 111320 * np.cos(np.radians(np.mean(dem_4326[dem_mask])))
# Actually, pyflwdir with latlon=True uses distances internally
# Let's just use constant cell size in meters at this latitude
lat_mean = abs(transform_4326[3] + transform_4326[5]*height/2)  # center lat
lat_center = abs((ymin + ymax) / 2)
cell_size_m = RES_DEG * 111320 * np.cos(np.radians(lat_center))
print(f"  Cell size at lat {lat_center:.1f}: {cell_size_m:.1f}m")
# cell diagonal for D4 diagonal directions
_diag = cell_size_m * np.sqrt(2)

# River length: sum of downstream path lengths per river cell snippet
ldd = flw.to_array()  # LDD direction encoding
river_length = np.where(river_mask, cell_size_m, -9999.0).astype(np.float32)

# River width: empirical power law width = a * (upstream_area)^b
# Compute only on river cells
_riv_area_km2 = np.where(river_mask, uparea * (cell_size_m**2) / 1e6, 1.0)
_riv_width = np.clip(1.5 * _riv_area_km2**0.35, 5, 200)
river_width = np.where(river_mask, _riv_width, -9999.0).astype(np.float32)

# Land slope
_gy, _gx = np.gradient(dem_4326, cell_size_m, cell_size_m)
land_slope = np.sqrt(_gx**2 + _gy**2)
land_slope = np.clip(land_slope, 0.0001, None).astype(np.float32)

# River slope: approximate from DEM gradient along rivers
# For screening, use land_slope on river cells; exact along-flow slope is expensive
river_slope = np.where(river_mask & dem_mask, np.clip(land_slope, 0.0001, None), -9999.0).astype(np.float32)

# ---------------------------------------------------------------------------
# 4. Soil & landcover parameters (literature defaults for tropical Java)
# ---------------------------------------------------------------------------
print("4. Setting soil & landcover parameters ...")

# Soil defaults: silty loam / tropical clay loam (van Genuchten -> Brooks-Corey approx)
thetaS = np.where(dem_mask, 0.48, -9999.0).astype(np.float32)  # saturated water content
thetaR = np.where(dem_mask, 0.08, -9999.0).astype(np.float32)  # residual water content
KsatVer = np.where(dem_mask, 150.0, -9999.0).astype(np.float32)  # mm/day, vertical Ksat
c = np.where(dem_mask, 4.0, -9999.0).astype(np.float32)  # Brooks-Corey exponent
KsatHorFrac = np.where(dem_mask, 50.0, -9999.0).astype(np.float32)
f = np.where(dem_mask, 0.1, -9999.0).astype(np.float32)

# Additional soil
SoilThickness = np.where(dem_mask, 1000.0, -9999.0).astype(np.float32)
cf_soil = np.where(dem_mask, 100.0, -9999.0).astype(np.float32)
InfiltCapPath = np.where(dem_mask, 20.0, -9999.0).astype(np.float32)
PathFrac = np.where(dem_mask, 0.01, -9999.0).astype(np.float32)
MaxLeakage = np.where(dem_mask, 0.0, -9999.0).astype(np.float32)
rootdistpar = np.where(dem_mask, -500.0, -9999.0).astype(np.float32)

# Manning's n
N = np.where(dem_mask, 0.1, -9999.0).astype(np.float32)  # land
N_River = np.where(river_mask, 0.04, -9999.0).astype(np.float32)

# Landcover defaults (tropical forest/shrub)
CanopyGap = np.where(dem_mask, 0.1, -9999.0).astype(np.float32)
EoverR = np.where(dem_mask, 0.08, -9999.0).astype(np.float32)
Kext = np.where(dem_mask, 0.5, -9999.0).astype(np.float32)
Sl = np.where(dem_mask, 0.1, -9999.0).astype(np.float32)
Swood = np.where(dem_mask, 0.0, -9999.0).astype(np.float32)
RootingDepth = np.where(dem_mask, 750.0, -9999.0).astype(np.float32)  # mm

# Water fraction
WaterFrac = np.where(dem_mask, 0.0, -9999.0).astype(np.float32)
WaterFrac = np.where(river_mask, 1.0, WaterFrac).astype(np.float32)

# Snow
TT = np.where(dem_mask, 0.0, -9999.0).astype(np.float32)
TTI = np.where(dem_mask, 0.0, -9999.0).astype(np.float32)
TTM = np.where(dem_mask, 0.0, -9999.0).astype(np.float32)
Cfmax = np.where(dem_mask, 1.0, -9999.0).astype(np.float32)

# LDD: use pyflwdir's native PCR-LDD output (values 1-9, 5=pit), north-first
ldd_pcr_north = flw.to_array("ldd").astype(np.uint8)
subcatch_data = np.where(dem_mask, subcatch2, -9999).astype(np.int32)
river_data = np.where(river_mask, 1, 0).astype(np.int32)

def _ldd_to_south(arr_north):
    # Wflow standardizes (lat, lon) to (x, y) and handles the axis direction
    # during read_standardized; remapping direction codes here creates cycles.
    return np.flipud(arr_north).astype(np.uint8)

# ---------------------------------------------------------------------------
# 5. Write staticmaps.nc
# ---------------------------------------------------------------------------
print("5. Writing staticmaps.nc ...")
static_path = OUT_DIR / "staticmaps.nc"

_x_origin = transform_4326.c
_y_origin = transform_4326.f  # north
_dx = transform_4326.a
_dy = transform_4326.e  # negative for north-up

# Build lat from south to north (ascending) for CF/NetCDF convention
_lat_south = _y_origin + _dy * (height - 1)
_lat_north = _y_origin
_lat_vals = np.linspace(_lat_south, _lat_north, height)
_lon_vals = np.linspace(_x_origin, _x_origin + _dx * (width - 1), width)

# Flip all 2D arrays to match south-first ordering (originally north-first from affine)
def _south_first(arr):
    return np.flipud(arr)

ds = xr.Dataset(coords={
    "layer": (["layer"], np.arange(1, 5, dtype=np.int32)),
    "lat": (["lat"], _lat_vals),
    "lon": (["lon"], _lon_vals),
})

# Variable dictionary (all arrays are currently north-first, will be flipped below)
variables = {
    "wflow_ldd": (ldd_pcr_north, "Local Drain Direction (PCRaster LDD)"),
    "wflow_subcatch": (subcatch_data.astype(np.float64), "Subcatchment ID"),
    "wflow_river": (river_data, "River mask"),
    "wflow_riverlength": (river_length, "m"),
    "wflow_riverwidth": (river_width, "m"),
    "RiverSlope": (river_slope, "m/m"),
    "Slope": (land_slope, "m/m"),
    "N": (N, "s/m^(1/3)"),
    "N_River": (N_River, "s/m^(1/3)"),
    "thetaS": (thetaS, "m3/m3"),
    "thetaR": (thetaR, "m3/m3"),
    "KsatVer": (KsatVer, "mm/day"),
    "KsatHorFrac": (KsatHorFrac, "-"),
    "c": (np.stack([c, c, c, c], axis=0), "-"),
    "f": (f, "-"),
    "SoilThickness": (SoilThickness, "mm"),
    "cf_soil": (cf_soil, "mm/day"),
    "InfiltCapPath": (InfiltCapPath, "mm/day"),
    "PathFrac": (PathFrac, "-"),
    "MaxLeakage": (MaxLeakage, "mm/day"),
    "rootdistpar": (rootdistpar, "-"),
    "CanopyGap": (CanopyGap, "-"),
    "EoverR": (EoverR, "-"),
    "Kext": (Kext, "-"),
    "Sl": (Sl, "mm"),
    "Swood": (Swood, "mm"),
    "RootingDepth": (RootingDepth, "mm"),
    "WaterFrac": (WaterFrac, "-"),
    "TT": (TT, "degC"),
    "TTI": (TTI, "degC"),
    "TTM": (TTM, "degC"),
    "Cfmax": (Cfmax, "mm/degC/day"),
    "soil__thickness_l1": (np.where(dem_mask, 100.0, -9999.0).astype(np.float32), "mm"),
    "soil__thickness_l2": (np.where(dem_mask, 300.0, -9999.0).astype(np.float32), "mm"),
    "soil__thickness_l3": (np.where(dem_mask, 800.0, -9999.0).astype(np.float32), "mm"),
}

for vname, (arr, units) in variables.items():
    if vname == "wflow_ldd":
        ds[vname] = (["lat", "lon"], _ldd_to_south(arr))
    elif vname == "c":
        ds[vname] = (["layer", "lat", "lon"], np.stack([_south_first(arr[0]), _south_first(arr[1]), _south_first(arr[2]), _south_first(arr[3])]))
    else:
        ds[vname] = (["lat", "lon"], _south_first(arr))
    ds[vname].attrs["units"] = units
    if arr.dtype in (np.float32, np.float64) and np.nanmin(arr) >= 0:
        ds[vname].attrs["_FillValue"] = -9999.0

ds["wflow_subcatch"].attrs["_FillValue"] = -9999
ds["wflow_ldd"].encoding["_FillValue"] = np.uint8(0)

# Global attrs
ds.attrs["title"] = "Wflow SBM Staticmaps — Citarum Hulu (screening)"
ds.attrs["source"] = "GLO-30 DEM conditioned (burn-before-fill, 5m), pyflwdir drainage"
ds.attrs["resolution_deg"] = RES_DEG
ds.attrs["crs"] = "EPSG:4326"

# Clean any stray _FillValue from attrs before encoding
for v in ds.data_vars:
    ds[v].attrs.pop("_FillValue", None)

encoding = {}
for v in ds.data_vars:
    enc = {"zlib": True, "complevel": 4}
    dtype_kind = ds[v].dtype.kind
    if v == "wflow_ldd":
        enc["dtype"] = "uint8"
    elif dtype_kind in ('i', 'u'):
        enc["_FillValue"] = np.int32(-9999)
    elif dtype_kind == 'f':
        enc["_FillValue"] = np.float32(-9999.0)
    encoding[v] = enc

ds.to_netcdf(static_path, encoding=encoding)
print(f"  Written: {static_path} ({os.path.getsize(static_path)/1024:.0f} KB)")

# ---------------------------------------------------------------------------
# 6. Build forcing NetCDF for March 2016 event
# ---------------------------------------------------------------------------
print("6. Building forcing NetCDF ...")
with open(FORCING_SRC) as f:
    fc_data = json.load(f)

import pandas as pd

# daily is a list of per-day dicts: [{date, precipitation_sum_mm, rain_sum_mm}, ...]
daily_list = fc_data["daily"]
times = [d["date"] for d in daily_list]
precip_vals = np.array([d.get("precipitation_sum_mm", 0.0) for d in daily_list], dtype=np.float32)
# PET: approximate Hargreaves ~ 3.5 mm/day tropical March
pet_vals = np.full(len(times), 3.5, dtype=np.float32)
temp_vals = np.full(len(times), 24.0, dtype=np.float32)  # approximate mean

# Convert string dates to CF-compliant numeric time (days since epoch)
_time_dt = pd.to_datetime(times)
_time_cf = (_time_dt - pd.Timestamp("2000-01-01")).days.values.astype(np.float64)

forcing_path = OUT_DIR / "forcing_2016.nc"
nsteps = len(times)

# Create spatially-uniform forcing (extend to full grid)
# Precip varies daily but uniform spatially (single station approximation)
precip_3d = np.zeros((nsteps, height, width), dtype=np.float32)
pet_3d = np.zeros((nsteps, height, width), dtype=np.float32)
temp_3d = np.zeros((nsteps, height, width), dtype=np.float32)

for i in range(nsteps):
    precip_3d[i] = _south_first(np.where(dem_mask, precip_vals[i], -9999.0))
    pet_3d[i] = _south_first(np.where(dem_mask, pet_vals[i], -9999.0))
    temp_3d[i] = _south_first(np.where(dem_mask, temp_vals[i], -9999.0))

ds_f = xr.Dataset(
    data_vars={
        "precip": (["time", "lat", "lon"], precip_3d, {"units": "mm/day", "_FillValue": -9999.0}),
        "pet": (["time", "lat", "lon"], pet_3d, {"units": "mm/day", "_FillValue": -9999.0}),
        "temp": (["time", "lat", "lon"], temp_3d, {"units": "degC", "_FillValue": -9999.0}),
    },
    coords={
        "time": (["time"], _time_cf, {"units": "days since 2000-01-01 00:00:00", "calendar": "standard"}),
        "lat": (["lat"], ds.lat.values),
        "lon": (["lon"], ds.lon.values),
    },
)
ds_f.attrs["title"] = "Wflow Forcing — Citarum Hulu March 2016 event"
ds_f.attrs["source"] = "Open-Meteo API, spatially uniform"
forcing_encoding = {v: {"zlib": True, "complevel": 4} for v in ds_f.data_vars}
forcing_encoding.update({coord: {"_FillValue": None} for coord in ("time", "lat", "lon")})
ds_f.to_netcdf(forcing_path, encoding=forcing_encoding)
print(f"  Written: {forcing_path} ({os.path.getsize(forcing_path)/1024:.0f} KB)")
print(f"  Time steps: {nsteps}, Period: {times[0]} to {times[-1]}")

# ---------------------------------------------------------------------------
# 7. Write Wflow TOML config
# ---------------------------------------------------------------------------
print("7. Writing Wflow config ...")
toml_path = OUT_DIR / "citarum_sbm.toml"
toml_content = f"""# Wflow SBM Config — Citarum Hulu Screening
# Auto-generated by build_citarum_model.py
# STATUS: screening_only — no calibration, no validation

[time]
starttime = 2016-03-10T00:00:00
endtime = 2016-03-16T00:00:00
timestepsecs = 86400

[logging]
silent = true

[input]
path_forcing = "forcing_2016.nc"
path_static = "staticmaps.nc"
basin__local_drain_direction = "wflow_ldd"
river_location__mask = "wflow_river"
subbasin_location__count = "wflow_subcatch"

[input.forcing]
atmosphere_water__precipitation_volume_flux = "precip"
land_surface_water__potential_evaporation_volume_flux = "pet"
atmosphere_air__temperature = "temp"

[input.static]
atmosphere_air__snowfall_temperature_threshold = "TT"
atmosphere_air__snowfall_temperature_interval = "TTI"
snowpack__melting_temperature_threshold = "TTM"
snowpack__degree_day_coefficient = "Cfmax"
soil_layer_water__brooks_corey_exponent = "c"
soil_surface_water__infiltration_reduction_parameter = "cf_soil"
soil_surface_water__vertical_saturated_hydraulic_conductivity = "KsatVer"
soil_water__vertical_saturated_hydraulic_conductivity_scale_parameter = "f"
compacted_soil_surface_water__infiltration_capacity = "InfiltCapPath"
soil_water__residual_volume_fraction = "thetaR"
soil_water__saturated_volume_fraction = "thetaS"
soil_water_saturated_zone_bottom__max_leakage_volume_flux = "MaxLeakage"
compacted_soil__area_fraction = "PathFrac"
soil_wet_root__sigmoid_function_shape_parameter = "rootdistpar"
soil__thickness = "SoilThickness"
vegetation_canopy_water__mean_evaporation_to_mean_precipitation_ratio = "EoverR"
vegetation_canopy__light_extinction_coefficient = "Kext"
vegetation__specific_leaf_storage = "Sl"
vegetation_wood_water__storage_capacity = "Swood"
vegetation_root__depth = "RootingDepth"
vegetation_canopy__gap_fraction = "CanopyGap"
river__length = "wflow_riverlength"
river_water_flow__manning_n_parameter = "N_River"
river__slope = "RiverSlope"
river__width = "wflow_riverwidth"
land_surface_water_flow__manning_n_parameter = "N"
land_surface__slope = "Slope"
subsurface_water__horizontal_to_vertical_saturated_hydraulic_conductivity_ratio = "KsatHorFrac"
land_water_covered__area_fraction = "WaterFrac"

[model]
type = "sbm"
soil_layer__thickness = [100, 300, 800]
water_mass_balance__flag = true

[output.csv]
path = "output.csv"

[[output.csv.column]]
header = "Q"
parameter = "river_water__volume_flow_rate"
reducer = "maximum"
"""
toml_path.write_text(toml_content)
print(f"  Written: {toml_path}")

# ---------------------------------------------------------------------------
# 8. Write receipt
# ---------------------------------------------------------------------------
receipt = {
    "schema_version": "0.1.0",
    "status": "screening_only",
    "build_time_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "inputs": {
        "dem": str(DEM_SRC),
        "dem_sha256": sha256(DEM_SRC),
        "forcing": str(FORCING_SRC),
    },
    "parameters": {
        "resolution_deg": RES_DEG,
        "resolution_m_approx": cell_size_m,
        "river_threshold_cells": river_threshold_cells,
        "crs": "EPSG:4326",
        "soil_model": "literature_defaults_tropical_clay_loam",
        "landcover_model": "literature_defaults_tropical_forest",
        "forcing_method": "spatially_uniform_open_meteo",
        "forcing_period": f"{times[0]}/{times[-1]}",
        "precip_range_mm_per_day": [float(precip_vals.min()), float(precip_vals.max())],
    },
    "outputs": {
        "staticmaps": str(static_path),
        "forcing": str(forcing_path),
        "config": str(toml_path),
        "grid_shape": [height, width],
        "valid_cells": int(valid),
        "river_cells": int(river_mask.sum()),
        "subcatchments": int(new_id),
    },
    "hashes_sha256": {
        "staticmaps.nc": sha256(static_path),
        "forcing_2016.nc": sha256(forcing_path),
        "citarum_sbm.toml": hashlib.sha256(toml_content.encode()).hexdigest(),
    },
    "limitations": [
        "Soil and landcover parameters are literature defaults, not local survey or ISRIC/ESA CCI.",
        "Forcing is spatially uniform from a single Open-Meteo grid cell; no rainfall spatial variability.",
        "PET is approximate (Hargreaves or constant), not Penman-Monteith.",
        "No river cross-sections, reservoir, lake, or groundwater module.",
        "No calibration or validation has been performed.",
        "30m DEM was resampled to ~557m; sub-grid topographic effects are lost.",
        "This is a screening-level simulation only.",
    ],
}

receipt_path = OUT_DIR / "build_receipt.json"
with open(receipt_path, "w") as f:
    json.dump(receipt, f, indent=2, default=str)
print(f"\nDone. Receipt: {receipt_path}")
print(f"Grid: {height}x{width}, Valid: {valid:,}, River cells: {river_mask.sum()}, Subcatchments: {new_id}")
print(f"Output dir: {OUT_DIR}/")
for fn in ["staticmaps.nc", "forcing_2016.nc", "citarum_sbm.toml", "build_receipt.json"]:
    p = OUT_DIR / fn
    print(f"  {fn}: {os.path.getsize(p)/1024:.0f} KB")
