use std::process::Command;

pub fn render_2d(sources_json: &str, wind_speed: f64, wind_dir: f64, stability: &str, output: &str, title: &str, grid_size: u32) -> String {
    run_engine("2d", sources_json, wind_speed, wind_dir, stability, output, title, grid_size, "", "")
}

pub fn render_3d(sources_json: &str, wind_speed: f64, wind_dir: f64, stability: &str, output: &str, title: &str, grid_size: u32) -> String {
    run_engine("3d", sources_json, wind_speed, wind_dir, stability, output, title, grid_size, "", "")
}

pub fn render_4d(sources_json: &str, wind_speeds: &str, wind_dirs: &str, stability: &str, output: &str, title: &str, grid_size: u32) -> String {
    run_engine("4d", sources_json, 0.0, 0.0, stability, output, title, grid_size, wind_speeds, wind_dirs)
}

fn run_engine(mode: &str, sources: &str, ws: f64, wd: f64, stab: &str, output: &str, title: &str, grid: u32, ws_list: &str, wd_list: &str) -> String {
    let script = "/home/awan/Documents/env-ntb-mcp/src/tools/airquality/dispersion_engine.py";
    let mut cmd = Command::new("python3");
    cmd.arg(script)
       .arg("--mode").arg(mode)
       .arg("--sources").arg(sources)
       .arg("--stability").arg(stab)
       .arg("--output").arg(output)
       .arg("--title").arg(title)
       .arg("--grid_size").arg(grid.to_string())
       .arg("--resolution").arg("100");

    if mode == "4d" {
        cmd.arg("--wind_speeds").arg(ws_list);
        cmd.arg("--wind_dirs").arg(wd_list);
    } else {
        cmd.arg("--wind_speed").arg(ws.to_string());
        cmd.arg("--wind_dir").arg(wd.to_string());
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
