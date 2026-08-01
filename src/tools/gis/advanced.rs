use std::process::Command;

fn run_raster_engine(args: &[&str]) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/gis/raster_engine.py";
    match Command::new("python3").arg(script).args(args).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") || out.contains("Zone") {
                out
            } else {
                format!(
                    "ERROR [E502]: Python Engine Failed: {}\nStderr: {}",
                    out,
                    &err[..err.len().min(500)]
                )
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

/// Real DEM slope analysis via GEE SRTM
pub fn dem_slope(lat: f64, lon: f64, buffer_km: f64, output_path: &str) -> String {
    run_raster_engine(&[
        "dem_slope",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        output_path,
    ])
}

/// Real DEM aspect analysis via GEE SRTM
pub fn dem_aspect(lat: f64, lon: f64, buffer_km: f64, output_path: &str) -> String {
    run_raster_engine(&[
        "dem_aspect",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        output_path,
    ])
}

/// Real DEM hillshade via GEE SRTM
pub fn dem_hillshade(lat: f64, lon: f64, buffer_km: f64, output_path: &str) -> String {
    run_raster_engine(&[
        "dem_hillshade",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        output_path,
    ])
}

/// Real zonal statistics via GEE reduceRegion
pub fn raster_stats(
    image_id: &str,
    band: &str,
    geojson: &str,
    lat: f64,
    lon: f64,
    buffer_km: f64,
    output_path: &str,
) -> String {
    run_raster_engine(&[
        "zonal_gee",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        image_id,
        band,
        geojson,
        output_path,
    ])
}

/// Real raster band math (NDVI/NDWI/SAVI/EVI/MNDWI/NDBI) via GEE Sentinel-2
pub fn band_math_gee(
    lat: f64,
    lon: f64,
    buffer_km: f64,
    index_type: &str,
    start_date: &str,
    end_date: &str,
    output_path: &str,
) -> String {
    run_raster_engine(&[
        "band_math_gee",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        index_type,
        start_date,
        end_date,
        output_path,
    ])
}

/// Raster band math on local GeoTIFF file
pub fn band_math_local(input_path: &str, expression: &str, output_path: &str) -> String {
    run_raster_engine(&["band_math_local", input_path, expression, output_path])
}

/// Zonal statistics on local files
pub fn zonal_stats_local(raster_path: &str, vector_path: &str, stats: &str) -> String {
    run_raster_engine(&["zonal_local", raster_path, vector_path, stats])
}

/// Topographic C-correction for Sentinel-2 (Teillet et al. 1982)
pub fn topo_correction(
    lat: f64,
    lon: f64,
    buffer_km: f64,
    start_date: &str,
    end_date: &str,
    output_path: &str,
) -> String {
    run_raster_engine(&[
        "topo_correct",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        start_date,
        end_date,
        output_path,
    ])
}

/// NDVI annual trend analysis (Saifulloh et al. 2025)
pub fn ndvi_timeseries(
    lat: f64,
    lon: f64,
    buffer_km: f64,
    start_year: i32,
    end_year: i32,
    output_path: &str,
) -> String {
    run_raster_engine(&[
        "ndvi_timeseries",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        &start_year.to_string(),
        &end_year.to_string(),
        output_path,
    ])
}
