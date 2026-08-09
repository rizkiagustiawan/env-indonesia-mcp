// STUB (dead code, not wired to server.rs): returns index=1.0, no actual water quality model.
// To implement: call calculators::streeter_phelps + compliance::baku_mutu_air_permukaan.
use crate::result_contract::{ScientificResult, ResultStatus, Claim};

pub fn run_water_quality_assessment(
    wwtp_flow_m3d: f64,
    river_width_m: f64,
    river_depth_m: f64,
    river_velocity_ms: f64,
) -> Result<ScientificResult, String> {
    // 1. Physical Continuity Check (Guardrail)
    // River discharge in m3/s
    let river_discharge_m3s = river_width_m * river_depth_m * river_velocity_ms;
    let river_discharge_m3d = river_discharge_m3s * 86400.0;

    // If WWTP flow is > 50% of the river flow, the river cannot properly dilute it.
    // In many environmental regs, mixing zone models become invalid if effluent dominates the flow.
    if wwtp_flow_m3d > (0.5 * river_discharge_m3d) {
        return Err(format!(
            "Hydrological continuity violation: WWTP discharge ({:.1} m3/d) exceeds 50% of river baseflow ({:.1} m3/d). Dilution assumption invalid.",
            wwtp_flow_m3d, river_discharge_m3d
        ));
    }

    let res = ScientificResult::new("integrated_water_quality", 1.0, "index")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_claim(Claim::new("hydrology", "Discharge ratio is within valid mixing bounds."));

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_water_quality_assessment_fails_on_hydrological_violation() {
        // Small river: 1m wide, 0.5m deep, 0.1 m/s = 0.05 m3/s = 4320 m3/d
        // WWTP: 5000 m3/d
        let res = run_water_quality_assessment(5000.0, 1.0, 0.5, 0.1);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Hydrological continuity violation"));
    }

    #[test]
    fn test_water_quality_assessment_passes_on_valid_flow() {
        // River: 10m wide, 2m deep, 1 m/s = 20 m3/s = 1,728,000 m3/d
        // WWTP: 5000 m3/d
        let res = run_water_quality_assessment(5000.0, 10.0, 2.0, 1.0);
        assert!(res.is_ok());
    }
}
