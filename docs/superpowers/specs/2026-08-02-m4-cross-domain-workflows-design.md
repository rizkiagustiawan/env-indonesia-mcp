# M4: Cross-Domain Workflow Orchestrator Design

## Goal
Implement a cross-domain workflow orchestrator that ties isolated environmental tools together (Air Dispersion and Water Quality) while enforcing physical and meteorological continuity guardrails.

## Architecture
We will create a new Rust module `src/tools/workflows/mod.rs` with two submodules: `air_dispersion.rs` and `water_quality.rs`. 
Each orchestrator function will take raw parameters, pass them through the `spatial_validation.rs` or local checks (to prevent silent bad assumptions), and then invoke the underlying tools, returning a single, unified `ScientificResult`.

### 1. Water Quality Assessment Workflow
**Chain:** `treatment_train` (WWTP Effluent) → River Mixing (`dispersion_coefficient`) → Decay (`streeter_phelps`)
**Guardrail:** If the WWTP discharge flow rate (`q_m3d`) is larger than the receiving river's baseflow (`river_velocity * width * depth`), the assessment must fail or be flagged because the river cannot dilute the wastewater (a physical violation).

### 2. Air Dispersion Assessment Workflow
**Chain:** `plume_rise` → `stability` (Pasquill-Gifford) → `dispersion` (Gaussian Plume)
**Guardrail:** Elevation mismatch check. If the wind speed measurement height is standard (10m) but the stack height is >50m, wind speed must be extrapolated using the power-law profile before passing to the dispersion model. If extrapolated wind speed is not computed, fail the workflow.

## Deliverables
- `src/tools/workflows/mod.rs`
- `src/tools/workflows/water_quality.rs`
- `src/tools/workflows/air_dispersion.rs`
- Tests validating the physical guardrails.
