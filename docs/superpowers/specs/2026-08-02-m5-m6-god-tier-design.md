# M5 & M6: God Tier Integration Design

## Goal
Implement a fully compliant scientific integration for M5 (Olofsson Area Estimation in `landcover_engine.py`) and M6 (2D-MCA Risk Assessment in `monte_carlo_risk.rs`) to ensure end-to-end scientific correctness and avoid producing naive deterministic or raw pixel count outputs.

## Architecture

### M5: GIS Accuracy Framework Integration
**Changes:** Modify `src/tools/gis/landcover_engine.py`
- We currently use `landcover_engine.py` to classify satellite imagery. We need to parse an optional (but highly recommended) validation GeoJSON parameter.
- If the validation GeoJSON is provided, the script computes a confusion matrix internally.
- The script must then use the Python equivalent of the `olofsson.rs` logic (or call a robust library / Rust binary if it were easier, but doing the math natively in the script is cleaner for integration) to emit *Adjusted Area* and *Confidence Intervals* alongside the raw pixel count.
- The Python script must return JSON if a `--json-result` flag is passed, which maps directly to the `ScientificResult` contract (M2) in Rust.

### M6: 2D-MCA Risk Assessment
**Changes:** Modify `src/tools/calculators/monte_carlo_risk.rs`
- Instead of a 1D loop mapping random samples over all parameters, we use a 2D nested loop structure.
- **Outer Loop (Epistemic):** Represents ignorance or structural uncertainty. E.g., the true median concentration of a pollutant might have an error bound. We sample the true `concentration_mean` from its confidence interval for each outer loop iteration.
- **Inner Loop (Aleatory):** Represents population variability (Body Weight, Intake Rate).
- We output the 95th percentile risk *for each* outer loop iteration, generating a distribution of P95 values.
- We then report the Median of the P95s and the 95th Percentile of the P95s (The 95/95 Risk).

## Deliverables
- `src/tools/calculators/monte_carlo_risk.rs` (Refactored to 2D-MCA)
- `src/tools/gis/landcover_engine.py` (Olofsson Integration & JSON output)
- `src/server.rs` (Adaptations to parse the new `--json-result` output from `landcover_engine.py`)
