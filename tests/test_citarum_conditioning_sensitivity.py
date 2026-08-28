import unittest

import numpy as np

from scripts.citarum_conditioning_sensitivity import run_sensitivity


class TestCitarumConditioningSensitivity(unittest.TestCase):
    def test_run_sensitivity_returns_one_record_per_depth(self):
        dem = np.array([
            [10.0, 9.0, 8.0, 7.0],
            [11.0, 10.0, 9.0, 8.0],
            [12.0, 11.0, 10.0, 9.0],
            [13.0, 12.0, 11.0, 10.0],
        ])
        mask = np.zeros_like(dem, dtype=np.uint8)
        mask[:, 0] = 1

        records = run_sensitivity(dem, mask, burn_depths=(0.0, 2.0, 5.0), connectivity=8)

        self.assertEqual([record["burn_depth_m"] for record in records], [0.0, 2.0, 5.0])
        self.assertEqual([record["conditioned_pit_count"] for record in records], [0, 0, 0])
        self.assertEqual(records[0]["burned_cells"], 4)
        self.assertEqual(records[0]["conditioned_min_m"], 7.0)
        self.assertLess(records[2]["conditioned_min_m"], records[0]["conditioned_min_m"])

    def test_run_sensitivity_rejects_empty_depths(self):
        with self.assertRaises(ValueError):
            run_sensitivity(np.ones((2, 2)), np.zeros((2, 2), dtype=np.uint8), burn_depths=())


if __name__ == "__main__":
    unittest.main()
