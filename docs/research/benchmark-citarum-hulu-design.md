# Benchmark Citarum Hulu: Flood Screening and Evidence Validation

Status: design frozen before execution
Date: 2026-08-22

## Goal

Evaluate whether the environmental workflow can perform reproducible flood
occurrence screening for Citarum Hulu while keeping occurrence labels,
spatial flood extent, model inputs, and scientific claims separate.

This is a benchmark of an auditable decision-support workflow. It is not a
claim of pixel-level flood-map truth and not an operational warning system.

## Scope

- Primary AOI: Citarum Hulu / upper Bandung basin, using an official BBWS
  Citarum or BIG/data.go.id boundary artifact.
- Primary event: 2016-03-13, Bandung Selatan flood associated with Citarum
  overflow.
- Holdout event: 2025-03-15, Bandung Regency flood affecting multiple
  tributaries and four subdistrict groups.
- Primary analysis: flood occurrence screening using conditioned DEM,
  rainfall context, and Sentinel-1/Sentinel-2 evidence.
- Output status: `screening_only` unless independent spatial flood extent and
  sufficient observations are supplied.

## Evidence Roles

| Evidence | Role | Not allowed to claim |
|---|---|---|
| DIBI/BNPB event record | Event occurrence and date candidate | Pixel-level flood extent |
| BBWS/official basin boundary | AOI definition | Exact inundation boundary |
| BMKG station observations | Point rainfall validation/calibration | Complete spatial rainfall field |
| GSMaP/ERA5 | Spatial forcing/context | Ground truth observation |
| DEMNAS | Elevation input | Flood truth or absolute local elevation accuracy |
| RBI hydrography | DEM conditioning input | Complete drainage network |
| Sentinel-1 flood mask | Derived spatial observation | Unbiased ground truth without validation |
| Sentinel-2 land cover | Derived context/land-cover input | Ground truth without independent labels |
| InaRISK hazard | Independent consistency check | Event-specific observed extent |

## Frozen Inputs

Before any model run, record and hash:

- exact AOI geometry and source URL;
- event date and allowed temporal window;
- satellite collection, item IDs, acquisition timestamps, and cloud mask;
- DEM source, tile IDs, vertical datum, conditioning operations, and output
  hash;
- rainfall source, station IDs or grid product, units, aggregation window,
  and missing-value policy;
- model parameters, including Manning roughness, inflow basis, timestep,
  threshold, and boundary condition;
- software commit, environment, random seed, and execution receipt.

No threshold, input window, or evaluation metric may be selected after
reviewing the model output.

## Baselines

1. Terrain-only baseline: conditioned DEM / HAND or elevation-threshold
   screening with no learned model.
2. SAR observation baseline: Sentinel-1 change/backscatter screening with
   explicit orbit, polarization, calibration, and terrain-correction metadata.
3. Optional hydraulic baseline: existing 2D SWE screening only when discharge
   and boundary conditions are explicit. It must remain `screening_only`.

## Metrics

The primary benchmark metrics are separated by evidence level:

- Event-level occurrence: whether the workflow identifies the documented
  event within the frozen date window.
- Administrative-location agreement: overlap between predicted affected
  area and documented affected villages/subdistricts. This is coarse
  validation, not polygon ground truth.
- Spatial mask metrics: CSI, POD, FAR, and IoU only when an independently
  sourced flood-extent mask is available and its provenance is recorded.
- Hydrologic metrics: RMSE/KGE/NSE only when paired observed discharge or
  stage data exist with aligned timestamps and documented station metadata.
- Reliability: percentage of reported numeric claims found in the immutable
  execution receipt; percentage of runs passing CRS, temporal, provenance,
  and mass-balance gates.

## Required Negative Controls

The benchmark harness must reject or downgrade runs that contain:

- a missing or stale source record;
- a DEM without CRS or vertical datum;
- an invalid or reversed bbox;
- a rainfall series with incompatible temporal aggregation;
- a claimed flood polygon presented as DIBI ground truth;
- a reported number absent from the execution receipt;
- a failed mass-balance gate;
- random-pixel validation without spatial holdout disclosure;
- a causal claim inferred only from spatial correlation;
- a `validated` status without independent observations.

## Acceptance Criteria

The benchmark is not passed by producing a visually plausible map. A run is
accepted only when:

1. all frozen inputs and parameters are recorded;
2. deterministic validation gates pass;
3. all claims are traceable to executed artifacts;
4. the event-level result is correctly classified or explicitly abstains;
5. the report distinguishes event occurrence from spatial extent;
6. uncertainty and limitations are present;
7. the result never exceeds `screening_only` without independent paired
   observations and a documented spatial validation artifact.

## Known Limitations

- The 2016 official source provides detailed locations and water-depth
  descriptions but not a pixel-level inundation polygon.
- The 2025 event involves multiple tributaries and is not a pure single-river
  Citarum overflow event.
- BMKG free historical access is limited; long-term station data may require
  PTSP access.
- DEMNAS quality and drainage representation vary by tile.
- Satellite-derived masks are observations requiring their own validation,
  not automatically ground truth.

## Sources

- [Ditjen SDA: Banjir Kabupaten Bandung, 13 March 2016](https://sda.pu.go.id/post/detail/upaya_penanggulangan_bencana_banjir_kabupaten_bandung)
- [BNPB: Banjir Kabupaten Bandung, 15 March 2025](https://www.bnpb.go.id/index.php/berita/banjir-rendam-empat-kecamatan-di-kabupaten-bandung-ratusan-warga-mengungsi)
- [DIBI BNPB](https://dibi.bnpb.go.id/)
- [Peta Batas DAS Citarum, data.go.id](https://data.go.id/dataset/dataset/peta-batas-das-citarum-skala-1-250000)
- [BBWS Citarum DAS publication](https://sda.pu.go.id/balai/bbwscitarum/publikasi-aset/aset-daerah-aliran-sungai)
