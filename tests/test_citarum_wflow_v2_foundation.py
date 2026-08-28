import hashlib
import json
from pathlib import Path
import subprocess
import sys

import numpy as np
import pytest
import xarray as xr

from tools.wflow_env.validate_wflow_forcing import validate_forcing
from tools.wflow_env.citarum_outlet import validate_outlet


ROOT = Path(__file__).resolve().parents[1]


def _write_forcing(path, *, times=None, include_pet=True, dims=("time", "lat", "lon"), units=None):
    times = times or ["2016-03-10", "2016-03-11"]
    shape = (len(times), 2, 2)
    time_values = np.array(times, dtype="datetime64[D]")
    time_encoding = {"units": "days since 2000-01-01 00:00:00", "calendar": "standard"}
    if times == ["NaT"]:
        time_values = np.array([-9999.0])
        time_encoding["_FillValue"] = -9999.0
    data = {
        "precip": (dims, np.ones(shape, dtype=np.float32)),
        "temp": (dims, np.full(shape, 25.0, dtype=np.float32)),
    }
    if include_pet:
        data["pet"] = (dims, np.full(shape, 3.0, dtype=np.float32))
    ds = xr.Dataset(
        data,
        coords={
            "time": time_values,
            "lat": [-7.0, -6.995],
            "lon": [107.5, 107.505],
        },
    )
    ds["precip"].attrs["units"] = (units or {}).get("precip", "mm/day")
    if include_pet:
        ds["pet"].attrs["units"] = (units or {}).get("pet", "mm/day")
    ds["temp"].attrs["units"] = (units or {}).get("temp", "degC")
    ds.time.encoding.update(time_encoding)
    ds.to_netcdf(path)


def _write_staticmaps(path, *, shape=(2, 2), active=None):
    active = np.ones(shape, dtype=bool) if active is None else np.asarray(active)
    subcatch = np.where(active, 1.0, -9999.0).astype(np.float32)
    ds = xr.Dataset(
        {"wflow_subcatch": (("lat", "lon"), subcatch)},
        coords={
            "lat": np.arange(shape[0], dtype=float),
            "lon": np.arange(shape[1], dtype=float),
        },
    )
    ds["wflow_subcatch"].encoding["_FillValue"] = -9999.0
    ds.to_netcdf(path)


def _sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_valid_forcing_report_is_valid(tmp_path):
    forcing = tmp_path / "forcing.nc"
    _write_forcing(forcing)

    report = validate_forcing(forcing)

    assert report["status"] == "valid"
    assert report["summary"]["period"] == ["2016-03-10", "2016-03-11"]
    assert report["screening_status"] == "screening_only"


def test_missing_required_variable_is_invalid(tmp_path):
    forcing = tmp_path / "missing-pet.nc"
    _write_forcing(forcing, include_pet=False)
    report = validate_forcing(forcing)
    assert report["status"] == "invalid"
    assert any("pet" in error for error in report["errors"])


def test_wrong_dimensions_are_invalid(tmp_path):
    forcing = tmp_path / "wrong-dims.nc"
    _write_forcing(forcing, dims=("lat", "time", "lon"))
    report = validate_forcing(forcing)
    assert report["status"] == "invalid"
    assert any("dimensions" in error for error in report["errors"])


def test_time_gap_is_invalid(tmp_path):
    forcing = tmp_path / "time-gap.nc"
    _write_forcing(forcing, times=["2016-03-10", "2016-03-12"])
    report = validate_forcing(forcing)
    assert report["status"] == "invalid"
    assert any("daily" in error or "gap" in error for error in report["errors"])


def test_wrong_units_are_invalid(tmp_path):
    forcing = tmp_path / "wrong-units.nc"
    _write_forcing(forcing, units={"precip": "m", "pet": "m/day", "temp": "K"})
    report = validate_forcing(forcing)
    assert report["status"] == "invalid"
    assert len(report["errors"]) >= 3


def test_negative_precipitation_or_pet_is_invalid(tmp_path):
    forcing = tmp_path / "negative.nc"
    _write_forcing(forcing)
    with xr.open_dataset(forcing) as ds:
        updated = ds.load()
    updated["precip"].values[0, 0, 0] = -1.0
    updated["pet"].values[0, 0, 1] = -1.0
    updated.to_netcdf(forcing, mode="w")
    report = validate_forcing(forcing)
    assert report["status"] == "invalid"
    assert any("negative" in error for error in report["errors"])


def test_nonfinite_active_cell_is_invalid(tmp_path):
    forcing = tmp_path / "nan.nc"
    _write_forcing(forcing)
    with xr.open_dataset(forcing) as ds:
        updated = ds.load()
    updated["precip"].values[0, 0, 0] = np.nan
    updated.to_netcdf(forcing, mode="w")
    report = validate_forcing(forcing)
    assert report["status"] == "invalid"
    assert any("finite" in error for error in report["errors"])


def test_single_nat_timestamp_is_invalid(tmp_path):
    forcing = tmp_path / "nat.nc"
    _write_forcing(forcing, times=["NaT"])

    report = validate_forcing(forcing)

    assert report["status"] == "invalid"
    assert any("NaT" in error or "time" in error for error in report["errors"])


def test_staticmap_lat_lon_layout_ignores_inactive_fill_values(tmp_path):
    forcing = tmp_path / "forcing-with-inactive-fill.nc"
    staticmaps = tmp_path / "staticmaps.nc"
    _write_forcing(forcing)
    _write_staticmaps(staticmaps, active=[[True, False], [True, True]])
    with xr.open_dataset(forcing) as ds:
        updated = ds.load()
    for name in ("precip", "pet", "temp"):
        updated[name].values[:, 0, 1] = -9999.0
    updated.to_netcdf(forcing, mode="w")

    report = validate_forcing(forcing, staticmaps)

    assert report["status"] == "valid"


def test_staticmap_active_nonfinite_value_is_invalid(tmp_path):
    forcing = tmp_path / "forcing-with-active-nan.nc"
    staticmaps = tmp_path / "staticmaps.nc"
    _write_forcing(forcing)
    _write_staticmaps(staticmaps, active=[[True, False], [True, True]])
    with xr.open_dataset(forcing) as ds:
        updated = ds.load()
    updated["precip"].values[0, 0, 0] = np.nan
    updated.to_netcdf(forcing, mode="w")

    report = validate_forcing(forcing, staticmaps)

    assert report["status"] == "invalid"
    assert any("finite" in error for error in report["errors"])


def test_staticmap_incompatible_shape_is_invalid(tmp_path):
    forcing = tmp_path / "forcing.nc"
    staticmaps = tmp_path / "wrong-shape-staticmaps.nc"
    _write_forcing(forcing)
    _write_staticmaps(staticmaps, shape=(3, 2))

    report = validate_forcing(forcing, staticmaps)

    assert report["status"] == "invalid"
    assert any("shape" in error for error in report["errors"])


def test_validation_does_not_modify_forcing_or_staticmaps(tmp_path):
    forcing = tmp_path / "forcing.nc"
    staticmaps = tmp_path / "staticmaps.nc"
    _write_forcing(forcing)
    _write_staticmaps(staticmaps, active=[[True, False], [True, True]])
    before = {_path: _sha256(_path) for _path in (forcing, staticmaps)}

    report = validate_forcing(forcing, staticmaps)

    assert report["status"] == "valid"
    assert {_path: _sha256(_path) for _path in (forcing, staticmaps)} == before


def test_provisional_outlet_allows_null_grid_indices(tmp_path):
    outlet = tmp_path / "outlet.json"
    outlet.write_text(json.dumps({
        "schema_version": "1.0.0",
        "status": "screening_only",
        "outlet_id": "citarum_hulu_provisional",
        "name": "Citarum Hulu provisional outlet",
        "description": "Provisional extraction target pending gauge confirmation.",
        "longitude": 107.62025,
        "latitude": -6.994727,
        "grid_row": None,
        "grid_col": None,
        "source": "existing Citarum benchmark AOI target; not a gauge record",
        "discharge_variable": "Q",
        "extraction_rule": "explicit outlet cell when grid indices are resolved",
        "validation_state": "provisional",
        "limitations": ["No independent discharge validation."]
    }))
    report = validate_outlet(outlet)
    assert report["status"] == "valid"
    assert report["normalized"]["validation_state"] == "provisional"
    assert any("grid" in warning for warning in report["warnings"])


def test_outlet_missing_identity_is_invalid(tmp_path):
    outlet = tmp_path / "bad-outlet.json"
    outlet.write_text(json.dumps({"status": "screening_only"}))
    report = validate_outlet(outlet)
    assert report["status"] == "invalid"
    assert any("outlet_id" in error for error in report["errors"])


def test_outlet_grid_indices_must_fit_grid(tmp_path):
    outlet = tmp_path / "out-of-range.json"
    payload = json.loads((ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json").read_text())
    payload.update({"validation_state": "resolved", "grid_row": 106, "grid_col": 139})
    outlet.write_text(json.dumps(payload))
    report = validate_outlet(outlet, grid_shape=(106, 139))
    assert report["status"] == "invalid"
    assert any("grid_row" in error or "grid_col" in error for error in report["errors"])


def test_provisional_partial_indices_reject_invalid_supplied_index(tmp_path):
    outlet = tmp_path / "partial-outlet.json"
    payload = json.loads((ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json").read_text())
    payload.update({"grid_row": "not-an-index", "grid_col": None})
    outlet.write_text(json.dumps(payload))

    report = validate_outlet(outlet)

    assert report["status"] == "invalid"
    assert any("grid_row" in error and "integer" in error for error in report["errors"])


def test_resolved_non_integer_indices_are_invalid_without_grid_shape(tmp_path):
    outlet = tmp_path / "non-integer-outlet.json"
    payload = json.loads((ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json").read_text())
    payload.update({"validation_state": "resolved", "grid_row": 1.5, "grid_col": 2})
    outlet.write_text(json.dumps(payload))

    report = validate_outlet(outlet)

    assert report["status"] == "invalid"
    assert any("grid_row" in error and "integer" in error for error in report["errors"])


def test_malformed_grid_shape_returns_invalid_report(tmp_path):
    outlet = tmp_path / "outlet.json"
    payload = json.loads((ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json").read_text())
    outlet.write_text(json.dumps(payload))

    report = validate_outlet(outlet, grid_shape=106)

    assert report["status"] == "invalid"
    assert any("grid_shape" in error for error in report["errors"])


def test_malformed_required_field_types_and_content_are_invalid(tmp_path):
    outlet = tmp_path / "malformed-outlet.json"
    payload = json.loads((ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json").read_text())
    payload.update({
        "schema_version": 1,
        "outlet_id": "",
        "name": 42,
        "description": "",
        "source": ["benchmark"],
        "discharge_variable": "",
        "extraction_rule": None,
        "limitations": "none",
    })
    outlet.write_text(json.dumps(payload))

    report = validate_outlet(outlet)

    assert report["status"] == "invalid"
    for field in ("schema_version", "outlet_id", "name", "description", "source",
                  "discharge_variable", "extraction_rule", "limitations"):
        assert any(field in error for error in report["errors"])


def test_outlet_validation_does_not_modify_metadata():
    outlet = ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json"
    before = _sha256(outlet)

    report = validate_outlet(outlet)

    assert report["status"] == "valid"
    assert _sha256(outlet) == before


def test_list_validation_state_returns_invalid_report(tmp_path):
    outlet = tmp_path / "list-state-outlet.json"
    payload = json.loads((ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json").read_text())
    payload["validation_state"] = ["provisional"]
    outlet.write_text(json.dumps(payload))

    report = validate_outlet(outlet)

    assert report["status"] == "invalid"
    assert any("validation_state" in error for error in report["errors"])


def test_extremely_large_integer_coordinates_return_invalid_report(tmp_path):
    for coordinate in ("longitude", "latitude"):
        outlet = tmp_path / f"large-{coordinate}.json"
        payload = json.loads((ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json").read_text())
        payload[coordinate] = 10**400
        outlet.write_text(json.dumps(payload))

        report = validate_outlet(outlet)

        assert report["status"] == "invalid"
        assert any(coordinate in error for error in report["errors"])


def test_cli_returns_json_success_and_writes_receipt(tmp_path):
    forcing = tmp_path / "forcing.nc"
    _write_forcing(forcing)
    outlet = ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json"
    receipt = tmp_path / "receipt.json"

    result = subprocess.run(
        [
            sys.executable,
            "tools/wflow_env/validate_citarum_wflow.py",
            "--forcing",
            str(forcing),
            "--outlet",
            str(outlet),
            "--receipt",
            str(receipt),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0
    report = json.loads(result.stdout)
    assert report["status"] == "valid"
    assert report["screening_status"] == "screening_only"
    receipt_data = json.loads(receipt.read_text())
    assert receipt_data["forcing_sha256"] == _sha256(forcing)
    assert receipt_data["outlet_sha256"] == _sha256(outlet)
    assert receipt_data["staticmaps_sha256"] is None
    assert "discharge" not in json.dumps(receipt_data).lower()


def test_cli_returns_nonzero_for_invalid_forcing_without_receipt(tmp_path):
    forcing = tmp_path / "invalid.nc"
    _write_forcing(forcing, times=["2016-03-10", "2016-03-12"])
    receipt = tmp_path / "receipt.json"

    result = subprocess.run(
        [
            sys.executable,
            "tools/wflow_env/validate_citarum_wflow.py",
            "--forcing",
            str(forcing),
            "--receipt",
            str(receipt),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    assert json.loads(result.stdout)["status"] == "invalid"
    assert not receipt.exists()


@pytest.mark.parametrize("alias", ("forcing", "staticmaps", "outlet"))
def test_cli_rejects_receipt_path_aliasing_input_without_modifying_it(tmp_path, alias):
    forcing = tmp_path / "forcing.nc"
    staticmaps = tmp_path / "staticmaps.nc"
    outlet = ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json"
    _write_forcing(forcing)
    _write_staticmaps(staticmaps)
    inputs = {"forcing": forcing, "staticmaps": staticmaps, "outlet": outlet}
    before = {name: path.read_bytes() for name, path in inputs.items()}

    result = subprocess.run(
        [
            sys.executable,
            "tools/wflow_env/validate_citarum_wflow.py",
            "--forcing",
            str(forcing),
            "--staticmaps",
            str(staticmaps),
            "--outlet",
            str(outlet),
            "--receipt",
            str(inputs[alias]),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    assert json.loads(result.stdout)["status"] == "invalid"
    assert result.stderr == ""
    assert {name: path.read_bytes() for name, path in inputs.items()} == before


def test_cli_rejects_nan_outlet_without_nonstandard_json_stdout(tmp_path):
    forcing = tmp_path / "forcing.nc"
    outlet = tmp_path / "nan-outlet.json"
    _write_forcing(forcing)
    payload = json.loads(
        (ROOT / "data/benchmarks/citarum_hulu/wflow/citarum_hulu_outlet.json").read_text()
    )
    payload["longitude"] = float("nan")
    outlet.write_text(json.dumps(payload))

    result = subprocess.run(
        [
            sys.executable,
            "tools/wflow_env/validate_citarum_wflow.py",
            "--forcing",
            str(forcing),
            "--outlet",
            str(outlet),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    report = json.loads(
        result.stdout,
        parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)),
    )
    assert report["status"] == "invalid"
    assert any("outlet" in error.lower() for error in report["errors"])
    assert result.stderr == ""


def test_cli_reports_receipt_write_failure_as_json_without_traceback(tmp_path):
    forcing = tmp_path / "forcing.nc"
    receipt = tmp_path / "missing-parent" / "receipt.json"
    _write_forcing(forcing)

    result = subprocess.run(
        [
            sys.executable,
            "tools/wflow_env/validate_citarum_wflow.py",
            "--forcing",
            str(forcing),
            "--receipt",
            str(receipt),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    report = json.loads(result.stdout)
    assert report["status"] == "invalid"
    assert any("receipt" in error.lower() for error in report["errors"])
    assert result.stderr == ""
    assert not receipt.exists()
