"""Run declared stream-burn-depth sensitivity scenarios for Citarum Hulu."""

import argparse
import hashlib
import json
import sys
from pathlib import Path

import numpy as np
import rasterio

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from src.tools.gis.dem_conditioning import count_interior_pits, condition_dem


DEFAULT_DEPTHS = (0.0, 2.0, 5.0, 10.0)
NODATA = -9999.0


def _valid_mask(dem, nodata):
    return np.isfinite(dem) & (dem != nodata)


def _stats(values, valid):
    sample = values[valid]
    return {
        "min_m": float(sample.min()),
        "max_m": float(sample.max()),
        "mean_m": float(sample.mean()),
        "std_m": float(sample.std()),
    }


def run_sensitivity(dem, stream_mask, burn_depths=DEFAULT_DEPTHS, connectivity=8, valid_mask=None):
    """Return deterministic conditioning diagnostics for each burn depth."""
    depths = tuple(float(depth) for depth in burn_depths)
    if not depths:
        raise ValueError("burn_depths must contain at least one value")
    if any(not np.isfinite(depth) or depth < 0 for depth in depths):
        raise ValueError("burn_depths must contain finite non-negative values")
    elevation = np.asarray(dem, dtype=float)
    mask = np.asarray(stream_mask)
    if elevation.shape != mask.shape:
        raise ValueError("stream_mask must have the same shape as DEM")
    valid = np.ones(elevation.shape, dtype=bool) if valid_mask is None else np.asarray(valid_mask, dtype=bool)
    if valid.shape != elevation.shape:
        raise ValueError("valid_mask must have the same shape as DEM")
    records = []
    for depth in depths:
        burned = np.where(mask == 1, elevation - depth, elevation)
        conditioned = condition_dem(elevation, mask, depth, valid, connectivity)
        records.append({
            "burn_depth_m": depth,
            "burned_cells": int(((mask == 1) & valid).sum()),
            "conditioned_pit_count": count_interior_pits(conditioned, valid, connectivity),
            "conditioned": _stats(conditioned, valid),
            "conditioned_min_m": float(conditioned[valid].min()),
            "burn_delta_min_m": float((conditioned[valid] - burned[valid]).min()),
            "burn_delta_max_m": float((conditioned[valid] - burned[valid]).max()),
        })
    return records


def _sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _run_files(input_path, mask_path, output_dir, depths):
    output_dir.mkdir(parents=True, exist_ok=True)
    with rasterio.open(input_path) as dem_source, rasterio.open(mask_path) as mask_source:
        dem = dem_source.read(1).astype(float)
        stream_mask = mask_source.read(1)
        valid = _valid_mask(dem, dem_source.nodata if dem_source.nodata is not None else NODATA)
        records = run_sensitivity(dem, stream_mask, depths, connectivity=8, valid_mask=valid)
        profile = dem_source.profile.copy()
        profile.update(dtype="float32", count=1, compress="deflate", predictor=3, tiled=True, blockxsize=256, blockysize=256, nodata=dem_source.nodata)
        for record in records:
            depth = record["burn_depth_m"]
            output = output_dir / f"dem_citarum_hulu_conditioned_burn{depth:g}m.tif"
            conditioned = condition_dem(dem, stream_mask, depth, valid, 8).astype(np.float32)
            with rasterio.open(output, "w", **profile) as destination:
                destination.write(conditioned, 1)
                destination.update_tags(burn_depth_m=depth, connectivity=8, screening_only="true")
            record["path"] = output.name
            record["sha256"] = _sha256(output)
    receipt = {
        "schema_version": "0.1.0",
        "status": "screening_only",
        "method": "burn_before_priority_flood_fill",
        "connectivity": 8,
        "input": input_path.name,
        "stream_mask": mask_path.name,
        "burn_depths_m": list(depths),
        "scenarios": records,
        "limitations": [
            "Burn depth is a conditioning assumption, not a surveyed channel bed.",
            "GLO-30 is a DSM and can bias urban routing and depth interpretation.",
            "Sensitivity does not constitute hydraulic calibration or validation.",
        ],
    }
    (output_dir / "conditioning_sensitivity_receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dem", type=Path, required=True)
    parser.add_argument("--stream-mask", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--depths", nargs="+", type=float, default=DEFAULT_DEPTHS)
    args = parser.parse_args()
    print(json.dumps(_run_files(args.dem, args.stream_mask, args.output_dir, args.depths), indent=2))


if __name__ == "__main__":
    main()
