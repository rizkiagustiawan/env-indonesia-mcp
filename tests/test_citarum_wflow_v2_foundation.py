import hashlib

import numpy as np
import xarray as xr

from tools.wflow_env.validate_wflow_forcing import validate_forcing


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
