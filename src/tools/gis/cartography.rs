use std::process::Command;

/// Generate SNI 6502:2010 compliant map layout
/// 13 mandatory elements: title, scale bar, numeric scale, legend, north arrow,
/// coordinate grid, inset map, CRS info, data source, date, author, admin boundaries, map frame
pub fn generate_map(geojson: &str, output_path: &str, title: &str, realtime: bool,
                    author: Option<&str>, date: Option<&str>, show_admin: bool) -> String {
    let py_script = "/home/awan/Documents/env-indonesia-mcp/src/tools/gis/cartography.py";
    
    let mut cmd = Command::new("python3");
    cmd.arg(py_script)
       .arg("--geojson").arg(geojson)
       .arg("--output").arg(output_path)
       .arg("--title").arg(title);
       
    if realtime {
        cmd.arg("--realtime");
    }
    
    if let Some(a) = author {
        cmd.arg("--author").arg(a);
    }
    
    if let Some(d) = date {
        cmd.arg("--date").arg(d);
    }
    
    if !show_admin {
        cmd.arg("--no-admin");
    }
    
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stdout.contains("SUCCESS") { stdout }
            else { format!("{}\nStderr: {}", stdout, &stderr[..stderr.len().min(500)]) }
        }
        Err(e) => format!("ERROR [E502]: Cartography engine gagal: {}", e),
    }
}
