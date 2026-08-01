use reqwest::Client;
use std::process::Command;

pub async fn query(_client: &Client, lat: f64, lon: f64) -> String {
    let script =
        "/home/awan/Documents/env-indonesia-mcp/src/tools/satellite/satellite_query_engine.py";
    match Command::new("python3")
        .arg(script)
        .arg("modis")
        .arg(lat.to_string())
        .arg(lon.to_string())
        .output()
    {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") {
                out
            } else {
                format!("ERROR: {}\n{}", err, out)
            }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}
