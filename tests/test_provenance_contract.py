import unittest
import os
import json
from src.tools.gis.provenance import create_provenance, read_provenance, ScientificResult, Provenance, Uncertainty

class TestProvenanceContract(unittest.TestCase):
    def setUp(self):
        self.test_file = "test_output.tif"
        if os.path.exists(self.test_file + ".meta.json"):
            os.remove(self.test_file + ".meta.json")

    def tearDown(self):
        if os.path.exists(self.test_file + ".meta.json"):
            os.remove(self.test_file + ".meta.json")

    def test_legacy_create_provenance(self):
        path = create_provenance(self.test_file, tool="test", source="api")
        self.assertTrue(os.path.exists(path))
        meta = read_provenance(self.test_file)
        self.assertEqual(meta["processing"]["tool"], "test")
        self.assertEqual(meta["provenance"]["generator"], "env-indonesia-mcp")

    def test_strict_scientific_result_validation(self):
        # Missing fallback reason
        with self.assertRaises(ValueError) as cm:
            Provenance(source_kind="fallback", source_identifier="none", acquisition_timestamp="2026-08-02")
        self.assertIn("fallback reason", str(cm.exception).lower())

        # Invalid bounds
        with self.assertRaises(ValueError):
            Uncertainty(uncertainty_type="bound", lower=10.0, upper=5.0, method="test")

        # Missing seed for stochastic
        with self.assertRaises(ValueError):
            Uncertainty(uncertainty_type="confidence_interval", lower=5.0, upper=10.0, method="test")

        # Regulatory claim on screening
        result = ScientificResult(parameter="test", value=1.0, unit="mg/L", status="screening_only")
        with self.assertRaises(ValueError):
            result.add_claim("compliant", "test")

    def test_result_serialization(self):
        prov = Provenance("api", "landsat", "2026-08-02")
        unc = Uncertainty("bound", 5.0, 10.0, "range")
        result = ScientificResult("test_param", 10.0, "m", "valid", unc, prov)
        
        d = result.to_dict()
        self.assertEqual(d["value"], 10.0)
        self.assertEqual(d["status"], "valid")
        self.assertEqual(d["provenance"]["source_kind"], "api")
        self.assertEqual(d["uncertainty"]["lower"], 5.0)

if __name__ == '__main__':
    unittest.main()
