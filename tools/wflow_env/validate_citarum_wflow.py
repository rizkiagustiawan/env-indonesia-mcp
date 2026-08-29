#!/usr/bin/env python3
"""Validate Citarum Wflow inputs and optionally write a hash-only receipt."""

import argparse
import hashlib
import json
import os
import sys
import tempfile
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


def _invalid_report(error: str) -> dict[str, object]:
    return {
        "status": "invalid",
        "screening_status": "screening_only",
        "errors": [error],
        "warnings": [],
    }


def _receipt_alias(receipt_path: Path, input_paths: dict[str, Path | None]) -> str | None:
    resolved_receipt = receipt_path.resolve()
    for name, input_path in input_paths.items():
        if input_path is None:
            continue
        if resolved_receipt == input_path.resolve():
            return name
        try:
            if receipt_path.exists() and input_path.exists() and os.path.samefile(
                receipt_path, input_path
            ):
                return name
        except OSError:
            continue
    return None


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
    temporary_path = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=receipt_path.parent,
            prefix=f".{receipt_path.name}.",
            suffix=".tmp",
            delete=False,
        ) as temporary:
            temporary_path = Path(temporary.name)
            temporary.write(json.dumps(receipt, indent=2) + "\n")
            temporary.flush()
            os.fsync(temporary.fileno())
        os.replace(temporary_path, receipt_path)
        temporary_path = None
    finally:
        if temporary_path is not None:
            try:
                temporary_path.unlink()
            except FileNotFoundError:
                pass


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forcing", required=True, type=Path)
    parser.add_argument("--staticmaps", type=Path)
    parser.add_argument("--outlet", type=Path)
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args(argv)

    input_paths = {
        "forcing": args.forcing,
        "staticmaps": args.staticmaps,
        "outlet": args.outlet,
    }
    alias = (
        _receipt_alias(args.receipt, input_paths)
        if args.receipt is not None
        else None
    )
    if alias is not None:
        report = _invalid_report(
            f"receipt path must not alias the {alias} input path"
        )
    else:
        report = _report(args.forcing, args.staticmaps, args.outlet)

    if report["status"] == "valid" and args.receipt is not None:
        try:
            _write_receipt(args.receipt, args.forcing, args.staticmaps, args.outlet)
        except OSError as exc:
            report["status"] = "invalid"
            report["errors"].append(f"unable to write receipt: {exc}")
    print(json.dumps(report, indent=2, allow_nan=False))
    return 0 if report["status"] == "valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
