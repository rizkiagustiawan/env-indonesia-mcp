use std::process::Command;

pub fn generate(title: &str, sections_json: &str, output_path: &str) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/processing/pdf_report.py";
    match Command::new("python3")
        .arg(script)
        .arg("--title")
        .arg(title)
        .arg("--sections")
        .arg(sections_json)
        .arg("--output")
        .arg(output_path)
        .output()
    {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if !err.is_empty() && !out.contains("SUCCESS") {
                format!("Error: {}\n{}", err, out)
            } else {
                out
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}
