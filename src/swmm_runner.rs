//! Real EPA SWMM execution via a Python (`pyswmm`) subprocess wrapper.
//!
//! The Rust side never links SWMM directly. It shells out to
//! `scripts/swmm_run.py` with an argument vector (never a shell string),
//! bounds the runtime with a timeout, caps stdout, and parses exactly one
//! line of JSON into [`SwmmRunResult`].

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

/// Interpreter that has `pyswmm` installed. Override with
/// `ENV_INDONESIA_SWMM_PYTHON` for deployments with a different venv path.
pub const SWMM_PYTHON: &str = "/home/awan/env-indonesia/bin/python";

/// Absolute path of the Python wrapper, resolved at compile time so
/// `run_swmm` does not depend on the process working directory.
pub const SWMM_SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/swmm_run.py");

/// Resolve the interpreter, honouring the environment override.
pub fn swmm_python() -> String {
    std::env::var("ENV_INDONESIA_SWMM_PYTHON").unwrap_or_else(|_| SWMM_PYTHON.to_string())
}

/// Resolve the wrapper script, honouring the environment override.
pub fn swmm_script() -> String {
    std::env::var("ENV_INDONESIA_SWMM_SCRIPT").unwrap_or_else(|_| SWMM_SCRIPT.to_string())
}

/// Hard cap on captured stdout: 8 MiB.
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwmmRoutingBalance {
    pub external_inflow_m3: f64,
    pub flooding_m3: f64,
    pub outflow_m3: f64,
    pub initial_storage_m3: f64,
    pub final_storage_m3: f64,
    pub routing_error_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwmmNodeResult {
    pub node_id: String,
    pub flooding_volume_m3: f64,
    pub peak_flooding_rate_m3s: f64,
    pub flooding_duration_hr: f64,
    /// Structural full depth of the node (rim minus invert), NOT the simulated
    /// peak water depth. Used to sanity-check surcharge geometry only.
    pub max_depth_m: f64,
    pub invert_elevation_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwmmRunResult {
    pub status: String,
    pub pyswmm_version: String,
    pub inp_sha256: String,
    pub routing: SwmmRoutingBalance,
    #[serde(default)]
    pub nodes: Vec<SwmmNodeResult>,
}

/// Shape of the `{"status":"error"|"invalid_request","error":...}` payloads.
#[derive(Debug, Clone, Deserialize)]
struct SwmmFailure {
    status: String,
    #[serde(default)]
    error: String,
}

/// Validate that `path` exists, is a regular file, and ends in `.inp`.
pub fn validate_inp_path(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("SWMM inp path must not be empty".to_string());
    }
    let candidate = PathBuf::from(trimmed);
    let has_inp_extension = candidate
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("inp"))
        .unwrap_or(false);
    if !has_inp_extension {
        return Err(format!("SWMM inp path must end in .inp: {}", trimmed));
    }
    if !candidate.exists() {
        return Err(format!("SWMM inp path does not exist: {}", trimmed));
    }
    if !candidate.is_file() {
        return Err(format!("SWMM inp path is not a regular file: {}", trimmed));
    }
    Ok(candidate)
}

/// A routing continuity error is acceptable when its magnitude is within tolerance.
pub fn routing_error_acceptable(error_pct: f64, tolerance_pct: f64) -> bool {
    error_pct.is_finite() && error_pct.abs() <= tolerance_pct
}

/// Run a SWMM model to completion and return its routing/node statistics.
pub async fn run_swmm(inp_path: &str, timeout_secs: u64) -> Result<SwmmRunResult, String> {
    if timeout_secs == 0 {
        return Err("SWMM timeout_secs must be greater than zero".to_string());
    }
    let validated = validate_inp_path(inp_path)?;

    let python = swmm_python();
    let script = swmm_script();
    let mut command = tokio::process::Command::new(&python);
    command
        .arg(&script)
        .arg("--inp")
        .arg(&validated)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let child = command
        .spawn()
        .map_err(|error| format!("Failed to spawn SWMM runner ({}): {}", python, error))?;

    let output = tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output())
        .await
        .map_err(|_| format!("SWMM run exceeded {}s timeout", timeout_secs))?
        .map_err(|error| format!("SWMM runner failed: {}", error))?;

    if output.stdout.len() > MAX_STDOUT_BYTES {
        return Err(format!(
            "SWMM runner stdout exceeded {} bytes",
            MAX_STDOUT_BYTES
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().find(|line| !line.trim().is_empty()).unwrap_or("").trim();

    if line.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "SWMM runner produced no JSON (exit {:?}): {}",
            output.status.code(),
            stderr.trim()
        ));
    }

    if !output.status.success() {
        let detail = serde_json::from_str::<SwmmFailure>(line)
            .map(|failure| format!("{}: {}", failure.status, failure.error))
            .unwrap_or_else(|_| line.to_string());
        return Err(format!(
            "SWMM runner exited with {:?}: {}",
            output.status.code(),
            detail
        ));
    }

    let result: SwmmRunResult = serde_json::from_str(line)
        .map_err(|error| format!("SWMM runner emitted unparseable JSON: {}", error))?;

    if result.status != "ok" {
        return Err(format!("SWMM run status was {}", result.status));
    }

    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_documented_contract() {
        let json = r#"{"status":"ok","pyswmm_version":"2.1.0","inp_sha256":"aa","routing":{"external_inflow_m3":1800.03,"flooding_m3":1231.92,"outflow_m3":568.4,"initial_storage_m3":0.0069,"final_storage_m3":0.157,"routing_error_pct":-0.0164},"nodes":[{"node_id":"J1","flooding_volume_m3":1231.88,"peak_flooding_rate_m3s":0.3666,"flooding_duration_hr":1.11,"max_depth_m":2.0,"invert_elevation_m":10.0}]}"#;
        let parsed: SwmmRunResult = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.nodes[0].node_id, "J1");
        assert!((parsed.routing.flooding_m3 - 1231.92).abs() < 1e-6);
    }

    #[test]
    fn rejects_non_inp_path() {
        assert!(validate_inp_path("/tmp/model.txt").is_err());
        assert!(validate_inp_path("/tmp/does-not-exist.inp").is_err());
    }

    #[test]
    fn routing_error_beyond_tolerance_is_flagged() {
        assert!(!routing_error_acceptable(-12.0, 1.0));
        assert!(routing_error_acceptable(-0.016, 1.0));
    }

    #[test]
    fn wrapper_script_path_is_absolute_and_present() {
        let script = std::path::Path::new(SWMM_SCRIPT);
        assert!(script.is_absolute(), "SWMM_SCRIPT must not depend on CWD");
        assert!(script.is_file(), "missing wrapper script at {}", SWMM_SCRIPT);
    }

    #[tokio::test]
    async fn zero_timeout_is_rejected_before_spawning() {
        let error = run_swmm("/tmp/swmmtest/min.inp", 0).await.unwrap_err();
        assert!(error.contains("timeout_secs"), "unexpected error: {error}");
    }
}
