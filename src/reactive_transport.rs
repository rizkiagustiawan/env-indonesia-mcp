//! 1D advective-dispersive REACTIVE TRANSPORT via PHREEQC TRANSPORT, executed
//! as a Python subprocess.
//!
//! This is the last coupling in the AMD chain. `phreeqc_speciation` equilibrates
//! one batch; `pyrite_oxidation_kinetics` adds time to one batch; this adds
//! space and flow. It answers WHERE along a flow path, and after how many pore
//! volumes, a reactive barrier stops working.
//!
//! Four honesty guards travel with every result:
//!
//! * `numerical_dispersion_dominates` — PHREEQC's mixing-cell scheme carries
//!   numerical dispersion of roughly `cell_length / 2`. When the physical
//!   dispersivity is smaller than that, the simulated front spreading is a grid
//!   artifact rather than transport physics. Reported as the grid Peclet number.
//! * `front_traversed_column` / `breakthrough_reached` — if the influent front
//!   never reaches the outlet, a clean outlet means "the simulation was too
//!   short", not "the barrier works". This is the failure mode most likely to
//!   be misread as success.
//! * `buffer_exhausted` — the reactive mineral was consumed at the outlet, so
//!   the barrier has failed. This is the physically real answer and the reason
//!   to run the model at all.
//! * `equilibrium_assumed_at_each_cell` — always true and always reported: each
//!   cell reaches full thermodynamic equilibrium every shift, so kinetic
//!   limitation and preferential flow are absent by construction.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use crate::swmm_runner::swmm_python;

/// Absolute path of the Python wrapper, resolved at compile time.
pub const TRANSPORT_SCRIPT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/reactive_transport.py");

/// Hard cap on captured stdout: 8 MiB.
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;

/// Resolve the wrapper script, honouring an environment override.
pub fn transport_script() -> String {
    std::env::var("ENV_INDONESIA_TRANSPORT_SCRIPT")
        .unwrap_or_else(|_| TRANSPORT_SCRIPT.to_string())
}

/// The interpreter is the same venv that carries `phreeqpython`.
pub fn transport_python() -> String {
    swmm_python()
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ReactivePhase {
    pub phase: String,
    /// Initial moles of the phase present in every cell.
    pub moles: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ReactiveTransportRequest {
    /// Number of mixing cells in the column (2..=200).
    pub cells: u32,
    pub cell_length_m: f64,
    /// Number of advective shifts. `shifts / cells` is the pore volumes flushed.
    pub shifts: u32,
    pub time_step_s: f64,
    /// Physical dispersivity. Below `cell_length / 2` the scheme's own numerical
    /// dispersion dominates, which is reported rather than hidden.
    pub dispersivity_m: f64,
    /// Influent composition, e.g. `{"pH": 2.5, "Fe(3)": 30.0, "S(6)": 120.0}`.
    pub influent: BTreeMap<String, f64>,
    /// Initial pore water in every cell.
    pub pore_water: BTreeMap<String, f64>,
    #[serde(default)]
    pub units: Option<String>,
    /// Minerals available to react in every cell, e.g. Calcite for an ALD.
    #[serde(default)]
    pub reactive_phases: Vec<ReactivePhase>,
    #[serde(default)]
    pub tracked_elements: Option<Vec<String>>,
    #[serde(default)]
    pub punch_frequency: Option<u32>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportStep {
    pub shift: u32,
    #[serde(default)]
    pub pore_volumes: Option<f64>,
    pub time_days: f64,
    #[serde(default)]
    pub distance_m: Option<f64>,
    pub ph: f64,
    #[serde(default)]
    pub elements_mol_kgw: BTreeMap<String, f64>,
    #[serde(default)]
    pub phases_mol: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportGuards {
    #[serde(default)]
    pub grid_peclet: Option<f64>,
    pub grid_peclet_limit: f64,
    pub numerical_dispersion_dominates: bool,
    pub pore_volumes_flushed: f64,
    pub front_traversed_column: bool,
    pub breakthrough_reached: bool,
    #[serde(default)]
    pub breakthrough_element: Option<String>,
    #[serde(default)]
    pub breakthrough_pore_volumes: Option<f64>,
    pub buffer_exhausted: bool,
    #[serde(default)]
    pub exhausted_phases: Vec<String>,
    pub outlet_initial_ph: f64,
    pub outlet_final_ph: f64,
    pub equilibrium_assumed_at_each_cell: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactiveTransportResult {
    pub status: String,
    pub database: String,
    pub database_sha256: String,
    pub column_length_m: f64,
    pub cells: u32,
    pub shifts: u32,
    pub cell_length_m: f64,
    pub dispersivity_m: f64,
    pub time_step_s: f64,
    pub pore_velocity_m_day: f64,
    pub total_simulated_days: f64,
    #[serde(default)]
    pub tracked_elements: Vec<String>,
    #[serde(default)]
    pub reactive_phases: Vec<ReactivePhase>,
    pub outlet_series: Vec<TransportStep>,
    pub guards: TransportGuards,
}

#[derive(Debug, Clone, Deserialize)]
struct TransportFailure {
    status: String,
    #[serde(default)]
    error: String,
}

const MAX_CELLS: u32 = 200;
const MAX_SHIFTS: u32 = 5000;
const NON_ELEMENT_KEYS: [&str; 7] =
    ["pH", "pe", "temp", "temperature", "density", "water", "redox"];

/// Validate the request before spending a subprocess on it.
pub fn validate_request(request: &ReactiveTransportRequest) -> Result<(), String> {
    if !(2..=MAX_CELLS).contains(&request.cells) {
        return Err(format!("cells must be between 2 and {}", MAX_CELLS));
    }
    if !(1..=MAX_SHIFTS).contains(&request.shifts) {
        return Err(format!("shifts must be between 1 and {}", MAX_SHIFTS));
    }
    for (name, value) in [
        ("cell_length_m", request.cell_length_m),
        ("time_step_s", request.time_step_s),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(format!("{} must be a positive finite number", name));
        }
    }
    if !request.dispersivity_m.is_finite() || request.dispersivity_m < 0.0 {
        return Err("dispersivity_m must not be negative".to_string());
    }
    if request.influent.is_empty() {
        return Err("influent must specify at least one entry".to_string());
    }
    if request.pore_water.is_empty() {
        return Err("pore_water must specify at least one entry".to_string());
    }
    let influent_elements = request
        .influent
        .keys()
        .filter(|key| !NON_ELEMENT_KEYS.contains(&key.as_str()))
        .count();
    if influent_elements == 0 {
        return Err("influent must specify at least one element besides pH/pe".to_string());
    }
    for (label, block) in [("influent", &request.influent), ("pore_water", &request.pore_water)] {
        for (key, value) in block {
            if !value.is_finite() {
                return Err(format!("{}[{}] must be finite", label, key));
            }
        }
    }
    if let Some(units) = &request.units {
        if !matches!(units.as_str(), "mmol" | "mol" | "mg" | "umol" | "ug") {
            return Err(format!(
                "units must be one of mmol, mol, mg, umol, ug (got {})",
                units
            ));
        }
    }
    for (index, phase) in request.reactive_phases.iter().enumerate() {
        if phase.phase.trim().is_empty() {
            return Err(format!("reactive_phases[{}].phase must not be empty", index));
        }
        if !phase.moles.is_finite() || phase.moles < 0.0 {
            return Err(format!(
                "reactive_phases[{}].moles must not be negative",
                index
            ));
        }
    }
    if let Some(tracked) = &request.tracked_elements {
        if tracked.is_empty() || tracked.iter().any(|name| name.trim().is_empty()) {
            return Err("tracked_elements must contain non-empty element names".to_string());
        }
    }
    if let Some(punch) = request.punch_frequency {
        if punch < 1 {
            return Err("punch_frequency must be at least 1".to_string());
        }
    }
    Ok(())
}

/// True when the simulated column behaviour can be read as a screening-level
/// result rather than a grid or setup artifact.
///
/// A run whose front never left the inlet says nothing about the barrier, and a
/// run dominated by numerical dispersion describes the grid rather than the
/// medium.
pub fn transport_is_interpretable(result: &ReactiveTransportResult) -> bool {
    result.guards.front_traversed_column && !result.guards.numerical_dispersion_dominates
}

/// Run a 1D reactive transport simulation.
pub async fn run_reactive_transport(
    request: &ReactiveTransportRequest,
) -> Result<ReactiveTransportResult, String> {
    validate_request(request)?;
    let timeout_secs = request.timeout_secs.unwrap_or(300);
    if timeout_secs == 0 {
        return Err("reactive transport timeout_secs must be greater than zero".to_string());
    }

    let script = transport_script();
    if !PathBuf::from(&script).is_file() {
        return Err(format!("reactive transport wrapper script not found: {}", script));
    }
    let payload = serde_json::to_vec(request)
        .map_err(|error| format!("could not serialise reactive transport request: {}", error))?;

    let python = transport_python();
    let mut command = tokio::process::Command::new(&python);
    command
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = command.spawn().map_err(|error| {
        format!("Failed to spawn reactive transport runner ({}): {}", python, error)
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(&payload)
            .await
            .map_err(|error| format!("could not write reactive transport request: {}", error))?;
        stdin
            .shutdown()
            .await
            .map_err(|error| format!("could not close reactive transport stdin: {}", error))?;
    }

    let output = tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output())
        .await
        .map_err(|_| format!("reactive transport run exceeded {}s timeout", timeout_secs))?
        .map_err(|error| format!("reactive transport runner failed: {}", error))?;

    if output.stdout.len() > MAX_STDOUT_BYTES {
        return Err(format!(
            "reactive transport runner stdout exceeded {} bytes",
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
            "reactive transport runner produced no JSON (exit {:?}): {}",
            output.status.code(),
            stderr.trim()
        ));
    }

    if !output.status.success() {
        let detail = serde_json::from_str::<TransportFailure>(line)
            .map(|failure| format!("{}: {}", failure.status, failure.error))
            .unwrap_or_else(|_| line.to_string());
        return Err(format!(
            "reactive transport runner exited with {:?}: {}",
            output.status.code(),
            detail
        ));
    }

    let result: ReactiveTransportResult = serde_json::from_str(line)
        .map_err(|error| format!("reactive transport runner emitted unparseable JSON: {}", error))?;

    if result.status != "ok" {
        return Err(format!("reactive transport run status was {}", result.status));
    }
    if result.outlet_series.is_empty() {
        return Err("reactive transport run returned an empty outlet series".to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> ReactiveTransportRequest {
        ReactiveTransportRequest {
            cells: 10,
            cell_length_m: 0.2,
            shifts: 60,
            time_step_s: 3600.0,
            dispersivity_m: 0.1,
            influent: BTreeMap::from([
                ("pH".into(), 2.5),
                ("Fe(3)".into(), 30.0),
                ("S(6)".into(), 120.0),
            ]),
            pore_water: BTreeMap::from([("pH".into(), 7.0), ("Ca".into(), 1.0)]),
            units: Some("mmol".into()),
            reactive_phases: vec![ReactivePhase { phase: "Calcite".into(), moles: 0.02 }],
            tracked_elements: Some(vec!["Fe".into(), "S(6)".into()]),
            punch_frequency: Some(5),
            timeout_secs: Some(300),
        }
    }

    #[test]
    fn valid_request_is_accepted() {
        assert!(validate_request(&valid_request()).is_ok());
    }

    #[test]
    fn grid_and_transport_parameters_are_checked() {
        let mut r = valid_request();
        r.cells = 1;
        assert!(validate_request(&r).unwrap_err().contains("cells"));
        let mut r = valid_request();
        r.dispersivity_m = -1.0;
        assert!(validate_request(&r).unwrap_err().contains("dispersivity"));
        let mut r = valid_request();
        r.time_step_s = 0.0;
        assert!(validate_request(&r).unwrap_err().contains("time_step"));
    }

    #[test]
    fn empty_compositions_and_bad_phase_are_rejected() {
        let mut r = valid_request();
        r.influent.clear();
        assert!(validate_request(&r).unwrap_err().contains("influent"));
        let mut r = valid_request();
        r.reactive_phases[0].phase.clear();
        assert!(validate_request(&r).unwrap_err().contains("phase"));
    }

    #[test]
    fn peclet_and_front_guards_make_semantic_failures_uninterpretable() {
        let mut r = valid_request();
        r.dispersivity_m = 0.01;
        let mut result = ReactiveTransportResult {
            status: "ok".into(), database: "db".into(), database_sha256: "x".into(),
            column_length_m: 2.0, cells: 10, shifts: 5, cell_length_m: 0.2,
            dispersivity_m: 0.01, time_step_s: 3600.0, pore_velocity_m_day: 4.8,
            total_simulated_days: 0.2, tracked_elements: vec!["Fe".into()],
            reactive_phases: vec![], outlet_series: vec![],
            guards: TransportGuards {
                grid_peclet: Some(20.0), grid_peclet_limit: 2.0,
                numerical_dispersion_dominates: true, pore_volumes_flushed: 0.5,
                front_traversed_column: false, breakthrough_reached: false,
                breakthrough_element: None, breakthrough_pore_volumes: None,
                buffer_exhausted: false, exhausted_phases: vec![], outlet_initial_ph: 7.0,
                outlet_final_ph: 7.0, equilibrium_assumed_at_each_cell: true,
            },
        };
        assert!(!transport_is_interpretable(&result));
        r.dispersivity_m = 0.1;
        result.guards.grid_peclet = Some(2.0);
        result.guards.numerical_dispersion_dominates = false;
        result.guards.front_traversed_column = true;
        assert!(transport_is_interpretable(&result));
    }

    #[test]
    fn parses_honesty_guard_contract() {
        let json = r#"{"status":"ok","database":"db","database_sha256":"abc","column_length_m":1.0,"cells":5,"shifts":60,"cell_length_m":0.2,"dispersivity_m":0.1,"time_step_s":3600.0,"pore_velocity_m_day":4.8,"total_simulated_days":2.5,"tracked_elements":["Fe"],"reactive_phases":[{"phase":"Calcite","moles":0.02}],"outlet_series":[{"shift":60,"pore_volumes":12.0,"time_days":2.5,"distance_m":0.9,"ph":2.5,"elements_mol_kgw":{"Fe":0.03},"phases_mol":{"Calcite":0.0}}],"guards":{"grid_peclet":2.0,"grid_peclet_limit":2.0,"numerical_dispersion_dominates":false,"pore_volumes_flushed":12.0,"front_traversed_column":true,"breakthrough_reached":true,"breakthrough_element":"Fe","breakthrough_pore_volumes":1.0,"buffer_exhausted":true,"exhausted_phases":["Calcite"],"outlet_initial_ph":7.8,"outlet_final_ph":2.5,"equilibrium_assumed_at_each_cell":true}}"#;
        let parsed: ReactiveTransportResult = serde_json::from_str(json).unwrap();
        assert!(parsed.guards.buffer_exhausted);
        assert!(parsed.guards.breakthrough_reached);
        assert!(transport_is_interpretable(&parsed));
    }

    #[test]
    fn wrapper_script_path_is_absolute_and_present() {
        let script = std::path::Path::new(TRANSPORT_SCRIPT);
        assert!(script.is_absolute());
        assert!(script.is_file());
    }

    #[tokio::test]
    async fn zero_timeout_is_rejected() {
        let mut request = valid_request();
        request.timeout_secs = Some(0);
        assert!(run_reactive_transport(&request).await.unwrap_err().contains("timeout_secs"));
    }
}
