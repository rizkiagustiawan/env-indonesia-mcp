import json
import tempfile
import unittest
from pathlib import Path

from src.tools.processing.hydrology_animation import (
    depth_snapshot_frames,
    load_hydrograph_csv,
    render_depth_history,
    render_hydrograph,
)


class TestHydrologyAnimation(unittest.TestCase):
    def test_load_hydrograph_csv_returns_numeric_series(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "q.csv"
            path.write_text("time,Q\n2000-01-01T00:00:00,1.5\n2000-01-02T00:00:00,3.0\n")

            labels, values = load_hydrograph_csv(path, value_column="Q")

            self.assertEqual(labels, ["2000-01-01T00:00:00", "2000-01-02T00:00:00"])
            self.assertEqual(values, [1.5, 3.0])

    def test_depth_snapshot_frames_respects_x_major_flattening(self):
        history = [{"time_s": 10.0, "depth_grid_m": [1.0, 2.0, 3.0, 4.0], "volume_m3": 10.0}]

        frames = depth_snapshot_frames(history, nx=2, ny=2)

        self.assertEqual(frames[0]["time_s"], 10.0)
        self.assertEqual(frames[0]["depth_grid_m"], [[1.0, 2.0], [3.0, 4.0]])

    def test_renderers_write_nonempty_gif(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            hydro = tmp / "hydro.gif"
            hydraulic = tmp / "hydraulic.gif"
            hydro_mp4 = tmp / "hydro.mp4"
            history = [{"time_s": 0.0, "depth_grid_m": [0.0, 0.0, 0.0, 0.0], "volume_m3": 0.0},
                       {"time_s": 10.0, "depth_grid_m": [0.0, 1.0, 2.0, 0.0], "volume_m3": 30.0}]

            render_hydrograph(["t0", "t1"], [0.0, 2.0], hydro, title="Hydro")
            render_hydrograph(["t0", "t1"], [0.0, 2.0], hydro_mp4, title="Hydro")
            render_depth_history(history, 2, 2, hydraulic, title="Hydraulic")

            self.assertGreater(hydro.stat().st_size, 100)
            self.assertGreater(hydro_mp4.stat().st_size, 100)
            self.assertGreater(hydraulic.stat().st_size, 100)


if __name__ == "__main__":
    unittest.main()
