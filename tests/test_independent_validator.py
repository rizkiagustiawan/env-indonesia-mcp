import unittest

from src.tools.validation.independent_validator import validate_result


def valid_result():
    return {
        "status": "screening_only",
        "provenance": {
            "source_ids": ["dibi:event-1"],
            "input_hash": "abc123",
            "execution_id": "run-1",
        },
        "uncertainty": {"lower": 10.0, "upper": 20.0},
        "geospatial": {
            "crs": "EPSG:4326",
            "bbox": [106.0, -7.0, 107.0, -6.0],
            "resolution_m": 10.0,
        },
        "mass_balance": {
            "input_volume_m3": 100.0,
            "output_volume_m3": 99.0,
            "tolerance_fraction": 0.02,
        },
        "claims": [{"name": "flooded_area_m2", "value": 123.0}],
        "execution_receipt": {"reported_values": [123.0]},
    }


class TestIndependentValidator(unittest.TestCase):
    def test_accepts_complete_screening_result_without_promoting_status(self):
        result = validate_result(valid_result())

        self.assertEqual(result["validation_status"], "pass")
        self.assertEqual(result["result_status"], "screening_only")
        self.assertEqual(result["errors"], [])

    def test_rejects_missing_provenance(self):
        payload = valid_result()
        del payload["provenance"]

        result = validate_result(payload)

        self.assertEqual(result["validation_status"], "reject")
        self.assertIn("provenance", " ".join(result["errors"]).lower())

    def test_rejects_reversed_uncertainty_bounds(self):
        payload = valid_result()
        payload["uncertainty"] = {"lower": 20.0, "upper": 10.0}

        result = validate_result(payload)

        self.assertEqual(result["validation_status"], "reject")
        self.assertTrue(any("uncertainty" in error.lower() for error in result["errors"]))

    def test_rejects_invalid_crs_and_bbox(self):
        payload = valid_result()
        payload["geospatial"] = {
            "crs": "",
            "bbox": [107.0, -7.0, 106.0, -6.0],
            "resolution_m": 0.0,
        }

        result = validate_result(payload)

        self.assertEqual(result["validation_status"], "reject")
        errors = " ".join(result["errors"]).lower()
        self.assertIn("crs", errors)
        self.assertIn("bbox", errors)
        self.assertIn("resolution", errors)

    def test_rejects_mass_balance_beyond_tolerance(self):
        payload = valid_result()
        payload["mass_balance"]["output_volume_m3"] = 80.0

        result = validate_result(payload)

        self.assertEqual(result["validation_status"], "reject")
        self.assertTrue(any("mass balance" in error.lower() for error in result["errors"]))

    def test_rejects_claim_not_present_in_execution_receipt(self):
        payload = valid_result()
        payload["claims"][0]["value"] = 456.0

        result = validate_result(payload)

        self.assertEqual(result["validation_status"], "reject")
        self.assertTrue(any("execution receipt" in error.lower() for error in result["errors"]))


if __name__ == "__main__":
    unittest.main()
