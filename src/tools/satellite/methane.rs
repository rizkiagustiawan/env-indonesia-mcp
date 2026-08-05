use std::process::Command;

/// Query Sentinel-5P TROPOMI CH4 data for a specific location
pub fn query_methane(lat: f64, lon: f64, buffer_km: f64, start_date: &str, end_date: &str) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/satellite/methane_engine.py";
    match Command::new("python3")
        .arg(script)
        .arg("--lat").arg(lat.to_string())
        .arg("--lon").arg(lon.to_string())
        .arg("--buffer_km").arg(buffer_km.to_string())
        .arg("--start_date").arg(start_date)
        .arg("--end_date").arg(end_date)
        .output()
    {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if o.status.success() {
                out
            } else {
                format!(
                    "ERROR: {}\n{}",
                    out,
                    &err[..err.len().min(500)]
                )
            }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}

/// Scan seluruh Indonesia untuk hotspot metana (CH4 > 1950 ppb)
pub fn scan_indonesia() -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/satellite/methane_engine.py";
    match Command::new("python3")
        .arg(script)
        .arg("--scan_indonesia")
        .output()
    {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if o.status.success() {
                out
            } else {
                format!(
                    "ERROR: {}\n{}",
                    out,
                    &err[..err.len().min(500)]
                )
            }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}
