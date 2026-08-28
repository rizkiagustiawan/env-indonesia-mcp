import numpy as np
import csv
from pathlib import Path

from tools.wflow_env.build_chirps_forcing import build_forcing_dataset, chirps_url


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


def test_chirps_wflow_output_is_nonempty():
    assert CHIRPS_OUTPUT.is_file()
    rows = list(csv.DictReader(CHIRPS_OUTPUT.open()))
    values = [float(row["Q"]) for row in rows]
    assert len(rows) == 6
    assert all(np.isfinite(values))
    assert max(values) > 0.0
