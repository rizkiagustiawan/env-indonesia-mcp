#!/usr/bin/env python3
"""Build a CHIRPS daily rainfall forcing for the Citarum Wflow grid.

Rainfall comes from CHIRPS v2 daily 0.05-degree GeoTIFFs. Temperature and ET0
come from Open-Meteo at the AOI centre because the current model period has no
local daily BMKG series. The resulting product is screening-only.
"""

import gzip
import hashlib
import json
import argparse
import re
import shutil
import tempfile
import urllib.parse
import urllib.request
from datetime import datetime, timedelta, timezone
from pathlib import Path

import numpy as np
import rasterio
import xarray as xr
from scipy.interpolate import RegularGridInterpolator


ROOT = Path(__file__).resolve().parents[2]
BENCH = ROOT / "data/benchmarks/citarum_hulu"
MODEL = BENCH / "wflow"
STATIC = MODEL / "staticmaps.nc"
CHIRPS_BASE = "https://data.chc.ucsb.edu/products/CHIRPS-2.0/global_daily/tifs/p05/2016"
DEFAULT_START = "2016-03-10"
DEFAULT_END = "2016-03-16"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def interpolate_daily_grid(source_lat, source_lon, values, target_lat, target_lon):
    interpolator = RegularGridInterpolator(
        (np.asarray(source_lat), np.asarray(source_lon)),
        np.asarray(values, dtype=float),
        bounds_error=True,
    )
    lat_grid, lon_grid = np.meshgrid(target_lat, target_lon, indexing="ij")
    points = np.column_stack((lat_grid.ravel(), lon_grid.ravel()))
    return interpolator(points).reshape(lat_grid.shape).astype(np.float32)


def chirps_url(date: str) -> str:
    filename_date = date.replace("-", ".")
    base = CHIRPS_BASE.rsplit("/", 1)[0] + f"/{date[:4]}"
    return f"{base}/chirps-v2.0.{filename_date}.tif.gz"


def date_range(start: str, end: str):
    start_date = datetime.strptime(start, "%Y-%m-%d").date()
    end_date = datetime.strptime(end, "%Y-%m-%d").date()
    if end_date < start_date:
        raise ValueError("end date must not precede start date")
    return [
        (start_date + timedelta(days=offset)).isoformat()
        for offset in range((end_date - start_date).days + 1)
    ]


def artifact_paths(start: str, end: str, suffix: str):
    """Return stable artifact names, retaining names from the original run."""
    if start == DEFAULT_START and end == DEFAULT_END and suffix == "chirps":
        return (
            MODEL / "forcing_2016_chirps.nc",
            BENCH / "forcing_chirps_2016-03-10_2016-03-16.json",
            MODEL / "chirps_forcing_receipt.json",
            MODEL / "citarum_sbm_chirps.toml",
        )
    return (
        MODEL / f"forcing_{start}_{end}_{suffix}.nc",
        BENCH / f"forcing_chirps_{start}_{end}_{suffix}.json",
        MODEL / f"{suffix}_forcing_receipt.json",
        MODEL / f"citarum_sbm_{suffix}.toml",
    )


def download_chirps(date: str, directory: Path) -> Path:
    url = chirps_url(date)
    compressed = directory / f"{date}.tif.gz"
    tif = directory / f"{date}.tif"
    urllib.request.urlretrieve(url, compressed)
    with gzip.open(compressed, "rb") as source, tif.open("wb") as target:
        shutil.copyfileobj(source, target)
    return tif


def build_forcing_dataset(times, lat, lon, precip, pet, temp):
    cf_time = (
        np.asarray(times, dtype="datetime64[D]") - np.datetime64("2000-01-01")
    ).astype(float)
    return xr.Dataset(
        data_vars={
            "precip": (("time", "lat", "lon"), precip, {"units": "mm/day"}),
            "pet": (("time", "lat", "lon"), pet, {"units": "mm/day"}),
            "temp": (("time", "lat", "lon"), temp, {"units": "degC"}),
        },
        coords={
            "time": (("time",), cf_time, {"units": "days since 2000-01-01 00:00:00", "calendar": "standard"}),
            "lat": (("lat",), lat),
            "lon": (("lon",), lon),
        },
        attrs={"screening_status": "screening_only"},
    )


def fetch_open_meteo(dates):
    query = {
        "latitude": "-6.994727",
        "longitude": "107.62025",
        "start_date": dates[0],
        "end_date": dates[-1],
        "daily": "temperature_2m_mean,et0_fao_evapotranspiration",
        "timezone": "Asia/Bangkok",
    }
    url = "https://archive-api.open-meteo.com/v1/archive?" + urllib.parse.urlencode(query)
    with urllib.request.urlopen(url, timeout=120) as response:
        return url, json.load(response)


def main(start=DEFAULT_START, end=DEFAULT_END, suffix="chirps"):
    dates = date_range(start, end)
    output, source, receipt, config = artifact_paths(start, end, suffix)
    MODEL.mkdir(parents=True, exist_ok=True)
    with xr.open_dataset(STATIC) as static:
        target_lat = static.lat.values
        target_lon = static.lon.values
        active = ~np.isnan(static.wflow_subcatch.values)

    source_records = []
    with tempfile.TemporaryDirectory(prefix="chirps-") as temp:
        temp_dir = Path(temp)
        for date in dates:
            tif = download_chirps(date, temp_dir)
            with rasterio.open(tif) as dataset:
                if dataset.crs is None:
                    raise ValueError(f"CHIRPS raster has no CRS: {tif}")
                margin = 0.1
                window = dataset.window(
                    float(target_lon.min()) - margin,
                    float(target_lat.min()) - margin,
                    float(target_lon.max()) + margin,
                    float(target_lat.max()) + margin,
                )
                values = dataset.read(1, window=window).astype(np.float32)
                transform = dataset.window_transform(window)
                nodata = dataset.nodata
                source_lon = transform.c + (np.arange(values.shape[1]) + 0.5) * transform.a
                source_lat = transform.f + (np.arange(values.shape[0]) + 0.5) * transform.e
                valid = np.isfinite(values)
                if nodata is not None:
                    valid &= values != nodata
                values = np.where(valid, values, np.nan)
                # CHIRPS latitude is descending; normalize for interpolation.
                if source_lat[0] > source_lat[-1]:
                    source_lat = source_lat[::-1]
                    values = values[::-1, :]
                source_records.append(
                    interpolate_daily_grid(
                        source_lat,
                        source_lon,
                        values,
                        target_lat,
                        target_lon,
                    )
                )

    meteo_url, meteo = fetch_open_meteo(dates)
    daily = meteo["daily"]
    temp_values = np.asarray(daily["temperature_2m_mean"], dtype=np.float32)
    pet_values = np.asarray(daily["et0_fao_evapotranspiration"], dtype=np.float32)
    shape = (len(dates), len(target_lat), len(target_lon))
    precip = np.full(shape, -9999.0, dtype=np.float32)
    temp = np.full(shape, -9999.0, dtype=np.float32)
    pet = np.full(shape, -9999.0, dtype=np.float32)
    for i, rainfall in enumerate(source_records):
        precip[i][active] = rainfall[active]
        temp[i][active] = temp_values[i]
        pet[i][active] = pet_values[i]

    dataset = build_forcing_dataset(dates, target_lat, target_lon, precip, pet, temp)
    encoding = {name: {"zlib": True, "complevel": 4, "_FillValue": -9999.0} for name in ("precip", "pet", "temp")}
    encoding.update({name: {"_FillValue": None} for name in ("time", "lat", "lon")})
    dataset.to_netcdf(output, encoding=encoding)
    spatial_config = (MODEL / "citarum_sbm.toml").read_text()
    spatial_config = re.sub(
        r"starttime = \d{4}-\d{2}-\d{2}T00:00:00",
        f"starttime = {dates[0]}T00:00:00",
        spatial_config,
    )
    spatial_config = re.sub(
        r"endtime = \d{4}-\d{2}-\d{2}T00:00:00",
        f"endtime = {dates[-1]}T00:00:00",
        spatial_config,
    )
    spatial_config = spatial_config.replace(
        'path_forcing = "forcing_2016.nc"',
        f'path_forcing = "{output.name}"',
    ).replace('path = "output.csv"', f'path = "output_{suffix}.csv"')
    spatial_config = spatial_config.replace(
        '[logging]\nsilent = true',
        f'[logging]\nsilent = true\npath_log = "log_{suffix}.txt"',
    )
    config.write_text(spatial_config)

    source.write_text(
        json.dumps(
            {
                "source": "CHIRPS v2 daily + Open-Meteo daily",
                "chirps_base_url": CHIRPS_BASE,
                "chirps_files": [chirps_url(date) for date in dates],
                "open_meteo_url": meteo_url,
                "retrieved_at": datetime.now(timezone.utc).isoformat(),
                "period": [dates[0], dates[-1]],
                "chirps_resolution_deg": 0.05,
                "temperature_pet_source": "Open-Meteo AOI centre",
                "ground_truth": False,
                "limitations": [
                    "CHIRPS is satellite/gauge blended rainfall, not BMKG station truth.",
                    "Temperature and ET0 are spatially uniform from one Open-Meteo point.",
                    "No gauge bias correction or hydrologic calibration is applied.",
                ],
            },
            indent=2,
        )
        + "\n"
    )
    receipt_data = {
        "schema_version": "0.1.0",
        "status": "screening_only",
        "source": str(source),
        "source_sha256": sha256(source),
        "output": str(output),
        "output_sha256": sha256(output),
        "config": str(config),
        "grid_shape": list(shape),
        "period": [dates[0], dates[-1]],
        "precipitation_range_mm_per_day": [float(np.nanmin(precip[precip > -9998])), float(np.nanmax(precip))],
        "limitations": [
            "CHIRPS is not BMKG observation and is not a legal/regulatory measurement.",
            "Temperature and ET0 use Open-Meteo at one point.",
            "Results remain screening_only until independent discharge validation exists.",
        ],
    }
    receipt.write_text(json.dumps(receipt_data, indent=2) + "\n")
    print(json.dumps(receipt_data, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--start-date", default=DEFAULT_START)
    parser.add_argument("--end-date", default=DEFAULT_END)
    parser.add_argument("--suffix", default="chirps")
    args = parser.parse_args()
    main(args.start_date, args.end_date, args.suffix)
