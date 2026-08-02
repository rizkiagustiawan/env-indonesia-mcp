use std::process::Command;
use crate::result_contract::ScientificResult;

fn run_landcover(args: &[&str]) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/gis/landcover_engine.py";
    match Command::new("python3").arg(script).args(args).output() {
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

fn run_landcover_json(args: &[&str]) -> Result<ScientificResult, String> {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/gis/landcover_engine.py";
    let mut full_args = args.to_vec();
    full_args.push("--json-result");

    match Command::new("python3").arg(script).args(&full_args).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            
            // Try to parse the last valid JSON object in stdout (Python script might print debug info before JSON)
            if let Some(json_start) = out.rfind('{') {
                if let Ok(res) = serde_json::from_str::<ScientificResult>(&out[json_start..]) {
                    return Ok(res);
                }
            }
            
            Err(format!("ERROR [E502]: Failed to parse JSON from Python Engine.\nStdout: {}\nStderr: {}", out, &err[..err.len().min(500)]))
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

pub fn classify(
    lat: f64,
    lon: f64,
    buffer_km: f64,
    start_date: &str,
    end_date: &str,
    output_path: &str,
) -> Result<ScientificResult, String> {
    run_landcover_json(&[
        "classify",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        start_date,
        end_date,
        output_path,
    ])
}

pub fn change_detection(
    lat: f64,
    lon: f64,
    buffer_km: f64,
    d1_start: &str,
    d1_end: &str,
    d2_start: &str,
    d2_end: &str,
    output_path: &str,
) -> String {
    run_landcover(&[
        "change",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        d1_start,
        d1_end,
        d2_start,
        d2_end,
        output_path,
    ])
}

pub fn accuracy_assessment(predicted_json: &str, actual_json: &str) -> String {
    run_landcover(&["accuracy", predicted_json, actual_json])
}

pub fn supervised_classify(
    lat: f64,
    lon: f64,
    buffer_km: f64,
    training_geojson: &str,
    start_date: &str,
    end_date: &str,
    n_trees: u32,
    output_path: &str,
) -> String {
    run_landcover(&[
        "supervised",
        &lat.to_string(),
        &lon.to_string(),
        &buffer_km.to_string(),
        training_geojson,
        start_date,
        end_date,
        &n_trees.to_string(),
        output_path,
    ])
}
