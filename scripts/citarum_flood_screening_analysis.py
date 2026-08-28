"""Compare conditioned DEM scenarios with SAR screening masks.

This module deliberately does not run SWE without an observed discharge. The
terrain masks produced here are consistency diagnostics, not flood validation.
"""

import argparse
import hashlib
import json
import sys
from pathlib import Path

import numpy as np
import rasterio
from rasterio.warp import reproject, Resampling

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
NODATA = -9999.0
MASK_NODATA = 255


def binary_metrics(reference, predicted, valid=None):
    """Return confusion and overlap metrics for binary arrays."""
    reference = np.asarray(reference).astype(bool)
    predicted = np.asarray(predicted).astype(bool)
    if reference.shape != predicted.shape:
        raise ValueError("reference and predicted must have the same shape")
    valid = np.ones(reference.shape, dtype=bool) if valid is None else np.asarray(valid, dtype=bool)
    if valid.shape != reference.shape:
        raise ValueError("valid must have the same shape as reference")
    tp = int((valid & reference & predicted).sum())
    fp = int((valid & ~reference & predicted).sum())
    fn = int((valid & reference & ~predicted).sum())
    tn = int((valid & ~reference & ~predicted).sum())
    union = tp + fp + fn
    return {
        "valid_cells": int(valid.sum()),
        "tp": tp,
        "fp": fp,
        "fn": fn,
        "tn": tn,
        "iou": float(tp / union) if union else None,
        "pod": float(tp / (tp + fn)) if tp + fn else None,
        "far": float(fp / (tp + fp)) if tp + fp else None,
        "accuracy": float((tp + tn) / valid.sum()) if valid.any() else None,
    }


def scenario_stability(baseline, scenario, valid=None):
    """Summarize absolute elevation differences for two aligned DEMs."""
    baseline = np.asarray(baseline, dtype=float)
    scenario = np.asarray(scenario, dtype=float)
    if baseline.shape != scenario.shape:
        raise ValueError("baseline and scenario must have the same shape")
    valid = np.ones(baseline.shape, dtype=bool) if valid is None else np.asarray(valid, dtype=bool)
    if valid.shape != baseline.shape:
        raise ValueError("valid must have the same shape as baseline")
    delta = np.abs(scenario - baseline)
    values = delta[valid]
    return {
        "valid_cells": int(valid.sum()),
        "changed_cells": int((valid & (delta > 1e-6)).sum()),
        "changed_fraction": float((valid & (delta > 1e-6)).sum() / valid.sum()) if valid.any() else None,
        "mean_abs_delta_m": float(values.mean()) if values.size else 0.0,
        "max_abs_delta_m": float(values.max()) if values.size else 0.0,
        "p95_abs_delta_m": float(np.percentile(values, 95)) if values.size else 0.0,
    }


def _sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _read_reprojected_mask(mask_path, target):
    with rasterio.open(mask_path) as source:
        result = np.full((target.height, target.width), MASK_NODATA, dtype=np.uint8)
        reproject(
            source.read(1), result,
            src_transform=source.transform,
            src_crs=source.crs,
            src_nodata=source.nodata,
            dst_transform=target.transform,
            dst_crs=target.crs,
            dst_nodata=MASK_NODATA,
            resampling=Resampling.nearest,
        )
        return result


def _terrain_proxy(dem, valid, quantile=0.2):
    return terrain_quantile_mask(dem, valid, quantile)


def terrain_quantile_mask(dem, valid, quantile=0.2):
    """Classify the low-elevation quantile inside an explicit valid domain."""
    if not 0 < quantile < 1:
        raise ValueError("quantile must be between 0 and 1")
    valid = np.asarray(valid, dtype=bool)
    dem = np.asarray(dem, dtype=float)
    if dem.shape != valid.shape:
        raise ValueError("dem and valid must have the same shape")
    if not valid.any():
        raise ValueError("valid must contain at least one cell")
    threshold = float(np.quantile(dem[valid], quantile))
    return valid & (dem <= threshold), threshold


def analyze(dem_dir, sar_dir, output_path, depths=(0, 2, 5, 10)):
    """Run terrain-vs-SAR consistency analysis and write a receipt."""
    dem_dir = Path(dem_dir)
    sar_dir = Path(sar_dir)
    scenario_paths = {float(depth): dem_dir / f"dem_citarum_hulu_conditioned_burn{depth:g}m.tif" for depth in depths}
    baseline_path = scenario_paths[float(depths[0])]
    with rasterio.open(baseline_path) as baseline_source:
        baseline = baseline_source.read(1).astype(float)
        valid = np.isfinite(baseline) & (baseline != (baseline_source.nodata if baseline_source.nodata is not None else NODATA))
        target = baseline_source
        scenarios = []
        for depth, path in scenario_paths.items():
            with rasterio.open(path) as source:
                dem = source.read(1).astype(float)
                scenario_valid = valid & np.isfinite(dem) & (dem != source.nodata)
                low_mask, low_threshold = _terrain_proxy(dem, scenario_valid)
                scenarios.append({
                    "burn_depth_m": depth,
                    "path": path.name,
                    "terrain_low_quantile": 0.2,
                    "terrain_low_threshold_m": low_threshold,
                    "terrain_low_cells": int(low_mask.sum()),
                    "stability_vs_zero_m": scenario_stability(baseline, dem, scenario_valid),
                })

        sar_results = {}
        for role in ("event", "holdout"):
            sar_mask = _read_reprojected_mask(sar_dir / f"{role}_vv_flood_screening_mask.tif", target)
            sar_valid = valid & (sar_mask != MASK_NODATA)
            role_results = []
            for scenario in scenarios:
                depth = scenario["burn_depth_m"]
                with rasterio.open(scenario_paths[depth]) as scenario_source:
                    dem = scenario_source.read(1).astype(float)
                comparison_valid = sar_valid & np.isfinite(dem) & (dem != (target.nodata if target.nodata is not None else NODATA))
                terrain_mask, terrain_threshold = _terrain_proxy(dem, comparison_valid)
                role_results.append({
                    "burn_depth_m": depth,
                    "terrain_low_threshold_m_on_sar_intersection": terrain_threshold,
                    "terrain_low_cells_on_sar_intersection": int(terrain_mask.sum()),
                    "terrain_vs_sar_consistency": binary_metrics(sar_mask == 1, terrain_mask, sar_valid),
                })
            sar_results[role] = role_results

    receipt = {
        "schema_version": "0.1.0",
        "status": "screening_only",
        "swe": {
            "status": "not_run_missing_discharge",
            "missing": ["observed_or_declared_inflow_discharge_m3s"],
        },
        "method": "terrain_low_quantile_vs_sar_vv_change",
        "terrain_quantile": 0.2,
        "scenarios": scenarios,
        "sar_comparison": sar_results,
        "inputs": {
            "baseline": baseline_path.name,
            "sar_masks": ["event_vv_flood_screening_mask.tif", "holdout_vv_flood_screening_mask.tif"],
        },
        "limitations": [
            "Terrain-low masks are a topographic proxy, not hydraulic flood simulations.",
            "SAR-derived masks are independent screening observations, not official ground truth.",
            "SWE was intentionally not run because no observed or authorized discharge artifact exists.",
            "The -3 dB SAR threshold remains an uncalibrated screening assumption.",
        ],
    }
    output_path = Path(output_path)
    output_path.write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dem-dir", type=Path, required=True)
    parser.add_argument("--sar-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(analyze(args.dem_dir, args.sar_dir, args.output), indent=2))


if __name__ == "__main__":
    main()
