import unittest

import numpy as np

from src.tools.gis.dem_conditioning import (
    condition_dem,
    count_interior_pits,
    priority_flood_fill,
)


class TestDemConditioning(unittest.TestCase):
    def test_priority_flood_fills_closed_depression_to_spill_level(self):
        dem = np.array([
            [10.0, 10.0, 10.0, 10.0, 10.0],
            [10.0, 8.0, 8.0, 8.0, 10.0],
            [10.0, 8.0, 3.0, 8.0, 10.0],
            [10.0, 8.0, 8.0, 8.0, 10.0],
            [10.0, 10.0, 10.0, 10.0, 10.0],
        ])

        filled = priority_flood_fill(dem)

        self.assertEqual(filled[2, 2], 10.0)
        np.testing.assert_array_equal(filled[[0, 0, 4, 4], [0, 4, 0, 4]], 10.0)

    def test_condition_dem_burns_before_fill_and_removes_fragment_pit(self):
        dem = np.array([
            [10.0, 10.0, 10.0, 10.0, 10.0],
            [10.0, 10.0, 10.0, 10.0, 10.0],
            [10.0, 10.0, 10.0, 10.0, 10.0],
            [10.0, 10.0, 10.0, 10.0, 10.0],
            [10.0, 10.0, 10.0, 10.0, 10.0],
        ])
        mask = np.array([
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ], dtype=np.uint8)

        conditioned = condition_dem(dem, mask, burn_depth_m=5.0, connectivity=8)

        self.assertEqual(conditioned[2, 2], 10.0)
        self.assertEqual(conditioned[0, 0], 10.0)
        self.assertEqual(count_interior_pits(conditioned, connectivity=8), 0)

    def test_priority_flood_rejects_unsupported_connectivity(self):
        with self.assertRaises(ValueError):
            priority_flood_fill(np.ones((3, 3)), connectivity=6)

    def test_count_interior_pits_respects_valid_mask_and_connectivity(self):
        dem = np.full((5, 5), 10.0)
        dem[2, 2] = 1.0
        valid = np.ones((5, 5), dtype=bool)
        valid[0, 0] = False

        self.assertEqual(count_interior_pits(dem, valid_mask=valid, connectivity=8), 1)
        self.assertEqual(count_interior_pits(dem, valid_mask=valid, connectivity=4), 1)

    def test_count_interior_pits_excludes_valid_mask_outlets(self):
        dem = np.full((3, 3), 10.0)
        dem[0, 1] = 1.0
        valid = np.ones((3, 3), dtype=bool)

        self.assertEqual(count_interior_pits(dem, valid_mask=valid, connectivity=8), 0)

    def test_rejects_shape_mismatch(self):
        with self.assertRaises(ValueError):
            condition_dem(np.ones((2, 2)), np.ones((3, 3), dtype=np.uint8), 5.0)


if __name__ == "__main__":
    unittest.main()
