import unittest

from src.tools.gis import flood_screening as fs
from src.tools.gis import location_resolver as lr


class TestPlanScreening(unittest.TestCase):
    def test_without_discharge_is_insufficient_data(self):
        plan = fs.plan_screening("kota bima", discharge_m3s=None,
                                 buffer_km=None, duration_hours=6)
        self.assertEqual(plan["status"], "insufficient_data")
        self.assertIn("inflow_discharge_m3s", plan["missing"])
        self.assertEqual(plan["resolution"]["type"], "Kota")
        self.assertIn("debit", " ".join(plan["limitations"]).lower())

    def test_with_discharge_is_screening_only(self):
        plan = fs.plan_screening("kota bima", discharge_m3s=50.0,
                                 buffer_km=None, duration_hours=6)
        self.assertEqual(plan["status"], "screening_only")
        self.assertGreater(plan["run"]["lat"], -11.5)
        self.assertLess(plan["run"]["lat"], 6.5)
        self.assertGreater(plan["run"]["buffer_km"], 0)
        self.assertEqual(plan["run"]["discharge_m3s"], 50.0)
        self.assertFalse(plan["synthetic"])

    def test_explicit_buffer_is_honored(self):
        plan = fs.plan_screening("kota bima", discharge_m3s=50.0,
                                 buffer_km=7.5, duration_hours=6)
        self.assertEqual(plan["run"]["buffer_km"], 7.5)

    def test_unknown_location_propagates(self):
        with self.assertRaises(lr.LocationError):
            fs.plan_screening("wakanda", discharge_m3s=50.0)

    def test_bare_ambiguous_location_propagates(self):
        with self.assertRaises(lr.LocationError):
            fs.plan_screening("bima", discharge_m3s=50.0)


if __name__ == "__main__":
    unittest.main()
