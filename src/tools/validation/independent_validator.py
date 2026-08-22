"""Independent hard checks for scientific result envelopes.

This validator checks claims against execution evidence and never upgrades a
screening result to a validated result.
"""

import math


def validate_result(payload):
    """Validate required evidence, geospatial metadata, and physical balance."""
    errors = []

    if not isinstance(payload, dict):
        return {
            "validation_status": "reject",
            "result_status": None,
            "errors": ["Result must be a JSON object"],
        }

    result_status = payload.get("status")
    provenance = payload.get("provenance")
    if not isinstance(provenance, dict) or not provenance:
        errors.append("Provenance is required")
    else:
        for field in ("source_ids", "input_hash", "execution_id"):
            if not provenance.get(field):
                errors.append(f"Provenance field is missing: {field}")

    uncertainty = payload.get("uncertainty")
    if not isinstance(uncertainty, dict):
        errors.append("Uncertainty bounds are required")
    else:
        lower = uncertainty.get("lower")
        upper = uncertainty.get("upper")
        if not _finite_number(lower) or not _finite_number(upper):
            errors.append("Uncertainty bounds must be finite numbers")
        elif lower > upper:
            errors.append("Uncertainty lower bound exceeds upper bound")

    geospatial = payload.get("geospatial")
    if not isinstance(geospatial, dict):
        errors.append("Geospatial metadata is required")
    else:
        if not geospatial.get("crs"):
            errors.append("CRS is required")
        bbox = geospatial.get("bbox")
        if not _valid_bbox(bbox):
            errors.append("BBox must be [west, south, east, north] with east/north greater than west/south")
        resolution = geospatial.get("resolution_m")
        if not _finite_number(resolution) or resolution <= 0:
            errors.append("Resolution must be a positive finite number")

    mass_balance = payload.get("mass_balance")
    if not isinstance(mass_balance, dict):
        errors.append("Mass balance is required")
    else:
        input_volume = mass_balance.get("input_volume_m3")
        output_volume = mass_balance.get("output_volume_m3")
        tolerance = mass_balance.get("tolerance_fraction")
        if not all(_finite_number(value) for value in (input_volume, output_volume, tolerance)):
            errors.append("Mass balance values must be finite numbers")
        elif input_volume < 0 or output_volume < 0 or tolerance < 0:
            errors.append("Mass balance values must be non-negative")
        elif abs(input_volume - output_volume) > max(abs(input_volume), 1.0) * tolerance:
            errors.append("Mass balance exceeds tolerance")

    receipt = payload.get("execution_receipt")
    reported_values = receipt.get("reported_values") if isinstance(receipt, dict) else None
    if not isinstance(reported_values, list):
        errors.append("Execution receipt with reported values is required")
    else:
        for claim in payload.get("claims", []):
            if not isinstance(claim, dict) or "value" not in claim:
                errors.append("Claim must contain a value")
                continue
            if not any(_same_value(claim["value"], value) for value in reported_values):
                errors.append("Claim value is not present in execution receipt")

    return {
        "validation_status": "pass" if not errors else "reject",
        "result_status": result_status,
        "errors": errors,
    }


def _finite_number(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def _valid_bbox(bbox):
    if not isinstance(bbox, (list, tuple)) or len(bbox) != 4:
        return False
    if not all(_finite_number(value) for value in bbox):
        return False
    west, south, east, north = bbox
    return -180 <= west < east <= 180 and -90 <= south < north <= 90


def _same_value(left, right):
    if _finite_number(left) and _finite_number(right):
        return math.isclose(left, right, rel_tol=1e-12, abs_tol=1e-12)
    return left == right
