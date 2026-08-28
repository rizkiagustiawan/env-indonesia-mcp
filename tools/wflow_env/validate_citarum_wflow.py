#!/usr/bin/env python3
"""Validate Citarum Wflow inputs and optionally write a hash-only receipt."""

import argparse
import hashlib
import json
import sys
from pathlib import Path


if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from tools.wflow_env.citarum_outlet import validate_outlet
from tools.wflow_env.validate_wflow_forcing import validate_forcing


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _report(forcing_path: Path, staticmaps_path: Path | None, outlet_path: Path | None):
    forcing = validate_forcing(forcing_path, staticmaps_path)
    report = {
        "status": "invalid",
        "screening_status": "screening_only",
        "forcing": forcing,
    }
    reports = [forcing]

    if outlet_path is not None:
        grid_shape = forcing.get("summary", {}).get("grid_shape")
        outlet = validate_outlet(
            outlet_path,
            grid_shape=tuple(grid_shape) if grid_shape is not None else None,
        )
        report["outlet"] = outlet
        reports.append(outlet)

    report["errors"] = [
        error
        for validation_report in reports
        for error in validation_report.get("errors", [])
    ]
    report["warnings"] = [
        warning
        for validation_report in reports
        for warning in validation_report.get("warnings", [])
    ]
    if all(validation_report.get("status") == "valid" for validation_report in reports):
        report["status"] = "valid"
    return report


def _write_receipt(
    receipt_path: Path,
    forcing_path: Path,
    staticmaps_path: Path | None,
    outlet_path: Path | None,
) -> None:
    receipt = {
        "schema_version": "0.1.0",
        "status": "screening_only",
        "forcing_sha256": _sha256(forcing_path),
        "staticmaps_sha256": _sha256(staticmaps_path) if staticmaps_path else None,
        "outlet_sha256": _sha256(outlet_path) if outlet_path else None,
    }
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forcing", required=True, type=Path)
    parser.add_argument("--staticmaps", type=Path)
    parser.add_argument("--outlet", type=Path)
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args(argv)

    report = _report(args.forcing, args.staticmaps, args.outlet)
    if report["status"] == "valid" and args.receipt is not None:
        _write_receipt(args.receipt, args.forcing, args.staticmaps, args.outlet)
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
