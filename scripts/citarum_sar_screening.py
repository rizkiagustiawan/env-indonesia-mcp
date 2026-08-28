"""Acquire same-orbit Sentinel-1 RTC bands and derive screening masks."""

import hashlib
import json
import math
import urllib.parse
import urllib.request
from pathlib import Path

import numpy as np
import rasterio
from rasterio.enums import Resampling
from rasterio.vrt import WarpedVRT


ROOT = Path(__file__).resolve().parents[1]
BENCHMARK = ROOT / "data/benchmarks/citarum_hulu"
OUTPUT = BENCHMARK / "sar"
STAC_SEARCH = "https://planetarycomputer.microsoft.com/api/stac/v1/search"
SAS_ENDPOINT = "https://planetarycomputer.microsoft.com/api/sas/v1/token/sentinel1euwestrtc/sentinel1-grd-rtc"
TARGET_REFERENCE = OUTPUT / "event_vv.tif"
THRESHOLD_DB = -3.0
NODATA = -9999.0
MASK_NODATA = 255

SCENES = {
    "event": "S1A_IW_GRDH_1SDV_20160314T222515_20160314T222528_010371_00F5F2_rtc",
    "dry_reference": "S1A_IW_GRDH_1SDV_20160829T222523_20160829T222536_012821_014392_rtc",
    "holdout": "S1A_IW_GRDH_1SDV_20250315T222549_20250315T222614_058321_0735B6_rtc",
}


def _request_json(url, payload=None):
    data = None if payload is None else json.dumps(payload).encode()
    request = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=120) as response:
        return json.load(response)


def _scene_metadata():
    response = _request_json(STAC_SEARCH, {"collections": ["sentinel-1-rtc"], "ids": list(SCENES.values()), "limit": 10})
    by_id = {feature["id"]: feature for feature in response["features"]}
    missing = sorted(set(SCENES.values()) - set(by_id))
    if missing:
        raise RuntimeError(f"STAC scenes missing: {missing}")
    return by_id


def _signed_url(href, token):
    separator = "&" if "?" in href else "?"
    return href + separator + token


def _sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _target_profile():
    with rasterio.open(TARGET_REFERENCE) as source:
        return {
            "crs": source.crs,
            "transform": source.transform,
            "width": source.width,
            "height": source.height,
            "bounds": [source.bounds.left, source.bounds.bottom, source.bounds.right, source.bounds.top],
        }


def _read_band(href, target):
    token = _request_json(SAS_ENDPOINT)["token"]
    with rasterio.open(_signed_url(href, token)) as source:
        with WarpedVRT(
            source,
            crs=target["crs"],
            transform=target["transform"],
            width=target["width"],
            height=target["height"],
            resampling=Resampling.bilinear,
            src_nodata=source.nodata,
            nodata=NODATA,
        ) as warped:
            return warped.read(1).astype(np.float32)


def _write_band(path, data, scene_id, polarization, href, target):
    profile = {
        "driver": "GTiff",
        "height": target["height"],
        "width": target["width"],
        "count": 1,
        "dtype": "float32",
        "crs": target["crs"],
        "transform": target["transform"],
        "nodata": NODATA,
        "compress": "deflate",
        "predictor": 3,
        "tiled": True,
        "blockxsize": 256,
        "blockysize": 256,
    }
    with rasterio.open(path, "w", **profile) as destination:
        destination.write(data, 1)
        destination.update_tags(
            source_scene_id=scene_id,
            source_asset_url=href,
            polarization=polarization,
            source_units="linear power",
            screening_only="true",
        )


def _write_mask(path, data, scene_id, target):
    profile = {
        "driver": "GTiff",
        "height": target["height"],
        "width": target["width"],
        "count": 1,
        "dtype": "uint8",
        "crs": target["crs"],
        "transform": target["transform"],
        "nodata": MASK_NODATA,
        "compress": "deflate",
        "tiled": True,
        "blockxsize": 256,
        "blockysize": 256,
    }
    with rasterio.open(path, "w", **profile) as destination:
        destination.write(data, 1)
        destination.update_tags(
            source_scene_id=scene_id,
            derivation="VV dB change relative to dry reference",
            threshold_db=THRESHOLD_DB,
            screening_only="true",
        )


def _stats(data, valid):
    values = data[valid]
    return {
        "valid_cells": int(valid.sum()),
        "valid_fraction": float(valid.mean()),
        "min": float(values.min()) if values.size else None,
        "max": float(values.max()) if values.size else None,
        "mean": float(values.mean()) if values.size else None,
    }


def _raster_stats(path):
    count = 0
    total = 0
    minimum = math.inf
    maximum = -math.inf
    sum_value = 0.0
    with rasterio.open(path) as source:
        for _, window in source.block_windows(1):
            data = source.read(1, window=window)
            valid = np.isfinite(data) & (data != NODATA) & (data > 0)
            values = data[valid]
            if values.size:
                count += int(values.size)
                total += int(data.size)
                minimum = min(minimum, float(values.min()))
                maximum = max(maximum, float(values.max()))
                sum_value += float(values.sum(dtype=np.float64))
            else:
                total += int(data.size)
    return {
        "valid_cells": count,
        "valid_fraction": float(count / total) if total else 0.0,
        "min": minimum if count else None,
        "max": maximum if count else None,
        "mean": sum_value / count if count else None,
    }


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    target = _target_profile()
    metadata = _scene_metadata()
    band_paths = {}
    receipt = {
        "schema_version": "0.1.0",
        "status": "screening_only",
        "catalog": "Planetary Computer sentinel-1-rtc",
        "target_crs": str(target["crs"]),
        "target_grid": {
            "width": target["width"],
            "height": target["height"],
            "transform": list(target["transform"]),
            "bounds": target["bounds"],
            "resolution_m": [10.0, 10.0],
        },
        "scenes": {},
        "derivation": {
            "polarization": "VV",
            "source_units": "linear power",
            "change_units": "dB",
            "formula": "10 * log10(observation_power) - 10 * log10(dry_reference_power)",
            "flood_screening_rule": f"delta_vv_db <= {THRESHOLD_DB}",
            "mask_nodata": MASK_NODATA,
        },
        "limitations": [
            "RTC pixels are an independent screening observation, not official ground truth.",
            "The 2016 event observation is acquired after the 2016-03-13 reported event date.",
            "The threshold is an explicit screening assumption and is not calibrated.",
            "AOI coverage differs by acquisition; masks only use pixels valid in both scenes.",
        ],
    }

    for role, scene_id in SCENES.items():
        feature = metadata[scene_id]
        receipt["scenes"][role] = {
            "id": scene_id,
            "datetime": feature["properties"]["datetime"],
            "relative_orbit": feature["properties"].get("sat:relative_orbit"),
            "orbit_state": feature["properties"].get("sat:orbit_state"),
            "source_crs": f"EPSG:{feature['properties'].get('proj:epsg')}",
            "assets": {},
        }
        for polarization in ("vv", "vh"):
            href = feature["assets"][polarization]["href"]
            output = OUTPUT / f"{role}_{polarization}.tif"
            if not output.exists():
                data = _read_band(href, target)
                _write_band(output, data, scene_id, polarization.upper(), href, target)
            with rasterio.open(output) as check:
                if check.crs != target["crs"] or check.width != target["width"] or check.height != target["height"]:
                    raise RuntimeError(f"existing output has incompatible grid: {output}")
            band_paths[(role, polarization)] = output
            receipt["scenes"][role]["assets"][polarization] = {
                "path": output.name,
                "source_asset_url": href,
                "stats_linear_power": _raster_stats(output),
                "sha256": _sha256(output),
            }

    with rasterio.open(band_paths[("dry_reference", "vv")]) as dry_source, rasterio.open(band_paths[("event", "vv")]) as event_source:
        for role, observed_source in (("event", event_source),):
            mask_path = OUTPUT / f"{role}_vv_flood_screening_mask.tif"
            _derive_mask(mask_path, dry_source, observed_source, SCENES[role], target)
    with rasterio.open(band_paths[("dry_reference", "vv")]) as dry_source, rasterio.open(band_paths[("holdout", "vv")]) as holdout_source:
        mask_path = OUTPUT / "holdout_vv_flood_screening_mask.tif"
        _derive_mask(mask_path, dry_source, holdout_source, SCENES["holdout"], target)
    for role in ("event", "holdout"):
        output = OUTPUT / f"{role}_vv_flood_screening_mask.tif"
        with rasterio.open(output) as source:
            values = source.read(1)
        valid = values != MASK_NODATA
        receipt.setdefault("masks", {})[role] = {
            "path": output.name,
            "valid_comparison_cells": int(valid.sum()),
            "flood_candidate_cells": int((values == 1).sum()),
            "flood_candidate_fraction_of_comparison": float((values == 1).sum() / valid.sum()) if valid.any() else None,
            "sha256": _sha256(output),
        }

    (OUTPUT / "sar_screening_receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


def _derive_mask(path, dry_source, observed_source, scene_id, target):
    profile = {
        "driver": "GTiff", "height": target["height"], "width": target["width"],
        "count": 1, "dtype": "uint8", "crs": target["crs"],
        "transform": target["transform"], "nodata": MASK_NODATA,
        "compress": "deflate", "tiled": True, "blockxsize": 256, "blockysize": 256,
    }
    with rasterio.open(path, "w", **profile) as destination:
        for _, window in observed_source.block_windows(1):
            dry = dry_source.read(1, window=window)
            observed = observed_source.read(1, window=window)
            valid = np.isfinite(dry) & np.isfinite(observed) & (dry != NODATA) & (observed != NODATA) & (dry > 0) & (observed > 0)
            mask = np.full(observed.shape, MASK_NODATA, dtype=np.uint8)
            delta = np.zeros(observed.shape, dtype=np.float32)
            delta[valid] = 10.0 * np.log10(observed[valid]) - 10.0 * np.log10(dry[valid])
            mask[valid] = (delta[valid] <= THRESHOLD_DB).astype(np.uint8)
            destination.write(mask, 1, window=window)
        destination.update_tags(
            source_scene_id=scene_id,
            derivation="VV dB change relative to dry reference",
            threshold_db=THRESHOLD_DB,
            screening_only="true",
        )


if __name__ == "__main__":
    main()
