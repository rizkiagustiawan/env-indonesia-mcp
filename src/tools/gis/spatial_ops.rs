use std::process::Command;

fn run_spatial(args: &[&str]) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/gis/spatial_engine.py";
    match Command::new("python3").arg(script).args(args).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("{}\nStderr: {}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn buffer(geojson: &str, distance_m: f64, output_path: &str) -> String {
    run_spatial(&["buffer", geojson, &distance_m.to_string(), output_path])
}

pub fn overlay(geojson_a: &str, geojson_b: &str, operation: &str, output_path: &str) -> String {
    run_spatial(&["overlay", geojson_a, geojson_b, operation, output_path])
}

pub fn suitability(criteria_json: &str, lat: f64, lon: f64, buffer_km: f64, output_path: &str) -> String {
    run_spatial(&["suitability", criteria_json, &lat.to_string(), &lon.to_string(), &buffer_km.to_string(), output_path])
}
