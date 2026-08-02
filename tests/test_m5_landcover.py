import unittest
import json
import sys
from unittest.mock import patch
import src.tools.gis.landcover_engine as le

class TestM5LandcoverIntegration(unittest.TestCase):
    def test_json_result_format(self):
        # We will test a helper function `format_scientific_result` that we will add to landcover_engine.py
        # It should take the area histogram and optionally Olofsson adjusted area, and return the M2 JSON.
        class_hist = {"0": 100.0, "1": 50.0} # raw areas in ha
        adjusted_areas = {"0": (110.0, 10.0), "1": (40.0, 5.0)} # adjusted area and CI bound
        
        result_json_str = le.format_scientific_result(
            parameter="landcover_area",
            class_hist=class_hist,
            adjusted_areas=adjusted_areas,
            sensor="Dynamic World",
            resolution_m=10
        )
        
        result = json.loads(result_json_str)
        self.assertEqual(result["parameter"], "landcover_area")
        self.assertEqual(result["status"], "valid")
        self.assertEqual(result["provenance"]["source_kind"], "api")
        self.assertEqual(result["provenance"]["sensor"], "Dynamic World")
        self.assertEqual(result["uncertainty"]["uncertainty_type"], "confidence_interval")

if __name__ == '__main__':
    unittest.main()
