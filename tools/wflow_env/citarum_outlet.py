"""Read-only validation for a Citarum Wflow outlet metadata record."""

import json
import math


_REQUIRED_FIELDS = (
    "schema_version",
    "status",
    "outlet_id",
    "name",
    "description",
    "longitude",
    "latitude",
    "grid_row",
    "grid_col",
    "source",
    "discharge_variable",
    "extraction_rule",
    "validation_state",
    "limitations",
)
_VALIDATION_STATES = {"provisional", "resolved"}
_STRING_FIELDS = (
    "schema_version",
    "outlet_id",
    "name",
    "description",
    "source",
    "discharge_variable",
    "extraction_rule",
)


def _report(path):
    return {
        "status": "invalid",
        "outlet_path": str(path),
        "normalized": {},
        "errors": [],
        "warnings": [],
    }


def _finite_coordinate(value, name, minimum, maximum, errors):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        errors.append(f"{name} must be a finite number")
        return
    if not math.isfinite(value) or not minimum <= value <= maximum:
        errors.append(f"{name} must be finite and between {minimum} and {maximum}")


def _grid_index(value, name, limit, errors):
    if isinstance(value, bool) or not isinstance(value, int):
        errors.append(f"{name} must be an integer")
        return
    if limit is None:
        return
    if value < 0 or value >= limit:
        errors.append(f"{name} must fit grid bounds 0 <= {name} < {limit}")


def _validate_required_fields(payload, errors):
    for field in _STRING_FIELDS:
        value = payload.get(field)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"{field} must be a non-empty string")

    limitations = payload.get("limitations")
    if (
        not isinstance(limitations, list)
        or not limitations
        or any(not isinstance(value, str) or not value.strip() for value in limitations)
    ):
        errors.append("limitations must be a non-empty list of non-empty strings")


def validate_outlet(outlet_path, grid_shape=None) -> dict[str, object]:
    """Validate outlet metadata without modifying the source JSON file."""
    report = _report(outlet_path)
    errors = report["errors"]
    warnings = report["warnings"]

    try:
        with open(outlet_path, encoding="utf-8") as handle:
            payload = json.load(handle)
    except (FileNotFoundError, OSError, json.JSONDecodeError) as exc:
        errors.append(f"unable to read outlet metadata: {exc}")
        return report

    if not isinstance(payload, dict):
        errors.append("outlet metadata must be a JSON object")
        return report

    report["normalized"] = dict(payload)
    for field in _REQUIRED_FIELDS:
        if field not in payload:
            errors.append(f"missing required field: {field}")
    _validate_required_fields(payload, errors)

    if payload.get("status") != "screening_only":
        errors.append("status must be screening_only")

    if "longitude" in payload:
        _finite_coordinate(payload["longitude"], "longitude", -180, 180, errors)
    if "latitude" in payload:
        _finite_coordinate(payload["latitude"], "latitude", -90, 90, errors)

    validation_state = payload.get("validation_state")
    if validation_state not in _VALIDATION_STATES:
        errors.append("validation_state must be provisional or resolved")

    row = payload.get("grid_row")
    col = payload.get("grid_col")
    if validation_state == "provisional" and (row is None or col is None):
        warnings.append("grid_row and grid_col are unresolved for this provisional outlet")
    elif row is None or col is None:
        errors.append("grid_row and grid_col may be null only for provisional metadata")

    if row is not None:
        _grid_index(row, "grid_row", None, errors)
    if col is not None:
        _grid_index(col, "grid_col", None, errors)

    if grid_shape is not None:
        if (
            not isinstance(grid_shape, (tuple, list))
            or len(grid_shape) != 2
            or any(isinstance(size, bool) or not isinstance(size, int) or size <= 0 for size in grid_shape)
        ):
            errors.append("grid_shape must contain two positive integers")
        elif row is not None and col is not None:
            if isinstance(row, int) and not isinstance(row, bool):
                if row < 0 or row >= grid_shape[0]:
                    errors.append(f"grid_row must fit grid bounds 0 <= grid_row < {grid_shape[0]}")
            if isinstance(col, int) and not isinstance(col, bool):
                if col < 0 or col >= grid_shape[1]:
                    errors.append(f"grid_col must fit grid bounds 0 <= grid_col < {grid_shape[1]}")

    if not errors:
        report["status"] = "valid"
    return report
