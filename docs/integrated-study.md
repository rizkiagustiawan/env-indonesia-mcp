# Integrated Environment Study

`integrated_environment_study` is the first end-to-end orchestration path for a user-defined study area. It accepts a GeoJSON AOI and optional domain inputs, validates the AOI, chooses an evidence level, runs available baselines, and returns an auditable JSON report.

## CLI Example

```bash
./target/debug/env-indonesia-mcp --test-tool integrated_environment_study '{"aoi_geojson":"{\"type\":\"Polygon\",\"coordinates\":[[[101.0,0.0],[101.2,0.0],[101.2,0.2],[101.0,0.2],[101.0,0.0]]]}" ,"domains":["urban_flood","landfill_leachate","acid_mine_drainage"],"satellite_fallback":true}'
```

GeoJSON coordinates must be WGS84 longitude/latitude. A Feature, FeatureCollection, Geometry, Point, Polygon, or MultiPolygon is accepted. The workflow returns `invalid_request` for malformed GeoJSON or unsupported domains.

## Domain Inputs

- `flood`: rectangular DEM plus grid size, Manning roughness, duration, and inflow. Runs the existing 2D SWE HLLC/MUSCL baseline.
- `leachate`: landfill area, twelve monthly rainfall values, twelve monthly ET values, storage, and runoff coefficient. Runs the monthly water-balance baseline.
- `amd`: sulfur, ANC, and optional NAG pH. Runs ABA MPA/NAPP static screening.

Missing inputs are reported as `insufficient_data`. Available baselines are reported as `screening_only`.

## Satellite Fallback

When enabled, the workflow queries Earth Search STAC for candidate Sentinel-1, Sentinel-2, and DEM scenes using the AOI bbox. This is a discovery step. It does not download or silently treat satellite proxies as discharge, groundwater chemistry, leachate quality, or mineral kinetics.

The separate `stac_download_asset` tool performs an actual HTTP retrieval bounded to 512 MiB. It requires successful item and asset responses, validates HTTPS/allowlisted asset hosts, TIFF media type and magic bytes, writes the raster and a `<raster>.manifest.json` containing the byte hash, source URL, license, and optional CRS, and reports `scientific interpretation was not performed` as a limitation.

## Evidence Boundaries

Example structured response fragment:

```json
{
  "status": "screening_only",
  "domain_results": [
    {
      "domain": "acid_mine_drainage",
      "status": "screening_only",
      "method": "ABA MPA/NAPP static screening",
      "summary": {
        "mpa_kg_h2so4_t": 61.2,
        "napp_kg_h2so4_t": 51.2,
        "nag_ph": 3.0
      },
      "limitations": [
        "No kinetic test simulation, PHREEQC execution, reactive transport, or field validation"
      ]
    }
  ],
  "validation": {
    "status": "not_run"
  }
}
```

`summary` is machine-readable; `output` retains the existing human-readable solver report.

The current vertical slice does not perform:

- rainfall-runoff calibration;
- PHREEQC script generation only (no reactive transport module execution or PhreeqcRM coupling);
- AMD kinetic tests or field reactive transport;
- trained FNO/PINO/PINN inference (models run conceptually but lack trained checkpoints, fallback mechanisms govern current behavior);
- calibration, independent validation, or parameter ensembles.

## 1D-2D Sewer Coupling (`swmm_1d2d_coupling`)

A separate tool couples a real EPA SWMM 1D sewer run to the 2D SWE overland solver. `integrated_environment_study` itself still runs the 2D baseline alone.

The tool executes `scripts/swmm_run.py` with `pyswmm` as an external subprocess (argument vector, timeout, bounded stdout), converts each flooding node's surcharge volume into an equivalent steady 2D point discharge over the simulation window, solves the 2D shallow-water equations with `solve_multi_source`, and then applies a **mass-balance gate** comparing the 1D surcharge volume against the 2D injected volume.

- Default tolerance is 1%. Exceeding it sets `gate_passed: false` and adds the limitation `mass-balance gate FAILED; coupling result must not be used`.
- Every SWMM invocation is written to the tamper-evident audit chain via `record_computation`, and the resulting `audit_event` hash is attached as a claim.
- The coupled result is capped at `screening_only` by the honesty ladder and can never be `valid`: the overland extent is not validated against observed flood extent.

Verified end to end against a 2-junction test model: 1231.92 m³ of 1D surcharge produced 1231.88 m³ of 2D injected volume (mass error 0.0030%, gate passed), peak coupled depth 1.845 m over 10 flooded cells.

## Data Maturity Gating (`assess_data_maturity`)

`assess_data_maturity` reports the highest honesty level the supplied data can support (`insufficient_data` → `screening` → `conceptual` → `calibrated` → `validated`) and lists what is missing for the level that was requested. Synthetic field data is capped at `conceptual` and never contributes toward `calibrated` or `validated`; a result flagged `synthetic: true` is rejected by the contract if it claims `valid` status.

## Earned Validation (`calibrate_and_validate`)

`validated` is not a flag a caller can set. `calibrate_and_validate` takes a paired predicted/observed series, splits it **contiguously** (Klemeš 1986 split-sample test — not randomly, which would leak information across autocorrelated series), and computes Moriasi et al. (2007) metrics separately for the train and test partitions.

The level is then earned from the numbers:

| Earned level | Requirement | Contract status |
| --- | --- | --- |
| `validated` | test partition: NSE > 0.5, \|PBIAS\| < 25%, n ≥ 5 | `valid` |
| `calibrated` | train partition clears the bar but test does not (overfitting) | `valid_with_assumptions` |
| `screening` | neither partition clears the bar | `screening_only` |

The result carries a `PredictionInterval` whose half-width is `z(1 - α/2) × RMSE_test`, so the interval reflects performance on data the model was not fitted to. When a `availability` block is supplied, `assess_level_with_evidence` takes the **minimum** of the declared and the earned level: evidence can only cap a claim, never inflate it. With no evidence at all, a declared calibration claim is capped at `conceptual`.

Verified behaviour: a near-perfect series earns `validated` (test n=6, NSE 0.9983); an anti-correlated series earns `screening` (test NSE −70.2); and a series that fits its train partition but collapses on test earns `calibrated`, not `validated` (train NSE 0.9999, test NSE −77.06).

## Multi-Source Evidence (`evidence_assess`)

`evidence_assess` scores claims from multiple reporting sources. It deduplicates claims by semantics, requires at least **two independent reporting lineages** before it will corroborate anything, and treats disagreement between independent lineages as a contradiction that routes to `human_review` rather than picking a winner.

- Two sources that share an `independence_group` are one lineage, not two — syndicated copies of the same report never corroborate each other.
- A tier-1 `official_finding` is sufficient alone and is given an effective confidence of `1.0` when it carries none.
- Aggregate confidence is the **lowest** of the strongest confidence per lineage, so the weakest independent link governs.
- Output is always `screening_only` and carries the abstention reason `Evidence core does not make legal or regulatory conclusions`. Regulatory claim types (`compliant`, `approved`, `safe`, `legal`) are rejected by the contract on screening results.

Every source, artifact, and claim is validated before assessment: unknown source or artifact references are rejected rather than silently dropped, and artifact payload hashes are SHA-256. Ingestion and review events append to a tamper-evident chain (`previous_event_sha256` / `event_sha256`) that `verify_chain` re-derives.

## PHREEQC Geochemistry (`phreeqc_speciation`)

`phreeqc_speciation` runs a real PHREEQC calculation through `phreeqpython` (`scripts/phreeqc_run.py`), not a script generator. Given a solution composition it reports speciation, saturation indices, and — when `lime_titration_target_ph` is set — the dissolved metals remaining after Ca(OH)2 neutralisation.

The bundled `wateq4f_PWN.dat` from `phreeqpython` cannot be loaded as shipped: it has CRLF line endings and three `log_k` entries with no value, so PHREEQC aborts with `Expecting log k` and every later call fails with `RunString: No database is loaded`. A repaired copy lives at `resources/phreeqc/wateq4f_PWN_repaired.dat`; the file header records each repair and its source value from the canonical `phreeqc.dat`. It is the only bundled database that defines master species for Ni and As.

Three honesty guards are part of the contract, because each one covers a case where PHREEQC returns a number that means something other than a measurement:

| Guard | What it prevents |
| --- | --- |
| `unsupported_elements` | PHREEQC accepts an element with no master species, reports 0 mg/L, and raises nothing. Chromium is not defined in any bundled database, so a Cr request would read as "not mobile" instead of "never modelled". |
| `sc_us_cm: null` + `sc_note` | Specific conductance needs `-dw` diffusion coefficients. Databases lacking them return 0.0 µS/cm, impossible for a solution with real ionic strength. |
| `supersaturated_but_unmodelled` + `concentrations_are_upper_bounds` | A phase with SI > 0 that was not listed in `equilibrium_phases` would precipitate in reality but was never removed, so the dissolved concentrations are an upper bound. |

Two further correctness notes on the implementation:

- Phases passed in `equilibrium_phases` are equilibrated with `in_phase = 0`, so they may only **precipitate**. `phreeqpython`'s default places 10 moles of the mineral in the system, which PHREEQC then dissolves — requesting `Zn(OH)2-e` as a treatment target injected 6513 mg/L of zinc into a solution that started with 131 mg/L.
- The lime dose is bracketed by coarse steps and then bisected onto the requested pH. Fixed stepping overshot pH 8.5 to pH 11.5, which changes which hydroxides precipitate and therefore the reported metal removal.

Verified end to end on a synthetic AMD (pH 2.8, Fe(3) 50 mmol, SO4 200 mmol, Al, Ca, Zn, Ni), titrated to pH 8.5 with Fe(OH)3(a), Gibbsite, Gypsum, Zn(OH)2-e, and Ni(OH)2 allowed to precipitate: achieved pH 8.52 with 99.7 mmol lime; Fe 2783 → 0.000 mg/L, Al 270 → 0.002, Ni 293 → 0.002, Zn 131 → 2.6 mg/L, sulfate 44.8% removed as gypsum.

The result is capped at `screening_only` and carries the limitation that this is equilibrium thermodynamics only: no reaction kinetics, no reactive transport, no field validation.





The report always exposes these limitations and does not emit a validated operational status without independent observations.

## MODFLOW 6 Groundwater (`modflow_groundwater`)

`modflow_groundwater` runs a real MODFLOW 6 simulation through FloPy (`scripts/modflow_run.py`), for aquifer drawdown, wellfield sustainability, and landfill/tailings groundwater screening.

Units are fixed and explicit: **metres and days**. Hydraulic conductivity is m/day, recharge is mm/yr converted internally, pumping is m3/day, and well indices are 1-based. The previous inline-Python implementation labelled conductivity `m/s` while running MODFLOW on a day time base and converted recharge as `mm/yr / 1000 / 365` while calling the result `m/s` — an error of roughly 10^5.

It also removed the analytical fallback. The old code substituted a Theis estimate whenever MODFLOW was missing, failed to converge, or produced unparseable output, so a broken model still returned a plausible drawdown number. A failed model is now an error.

Four gates travel with the result:

| Gate | What it catches |
| --- | --- |
| `converged` | Non-convergent heads are numerically meaningless. |
| `gate.percent_discrepancy` | MODFLOW's own volumetric budget error, checked against `mass_tolerance_pct` (default 1%). The groundwater analogue of the SWMM mass-balance gate. |
| `heads.dry_cell_count` | Dry and inactive cells carry ±1e30 sentinel heads. Averaged in blindly they destroy every head statistic, so they are excluded and counted. |
| `gate.wells_curtailed` | MODFLOW switches off a well whose cell goes dry. The budget then balances around a pump that extracted nothing, so `converged` and the discrepancy gate both pass while the requested scenario never ran. Requested extraction is compared against delivered extraction. |

A fifth signal, `gate.boundary_controlled`, flags runs where constant-head boundaries supply more than half the inflow — drawdown then reflects where the modeller drew the boundary rather than the aquifer.

Verified end to end on a 2-layer 20x20 grid at 100 m cells:

- Sustainable case (K 10 m/day, recharge 1800 mm/yr, 2000 m3/day well): converged, discrepancy −0.05%, 2000 m3 requested and 2000 m3 delivered, drawdown −1.20 m, gate passed.
- Over-pumped case (K 2 m/day, recharge 50 mm/yr, 8000 m3/day well): MODFLOW reported convergence and a budget discrepancy of only 0.42%, yet 82 cells went dry, the well head was a sentinel, and **0 m3 of the requested 8000 m3 was delivered**. The curtailment gate failed the result, which the convergence and budget checks alone would have passed.

The result is capped at `screening_only` and carries the limitation that the model is uncalibrated: conductivity, storage, and boundary heads were supplied by the caller, not fitted to observed heads. Raising it above screening requires `calibrate_and_validate` against measured heads.

## Pyrite Oxidation Kinetics (`pyrite_oxidation_kinetics`)

The AMD domain previously had two tools that could not answer the operative question. Static ABA screening (MPA/NAPP) says how much acid a rock *could* generate; `phreeqc_speciation` says what a given water *is*. Neither says how fast acid appears, which is what decides whether a pit needs treatment in month three or year thirty.

`pyrite_oxidation_kinetics` runs PHREEQC KINETICS with the Williamson & Rimstidt (1994) rate law, taken from the RATES block of the loaded database rather than reimplemented. It returns a pH / Fe / sulfate time series and the fraction of pyrite consumed.

Four guards travel with the result:

| Guard | What it catches |
| --- | --- |
| `oxygen_limited` | A closed system exhausts its dissolved O2 and the pH curve flattens. That plateau looks like a stable long-term outcome but is an artifact of the sealed box. |
| `pyrite_depleted` | The curve also flattens once the sulfide is spent — the opposite and physically real reason. The two must not be confused. |
| `stoichiometry_consistent` | FeS2 releases 2 mol S per mol Fe. When ferrihydrite precipitates the observed ratio reached 73,377, so dissolved Fe stops measuring how much pyrite oxidised. |
| `rate_is_laboratory_derived` | Always true and always reported: field rates are commonly one to two orders of magnitude slower than laboratory rates. |

A database portability trap is worth recording. `phreeqc.dat` and the repaired WATEQ4F database both implement Williamson & Rimstidt, but with different intercepts (−8.19 versus −10.19) and different meanings for `parm1`. The **same `-parms` values produce different rates in the two databases**, so the tool pins the database and reports its SHA-256 alongside the result.

Verified end to end (0.05 mol/kgw reactive pyrite, initial pH 6.5, one year):

- Open pit, atmosphere-connected: pH 4.97 → 2.99, S:Fe ratio exactly 2.00, 1.51% of pyrite consumed, all guards clear.
- Sealed system: pH flattens at 4.01 with a late change of −3.4e−5 → `oxygen_limited` true, result not interpretable.
- Calcite-buffered with ferrihydrite precipitating: pH held at 6.74, but S:Fe = 73,377 → `stoichiometry_consistent` false, dissolved Fe no longer tracks oxidation.

The result is capped at `screening_only`, and carries the further limitation that this is a single well-mixed batch: no gas diffusion through waste rock, no unsaturated flow, and no bacterial catalysis (*Acidithiobacillus*), all of which dominate real waste dumps.


## Reactive Transport 1D (`reactive_transport`)

`reactive_transport` is the spatial counterpart to `phreeqc_speciation` and `pyrite_oxidation_kinetics`: PHREEQC transports an influent through a mineral column and re-equilibrates each cell at each advective shift. The output is an outlet time series indexed by **pore volumes flushed**, not just a final number.

The tool makes the transport assumptions explicit:

- Pore velocity is derived as `cell_length / time_step`; PHREEQC's mixing-cell scheme advances one cell per shift. It is reported, not hidden.
- `grid_peclet = cell_length / dispersivity`. When it exceeds 2, numerical dispersion is larger than the physical dispersivity requested by the user; the result is flagged as grid-dominated.
- `front_traversed_column` is false until at least one pore volume has passed. A clean outlet before that is a simulation-too-short result, not evidence that a barrier works.
- `buffer_exhausted` detects when a reactive phase (e.g. Calcite) is spent at the outlet. This is a real breakthrough signal.
- `equilibrium_assumed_at_each_cell` is always true: this is thermodynamic equilibrium transport, not kinetic transport. Preferential flow, kinetic limitation, and 3D groundwater flow are not represented.

Verified on an AMD influent through a Calcite column: after 1 pore volume the outlet was partially buffered; by 2 pore volumes Calcite was exhausted and pH collapsed from approximately 7.9 to 2.5. With a short run of 0.5 pore volumes, `front_traversed_column: false` correctly warns that the apparently clean outlet is not interpretable. With dispersivity equal to half the cell length, grid Peclet is 2 and numerical-dispersion guard clears; with dispersivity one-tenth of the cell length, Peclet is 10 and the guard fails.

This is a **1D column executor**, not yet a 3D MODFLOW-GWT/PhreeqcRM coupling. The latter is the next spatial scale and requires a site-specific velocity field, porosity, dispersivity tensor, reaction cell mapping, and independent breakthrough observations.
