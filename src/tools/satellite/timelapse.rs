use std::process::Command;

pub fn generate_4d_timelapse(
    lat: f64,
    lon: f64,
    buffer_km: f64,
    start_year: u32,
    end_year: u32,
    sensor: &str,
    output_path: &str,
) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/satellite/timelapse_engine.py";

    let sensor_type =
        if sensor.to_lowercase().contains("radar") || sensor.to_lowercase().contains("s1") {
            "radar_s1"
        } else {
            "optik_s2"
        };

    match Command::new("python3")
        .arg(script)
        .arg("--lat")
        .arg(lat.to_string())
        .arg("--lon")
        .arg(lon.to_string())
        .arg("--buffer_km")
        .arg(buffer_km.to_string())
        .arg("--start_year")
        .arg(start_year.to_string())
        .arg("--end_year")
        .arg(end_year.to_string())
        .arg("--sensor")
        .arg(sensor_type)
        .arg("--output")
        .arg(output_path)
        .output()
    {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if out.contains("SUCCESS") {
                out
            } else {
                format!(
                    "ERROR [E502]: Python Engine Failed: {}\nStderr: {}",
                    out,
                    &err[..err.len().min(500)]
                )
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}
