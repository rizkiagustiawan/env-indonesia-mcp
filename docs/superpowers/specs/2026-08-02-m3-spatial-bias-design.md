# M3 Spatial Bias Validation Design

## Goal

Add spatial bias prevention to the `env-indonesia-mcp` tools, enforcing that predictive models and geo-analytics reject inputs when spatial autocorrelation, temporal mismatch, or sensor disparity invalidate the environmental assessment.

## Context

Recent literature (2026) demonstrates that standard statistical validation is insufficient for geospatial models due to spatial autocorrelation and sensor disparity. If data is spatially biased or temporally mismatched, predictions (e.g., land cover abandonment, flood risk) are artificially inflated or incorrect.

## Metrics & Validations

We add a new Rust module `src/tools/spatial_validation.rs` containing rigorous validation checks:

1.  **Spatial Autocorrelation Threshold (Moran's I Proxy / Separation)**
    *   Ensure training and testing polygons/areas are spatially separated.
    *   If a tool receives point/polygon inputs that are too densely clustered without independent validation regions, it should fail with a spatial bias error.
    *   *Implementation*: A simple point-distance check `check_spatial_independence(points, min_distance_m)` that fails if points are clustered within the minimum threshold.

2.  **Temporal Mismatch Flag**
    *   Ensure the temporal range of the input data aligns with the target phenomenon.
    *   *Implementation*: `check_temporal_alignment(data_season, target_season)` failing if there's a strict mismatch (e.g., using dry season data for a wet season flood prediction).

3.  **Sensor Disparity Guard**
    *   Reject operations where the sensor resolution is too coarse for the requested analysis scale (e.g., using 30m Landsat for a 0.1ha paddy field).
    *   *Implementation*: `check_sensor_resolution(resolution_m, area_sqm)` failing if the pixel size exceeds a threshold percentage of the target area.

## Integration

These validators will return specific `Result<(), String>` errors. We will write unit tests for these validators. In a future PR, they will be wired directly into the GIS tools (like `landcover_engine.py` or the Rust physics validators).
