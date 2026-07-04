use std::process::Command;

/// Map burned area using Sentinel-2 dNBR (differenced Normalized Burn Ratio)
/// Severity classification per USGS standards
pub fn map_burned_area(lat: f64, lon: f64, buffer_km: f64, fire_date: &str, output_path: &str) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/satellite/sar_engine.py";
    match Command::new("python3").arg(script)
        .arg("burned_area").arg(lat.to_string()).arg(lon.to_string())
        .arg(buffer_km.to_string()).arg(fire_date).arg(output_path)
        .output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("{}\nStderr: {}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}
