use std::process::Command;

fn run_sar_engine(args: &[&str]) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/satellite/sar_engine.py";
    match Command::new("python3").arg(script).args(args).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("ERROR [E502]: Python Engine Failed: {}\nStderr: {}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}

/// Sentinel-1 SAR flood detection using pre/post event change detection
pub fn flood_detection(lat: f64, lon: f64, buffer_km: f64, pre_date: &str, post_date: &str, output_path: &str) -> String {
    run_sar_engine(&["flood", &lat.to_string(), &lon.to_string(), &buffer_km.to_string(), pre_date, post_date, output_path])
}

/// Sentinel-1 SAR deforestation detection via temporal backscatter loss
pub fn deforestation(lat: f64, lon: f64, buffer_km: f64, start_date: &str, end_date: &str, output_path: &str) -> String {
    run_sar_engine(&["deforestation", &lat.to_string(), &lon.to_string(), &buffer_km.to_string(), start_date, end_date, output_path])
}

/// Local SAR image analysis (from downloaded GeoTIFF)
pub fn local_analysis(input_path: &str, output_path: &str, analysis_type: &str) -> String {
    run_sar_engine(&["local", input_path, output_path, analysis_type])
}

/// Simplified InSAR-like subsidence screening using Sentinel-1
pub fn subsidence_insar(lat: f64, lon: f64, buffer_km: f64, start_date: &str, end_date: &str, output_path: &str) -> String {
    run_sar_engine(&["subsidence", &lat.to_string(), &lon.to_string(), &buffer_km.to_string(), start_date, end_date, output_path])
}
