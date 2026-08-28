import unittest

import numpy as np

from scripts.citarum_flood_screening_analysis import (
    binary_metrics,
    scenario_stability,
    terrain_quantile_mask,
)


class TestCitarumFloodScreeningAnalysis(unittest.TestCase):
    def test_binary_metrics_uses_only_valid_cells(self):
        reference = np.array([[1, 1, 0], [0, 1, 0]], dtype=np.uint8)
        predicted = np.array([[1, 0, 1], [0, 1, 0]], dtype=np.uint8)
        valid = np.array([[1, 1, 1], [1, 0, 1]], dtype=bool)

        metrics = binary_metrics(reference, predicted, valid)

        self.assertEqual(metrics["tp"], 1)
        self.assertEqual(metrics["fp"], 1)
        self.assertEqual(metrics["fn"], 1)
        self.assertEqual(metrics["tn"], 2)
        self.assertAlmostEqual(metrics["iou"], 1 / 3)
        self.assertAlmostEqual(metrics["pod"], 0.5)
        self.assertAlmostEqual(metrics["far"], 0.5)

    def test_binary_metrics_rejects_shape_mismatch(self):
        with self.assertRaises(ValueError):
            binary_metrics(np.zeros((2, 2)), np.zeros((2, 3)), None)

    def test_scenario_stability_reports_absolute_change(self):
        baseline = np.array([[1.0, 2.0], [3.0, 4.0]])
        scenario = np.array([[1.0, 2.5], [2.0, 4.0]])
        valid = np.ones((2, 2), dtype=bool)

        result = scenario_stability(baseline, scenario, valid)

        self.assertEqual(result["changed_cells"], 2)
        self.assertAlmostEqual(result["mean_abs_delta_m"], 0.375)
        self.assertAlmostEqual(result["max_abs_delta_m"], 1.0)

    def test_terrain_quantile_uses_requested_valid_intersection(self):
        dem = np.array([[1.0, 2.0, 100.0], [3.0, 4.0, 100.0]])
        valid = np.array([[0, 1, 1], [0, 1, 1]], dtype=bool)

        mask, threshold = terrain_quantile_mask(dem, valid, quantile=0.5)

        self.assertAlmostEqual(threshold, 52.0)
        self.assertEqual(mask.sum(), 2)


if __name__ == "__main__":
    unittest.main()
