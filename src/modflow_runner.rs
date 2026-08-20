//! Real MODFLOW 6 groundwater execution via a Python (FloPy) subprocess.
//!
//! Replaces the previous inline-Python path in `tools/water/modflow_3d.rs`,
//! which had two defects that produced confident wrong numbers:
//!
//! * It labelled hydraulic conductivity `m/s` but fed MODFLOW a model whose
//!   time unit was days, and converted recharge as `mm/yr -> /1000/365` while
//!   calling the result `m/s`. Units are now fixed and explicit: metres, days.
//! * It fell back to an analytical Theis solution whenever MODFLOW failed, so a
//!   non-convergent or unavailable model still returned a plausible drawdown.
//!   Failure is now an error, not a silent substitution.
//!
//! Four honesty guards travel with every result:
//!
//! * `converged` — non-convergent heads are meaningless.
//! * `gate.percent_discrepancy` — MODFLOW's own volumetric budget error, the
//!   groundwater analogue of the SWMM mass-balance gate.
//! * `heads.dry_cell_count` — dry/inactive cells carry ±1e30 sentinels that
//!   destroy head statistics if averaged; they are excluded and counted.
//! * `gate.wells_curtailed` — MODFLOW switches off a well whose cell goes dry.
//!   The budget then balances around a pump that extracted nothing while
//!   `converged` and the discrepancy gate both pass. Requested extraction is
//!   compared against delivered extraction to catch it.
//! * `gate.boundary_controlled` — when constant-head boundaries supply most of
//!   the inflow, drawdown reflects where the modeller drew the boundary.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use crate::swmm_runner::swmm_python;

/// Absolute path of the Python wrapper, resolved at compile time.
pub const MODFLOW_SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/modflow_run.py");

/// Hard cap on captured stdout: 8 MiB.
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;

/// Resolve the wrapper script, honouring an environment override.
pub fn modflow_script() -> String {
    std::env::var("ENV_INDONESIA_MODFLOW_SCRIPT").unwrap_or_else(|_| MODFLOW_SCRIPT.to_string())
}

/// The interpreter is the same venv that carries `flopy`.
pub fn modflow_python() -> String {
    swmm_python()
}

/// Directory holding the `mf6` executable, prepended to PATH for the child.
pub fn modflow_bin_dir() -> Option<String> {
    std::env::var("ENV_INDONESIA_MODFLOW_BIN").ok().or_else(|| {
        PathBuf::from(swmm_python())
            .parent()
            .map(|dir| dir.to_string_lossy().to_string())
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WellSpec {
    /// 1-based layer index.
    pub layer: u32,
    /// 1-based row index.
    pub row: u32,
    /// 1-based column index.
    pub col: u32,
    /// Positive extraction rate in m3/day.
    pub rate_m3_day: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ModflowRequest {
    pub nlay: u32,
    pub nrow: u32,
    pub ncol: u32,
    pub cell_size_m: f64,
    pub top_m: f64,
    /// One bottom elevation per layer, strictly decreasing from `top_m`.
    pub layer_bottoms_m: Vec<f64>,
    /// Horizontal hydraulic conductivity in m/day.
    pub hk_m_day: f64,
    /// Vertical hydraulic conductivity in m/day.
    pub vk_m_day: f64,
    pub sy: f64,
    pub ss_per_m: f64,
    pub initial_head_m: f64,
    pub boundary_head_m: f64,
    pub recharge_mm_yr: f64,
    #[serde(default)]
    pub wells: Vec<WellSpec>,
    #[serde(default = "default_true")]
    pub steady_state: bool,
    #[serde(default)]
    pub duration_days: Option<f64>,
    #[serde(default)]
    pub mass_tolerance_pct: Option<f64>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadSummary {
    pub available: bool,
    pub dry_cell_count: u64,
    pub active_cell_count: u64,
    #[serde(default)]
    pub grid_shape: Vec<u32>,
    #[serde(default)]
    pub min_head_m: Option<f64>,
    #[serde(default)]
    pub max_head_m: Option<f64>,
    #[serde(default)]
    pub mean_head_m: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellResult {
    pub layer: u32,
    pub row: u32,
    pub col: u32,
    pub rate_m3_day: f64,
    #[serde(default)]
    pub head_m: Option<f64>,
    #[serde(default)]
    pub drawdown_m: Option<f64>,
    pub cell_is_dry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummary {
    pub available: bool,
    #[serde(default)]
    pub cumulative_m3: BTreeMap<String, f64>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModflowGate {
    #[serde(default)]
    pub percent_discrepancy: Option<f64>,
    pub tolerance_pct: f64,
    pub gate_passed: bool,
    #[serde(default)]
    pub boundary_inflow_fraction: Option<f64>,
    #[serde(default)]
    pub boundary_controlled: Option<bool>,
    #[serde(default)]
    pub requested_extraction_m3: Option<f64>,
    #[serde(default)]
    pub delivered_extraction_m3: Option<f64>,
    #[serde(default)]
    pub extraction_delivery_fraction: Option<f64>,
    #[serde(default)]
    pub wells_curtailed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModflowUnits {
    pub length: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModflowRunResult {
    pub status: String,
    pub mf6_version: String,
    pub mf6_executable: String,
    pub converged: bool,
    pub units: ModflowUnits,
    pub steady_state: bool,
    pub duration_days: f64,
    pub recharge_m_day: f64,
    pub heads: HeadSummary,
    #[serde(default)]
    pub wells: Vec<WellResult>,
    pub budget: BudgetSummary,
    pub gate: ModflowGate,
}

#[derive(Debug, Clone, Deserialize)]
struct ModflowFailure {
    status: String,
    #[serde(default)]
    error: String,
}

/// Validate the request before spending a subprocess on it.
pub fn validate_request(request: &ModflowRequest) -> Result<(), String> {
    if request.nlay < 1 || request.nrow < 3 || request.ncol < 3 {
        return Err("grid must have nlay >= 1, nrow >= 3, ncol >= 3".to_string());
    }
    if request.layer_bottoms_m.len() != request.nlay as usize {
        return Err(format!(
            "layer_bottoms_m must have exactly nlay ({}) entries, got {}",
            request.nlay,
            request.layer_bottoms_m.len()
        ));
    }
    for (name, value) in [
        ("cell_size_m", request.cell_size_m),
        ("hk_m_day", request.hk_m_day),
        ("vk_m_day", request.vk_m_day),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(format!("{} must be a positive finite number", name));
        }
    }
    let mut previous = request.top_m;
    if !previous.is_finite() {
        return Err("top_m must be finite".to_string());
    }
    for (index, bottom) in request.layer_bottoms_m.iter().enumerate() {
        if !bottom.is_finite() {
            return Err("layer_bottoms_m must contain only finite numbers".to_string());
        }
        if *bottom >= previous {
            return Err(format!(
                "layer {} bottom ({}) must be below the surface above it ({})",
                index + 1,
                bottom,
                previous
            ));
        }
        previous = *bottom;
    }
    if !(0.0..1.0).contains(&request.sy) || request.sy <= 0.0 {
        return Err("sy must be between 0 and 1".to_string());
    }
    if !(0.0..1.0).contains(&request.ss_per_m) {
        return Err("ss_per_m must be between 0 and 1".to_string());
    }
    if !request.recharge_mm_yr.is_finite() || request.recharge_mm_yr < 0.0 {
        return Err("recharge_mm_yr must not be negative".to_string());
    }
    for (index, well) in request.wells.iter().enumerate() {
        if well.layer < 1
            || well.layer > request.nlay
            || well.row < 1
            || well.row > request.nrow
            || well.col < 1
            || well.col > request.ncol
        {
            return Err(format!(
                "wells[{}] index out of range (1-based, layer<={} row<={} col<={})",
                index, request.nlay, request.nrow, request.ncol
            ));
        }
        if !well.rate_m3_day.is_finite() || well.rate_m3_day < 0.0 {
            return Err(format!(
                "wells[{}].rate_m3_day must be a non-negative extraction rate",
                index
            ));
        }
    }
    if !request.steady_state {
        let duration = request.duration_days.unwrap_or(0.0);
        if !duration.is_finite() || duration <= 0.0 {
            return Err("duration_days must be positive for a transient run".to_string());
        }
    }
    if let Some(tolerance) = request.mass_tolerance_pct {
        if !tolerance.is_finite() || !(0.0..=100.0).contains(&tolerance) || tolerance <= 0.0 {
            return Err("mass_tolerance_pct must be between 0 and 100".to_string());
        }
    }
    Ok(())
}

/// True when the run is trustworthy enough to interpret at screening level.
pub fn result_is_interpretable(result: &ModflowRunResult) -> bool {
    result.converged && result.gate.gate_passed && result.heads.available
}

/// Run a MODFLOW 6 model and return its parsed result.
pub async fn run_modflow(request: &ModflowRequest) -> Result<ModflowRunResult, String> {
    validate_request(request)?;
    let timeout_secs = request.timeout_secs.unwrap_or(300);
    if timeout_secs == 0 {
        return Err("MODFLOW timeout_secs must be greater than zero".to_string());
    }

    let script = modflow_script();
    if !PathBuf::from(&script).is_file() {
        return Err(format!("MODFLOW wrapper script not found: {}", script));
    }
    let payload = serde_json::to_vec(request)
        .map_err(|error| format!("could not serialise MODFLOW request: {}", error))?;

    let python = modflow_python();
    let mut command = tokio::process::Command::new(&python);
    command
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    // MODFLOW is invoked by FloPy through PATH; make sure the mf6 binary that
    // ships beside the interpreter is findable.
    if let Some(bin_dir) = modflow_bin_dir() {
        let existing = std::env::var("PATH").unwrap_or_default();
        command.env("PATH", format!("{}:{}", bin_dir, existing));
    }

    let mut child = command
        .spawn()
        .map_err(|error| format!("Failed to spawn MODFLOW runner ({}): {}", python, error))?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(&payload)
            .await
            .map_err(|error| format!("could not write MODFLOW request: {}", error))?;
        stdin
            .shutdown()
            .await
            .map_err(|error| format!("could not close MODFLOW stdin: {}", error))?;
    }

    let output = tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output())
        .await
        .map_err(|_| format!("MODFLOW run exceeded {}s timeout", timeout_secs))?
        .map_err(|error| format!("MODFLOW runner failed: {}", error))?;

    if output.stdout.len() > MAX_STDOUT_BYTES {
        return Err(format!(
            "MODFLOW runner stdout exceeded {} bytes",
            MAX_STDOUT_BYTES
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();

    if line.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "MODFLOW runner produced no JSON (exit {:?}): {}",
            output.status.code(),
            stderr.trim()
        ));
    }

    if !output.status.success() {
        let detail = serde_json::from_str::<ModflowFailure>(line)
            .map(|failure| format!("{}: {}", failure.status, failure.error))
            .unwrap_or_else(|_| line.to_string());
        return Err(format!(
            "MODFLOW runner exited with {:?}: {}",
            output.status.code(),
            detail
        ));
    }

    let result: ModflowRunResult = serde_json::from_str(line)
        .map_err(|error| format!("MODFLOW runner emitted unparseable JSON: {}", error))?;

    if result.status != "ok" {
        return Err(format!("MODFLOW run status was {}", result.status));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> ModflowRequest {
        ModflowRequest {
            nlay: 2,
            nrow: 20,
            ncol: 20,
            cell_size_m: 100.0,
            top_m: 50.0,
            layer_bottoms_m: vec![30.0, 0.0],
            hk_m_day: 10.0,
            vk_m_day: 1.0,
            sy: 0.15,
            ss_per_m: 1e-5,
            initial_head_m: 45.0,
            boundary_head_m: 45.0,
            recharge_mm_yr: 1800.0,
            wells: vec![WellSpec { layer: 1, row: 10, col: 10, rate_m3_day: 2000.0 }],
            steady_state: true,
            duration_days: None,
            mass_tolerance_pct: None,
            timeout_secs: Some(300),
        }
    }

    #[test]
    fn accepts_a_well_formed_request() {
        assert!(validate_request(&valid_request()).is_ok());
    }

    #[test]
    fn layer_bottoms_must_descend_from_the_top() {
        let mut request = valid_request();
        request.layer_bottoms_m = vec![60.0, 0.0];
        let error = validate_request(&request).unwrap_err();
        assert!(error.contains("must be below"), "unexpected: {error}");

        let mut inverted = valid_request();
        inverted.layer_bottoms_m = vec![10.0, 20.0];
        assert!(validate_request(&inverted).unwrap_err().contains("must be below"));
    }

    #[test]
    fn layer_bottoms_count_must_match_nlay() {
        let mut request = valid_request();
        request.layer_bottoms_m = vec![30.0];
        assert!(validate_request(&request).unwrap_err().contains("exactly nlay"));
    }

    #[test]
    fn out_of_range_well_index_is_rejected() {
        let mut request = valid_request();
        request.wells[0].col = 99;
        assert!(validate_request(&request).unwrap_err().contains("out of range"));

        let mut zero_based = valid_request();
        zero_based.wells[0].layer = 0;
        assert!(validate_request(&zero_based).unwrap_err().contains("out of range"));
    }

    #[test]
    fn negative_extraction_and_bad_parameters_are_rejected() {
        let mut negative = valid_request();
        negative.wells[0].rate_m3_day = -5.0;
        assert!(validate_request(&negative).unwrap_err().contains("non-negative"));

        let mut bad_k = valid_request();
        bad_k.hk_m_day = 0.0;
        assert!(validate_request(&bad_k).unwrap_err().contains("hk_m_day"));

        let mut bad_sy = valid_request();
        bad_sy.sy = 1.5;
        assert!(validate_request(&bad_sy).unwrap_err().contains("sy"));

        let mut bad_recharge = valid_request();
        bad_recharge.recharge_mm_yr = -1.0;
        assert!(validate_request(&bad_recharge).unwrap_err().contains("recharge"));
    }

    #[test]
    fn transient_run_requires_a_duration() {
        let mut request = valid_request();
        request.steady_state = false;
        request.duration_days = None;
        assert!(validate_request(&request).unwrap_err().contains("duration_days"));
    }

    #[test]
    fn parses_the_documented_contract_with_all_gates() {
        let json = r#"{"status":"ok","mf6_version":"mf6: 6.7.0 02/05/2026","mf6_executable":"/x/mf6","converged":true,"units":{"length":"meters","time":"days"},"steady_state":true,"duration_days":1.0,"recharge_m_day":0.0049,"heads":{"available":true,"dry_cell_count":0,"active_cell_count":800,"grid_shape":[2,20,20],"min_head_m":45.0,"max_head_m":49.57,"mean_head_m":47.86},"wells":[{"layer":1,"row":10,"col":10,"rate_m3_day":2000.0,"head_m":46.2,"drawdown_m":-1.2,"cell_is_dry":false}],"budget":{"available":true,"cumulative_m3":{"TOTAL_IN":17741.27,"WEL_OUT":2000.0,"CHD_IN":0.0}},"gate":{"percent_discrepancy":-0.05,"tolerance_pct":1.0,"gate_passed":true,"boundary_inflow_fraction":0.0,"boundary_controlled":false,"requested_extraction_m3":2000.0,"delivered_extraction_m3":2000.0,"extraction_delivery_fraction":1.0,"wells_curtailed":false}}"#;
        let parsed: ModflowRunResult = serde_json::from_str(json).unwrap();
        assert!(parsed.converged);
        assert_eq!(parsed.units.length, "meters");
        assert_eq!(parsed.units.time, "days");
        assert_eq!(parsed.heads.dry_cell_count, 0);
        assert_eq!(parsed.gate.wells_curtailed, Some(false));
        assert!(result_is_interpretable(&parsed));
    }

    #[test]
    fn curtailed_wells_make_the_result_uninterpretable() {
        // MODFLOW switches off a well whose cell dries out. The budget then
        // balances around a pump that extracted nothing, so `converged` and the
        // discrepancy gate both pass while the requested scenario never ran.
        let json = r#"{"status":"ok","mf6_version":"mf6: 6.7.0","mf6_executable":"/x/mf6","converged":true,"units":{"length":"meters","time":"days"},"steady_state":true,"duration_days":1.0,"recharge_m_day":0.0001,"heads":{"available":true,"dry_cell_count":82,"active_cell_count":718,"grid_shape":[2,20,20],"min_head_m":30.1,"max_head_m":45.0,"mean_head_m":41.0},"wells":[{"layer":1,"row":10,"col":10,"rate_m3_day":8000.0,"head_m":null,"drawdown_m":null,"cell_is_dry":true}],"budget":{"available":true,"cumulative_m3":{"TOTAL_IN":491.44,"WEL_OUT":0.0}},"gate":{"percent_discrepancy":0.42,"tolerance_pct":1.0,"gate_passed":false,"boundary_inflow_fraction":0.0,"boundary_controlled":false,"requested_extraction_m3":8000.0,"delivered_extraction_m3":0.0,"extraction_delivery_fraction":0.0,"wells_curtailed":true}}"#;
        let parsed: ModflowRunResult = serde_json::from_str(json).unwrap();
        assert!(parsed.converged, "MODFLOW itself reported convergence");
        assert!(parsed.gate.percent_discrepancy.unwrap().abs() < 1.0, "budget balances");
        assert_eq!(parsed.gate.wells_curtailed, Some(true));
        assert_eq!(parsed.gate.delivered_extraction_m3, Some(0.0));
        assert!(parsed.wells[0].cell_is_dry);
        assert!(parsed.wells[0].head_m.is_none());
        assert_eq!(parsed.heads.dry_cell_count, 82);
        assert!(!result_is_interpretable(&parsed));
    }

    #[test]
    fn wrapper_script_path_is_absolute_and_present() {
        let script = std::path::Path::new(MODFLOW_SCRIPT);
        assert!(script.is_absolute(), "MODFLOW_SCRIPT must not depend on CWD");
        assert!(script.is_file(), "missing wrapper script at {}", MODFLOW_SCRIPT);
    }

    #[tokio::test]
    async fn zero_timeout_is_rejected_before_spawning() {
        let mut request = valid_request();
        request.timeout_secs = Some(0);
        let error = run_modflow(&request).await.unwrap_err();
        assert!(error.contains("timeout_secs"), "unexpected: {error}");
    }
}
