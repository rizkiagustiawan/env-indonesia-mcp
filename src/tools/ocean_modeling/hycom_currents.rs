use std::process::Command;

pub fn fetch_currents(lat: f64, lon: f64, buffer_km: f64) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/ocean_modeling/hycom_currents.py";
    match Command::new("python3")
        .arg(script)
        .arg("--lat").arg(lat.to_string())
        .arg("--lon").arg(lon.to_string())
        .arg("--buffer_km").arg(buffer_km.to_string())
        .output()
    {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if o.status.success() { out } else { format!("ERROR: {}\n{}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}
