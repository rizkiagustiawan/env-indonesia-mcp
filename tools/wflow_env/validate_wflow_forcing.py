"""Read-only validation for Wflow forcing NetCDF files."""

import numpy as np
import xarray as xr


_DIMENSIONS = ("time", "lat", "lon")
_UNITS = {"precip": "mm/day", "pet": "mm/day", "temp": "degC"}


def _base_report(forcing_path, staticmaps_path):
    return {
        "status": "invalid",
        "forcing_path": str(forcing_path),
        "staticmaps_path": str(staticmaps_path) if staticmaps_path else None,
        "summary": {},
        "errors": [],
        "warnings": [],
        "screening_status": "screening_only",
    }


def _coordinate_values(dataset, name, errors):
    if name not in dataset.coords:
        errors.append(f"missing required coordinate: {name}")
        return None
    values = np.asarray(dataset.coords[name].values)
    if values.size == 0:
        errors.append(f"coordinate {name} must not be empty")
        return None
    return values


def _read_active_mask(staticmaps_path, spatial_shape, errors):
    try:
        with xr.open_dataset(staticmaps_path, decode_times=True) as static:
            if "wflow_subcatch" not in static:
                errors.append("static maps missing required variable: wflow_subcatch")
                return None
            subcatch = static["wflow_subcatch"].load()
            values = np.asarray(subcatch.values)
            active = ~np.isnan(values)
            if tuple(subcatch.dims) == ("lon", "lat"):
                active = active.T
            if active.shape != spatial_shape:
                errors.append(
                    "static map active mask shape is incompatible with forcing "
                    f"grid: {list(active.shape)} != {list(spatial_shape)}"
                )
                return None
            return active
    except (FileNotFoundError, OSError, TypeError, ValueError, OverflowError) as exc:
        errors.append(f"unable to open or decode input dataset: {exc}")
        return None


def validate_forcing(forcing_path, staticmaps_path=None) -> dict[str, object]:
    """Validate the structural and value contract of a Wflow forcing file."""
    report = _base_report(forcing_path, staticmaps_path)
    errors = report["errors"]
    summary = report["summary"]

    try:
        with xr.open_dataset(forcing_path, decode_times=True) as forcing:
            coordinates = {
                name: _coordinate_values(forcing, name, errors)
                for name in ("time", "lat", "lon")
            }

            time_values = coordinates["time"]
            lat_values = coordinates["lat"]
            lon_values = coordinates["lon"]
            if time_values is not None:
                try:
                    days = time_values.astype("datetime64[D]")
                    period = [
                        np.datetime_as_string(value, unit="D") for value in days
                    ]
                    summary["period"] = period
                    summary["time_count"] = int(days.size)
                    if np.any(np.isnat(days)):
                        errors.append("time coordinates must not contain NaT")
                    elif days.size > 1:
                        deltas = np.diff(days)
                        if np.any(deltas != np.timedelta64(1, "D")):
                            errors.append(
                                "time coordinates must be daily without gaps or "
                                f"duplicates; found deltas {deltas.astype(int).tolist()}"
                            )
                except (TypeError, ValueError, OverflowError) as exc:
                    errors.append(f"time coordinate is not decodable as dates: {exc}")

            if lat_values is not None:
                summary["lat_range"] = [
                    float(np.min(lat_values)),
                    float(np.max(lat_values)),
                ]
            if lon_values is not None:
                summary["lon_range"] = [
                    float(np.min(lon_values)),
                    float(np.max(lon_values)),
                ]

            for name, expected_units in _UNITS.items():
                if name not in forcing:
                    errors.append(f"missing required variable: {name}")
                    continue
                variable = forcing[name]
                if tuple(variable.dims) != _DIMENSIONS:
                    errors.append(
                        f"{name} dimensions must be exactly {_DIMENSIONS}; "
                        f"got {tuple(variable.dims)}"
                    )
                if variable.attrs.get("units") != expected_units:
                    errors.append(
                        f"{name} units must be {expected_units}; "
                        f"got {variable.attrs.get('units')!r}"
                    )

            if all(value is not None for value in (time_values, lat_values, lon_values)):
                spatial_shape = (lat_values.size, lon_values.size)
                summary["grid_shape"] = [int(size) for size in spatial_shape]
                active = None
                if staticmaps_path:
                    active = _read_active_mask(staticmaps_path, spatial_shape, errors)

                for name in _UNITS:
                    if name not in forcing:
                        continue
                    variable = forcing[name]
                    try:
                        values = np.asarray(variable.load().values)
                        expected_shape = (
                            time_values.size,
                            lat_values.size,
                            lon_values.size,
                        )
                        if values.shape != expected_shape:
                            errors.append(
                                f"{name} shape must match (time, lat, lon) coordinates; "
                                f"got {list(values.shape)}"
                            )
                            continue
                        checked_values = values if active is None else values[:, active]
                        if not np.all(np.isfinite(checked_values)):
                            errors.append(
                                f"{name} contains non-finite values in active cells"
                            )
                        if name in ("precip", "pet") and np.any(checked_values < 0):
                            errors.append(f"{name} contains negative values")
                    except (TypeError, ValueError) as exc:
                        errors.append(f"{name} values are not numeric: {exc}")
    except (FileNotFoundError, OSError, TypeError, ValueError, OverflowError) as exc:
        errors.append(f"unable to open or decode input dataset: {exc}")

    if not errors:
        report["status"] = "valid"
    return report
