use std::process::Command;

pub fn render(dem_path: &str, output_path: &str, title: &str, exaggeration: f64) -> String {
    let script = "/home/awan/Documents/env-ntb-mcp/src/tools/processing/terrain3d.py";
    match Command::new("python3").arg(script)
        .arg("--dem").arg(dem_path)
        .arg("--output").arg(output_path)
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
