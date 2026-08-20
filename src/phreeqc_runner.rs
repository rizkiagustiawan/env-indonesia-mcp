//! Real PHREEQC geochemical execution via a Python (`phreeqpython`) subprocess.
//!
//! This replaces the old script *generator* path: `scripts/phreeqc_run.py` runs
//! an actual speciation / lime-titration calculation and returns a strict JSON
//! contract, which is parsed here into [`PhreeqcRunResult`].
//!
//! Three honesty guards travel with every result and are deliberately modelled
//! as first-class fields rather than free-text notes:
//!
//! * `unsupported_elements` — PHREEQC silently accepts an element that has no
//!   master species in the loaded database, reports 0 mg/L, and raises nothing.
//!   A caller would read that zero as "not mobile" when it means "never
//!   modelled". Chromium is the concrete case: no bundled database defines it.
//! * `sc_us_cm: null` + `sc_note` — specific conductance needs `-dw` diffusion
//!   coefficients. Databases lacking them return 0.0 uS/cm, which is impossible
//!   for a solution with real ionic strength.
//! * `supersaturated_but_unmodelled` — a phase with SI > 0 that was not passed
//!   in `equilibrium_phases` would precipitate in reality but was never removed,
//!   so the dissolved concentrations are an upper bound, not a prediction.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use crate::swmm_runner::swmm_python;

/// Absolute path of the Python wrapper, resolved at compile time so the runner
/// does not depend on the process working directory.
pub const PHREEQC_SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/phreeqc_run.py");

/// Default repaired WATEQ4F database shipped in `resources/phreeqc`.
pub const DEFAULT_DATABASE: &str = "wateq4f_PWN_repaired.dat";

/// Hard cap on captured stdout: 8 MiB.
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;

/// Resolve the wrapper script, honouring an environment override.
pub fn phreeqc_script() -> String {
    std::env::var("ENV_INDONESIA_PHREEQC_SCRIPT").unwrap_or_else(|_| PHREEQC_SCRIPT.to_string())
}

/// The interpreter is the same venv that carries `pyswmm`/`phreeqpython`.
pub fn phreeqc_python() -> String {
    swmm_python()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupersaturatedPhase {
    pub phase: String,
    pub si: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRecovery {
    pub element: String,
    pub requested: f64,
    pub requested_units: String,
    pub reported_mmol: f64,
    pub recovered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionState {
    pub ph: f64,
    pub pe: f64,
    /// `None` when the database cannot compute conductance (see `sc_note`).
    pub sc_us_cm: Option<f64>,
    pub sc_note: Option<String>,
    pub ionic_strength_mol_kgw: f64,
    pub elements_mg_l: std::collections::BTreeMap<String, f64>,
    pub elements_mmol: std::collections::BTreeMap<String, f64>,
    pub saturation_indices: std::collections::BTreeMap<String, f64>,
    #[serde(default)]
    pub saturation_indices_not_computed: Vec<String>,
    #[serde(default)]
    pub supersaturated_but_unmodelled: Vec<SupersaturatedPhase>,
    #[serde(default)]
    pub concentrations_are_upper_bounds: bool,
    #[serde(default)]
    pub target_ph: Option<f64>,
    #[serde(default)]
    pub ph_error: Option<f64>,
    #[serde(default)]
    pub reached_target: Option<bool>,
    #[serde(default)]
    pub titration_steps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhreeqcRunResult {
    pub status: String,
    pub database: String,
    pub database_sha256: String,
    pub units: String,
    pub raw: SolutionState,
    #[serde(default)]
    pub treated: Option<SolutionState>,
    #[serde(default)]
    pub lime_added_mmol: f64,
    #[serde(default)]
    pub element_recovery: Vec<ElementRecovery>,
    #[serde(default)]
    pub unsupported_elements: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PhreeqcFailure {
    status: String,
    #[serde(default)]
    error: String,
}

/// Request forwarded verbatim to the Python wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhreeqcRequest {
    /// Element amounts plus optional `pH` / `pe`, e.g.
    /// `{"pH": 2.8, "Fe(3)": 50.0, "S(6)": 200.0, "Zn": 2.0}`.
    pub solution: std::collections::BTreeMap<String, f64>,
    /// One of `mmol`, `mol`, `mg`, `umol`, `ug`. Defaults to `mmol`.
    pub units: Option<String>,
    pub temperature_c: Option<f64>,
    /// Phases equilibrated (allowed to precipitate). Anything supersaturated and
    /// absent from this list is reported as an unmodelled upper bound.
    pub equilibrium_phases: Option<Vec<String>>,
    /// When set, Ca(OH)2 is titrated in until this pH is reached.
    pub lime_titration_target_ph: Option<f64>,
    /// Extra phases to report saturation indices for.
    pub saturation_indices: Option<Vec<String>>,
    pub timeout_secs: Option<u64>,
}

/// Validate the request before spending a subprocess on it.
pub fn validate_request(request: &PhreeqcRequest) -> Result<(), String> {
    if request.solution.is_empty() {
        return Err("solution must specify at least one element".to_string());
    }
    let element_count = request
        .solution
        .keys()
        .filter(|key| !matches!(key.as_str(), "pH" | "pe" | "temp" | "temperature" | "density" | "water"))
        .count();
    if element_count == 0 {
        return Err("solution must specify at least one element besides pH/pe".to_string());
    }
    for (key, value) in &request.solution {
        if !value.is_finite() {
            return Err(format!("solution value for {} must be finite", key));
        }
    }
    if let Some(units) = &request.units {
        if !matches!(units.as_str(), "mmol" | "mol" | "mg" | "umol" | "ug") {
            return Err(format!("units must be one of mmol, mol, mg, umol, ug (got {})", units));
        }
    }
    if let Some(target) = request.lime_titration_target_ph {
        if !target.is_finite() || !(0.0..14.0).contains(&target) {
            return Err(format!(
                "lime_titration_target_ph must be between 0 and 14 (got {})",
                target
            ));
        }
    }
    if let Some(temperature) = request.temperature_c {
        if !temperature.is_finite() || !(-10.0..=200.0).contains(&temperature) {
            return Err(format!(
                "temperature_c must be between -10 and 200 (got {})",
                temperature
            ));
        }
    }
    Ok(())
}

/// True when every requested element was actually carried by the database.
pub fn all_elements_modelled(result: &PhreeqcRunResult) -> bool {
    result.unsupported_elements.is_empty()
}

/// Run a PHREEQC calculation and return its parsed result.
pub async fn run_phreeqc(request: &PhreeqcRequest) -> Result<PhreeqcRunResult, String> {
    validate_request(request)?;
    let timeout_secs = request.timeout_secs.unwrap_or(120);
    if timeout_secs == 0 {
        return Err("PHREEQC timeout_secs must be greater than zero".to_string());
    }

    let script = phreeqc_script();
    if !PathBuf::from(&script).is_file() {
        return Err(format!("PHREEQC wrapper script not found: {}", script));
    }
    let payload = serde_json::to_vec(request)
        .map_err(|error| format!("could not serialise PHREEQC request: {}", error))?;

    let python = phreeqc_python();
    let mut command = tokio::process::Command::new(&python);
    command
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = command
        .spawn()
        .map_err(|error| format!("Failed to spawn PHREEQC runner ({}): {}", python, error))?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(&payload)
            .await
            .map_err(|error| format!("could not write PHREEQC request: {}", error))?;
        stdin
            .shutdown()
            .await
            .map_err(|error| format!("could not close PHREEQC stdin: {}", error))?;
    }

    let output = tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output())
        .await
        .map_err(|_| format!("PHREEQC run exceeded {}s timeout", timeout_secs))?
        .map_err(|error| format!("PHREEQC runner failed: {}", error))?;

    if output.stdout.len() > MAX_STDOUT_BYTES {
        return Err(format!(
            "PHREEQC runner stdout exceeded {} bytes",
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
            "PHREEQC runner produced no JSON (exit {:?}): {}",
            output.status.code(),
            stderr.trim()
        ));
    }

    if !output.status.success() {
        let detail = serde_json::from_str::<PhreeqcFailure>(line)
            .map(|failure| format!("{}: {}", failure.status, failure.error))
            .unwrap_or_else(|_| line.to_string());
        return Err(format!(
            "PHREEQC runner exited with {:?}: {}",
            output.status.code(),
            detail
        ));
    }

    let result: PhreeqcRunResult = serde_json::from_str(line)
        .map_err(|error| format!("PHREEQC runner emitted unparseable JSON: {}", error))?;

    if result.status != "ok" {
        return Err(format!("PHREEQC run status was {}", result.status));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amd_request() -> PhreeqcRequest {
        let mut solution = std::collections::BTreeMap::new();
        solution.insert("pH".to_string(), 2.8);
        solution.insert("Fe(3)".to_string(), 50.0);
        solution.insert("S(6)".to_string(), 200.0);
        PhreeqcRequest {
            solution,
            units: Some("mmol".to_string()),
            temperature_c: None,
            equilibrium_phases: None,
            lime_titration_target_ph: None,
            saturation_indices: None,
            timeout_secs: Some(120),
        }
    }

    #[test]
    fn parses_the_documented_contract_including_honesty_guards() {
        let json = r#"{"status":"ok","database":"wateq4f_PWN_repaired.dat","database_sha256":"c0f6","units":"mmol","raw":{"ph":3.02,"pe":13.4,"sc_us_cm":null,"sc_note":"database lacks -dw","ionic_strength_mol_kgw":0.279,"elements_mg_l":{"Fe":2835.6,"Cr":0.0},"elements_mmol":{"Fe":50.7,"Cr":0.0},"saturation_indices":{"Goethite":5.89},"saturation_indices_not_computed":["Gypsum"],"supersaturated_but_unmodelled":[{"phase":"Goethite","si":5.89}],"concentrations_are_upper_bounds":true},"treated":null,"lime_added_mmol":0.0,"element_recovery":[{"element":"Cr","requested":0.05,"requested_units":"mmol","reported_mmol":0.0,"recovered":false}],"unsupported_elements":["Cr"]}"#;
        let parsed: PhreeqcRunResult = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.unsupported_elements, vec!["Cr".to_string()]);
        assert!(parsed.raw.sc_us_cm.is_none());
        assert!(parsed.raw.sc_note.is_some());
        assert!(parsed.raw.concentrations_are_upper_bounds);
        assert_eq!(parsed.raw.supersaturated_but_unmodelled.len(), 1);
        assert_eq!(parsed.raw.saturation_indices_not_computed, vec!["Gypsum".to_string()]);
        assert!(!all_elements_modelled(&parsed));
    }

    #[test]
    fn parses_titration_outcome_fields() {
        let json = r#"{"status":"ok","database":"db.dat","database_sha256":"aa","units":"mmol","raw":{"ph":2.8,"pe":16.8,"sc_us_cm":null,"sc_note":null,"ionic_strength_mol_kgw":0.25,"elements_mg_l":{"Fe":2782.9},"elements_mmol":{"Fe":49.8},"saturation_indices":{},"saturation_indices_not_computed":[],"supersaturated_but_unmodelled":[],"concentrations_are_upper_bounds":false},"treated":{"ph":8.52,"pe":-0.6,"sc_us_cm":null,"sc_note":null,"ionic_strength_mol_kgw":0.35,"elements_mg_l":{"Fe":0.0},"elements_mmol":{"Fe":0.0},"saturation_indices":{},"saturation_indices_not_computed":[],"supersaturated_but_unmodelled":[],"concentrations_are_upper_bounds":false,"target_ph":8.5,"ph_error":0.0197,"reached_target":true,"titration_steps":21},"lime_added_mmol":99.69,"element_recovery":[],"unsupported_elements":[]}"#;
        let parsed: PhreeqcRunResult = serde_json::from_str(json).unwrap();
        assert!(all_elements_modelled(&parsed));
        let treated = parsed.treated.as_ref().expect("treated state");
        assert_eq!(treated.target_ph, Some(8.5));
        assert_eq!(treated.reached_target, Some(true));
        assert_eq!(treated.titration_steps, Some(21));
        // The titration must land on the requested pH, not merely exceed it:
        // a coarse-step overshoot to pH 11.5 changes which hydroxides
        // precipitate and therefore the reported metal removal.
        let error = treated.ph_error.expect("ph_error");
        assert!(error < 0.05, "ph_error too large: {error}");
    }

    #[test]
    fn empty_or_ph_only_solution_is_rejected() {
        let mut request = amd_request();
        request.solution.clear();
        assert!(validate_request(&request).is_err());

        let mut ph_only = amd_request();
        ph_only.solution.clear();
        ph_only.solution.insert("pH".to_string(), 7.0);
        let error = validate_request(&ph_only).unwrap_err();
        assert!(error.contains("besides pH/pe"), "unexpected: {error}");
    }

    #[test]
    fn non_finite_and_out_of_range_inputs_are_rejected() {
        let mut nan = amd_request();
        nan.solution.insert("Fe(3)".to_string(), f64::NAN);
        assert!(validate_request(&nan).unwrap_err().contains("finite"));

        let mut bad_units = amd_request();
        bad_units.units = Some("kg".to_string());
        assert!(validate_request(&bad_units).unwrap_err().contains("units"));

        let mut bad_ph = amd_request();
        bad_ph.lime_titration_target_ph = Some(20.0);
        assert!(validate_request(&bad_ph)
            .unwrap_err()
            .contains("lime_titration_target_ph"));

        let mut bad_temp = amd_request();
        bad_temp.temperature_c = Some(9000.0);
        assert!(validate_request(&bad_temp).unwrap_err().contains("temperature_c"));
    }

    #[test]
    fn wrapper_script_path_is_absolute_and_present() {
        let script = std::path::Path::new(PHREEQC_SCRIPT);
        assert!(script.is_absolute(), "PHREEQC_SCRIPT must not depend on CWD");
        assert!(script.is_file(), "missing wrapper script at {}", PHREEQC_SCRIPT);
    }

    #[test]
    fn shipped_database_is_present() {
        let database = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/phreeqc/",
            "wateq4f_PWN_repaired.dat"
        ));
        assert!(database.is_file(), "missing repaired database at {:?}", database);
    }

    #[tokio::test]
    async fn zero_timeout_is_rejected_before_spawning() {
        let mut request = amd_request();
        request.timeout_secs = Some(0);
        let error = run_phreeqc(&request).await.unwrap_err();
        assert!(error.contains("timeout_secs"), "unexpected: {error}");
    }
}
