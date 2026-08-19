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

- 1D sewer and 2D surface coupling;
- rainfall-runoff calibration;
- PHREEQC script generation only (no reactive transport module execution or PhreeqcRM coupling);
- AMD kinetic tests or field reactive transport;
- trained FNO/PINO/PINN inference (models run conceptually but lack trained checkpoints, fallback mechanisms govern current behavior);
- calibration, independent validation, or parameter ensembles.

The report always exposes these limitations and does not emit a validated operational status without independent observations.
