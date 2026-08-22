import unittest

import numpy as np

from src.tools.satellite.rsfm_adapter import run_rsfm_inference


class TestRsfmAdapter(unittest.TestCase):
    def test_baseline_returns_deterministic_screening_mask(self):
        inputs = {
            "vv": np.array([[-20.0, -8.0], [-18.0, -7.0]]),
            "vh": np.array([[-25.0, -14.0], [-24.0, -13.0]]),
        }

        result = run_rsfm_inference(inputs)

        self.assertEqual(result["status"], "ok")
        self.assertEqual(result["backend"], "baseline")
        self.assertEqual(result["validation"], "screening_only")
        self.assertEqual(result["prediction"], [[True, False], [True, False]])
        self.assertEqual(result["positive_fraction"], 0.5)

    def test_rejects_mismatched_modalities(self):
        result = run_rsfm_inference({
            "vv": np.zeros((2, 2)),
            "vh": np.zeros((2, 3)),
        })

        self.assertEqual(result["status"], "invalid_input")
        self.assertIn("same shape", result["error"])

    def test_rejects_nonfinite_values(self):
        result = run_rsfm_inference({
            "vv": np.array([[-20.0, np.nan]]),
        })

        self.assertEqual(result["status"], "invalid_input")
        self.assertIn("finite", result["error"])

    def test_rejects_empty_arrays(self):
        result = run_rsfm_inference({"vv": np.empty((0, 2))})

        self.assertEqual(result["status"], "invalid_input")
        self.assertIn("non-empty", result["error"])

    def test_rejects_non_numeric_values_instead_of_coercing_them(self):
        result = run_rsfm_inference({"vv": [["-20.0"]]})

        self.assertEqual(result["status"], "invalid_input")
        self.assertIn("numeric", result["error"])

    def test_invalid_result_keeps_screening_envelope(self):
        result = run_rsfm_inference(
            {"vv": np.zeros((1, 1))},
            task="unsupported_task",
            provenance={"source": "test"},
        )

        self.assertEqual(result["validation"], "screening_only")
        self.assertEqual(result["provenance"], {"source": "test"})
        self.assertIn("limitations", result)

    def test_rejects_non_json_safe_provenance(self):
        result = run_rsfm_inference(
            {"vv": np.zeros((1, 1))},
            provenance={"score": np.nan},
        )

        self.assertEqual(result["status"], "invalid_input")
        self.assertIn("JSON", result["error"])

    def test_preserves_provenance_and_screening_status(self):
        provenance = {
            "source": "sentinel-1",
            "acquisition": "2026-08-21T00:00:00Z",
        }

        result = run_rsfm_inference(
            {"vv": np.array([[-20.0]])},
            provenance=provenance,
        )

        self.assertEqual(result["provenance"], provenance)
        self.assertEqual(result["validation"], "screening_only")
        self.assertIn("not validated", " ".join(result["limitations"]).lower())

    def test_rejects_unknown_task(self):
        result = run_rsfm_inference(
            {"vv": np.array([[-20.0]])},
            task="unsupported_task",
        )

        self.assertEqual(result["status"], "invalid_input")
        self.assertIn("task", result["error"])

    def test_pretrained_backend_reports_missing_runtime(self):
        result = run_rsfm_inference(
            {"vv": np.array([[-20.0]])},
            backend="pretrained",
        )

        self.assertEqual(result["status"], "insufficient_data")
        self.assertIn("weights", result["missing"])


if __name__ == "__main__":
    unittest.main()
