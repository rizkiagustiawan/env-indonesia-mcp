//! 1D (EPA SWMM sewer surcharge) → 2D (shallow-water overland flow) coupling.
//!
//! The coupling is deliberately one-way and volume-based: each SWMM node that
//! floods is converted into an equivalent steady point discharge over the 2D
//! simulation window, injected into the SWE solver, and then the injected
//! volume is compared against the 1D surcharge volume by a **mass-balance
//! gate**. A coupled result that fails the gate is still emitted, but carries
//! an explicit limitation saying it must not be used.
//!
//! Coupled results are never `ResultStatus::Valid`: the 2D overland extent is
//! not validated against observed flood extent, so the honesty ladder caps the
//! result at [`MaturityLevel::Screening`].

use crate::honesty::{self, MaturityLevel};
use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use crate::swmm_runner::SwmmRunResult;
use crate::tools::advanced_physics::swe_solver::InflowSource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Maps one SWMM node id onto a cell of the 2D DEM grid.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeGridMapping {
    pub node_id: String,
    pub grid_x: usize,
    pub grid_y: usize,
}

/// Outcome of the volume conservation check between the 1D and 2D models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingGate {
    /// Total 1D flooding (surcharge) volume reported by SWMM.
    pub surcharge_volume_m3: f64,
    /// Total water volume actually injected into / retained by the 2D solver.
    pub injected_volume_m3: f64,
    pub mass_error_pct: f64,
    pub gate_passed: bool,
    pub tolerance_pct: f64,
}

/// Default mass-balance tolerance, matching the SWE solver's own 1% gate.
pub const DEFAULT_MASS_TOLERANCE_PCT: f64 = 1.0;

/// Convert every flooded SWMM node into an equivalent steady 2D point source.
///
/// The discharge is the node's flooding volume spread evenly over `duration_s`
/// (the 2D simulation window), so injecting it at `duty_fraction = 1.0`
/// reproduces the 1D surcharge volume exactly.
///
/// Errors when `duration_s` is not finite and positive, or when any node in the
/// SWMM result lacks a grid mapping (silently dropping surcharge volume would
/// break the mass-balance gate that follows).
pub fn build_sources(
    run: &SwmmRunResult,
    mapping: &[NodeGridMapping],
    duration_s: f64,
) -> Result<Vec<InflowSource>, String> {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return Err(format!(
            "coupling duration_s must be finite and greater than zero (got {})",
            duration_s
        ));
    }

    let mut sources = Vec::with_capacity(run.nodes.len());
    for node in &run.nodes {
        let cell = mapping
            .iter()
            .find(|entry| entry.node_id == node.node_id)
            .ok_or_else(|| {
                format!(
                    "SWMM node {} has no grid mapping; every flooding node must be mapped",
                    node.node_id
                )
            })?;
        if !node.flooding_volume_m3.is_finite() {
            return Err(format!(
                "SWMM node {} reported a non-finite flooding volume",
                node.node_id
            ));
        }
        sources.push(InflowSource {
            x: cell.grid_x,
            y: cell.grid_y,
            discharge_m3s: node.flooding_volume_m3 / duration_s,
        });
    }
    Ok(sources)
}

/// Compare the 2D injected volume against the 1D surcharge volume.
pub fn check_mass_balance(surcharge_m3: f64, injected_m3: f64, tolerance_pct: f64) -> CouplingGate {
    let mass_error_pct = if surcharge_m3 > 0.0 {
        (injected_m3 - surcharge_m3).abs() / surcharge_m3 * 100.0
    } else if injected_m3 == 0.0 {
        0.0
    } else {
        100.0
    };
    CouplingGate {
        surcharge_volume_m3: surcharge_m3,
        injected_volume_m3: injected_m3,
        mass_error_pct,
        gate_passed: mass_error_pct <= tolerance_pct,
        tolerance_pct,
    }
}

/// Wrap a coupled 2D depth into the scientific result contract.
///
/// Status is derived from the honesty ladder at [`MaturityLevel::Screening`],
/// so it can never be `ResultStatus::Valid`.
pub fn coupling_result(gate: &CouplingGate, max_depth_m: f64, flooded_cells: usize) -> ScientificResult {
    let mut result = ScientificResult::new("coupled_max_flood_depth", max_depth_m, "m")
        .with_status(honesty::to_result_status(MaturityLevel::Screening))
        .with_provenance(Provenance::new(
            "model",
            "epa_swmm_1d + swe_hllc_2d coupling",
            &chrono::Utc::now().to_rfc3339(),
        ))
        .with_claim(Claim::new(
            "mass_balance",
            &format!(
                "1D surcharge {:.3} m3 vs 2D injected {:.3} m3; mass_error_pct = {:.4} (tolerance {:.4}%)",
                gate.surcharge_volume_m3, gate.injected_volume_m3, gate.mass_error_pct, gate.tolerance_pct
            ),
        ))
        .with_claim(Claim::new(
            "coupling_gate",
            &format!("gate_passed = {}", gate.gate_passed),
        ))
        .with_claim(Claim::new(
            "flood_extent",
            &format!("flooded_cells = {}", flooded_cells),
        ))
        .with_limitation(
            "1D sewer surcharge coupled to 2D overland flow without observed flood extent validation",
        )
        .with_limitation(
            "EPA SWMM executed as an external untrusted process; its invocation is recorded in the audit chain",
        );

    if !gate.gate_passed {
        result = result.with_limitation("mass-balance gate FAILED; coupling result must not be used");
    }
    result
}

/// Convenience guard used by callers that must refuse to interpret a failed gate.
pub fn gate_blocks_interpretation(gate: &CouplingGate) -> bool {
    !gate.gate_passed
}

/// Emit a contract-shaped failure for any coupling error path.
pub fn coupling_failure(message: &str) -> ScientificResult {
    ScientificResult::new("coupled_max_flood_depth", 0.0, "m")
        .with_status(ResultStatus::ValidationFailed)
        .with_claim(Claim::new("coupling_error", message))
        .with_limitation("1D/2D coupling did not execute; no flood depth was computed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swmm_runner::{SwmmNodeResult, SwmmRoutingBalance};

    /// Build a SWMM result carrying exactly one flooding node.
    fn run_with_node(node_id: &str, flooding_volume: f64) -> SwmmRunResult {
        SwmmRunResult {
            status: "ok".to_string(),
            pyswmm_version: "2.1.0".to_string(),
            inp_sha256: "a".repeat(64),
            routing: SwmmRoutingBalance {
                external_inflow_m3: flooding_volume,
                flooding_m3: flooding_volume,
                outflow_m3: 0.0,
                initial_storage_m3: 0.0,
                final_storage_m3: 0.0,
                routing_error_pct: 0.0,
            },
            nodes: vec![SwmmNodeResult {
                node_id: node_id.to_string(),
                flooding_volume_m3: flooding_volume,
                peak_flooding_rate_m3s: 0.0,
                flooding_duration_hr: 1.0,
                max_depth_m: 2.0,
                invert_elevation_m: 10.0,
            }],
        }
    }

    #[test]
    fn node_flooding_volume_becomes_equivalent_steady_discharge() {
        let run = run_with_node("J1", 1800.0);
        let mapping = vec![NodeGridMapping { node_id: "J1".into(), grid_x: 4, grid_y: 4 }];
        let sources = build_sources(&run, &mapping, 3600.0).unwrap();
        assert_eq!(sources.len(), 1);
        assert!((sources[0].discharge_m3s - 0.5).abs() < 1e-9);
        assert_eq!((sources[0].x, sources[0].y), (4, 4));
    }

    #[test]
    fn unmapped_flooded_node_is_rejected() {
        let run = run_with_node("J9", 100.0);
        // `.err().unwrap()` rather than `.unwrap_err()`: `InflowSource` (owned by
        // the SWE solver module) does not derive `Debug`, which `unwrap_err`
        // requires on the Ok type.
        let err = build_sources(&run, &[], 3600.0).err().unwrap();
        assert!(err.contains("J9"));
    }

    #[test]
    fn zero_duration_is_rejected() {
        let run = run_with_node("J1", 100.0);
        let mapping = vec![NodeGridMapping { node_id: "J1".into(), grid_x: 1, grid_y: 1 }];
        assert!(build_sources(&run, &mapping, 0.0).is_err());
    }

    #[test]
    fn mass_balance_gate_passes_within_tolerance() {
        let gate = check_mass_balance(1000.0, 1005.0, 1.0);
        assert!(gate.gate_passed);
        assert!(gate.mass_error_pct.abs() < 1.0);
    }

    #[test]
    fn mass_balance_gate_fails_beyond_tolerance() {
        let gate = check_mass_balance(1000.0, 1200.0, 1.0);
        assert!(!gate.gate_passed);
        assert!((gate.mass_error_pct - 20.0).abs() < 1e-6);
    }

    #[test]
    fn coupled_result_is_never_valid_status() {
        let gate = check_mass_balance(1000.0, 1000.0, 1.0);
        let result = coupling_result(&gate, 1.23, 7);
        assert_ne!(result.status, ResultStatus::Valid);
        assert!(result.validate().is_ok());
    }
}
