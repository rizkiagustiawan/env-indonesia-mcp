import numpy as np
import csv
from pathlib import Path

from tools.wflow_env.build_spatial_forcing import interpolate_daily_grid


ROOT = Path(__file__).resolve().parents[1]
MODEL = ROOT / "data/benchmarks/citarum_hulu/wflow"


def test_interpolate_daily_grid_preserves_center_and_fills_valid_grid():
    source_lat = np.array([-7.2, -6.98, -6.76])
    source_lon = np.array([107.35, 107.60, 107.85])
    values = np.array(
        [
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ]
    )
    target_lat = np.array([-7.20, -6.98, -6.76])
    target_lon = np.array([107.35, 107.60, 107.85])

    result = interpolate_daily_grid(source_lat, source_lon, values, target_lat, target_lon)

    assert result.shape == (3, 3)
    assert np.allclose(result, values)


def test_spatial_wflow_output_is_reproducible_artifact():
    output = MODEL / "output_spatial.csv"
    assert output.is_file()
    rows = list(csv.DictReader(output.open()))
    discharge = [float(row["Q"]) for row in rows]
    assert len(rows) == 6
    assert all(np.isfinite(discharge))
    assert max(discharge) > 0.0
