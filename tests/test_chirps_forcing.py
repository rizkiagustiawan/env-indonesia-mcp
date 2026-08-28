import numpy as np
import csv
from pathlib import Path

from tools.wflow_env.build_chirps_forcing import artifact_paths, build_forcing_dataset, chirps_url, date_range


ROOT = Path(__file__).resolve().parents[1]
CHIRPS_OUTPUT = ROOT / "data/benchmarks/citarum_hulu/wflow/output_chirps.csv"


def test_build_forcing_dataset_uses_cf_time_and_wflow_dimensions():
    times = ["2016-03-10", "2016-03-11"]
    lat = np.array([-7.0, -6.995])
    lon = np.array([107.5, 107.505])
    shape = (2, 2, 2)
    precip = np.ones(shape, dtype=np.float32)
    pet = np.full(shape, 3.0, dtype=np.float32)
    temp = np.full(shape, 25.0, dtype=np.float32)

    dataset = build_forcing_dataset(times, lat, lon, precip, pet, temp)

    assert dataset["precip"].dims == ("time", "lat", "lon")
    assert dataset.sizes == {"time": 2, "lat": 2, "lon": 2}
    assert dataset.time.attrs["units"] == "days since 2000-01-01 00:00:00"
    assert np.all(dataset.precip.values == 1.0)


def test_chirps_filename_uses_dotted_date():
    assert chirps_url("2016-03-10").endswith(
        "chirps-v2.0.2016.03.10.tif.gz"
    )


def test_date_range_is_inclusive():
    assert date_range("2016-01-01", "2016-01-03") == [
        "2016-01-01",
        "2016-01-02",
        "2016-01-03",
    ]


def test_default_artifact_names_remain_stable():
    output, source, receipt, config = artifact_paths("2016-03-10", "2016-03-16", "chirps")
    assert output.name == "forcing_2016_chirps.nc"
    assert source.name == "forcing_chirps_2016-03-10_2016-03-16.json"
    assert receipt.name == "chirps_forcing_receipt.json"
    assert config.name == "citarum_sbm_chirps.toml"


def test_chirps_wflow_output_is_nonempty():
    assert CHIRPS_OUTPUT.is_file()
    rows = list(csv.DictReader(CHIRPS_OUTPUT.open()))
    values = [float(row["Q"]) for row in rows]
    assert len(rows) == 6
    assert all(np.isfinite(values))
    assert max(values) > 0.0


def test_warmup_artifacts_cover_more_than_event_window():
    forcing = ROOT / "data/benchmarks/citarum_hulu/wflow/forcing_2016-01-01_2016-03-16_chirps_warmup.nc"
    output = ROOT / "data/benchmarks/citarum_hulu/wflow/output_chirps_warmup.csv"
    assert forcing.is_file()
    assert output.is_file()
    rows = list(csv.DictReader(output.open()))
    assert len(rows) == 75
    assert rows[0]["time"] == "2016-01-02T00:00:00"
    assert rows[-1]["time"] == "2016-03-16T00:00:00"
