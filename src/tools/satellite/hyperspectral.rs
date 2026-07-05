use std::process::Command;

pub fn extract_signature(lat: f64, lon: f64, output_path: &str) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/satellite/hyperspectral_engine.py";
    match Command::new("python3").arg(script)
        .arg("--lat").arg(lat.to_string())
        .arg("--lon").arg(lon.to_string())
        .arg("--output").arg(output_path)
        .output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("ERROR [E502]: Python Engine Failed: {}\nStderr: {}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error mengeksekusi hyperspectral engine: {}", e),
    }
}
