#!/usr/bin/env python3
"""Write a provenance receipt for a completed Citarum Wflow run."""

import csv
import hashlib
import json
import re
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODEL = ROOT / "data/benchmarks/citarum_hulu/wflow"
OUTPUT = MODEL / "output.csv"
LOG = MODEL / "log.txt"
BUILD = MODEL / "build_receipt.json"
RECEIPT = MODEL / "run_receipt.json"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    build = json.loads(BUILD.read_text())
    with OUTPUT.open(newline="") as stream:
        rows = list(csv.DictReader(stream))
    discharge = [float(row["Q"]) for row in rows]

    log_text = LOG.read_text()
    duration = re.search(r"Simulation duration: (.+)", log_text)
    receipt = {
        "schema_version": "0.1.0",
        "status": "screening_only",
        "runtime": {
            "engine": "Wflow.jl",
            "wflow_version": "1.0.4",
            "julia_version": "1.12.7",
            "simulation_duration": duration.group(1).strip() if duration else None,
        },
        "inputs": {
            "build_receipt": str(BUILD),
            "build_receipt_sha256": sha256(BUILD),
            "staticmaps": build["outputs"]["staticmaps"],
            "forcing": build["outputs"]["forcing"],
            "config": build["outputs"]["config"],
        },
        "simulation": {
            "model": "sbm",
            "start": rows[0]["time"] if rows else None,
            "end": rows[-1]["time"] if rows else None,
            "output_rows": len(rows),
            "precipitation_range_mm_per_day": build["parameters"]["precip_range_mm_per_day"],
            "discharge_parameter": "river_water__volume_flow_rate",
            "discharge_unit": "m3/s",
            "discharge_reducer": "maximum",
            "discharge_min_m3_per_s": min(discharge) if discharge else None,
            "discharge_max_m3_per_s": max(discharge) if discharge else None,
            "discharge_last_m3_per_s": discharge[-1] if discharge else None,
        },
        "outputs": {
            "hydrograph_csv": str(OUTPUT),
            "log": str(LOG),
            "hydrograph_sha256": sha256(OUTPUT),
            "log_sha256": sha256(LOG),
        },
        "limitations": [
            "The run is a technical and screening execution, not calibration or validation.",
            "Rainfall is spatially uniform from one Open-Meteo source record.",
            "Soil and landcover parameters are literature defaults.",
            "PET and temperature are approximations.",
            "The CSV records the maximum active-cell river discharge, not an independently observed gauge discharge.",
            "The seven-day forcing window is not sufficient for event calibration or model skill assessment.",
        ],
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
    }
    RECEIPT.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
