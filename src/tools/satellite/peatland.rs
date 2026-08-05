use serde_json::json;
use std::process::Command;

pub fn classify_peatland_fire(lat: f64, lon: f64) -> String {
    let script_path = "src/tools/satellite/peatland_classifier.py";
    
    let output = Command::new("python3")
        .arg(script_path)
        .arg("--lat")
        .arg(lat.to_string())
        .arg("--lon")
        .arg(lon.to_string())
        .output();
        
    match output {
        Ok(out) => {
            if out.status.success() {
                String::from_utf8_lossy(&out.stdout).to_string()
            } else {
                format!("Error executing python script: {}", String::from_utf8_lossy(&out.stderr))
            }
        },
        Err(e) => format!("Failed to run python3: {}", e)
    }
}
