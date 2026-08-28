import numpy as np
import xarray as xr

from tools.wflow_env.validate_wflow_forcing import validate_forcing


def _write_forcing(path, *, times=None, include_pet=True, dims=("time", "lat", "lon"), units=None):
    times = times or ["2016-03-10", "2016-03-11"]
    shape = (len(times), 2, 2)
    data = {
        "precip": (dims, np.ones(shape, dtype=np.float32)),
        "temp": (dims, np.full(shape, 25.0, dtype=np.float32)),
    }
    if include_pet:
        data["pet"] = (dims, np.full(shape, 3.0, dtype=np.float32))
    ds = xr.Dataset(
        data,
        coords={
            "time": np.array(times, dtype="datetime64[D]"),
            "lat": [-7.0, -6.995],
            "lon": [107.5, 107.505],
        },
    )
    ds["precip"].attrs["units"] = (units or {}).get("precip", "mm/day")
    if include_pet:
        ds["pet"].attrs["units"] = (units or {}).get("pet", "mm/day")
    ds["temp"].attrs["units"] = (units or {}).get("temp", "degC")
    ds.time.encoding.update(
        {"units": "days since 2000-01-01 00:00:00", "calendar": "standard"}
    )
    ds.to_netcdf(path)


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
