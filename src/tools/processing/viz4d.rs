use std::process::Command;

pub fn terrain_rotation(dem_path: &str, output_path: &str, title: &str, exaggeration: f64, frames: u32) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/processing/viz4d.py";
    match Command::new("python3").arg(script)
        .arg("--mode").arg("terrain")
        .arg("--dem").arg(dem_path)
        .arg("--output").arg(output_path)
        .arg("--title").arg(title)
        .arg("--exaggeration").arg(exaggeration.to_string())
        .arg("--frames").arg(frames.to_string())
        .output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("{}\n{}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn timeseries_animation(values: &str, labels: &str, output_path: &str, title: &str, ylabel: &str) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/processing/viz4d.py";
    let mut cmd = Command::new("python3");
    cmd.arg(script)
       .arg("--mode").arg("timeseries")
       .arg("--values").arg(values)
       .arg("--output").arg(output_path)
       .arg("--title").arg(title)
       .arg("--ylabel").arg(ylabel);
    if !labels.is_empty() {
        cmd.arg("--labels").arg(labels);
    }
    match cmd.output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("{}\n{}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}
