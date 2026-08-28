# Wflow Citarum Environment

This directory pins the Julia environment used for the Wflow integration
smoke test.

## Runtime

- Julia: 1.12.7
- Wflow.jl: 1.0.4
- NCDatasets.jl: 0.14.15
- Platform: x86_64-pc-linux-gnu

Run the smoke test from the repository root:

```bash
julia --project=tools/wflow_env tools/wflow_env/smoke_test.jl
```

The test creates a 2 x 2 synthetic SBM grid, applies three daily forcing
steps, writes a CSV output, and asserts that Wflow produced at least one data
row. The fixture is an execution check only; it is not a Citarum calibration.

Animation outputs are kept under `output_maps/hydrology_animation/`. The Wflow
hydrograph and SWE depth examples are synthetic smoke demonstrations until a
spatial Citarum Wflow model and declared flood boundary forcing are available.
