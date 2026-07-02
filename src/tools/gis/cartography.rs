use std::process::Command;

pub fn generate_map(geojson: &str, output_path: &str, title: &str, realtime: bool) -> String {
    let py_script = "/home/awan/Documents/env-ntb-mcp/src/tools/gis/cartography.py";
    
    let mut cmd = Command::new("python3");
    cmd.arg(py_script)
       .arg("--geojson").arg(geojson)
       .arg("--output").arg(output_path)
       .arg("--title").arg(title);
       
    if realtime {
        cmd.arg("--realtime");
    }
    
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                format!("Error/Log: {}\n{}", stderr, stdout)
            } else {
                stdout.to_string()
            }
        }
        Err(e) => format!("Gagal memanggil Python Cartography Engine: {}", e),
    }
}
