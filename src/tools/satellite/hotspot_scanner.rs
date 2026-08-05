use std::process::Command;

pub fn scan_hotspots(min_frp: f64, days_back: u32) -> String {
    let script =
        "/home/awan/Documents/env-indonesia-mcp/src/tools/satellite/hotspot_scanner.py";
    match Command::new("python3")
        .arg(script)
        .arg("--min_frp")
        .arg(min_frp.to_string())
        .arg("--days_back")
        .arg(days_back.to_string())
        .output()
    {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if o.status.success() {
                out
            } else {
                format!("ERROR: {}\n{}", out, &err[..err.len().min(500)])
            }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}
