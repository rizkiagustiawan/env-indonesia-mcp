use std::process::Command;
use serde::{Deserialize, Serialize};
use rmcp::schemars::{self, JsonSchema};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SoilGridsParam {
    #[schemars(description = "Latitude of the point")]
    pub lat: f64,
    #[schemars(description = "Longitude of the point")]
    pub lon: f64,
}

pub fn fetch(p: &SoilGridsParam) -> String {
    let script = "src/tools/datasources/soilgrids_engine.py";
    let out = Command::new("python3")
        .arg(script)
        .arg(p.lat.to_string())
        .arg(p.lon.to_string())
        .output();
    match out {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
