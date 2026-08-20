# env-indonesia-mcp

**An auditable MCP server for environmental engineering analysis and modeling in Indonesia.**

[🇮🇩 Dokumentasi Bahasa Indonesia](README.md)

`env-indonesia-mcp` connects AI agents to Rust calculators, spatial data acquisition, real hydrology/geochemistry solvers, and scientific result contracts. Its goal is not to make an AI answer every question. Its goal is to make the AI **know when to stop, which data is missing, and which assumptions limit the result**.

> **Scientific status:** most outputs are `screening_only` or `valid_with_assumptions`. `valid` can only be earned through split-sample validation that passes defined metrics and supporting data. This system does not replace engineers, surveyors, laboratories, regulators, or field validation.

## Contents

- [What It Builds](#what-it-builds)
- [Working Physics Engines](#working-physics-engines)
- [Scientific Honesty Layer](#scientific-honesty-layer)
- [Architecture](#architecture)
- [Installation](#installation)
- [Usage](#usage)
- [Status and Limitations](#status-and-limitations)
- [Testing](#testing)
- [Project Structure](#project-structure)
- [License and Data](#license-and-data)

## What It Builds

The server exposes a broad MCP catalog, but tool maturity is not uniform. Each domain must be interpreted according to its evidence and implementation level:

| Level | Meaning |
|---|---|
| `insufficient_data` | Required inputs are missing; analysis is refused. |
| `screening_only` | The calculation runs, but it is not field-calibrated or independently validated. |
| `valid_with_assumptions` | Limited data or calibration is available; assumptions still constrain interpretation. |
| `valid` | Granted only after independent validation passes the gate. |
| `validation_failed` | The contract, solver, input, or physical gate failed. |

The system combines:

- Satellite/STAC data: Sentinel-1/2, Landsat, DEM, GPM/IMERG and related catalogs.
- GIS: GeoJSON, bounding boxes, rasters, CRS, clipping, manifests, and provenance.
- Hydrology: 2D SWE and 1D SWMM sewer coupling.
- Geochemistry: PHREEQC equilibrium, lime titration, pyrite oxidation kinetics, and 1D reactive transport.
- Hydrogeology: MODFLOW 6 through FloPy.
- Evidence: sources, artifacts, claims, independent lineages, conflicts, human review, and a SHA-256 audit chain.

## Working Physics Engines

### Flooding: SWMM 1D + 2D SWE

`swmm_1d2d_coupling` runs EPA SWMM through `pyswmm`, reads surcharge volume from each flooding node, maps nodes to DEM cells, and injects them into the multi-source SWE solver.

Guards include:

- SWMM surcharge volume versus volume injected into the 2D domain.
- Default mass-balance tolerance: 1%.
- Unmapped sewer nodes are rejected.
- The result remains `screening_only` because it is not validated against observed flood extent/depth.

Verified synthetic run:

```text
SWMM surcharge: 1231.92 m³
SWE injected:   1231.88 m³
Mass error:     0.0030%
Gate:            passed
Status:          screening_only
```

### PHREEQC: Speciation and Leaching

`phreeqc_speciation` runs real PHREEQC through `phreeqpython`, rather than only generating an input script. It supports:

- Solution speciation and saturation indices.
- Ca(OH)2 titration to a target pH using bracketing and bisection.
- Equilibrium phases in precipitation-only mode (`in_phase = 0`).
- Dissolved metals before and after treatment.

The database `wateq4f_PWN_repaired.dat` is stored in `resources/phreeqc/`. It is a repaired copy with provenance and SHA-256 metadata. The repair was necessary because the source file contained three empty `log_k` entries that prevented PHREEQC from loading it.

### Pyrite Oxidation Kinetics

`pyrite_oxidation_kinetics` uses the database RATES block and the Williamson & Rimstidt (1994) rate law. It returns pH, Fe, sulfate, remaining pyrite, and guards for:

- `oxygen_limited`: distinguishes a sealed system that ran out of O2 from a genuinely stable reaction.
- `pyrite_depleted`: distinguishes a flat pH curve caused by consumed sulfide.
- `stoichiometry_consistent`: FeS2 should release approximately 2 mol S per mol Fe; Fe precipitation can break that relationship.
- `rate_is_laboratory_derived`: always true; the field rate is not calibrated.

### 1D Reactive Transport

`reactive_transport` runs PHREEQC `TRANSPORT` through a mineral column:

- Advection, dispersion, and reactions per cell.
- Output indexed by pore volumes rather than only a final value.
- `grid_peclet`: if greater than 2, grid numerical dispersion dominates.
- `front_traversed_column`: the front must pass at least one pore volume before the outlet is interpretable.
- `buffer_exhausted`: detects mineral exhaustion and breakthrough.
- The full-equilibrium-per-cell assumption is reported explicitly.

This is a 1D column model, not a 3D MODFLOW-GWT/PhreeqcRM coupling.

### MODFLOW 6 Groundwater

`modflow_groundwater` runs MODFLOW 6.7.0 through FloPy with explicit units:

- Length: metres.
- Time: days.
- Conductivity: m/day.
- Recharge: mm/year, converted internally.
- Well extraction: m³/day.

Guards include convergence, `PERCENT_DISCREPANCY`, dry-cell sentinels, and silent well curtailment. There is no automatic Theis fallback. If MODFLOW fails, the result is an error rather than a substituted number.

## Scientific Honesty Layer

### Data Maturity Ladder

`assess_data_maturity` determines the highest level supported by the supplied data:

```text
insufficient_data → screening → conceptual → calibrated → validated
```

Synthetic data must be marked `synthetic: true` and can never produce `valid`.

### Earned Validation

`calibrate_and_validate` uses a contiguous split rather than a random split, avoiding information leakage in autocorrelated time series.

`validated` is earned only when the test partition satisfies:

- NSE > 0.5
- |PBIAS| < 25%
- at least 5 test points

A model that fits training data but fails on test data is capped at `valid_with_assumptions` as an overfitting signal. Results carry a `PredictionInterval` derived from test RMSE.

### Evidence Assessment

`evidence_assess`:

- Requires at least two independent lineages for corroboration.
- Treats syndicated reports with the same `independence_group` as one source.
- Routes conflicts between independent lineages to `human_review`.
- Supports a tier-1 official finding as a single-source rule.
- Always returns `screening_only` and never makes a legal or regulatory conclusion.

### Provenance and Audit

`record_computation` records external software executions such as QGIS, SWMM, PHREEQC, and MODFLOW. Records include tool, version, arguments, input/output hashes, timestamps, exit code, and the audit-event hash.

All external solver results are treated as **untrusted execution** until the contract and physical gates have been checked.

## Architecture

```text
AI Agent / MCP Client
        │ stdio MCP
        ▼
env-indonesia-mcp (Rust)
  ├─ result contract + honesty ladder
  ├─ evidence + SHA-256 audit chain
  ├─ satellite/STAC + artifact manifests
  ├─ SWE / SWMM coupling
  ├─ PHREEQC / reactive transport
  ├─ MODFLOW 6 / FloPy runner
  └─ legacy calculators and domain tools
        │ guarded subprocesses and recorded provenance
        ├─ Python environmental venv
        └─ QGIS Agent MCP (optional, live QGIS session)
```

## Installation

### Rust

```bash
git clone https://github.com/rizkiagustiawan/env-indonesia-mcp.git
cd env-indonesia-mcp
cargo build --release
```

### Python solver environment

```bash
python3 -m venv /path/to/env-indonesia
/path/to/env-indonesia/bin/pip install -r requirements.txt
```

Solver stack:

- `pyswmm==2.1.0`
- `swmm-toolkit==0.17.0`
- `wntr==1.5.0`
- `phreeqcrm==0.0.20`
- `phreeqpython==1.6.2`
- `flopy==3.10.0`
- `numpy`, `scipy`, `pandas`

Install MODFLOW executables separately:

```bash
get-modflow /path/to/env-indonesia/bin --subset mf6,mf2005,mp7
```

Override the solver interpreter without changing code:

```bash
export ENV_INDONESIA_SWMM_PYTHON=/path/to/env-indonesia/bin/python
```

### MCP server

```bash
cargo run --release
```

Example client configuration:

```json
{
  "mcpServers": {
    "env-indonesia": {
      "command": "cargo",
      "args": ["run", "--release", "--manifest-path", "/path/env-indonesia-mcp/Cargo.toml"]
    }
  }
}
```

## Usage

```bash
# Data maturity
cargo run -- --test-tool assess_data_maturity \
  '{"requested_level":"validated","availability":{"regional_dem":true}}'

# Earned validation
cargo run -- --test-tool calibrate_and_validate \
  '{"model_name":"example","predicted":[1,2,3,4,5,6,7,8,9,10],"observed":[1,2,3,4,5,6,7,8,9,10],"unit":"m"}'

# PHREEQC equilibrium
cargo run -- --test-tool phreeqc_speciation \
  '{"solution":{"pH":2.8,"Fe(3)":50,"S(6)":200,"Ni":5},"units":"mmol"}'

# Pyrite kinetics
cargo run -- --test-tool pyrite_oxidation_kinetics \
  '{"pyrite_mol_per_kgw":0.05,"initial_ph":6.5,"replenish_o2":true,"steps_days":[1,30,90,365]}'

# Reactive transport column
cargo run -- --test-tool reactive_transport \
  '{"cells":5,"cell_length_m":0.2,"shifts":60,"time_step_s":3600,"dispersivity_m":0.1,"influent":{"pH":2.5,"Fe(3)":30,"S(6)":120},"pore_water":{"pH":7,"Ca":1},"units":"mmol","reactive_phases":[{"phase":"Calcite","moles":0.02}],"tracked_elements":["Fe"]}'

# MODFLOW groundwater
cargo run -- --test-tool modflow_groundwater \
  '{"nlay":2,"nrow":20,"ncol":20,"cell_size_m":100,"top_m":50,"layer_bottoms_m":[30,0],"hk_m_day":10,"vk_m_day":1,"sy":0.15,"ss_per_m":0.00001,"initial_head_m":45,"boundary_head_m":45,"recharge_mm_yr":1800,"wells":[{"layer":1,"row":10,"col":10,"rate_m3_day":2000}],"steady_state":true}'
```

## Status and Limitations

### Working

- Scientific result contract with finite-value, uncertainty, provenance, CRS, stale-source, and regulatory-claim guards.
- Evidence assessment and SHA-256 audit chain.
- STAC asset download with host allowlist, content validation, manifest, and hash.
- SWMM 1D + SWE 2D with a mass-balance gate.
- PHREEQC equilibrium, pyrite kinetics, and 1D reactive transport.
- MODFLOW 6.7.0 + FloPy with budget, dry-cell, and well-curtailment gates.
- Earned split-sample validation and prediction intervals.

### Not claimed

- Not automatically field-calibrated or field-validated.
- No real-time digital twin.
- No trained PINO/FNO checkpoint validated across regions.
- Reactive transport is 1D; no 3D MODFLOW-GWT/PhreeqcRM coupling yet.
- Pyrite kinetics use laboratory-derived rates; field rates are not calibrated.
- DEMNAS and satellite proxies do not replace LiDAR, surveys, gauges, discharge measurements, or flood observations.
- Drainage data, boundary conditions, porosity, dispersivity, roughness, and chemical parameters must come from engineers or field data; the system does not silently invent them.

## Testing

```bash
cargo test
python3 -m pytest -q
python3 -m py_compile scripts/*.py
git diff --check
```

Verified baseline when this documentation was updated:

- Rust: **247 tests passed**
- Python: **11 tests passed**
- API gateway: `cargo check` passed
- Solver scripts: `py_compile` passed

The Python suite currently emits two dependency/deprecation warnings but no failures.

## Project Structure

```text
src/
├── main.rs                 # entry point, MCP router, CLI dispatch
├── server.rs               # MCP tool definitions
├── result_contract.rs      # ScientificResult and contract validation
├── honesty.rs              # maturity ladder and synthetic lock
├── evidence/               # source/artifact/claim/audit evidence
├── computation.rs          # external computation run manifest
├── calibration.rs          # earned validation and prediction interval
├── coupling.rs             # SWMM 1D -> SWE 2D mapping and mass gate
├── swmm_runner.rs          # pyswmm subprocess contract
├── phreeqc_runner.rs       # equilibrium PHREEQC subprocess contract
├── pyrite_kinetics.rs      # PHREEQC KINETICS contract
├── reactive_transport.rs   # PHREEQC TRANSPORT contract
├── modflow_runner.rs       # MODFLOW 6/FloPy subprocess contract
└── tools/                  # legacy calculators and domain tools
scripts/
├── swmm_run.py
├── phreeqc_run.py
├── pyrite_kinetics.py
├── reactive_transport.py
└── modflow_run.py
resources/phreeqc/
└── wateq4f_PWN_repaired.dat
```

## License and Data

Project source: [GitHub](https://github.com/rizkiagustiawan/env-indonesia-mcp). Check the license of each solver, thermodynamic database, satellite dataset, and official source before commercial or regulatory use.

Never commit credentials, DEMNAS passwords, Telegram tokens, or private endpoints. Use environment variables or a secret manager.

## Project Principle

> A good environmental system is not one that always produces a number. It is one that can show the number's source, assumptions, error budget, limits of use, and the reason it refuses when evidence is insufficient.
