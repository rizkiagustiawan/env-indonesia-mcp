use std::process::Command;

pub fn flood_3d(dem_path: &str, output_path: &str, water_level: f64, title: &str, exaggeration: f64) -> String {
    let script = "/home/awan/Documents/env-ntb-mcp/src/tools/processing/flood_sim.py";
    match Command::new("python3").arg(script)
        .arg("--mode").arg("3d")
        .arg("--dem").arg(dem_path)
        .arg("--output").arg(output_path)
        .arg("--water_level").arg(water_level.to_string())
        .arg("--title").arg(title)
        .arg("--exaggeration").arg(exaggeration.to_string())
        .output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("{}\n{}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn flood_4d(dem_path: &str, output_path: &str, water_start: f64, water_end: f64, steps: u32, title: &str, exaggeration: f64) -> String {
    let script = "/home/awan/Documents/env-ntb-mcp/src/tools/processing/flood_sim.py";
    match Command::new("python3").arg(script)
        .arg("--mode").arg("4d")
        .arg("--dem").arg(dem_path)
        .arg("--output").arg(output_path)
        .arg("--water_start").arg(water_start.to_string())
        .arg("--water_end").arg(water_end.to_string())
        .arg("--steps").arg(steps.to_string())
        .arg("--title").arg(title)
        .arg("--exaggeration").arg(exaggeration.to_string())
        .output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("{}\n{}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}
