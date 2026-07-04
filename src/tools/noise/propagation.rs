use std::process::Command;

fn run_noise_engine(args: &[&str]) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/noise/noise_engine.py";
    match Command::new("python3").arg(script).args(args).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") { out } else { format!("{}\nStderr: {}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}

/// Render 2D noise contour map
/// sources_json: [{"x_m": 0, "y_m": 0, "power_db": 95, "type": "point"}, ...]
/// barrier_json: [{"x1": 100, "y1": -50, "x2": 100, "y2": 50, "height_m": 3.0, "il_db": 10}] or "[]"
pub fn render_2d(sources_json: &str, output_path: &str, title: &str, grid_size: u32, barrier_json: &str) -> String {
    run_noise_engine(&["2d", sources_json, output_path, title, &grid_size.to_string(), barrier_json])
}

/// Render 3D surface plot of noise levels
pub fn render_3d(sources_json: &str, output_path: &str, title: &str, grid_size: u32) -> String {
    run_noise_engine(&["3d", sources_json, output_path, title, &grid_size.to_string()])
}
