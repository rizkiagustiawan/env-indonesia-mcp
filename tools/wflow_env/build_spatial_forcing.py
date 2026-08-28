#!/usr/bin/env python3
"""Build spatial Open-Meteo forcing for the Citarum Wflow grid.

This remains a screening product: Open-Meteo is a reanalysis/model product,
not BMKG station truth. The script only interpolates downloaded source values;
it does not invent rainfall observations.
"""

import hashlib
import json
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
import xarray as xr
from scipy.interpolate import RegularGridInterpolator


ROOT = Path(__file__).resolve().parents[2]
BENCH = ROOT / "data/benchmarks/citarum_hulu"
MODEL = BENCH / "wflow"
RAW = BENCH / "forcing_open_meteo_2016-03-10_2016-03-16_spatial.raw.json"
SOURCE = BENCH / "forcing_open_meteo_2016-03-10_2016-03-16_spatial.json"
OUTPUT = MODEL / "forcing_2016_spatial.nc"
RECEIPT = MODEL / "spatial_forcing_receipt.json"
CONFIG = MODEL / "citarum_sbm_spatial.toml"
STATIC = MODEL / "staticmaps.nc"

# Bounds cover the complete ~557 m target grid without extrapolation.
SOURCE_LAT = np.array([-7.30, -7.00, -6.70], dtype=float)
SOURCE_LON = np.array([107.20, 107.60, 108.00], dtype=float)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def interpolate_daily_grid(source_lat, source_lon, values, target_lat, target_lon):
    """Interpolate a source lat/lon matrix to a target lat/lon grid."""
    source_lat = np.asarray(source_lat, dtype=float)
    source_lon = np.asarray(source_lon, dtype=float)
    values = np.asarray(values, dtype=float)
    target_lat = np.asarray(target_lat, dtype=float)
    target_lon = np.asarray(target_lon, dtype=float)
    interpolator = RegularGridInterpolator(
        (source_lat, source_lon), values, bounds_error=True
    )
    lat_grid, lon_grid = np.meshgrid(target_lat, target_lon, indexing="ij")
    points = np.column_stack((lat_grid.ravel(), lon_grid.ravel()))
    return interpolator(points).reshape(lat_grid.shape).astype(np.float32)


def fetch_source():
    latitudes = [float(lat) for lat in SOURCE_LAT for _ in SOURCE_LON]
    longitudes = [float(lon) for _ in SOURCE_LAT for lon in SOURCE_LON]
    query = {
        "latitude": ",".join(map(str, latitudes)),
        "longitude": ",".join(map(str, longitudes)),
        "start_date": "2016-03-10",
        "end_date": "2016-03-16",
        "daily": "precipitation_sum,temperature_2m_mean,et0_fao_evapotranspiration",
        "timezone": "Asia/Bangkok",
    }
    url = "https://archive-api.open-meteo.com/v1/archive?" + urllib.parse.urlencode(query)
    with urllib.request.urlopen(url, timeout=120) as response:
        payload = json.load(response)
    RAW.write_text(json.dumps(payload, indent=2) + "\n")
    SOURCE.write_text(
        json.dumps(
            {
                "source": "Open-Meteo Historical Weather API",
                "source_url": url,
                "retrieved_at": datetime.now(timezone.utc).isoformat(),
                "role": "spatial_forcing_input",
                "ground_truth": False,
                "grid_latitude": SOURCE_LAT.tolist(),
                "grid_longitude": SOURCE_LON.tolist(),
                "variables": [
                    "precipitation_sum_mm",
                    "temperature_2m_mean_degC",
                    "et0_fao_evapotranspiration_mm",
                ],
                "raw_response": str(RAW),
                "limitations": [
                    "Open-Meteo is a model/reanalysis-derived forcing product, not BMKG station truth.",
                    "The source grid is interpolated to the Wflow grid; no station bias correction is applied.",
                ],
            },
            indent=2,
        )
        + "\n"
    )
    return payload


def main():
    MODEL.mkdir(parents=True, exist_ok=True)
    payload = fetch_source()
    with xr.open_dataset(STATIC) as static:
        target_lat = static.lat.values
        target_lon = static.lon.values
        active = ~np.isnan(static.wflow_subcatch.values)

    responses = np.asarray(payload, dtype=object).reshape(len(SOURCE_LAT), len(SOURCE_LON))
    times = responses[0, 0]["daily"]["time"]
    ntime = len(times)
    shape = (ntime, len(target_lat), len(target_lon))
    precip = np.full(shape, -9999.0, dtype=np.float32)
    temp = np.full(shape, -9999.0, dtype=np.float32)
    pet = np.full(shape, -9999.0, dtype=np.float32)

    for t in range(ntime):
        fields = {}
        for name, key in {
            "precip": "precipitation_sum",
            "temp": "temperature_2m_mean",
            "pet": "et0_fao_evapotranspiration",
        }.items():
            source_values = np.array(
                [[responses[i, j]["daily"][key][t] for j in range(len(SOURCE_LON))]
                 for i in range(len(SOURCE_LAT))],
                dtype=float,
            )
            fields[name] = interpolate_daily_grid(
                SOURCE_LAT, SOURCE_LON, source_values, target_lat, target_lon
            )
        precip[t][active] = fields["precip"][active]
        temp[t][active] = fields["temp"][active]
        pet[t][active] = fields["pet"][active]

    cf_time = (np.asarray(times, dtype="datetime64[D]") - np.datetime64("2000-01-01")).astype(float)
    dataset = xr.Dataset(
        data_vars={
            "precip": (("time", "lat", "lon"), precip, {"units": "mm/day"}),
            "pet": (("time", "lat", "lon"), pet, {"units": "mm/day"}),
            "temp": (("time", "lat", "lon"), temp, {"units": "degC"}),
        },
        coords={
            "time": (("time",), cf_time, {"units": "days since 2000-01-01 00:00:00", "calendar": "standard"}),
            "lat": (("lat",), target_lat),
            "lon": (("lon",), target_lon),
        },
        attrs={
            "title": "Citarum spatial Open-Meteo forcing",
            "source": "Open-Meteo 3x3 source grid interpolated to Wflow grid",
            "screening_status": "screening_only",
        },
    )
    encoding = {name: {"zlib": True, "complevel": 4, "_FillValue": -9999.0} for name in ("precip", "pet", "temp")}
    encoding.update({name: {"_FillValue": None} for name in ("time", "lat", "lon")})
    dataset.to_netcdf(OUTPUT, encoding=encoding)

    base_config = (MODEL / "citarum_sbm.toml").read_text()
    spatial_config = base_config.replace(
        'path_forcing = "forcing_2016.nc"',
        'path_forcing = "forcing_2016_spatial.nc"',
    ).replace('path = "output.csv"', 'path = "output_spatial.csv"')
    spatial_config = spatial_config.replace(
        '[logging]\nsilent = true',
        '[logging]\nsilent = true\npath_log = "log_spatial.txt"',
    )
    CONFIG.write_text(spatial_config)

    receipt = {
        "schema_version": "0.1.0",
        "status": "screening_only",
        "source": str(SOURCE),
        "source_sha256": sha256(SOURCE),
        "raw_source": str(RAW),
        "raw_source_sha256": sha256(RAW),
        "output": str(OUTPUT),
        "output_sha256": sha256(OUTPUT),
        "config": str(CONFIG),
        "grid_shape": list(shape),
        "source_grid": {"lat": SOURCE_LAT.tolist(), "lon": SOURCE_LON.tolist()},
        "period": [times[0], times[-1]],
        "variables": {
            "precip": {"min_mm_per_day": float(np.nanmin(precip[precip > -9998])), "max_mm_per_day": float(np.nanmax(precip))},
            "pet": {"min_mm_per_day": float(np.nanmin(pet[pet > -9998])), "max_mm_per_day": float(np.nanmax(pet))},
            "temp": {"min_degC": float(np.nanmin(temp[temp > -9998])), "max_degC": float(np.nanmax(temp))},
        },
        "limitations": [
            "Open-Meteo is not BMKG observation and has no station bias correction.",
            "The 3x3 source grid is interpolated spatially; it is not a new observation.",
            "No calibration, validation, or independent discharge comparison is implied.",
        ],
    }
    RECEIPT.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
