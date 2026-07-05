use std::process::Command;

pub fn analyze(dem_path: &str, observer_lat: f64, observer_lon: f64, observer_height_m: f64, max_distance_m: f64, output_path: &str) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/gis/viewshed_engine.py";
    match Command::new("python3").arg(script)
        .arg(dem_path)
        .arg(observer_lat.to_string()).arg(observer_lon.to_string())
        .arg(observer_height_m.to_string()).arg(max_distance_m.to_string())
        .arg(output_path)
        .output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("ERROR [E502]: Python Engine Failed: {}\nStderr: {}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}
