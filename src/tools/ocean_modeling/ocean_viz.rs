use std::process::Command;

fn run_ocean_viz(mode: &str, args: &[(&str, &str)]) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/ocean_modeling/ocean_viz.py";
    let mut cmd = Command::new("python3");
    cmd.arg(script).arg("--mode").arg(mode);
    for (k, v) in args {
        cmd.arg(format!("--{}", k)).arg(v);
    }
    match cmd.output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") {
                out
            } else {
                format!(
                    "ERROR [E502]: Python Engine Failed: {}\n{}",
                    out,
                    &err[..err.len().min(500)]
                )
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn bathymetry_3d(lat: f64, lon: f64, output: &str, title: &str) -> String {
    run_ocean_viz(
        "bathymetry3d",
        &[
            ("lat", &lat.to_string()),
            ("lon", &lon.to_string()),
            ("output", output),
            ("title", title),
        ],
    )
}

pub fn current_2d(
    lat: f64,
    lon: f64,
    wind_speed: f64,
    wind_dir: f64,
    output: &str,
    title: &str,
) -> String {
    run_ocean_viz(
        "current2d",
        &[
            ("lat", &lat.to_string()),
            ("lon", &lon.to_string()),
            ("wind_speed", &wind_speed.to_string()),
            ("wind_dir", &wind_dir.to_string()),
            ("output", output),
            ("title", title),
        ],
    )
}

pub fn thermal_3d(discharge_temp: f64, ambient_temp: f64, output: &str, title: &str) -> String {
    run_ocean_viz(
        "thermal3d",
        &[
            ("discharge_temp", &discharge_temp.to_string()),
            ("ambient_temp", &ambient_temp.to_string()),
            ("output", output),
            ("title", title),
        ],
    )
}

pub fn pollution_4d(current_speeds: &str, current_dirs: &str, output: &str, title: &str) -> String {
    run_ocean_viz(
        "pollution4d",
        &[
            ("current_speeds", current_speeds),
            ("current_dirs", current_dirs),
            ("output", output),
            ("title", title),
        ],
    )
}
