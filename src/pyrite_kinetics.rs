//! Time-dependent pyrite oxidation (acid mine drainage generation) via PHREEQC
//! KINETICS, executed as a Python subprocess.
//!
//! This closes the last gap between the two AMD tools that already exist:
//! static ABA screening (MPA/NAPP) says how much acid a rock *could* produce,
//! and `phreeqc_speciation` says what a given water *is*. Neither answers how
//! fast acid appears, which is what decides whether a pit needs treatment in
//! month three or year thirty.
//!
//! Rate law: Williamson & Rimstidt (1994), taken from the RATES block of the
//! loaded database rather than reimplemented here.
//!
//! Four honesty guards travel with every result:
//!
//! * `oxygen_limited` — a closed system consumes its dissolved O2 and the
//!   reaction stalls. The pH curve then flattens at a value that looks like a
//!   stable long-term outcome but is purely an artifact of the sealed box.
//! * `pyrite_depleted` — once the sulfide is spent the curve also flattens, for
//!   the opposite and physically real reason. The two must not be confused.
//! * `stoichiometry_consistent` — FeS2 releases 2 mol S per mol Fe. When iron
//!   precipitates as ferrihydrite the ratio explodes (observed: 73,377), so
//!   dissolved Fe stops being a measure of how much pyrite oxidised.
//! * `rate_is_laboratory_derived` — always true, and always reported: field
//!   rates are commonly one to two orders of magnitude slower than laboratory
//!   rates, so an uncalibrated absolute timescale is not a prediction.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use crate::swmm_runner::swmm_python;

/// Absolute path of the Python wrapper, resolved at compile time.
pub const PYRITE_SCRIPT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/pyrite_kinetics.py");

/// Hard cap on captured stdout: 8 MiB.
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;

/// Resolve the wrapper script, honouring an environment override.
pub fn pyrite_script() -> String {
    std::env::var("ENV_INDONESIA_PYRITE_SCRIPT").unwrap_or_else(|_| PYRITE_SCRIPT.to_string())
}

/// The interpreter is the same venv that carries `phreeqpython`.
pub fn pyrite_python() -> String {
    swmm_python()
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PyriteKineticsRequest {
    /// Reactive pyrite available per kg of pore water, mol/kgw.
    pub pyrite_mol_per_kgw: f64,
    #[serde(default)]
    pub initial_ph: Option<f64>,
    #[serde(default)]
    pub initial_o2_mmol: Option<f64>,
    /// When true the system stays in contact with the atmosphere (open pit or
    /// waste dump). When false the dissolved O2 is finite and the reaction will
    /// stall, which is reported as `oxygen_limited`.
    #[serde(default = "default_true")]
    pub replenish_o2: bool,
    #[serde(default)]
    pub o2_partial_pressure_log10: Option<f64>,
    /// Cumulative, strictly increasing output times in days.
    #[serde(default)]
    pub steps_days: Option<Vec<f64>>,
    /// Williamson & Rimstidt parameters: [log10(A/V), exp(m/m0), exp(O2), exp(H+)].
    #[serde(default)]
    pub parms: Option<Vec<f64>>,
    #[serde(default)]
    pub temperature_c: Option<f64>,
    /// Buffering / precipitating phases held at equilibrium, e.g. `["Calcite"]`.
    #[serde(default)]
    pub neutralising_phases: Option<Vec<String>>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KineticStep {
    pub time_days: f64,
    pub ph: f64,
    #[serde(default)]
    pub pe: Option<f64>,
    pub fe_mol_kgw: f64,
    pub sulfate_mol_kgw: f64,
    #[serde(default)]
    pub pyrite_remaining_mol_kgw: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KineticGuards {
    pub oxygen_replenished: bool,
    pub oxygen_limited: bool,
    pub late_ph_change: f64,
    #[serde(default)]
    pub pyrite_consumed_fraction: Option<f64>,
    pub pyrite_depleted: bool,
    #[serde(default)]
    pub sulfate_to_iron_ratio: Option<f64>,
    pub stoichiometry_consistent: bool,
    pub initial_ph: f64,
    pub final_ph: f64,
    pub simulated_days: f64,
    pub rate_is_laboratory_derived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyriteKineticsResult {
    pub status: String,
    pub database: String,
    pub database_sha256: String,
    pub rate_law: String,
    pub parms: Vec<f64>,
    pub replenish_o2: bool,
    #[serde(default)]
    pub neutralising_phases: Vec<String>,
    pub initial_pyrite_mol_kgw: f64,
    pub series: Vec<KineticStep>,
    pub guards: KineticGuards,
}

#[derive(Debug, Clone, Deserialize)]
struct PyriteFailure {
    status: String,
    #[serde(default)]
    error: String,
}

/// Validate the request before spending a subprocess on it.
pub fn validate_request(request: &PyriteKineticsRequest) -> Result<(), String> {
    if !request.pyrite_mol_per_kgw.is_finite() || request.pyrite_mol_per_kgw <= 0.0 {
        return Err("pyrite_mol_per_kgw must be a positive finite number".to_string());
    }
    if let Some(ph) = request.initial_ph {
        if !ph.is_finite() || !(0.0..14.0).contains(&ph) {
            return Err(format!("initial_ph must be between 0 and 14 (got {})", ph));
        }
    }
    if let Some(o2) = request.initial_o2_mmol {
        if !o2.is_finite() || o2 < 0.0 {
            return Err("initial_o2_mmol must not be negative".to_string());
        }
    }
    if let Some(temperature) = request.temperature_c {
        if !temperature.is_finite() || !(-10.0..=100.0).contains(&temperature) {
            return Err(format!(
                "temperature_c must be between -10 and 100 (got {})",
                temperature
            ));
        }
    }
    if let Some(parms) = &request.parms {
        if parms.len() != 4 {
            return Err(format!(
                "parms must contain exactly 4 numbers, got {}",
                parms.len()
            ));
        }
        if parms.iter().any(|value| !value.is_finite()) {
            return Err("parms must contain only finite numbers".to_string());
        }
    }
    if let Some(steps) = &request.steps_days {
        if steps.is_empty() {
            return Err("steps_days must not be empty".to_string());
        }
        let mut previous = 0.0_f64;
        for (index, step) in steps.iter().enumerate() {
            if !step.is_finite() || *step <= 0.0 {
                return Err(format!("steps_days[{}] must be positive", index));
            }
            if *step <= previous {
                return Err(
                    "steps_days must be strictly increasing cumulative times".to_string()
                );
            }
            previous = *step;
        }
    }
    Ok(())
}

/// True when the simulated acid-generation trajectory can be read as a
/// screening-level trend rather than an artifact.
///
/// A run that stalled because its sealed box ran out of oxygen, or whose
/// dissolved iron no longer tracks oxidation, describes the model setup rather
/// than the waste rock.
pub fn trajectory_is_interpretable(result: &PyriteKineticsResult) -> bool {
    !result.guards.oxygen_limited && result.guards.stoichiometry_consistent
}

/// Run a pyrite oxidation kinetics simulation.
pub async fn run_pyrite_kinetics(
    request: &PyriteKineticsRequest,
) -> Result<PyriteKineticsResult, String> {
    validate_request(request)?;
    let timeout_secs = request.timeout_secs.unwrap_or(180);
    if timeout_secs == 0 {
        return Err("pyrite kinetics timeout_secs must be greater than zero".to_string());
    }

    let script = pyrite_script();
    if !PathBuf::from(&script).is_file() {
        return Err(format!("pyrite kinetics wrapper script not found: {}", script));
    }
    let payload = serde_json::to_vec(request)
        .map_err(|error| format!("could not serialise pyrite kinetics request: {}", error))?;

    let python = pyrite_python();
    let mut command = tokio::process::Command::new(&python);
    command
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = command.spawn().map_err(|error| {
        format!("Failed to spawn pyrite kinetics runner ({}): {}", python, error)
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(&payload)
            .await
            .map_err(|error| format!("could not write pyrite kinetics request: {}", error))?;
        stdin
            .shutdown()
            .await
            .map_err(|error| format!("could not close pyrite kinetics stdin: {}", error))?;
    }

    let output = tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output())
        .await
        .map_err(|_| format!("pyrite kinetics run exceeded {}s timeout", timeout_secs))?
        .map_err(|error| format!("pyrite kinetics runner failed: {}", error))?;

    if output.stdout.len() > MAX_STDOUT_BYTES {
        return Err(format!(
            "pyrite kinetics runner stdout exceeded {} bytes",
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
            "pyrite kinetics runner produced no JSON (exit {:?}): {}",
            output.status.code(),
            stderr.trim()
        ));
    }

    if !output.status.success() {
        let detail = serde_json::from_str::<PyriteFailure>(line)
            .map(|failure| format!("{}: {}", failure.status, failure.error))
            .unwrap_or_else(|_| line.to_string());
        return Err(format!(
            "pyrite kinetics runner exited with {:?}: {}",
            output.status.code(),
            detail
        ));
    }

    let result: PyriteKineticsResult = serde_json::from_str(line)
        .map_err(|error| format!("pyrite kinetics runner emitted unparseable JSON: {}", error))?;

    if result.status != "ok" {
        return Err(format!("pyrite kinetics run status was {}", result.status));
    }
    if result.series.is_empty() {
        return Err("pyrite kinetics run returned an empty time series".to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> PyriteKineticsRequest {
        PyriteKineticsRequest {
            pyrite_mol_per_kgw: 0.05,
            initial_ph: Some(6.5),
            initial_o2_mmol: Some(0.27),
            replenish_o2: true,
            o2_partial_pressure_log10: None,
            steps_days: Some(vec![1.0, 30.0, 90.0, 180.0, 365.0]),
            parms: None,
            temperature_c: None,
            neutralising_phases: None,
            timeout_secs: Some(180),
        }
    }

    fn result_with(oxygen_limited: bool, stoichiometry_consistent: bool) -> PyriteKineticsResult {
        PyriteKineticsResult {
            status: "ok".to_string(),
            database: "wateq4f_PWN_repaired.dat".to_string(),
            database_sha256: "c0f6".to_string(),
            rate_law: "Williamson & Rimstidt (1994)".to_string(),
            parms: vec![1.0, 0.67, 0.5, -0.11],
            replenish_o2: !oxygen_limited,
            neutralising_phases: vec![],
            initial_pyrite_mol_kgw: 0.05,
            series: vec![KineticStep {
                time_days: 365.0,
                ph: 2.99,
                pe: Some(13.0),
                fe_mol_kgw: 7.57e-4,
                sulfate_mol_kgw: 1.51e-3,
                pyrite_remaining_mol_kgw: Some(0.04924),
            }],
            guards: KineticGuards {
                oxygen_replenished: !oxygen_limited,
                oxygen_limited,
                late_ph_change: if oxygen_limited { -3e-5 } else { 0.381 },
                pyrite_consumed_fraction: Some(0.0151),
                pyrite_depleted: false,
                sulfate_to_iron_ratio: Some(if stoichiometry_consistent { 2.0 } else { 73377.0 }),
                stoichiometry_consistent,
                initial_ph: 4.97,
                final_ph: 2.99,
                simulated_days: 365.0,
                rate_is_laboratory_derived: true,
            },
        }
    }

    #[test]
    fn accepts_a_well_formed_request() {
        assert!(validate_request(&valid_request()).is_ok());
    }

    #[test]
    fn non_positive_pyrite_is_rejected() {
        let mut request = valid_request();
        request.pyrite_mol_per_kgw = 0.0;
        assert!(validate_request(&request)
            .unwrap_err()
            .contains("pyrite_mol_per_kgw"));

        let mut nan = valid_request();
        nan.pyrite_mol_per_kgw = f64::NAN;
        assert!(validate_request(&nan).is_err());
    }

    #[test]
    fn out_of_range_ph_and_temperature_are_rejected() {
        let mut bad_ph = valid_request();
        bad_ph.initial_ph = Some(20.0);
        assert!(validate_request(&bad_ph).unwrap_err().contains("initial_ph"));

        let mut bad_temp = valid_request();
        bad_temp.temperature_c = Some(500.0);
        assert!(validate_request(&bad_temp)
            .unwrap_err()
            .contains("temperature_c"));
    }

    #[test]
    fn steps_must_be_strictly_increasing() {
        let mut request = valid_request();
        request.steps_days = Some(vec![30.0, 30.0]);
        assert!(validate_request(&request)
            .unwrap_err()
            .contains("strictly increasing"));

        let mut backwards = valid_request();
        backwards.steps_days = Some(vec![90.0, 30.0]);
        assert!(validate_request(&backwards).is_err());

        let mut empty = valid_request();
        empty.steps_days = Some(vec![]);
        assert!(validate_request(&empty).unwrap_err().contains("empty"));
    }

    #[test]
    fn parms_must_have_exactly_four_entries() {
        let mut request = valid_request();
        request.parms = Some(vec![1.0, 0.67]);
        assert!(validate_request(&request).unwrap_err().contains("exactly 4"));
    }

    #[test]
    fn oxygen_limited_run_is_not_interpretable() {
        // A sealed system exhausts its dissolved O2 and the pH curve flattens.
        // That plateau describes the box, not the waste rock.
        let stalled = result_with(true, true);
        assert!(stalled.guards.oxygen_limited);
        assert!(!trajectory_is_interpretable(&stalled));
    }

    #[test]
    fn iron_precipitation_breaks_the_stoichiometry_guard() {
        // FeS2 gives 2 mol S per mol Fe. When ferrihydrite precipitates the
        // observed ratio reached 73,377, so dissolved Fe no longer measures how
        // much pyrite oxidised.
        let precipitating = result_with(false, false);
        assert!(precipitating.guards.sulfate_to_iron_ratio.unwrap() > 100.0);
        assert!(!precipitating.guards.stoichiometry_consistent);
        assert!(!trajectory_is_interpretable(&precipitating));
    }

    #[test]
    fn healthy_open_system_run_is_interpretable() {
        let healthy = result_with(false, true);
        assert!(trajectory_is_interpretable(&healthy));
        // The laboratory-rate caveat is never dropped, even for a good run.
        assert!(healthy.guards.rate_is_laboratory_derived);
    }

    #[test]
    fn parses_the_documented_contract() {
        let json = r#"{"status":"ok","database":"wateq4f_PWN_repaired.dat","database_sha256":"c0f6","rate_law":"Williamson & Rimstidt (1994)","parms":[1.0,0.67,0.5,-0.11],"replenish_o2":true,"neutralising_phases":["Calcite"],"initial_pyrite_mol_kgw":0.05,"series":[{"time_days":1.0,"ph":4.969,"pe":13.2,"fe_mol_kgw":3.528e-06,"sulfate_mol_kgw":7.056e-06,"pyrite_remaining_mol_kgw":0.05},{"time_days":365.0,"ph":2.986,"pe":13.4,"fe_mol_kgw":7.572e-04,"sulfate_mol_kgw":1.5144e-03,"pyrite_remaining_mol_kgw":0.04924}],"guards":{"oxygen_replenished":true,"oxygen_limited":false,"late_ph_change":0.381,"pyrite_consumed_fraction":0.0151,"pyrite_depleted":false,"sulfate_to_iron_ratio":2.0,"stoichiometry_consistent":true,"initial_ph":4.969,"final_ph":2.986,"simulated_days":365.0,"rate_is_laboratory_derived":true}}"#;
        let parsed: PyriteKineticsResult = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.series.len(), 2);
        assert_eq!(parsed.neutralising_phases, vec!["Calcite".to_string()]);
        assert!((parsed.guards.final_ph - 2.986).abs() < 1e-9);
        assert!(trajectory_is_interpretable(&parsed));
    }

    #[test]
    fn wrapper_script_path_is_absolute_and_present() {
        let script = std::path::Path::new(PYRITE_SCRIPT);
        assert!(script.is_absolute(), "PYRITE_SCRIPT must not depend on CWD");
        assert!(script.is_file(), "missing wrapper script at {}", PYRITE_SCRIPT);
    }

    #[tokio::test]
    async fn zero_timeout_is_rejected_before_spawning() {
        let mut request = valid_request();
        request.timeout_secs = Some(0);
        let error = run_pyrite_kinetics(&request).await.unwrap_err();
        assert!(error.contains("timeout_secs"), "unexpected: {error}");
    }
}
