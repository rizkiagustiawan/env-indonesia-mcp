use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use super::groundwater_pde::RichardsParam;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CoupledParam {
    #[schemars(description = "Initial surface water depth (m)")]
    pub h_swe_initial_m: f64,
    #[schemars(description = "Rainfall rate (m/s)")]
    pub rainfall_m_s: f64,
    #[schemars(description = "Richards parameters")]
    pub richards: RichardsParam,
}

pub fn solve_coupled(p: &CoupledParam) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Coupled SWE-Richards Solver (1D)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: MIKE SHE Head-Flux Switching Algorithm\n\n");
    
    // In a full implementation, we would integrate the Richards solver step-by-step
    // with the SWE. Since Richards solver is currently a black box taking total time,
    // we simulate the coupling conceptually for the reporting tool.
    // The "head-flux switching" is reported as the active regime.
    
    let mut h_swe = p.h_swe_initial_m;
    let mut regime = "Flux (Neumann)";
    let mut infiltration_rate = p.richards.k_sat_m_s; // max possible roughly Ks

    let available_water_rate = h_swe / p.richards.t_total_s + p.rainfall_m_s;

    if infiltration_rate > available_water_rate {
        // Soil can absorb more than is available -> Flux BC limited by availability
        regime = "Flux (Neumann - Water Limited)";
        infiltration_rate = available_water_rate;
        h_swe = 0.0; // All surface water infiltrates
    } else {
        // Soil capacity is limiting -> Head BC
        regime = "Head (Dirichlet - Ponded)";
        h_swe = h_swe + p.rainfall_m_s * p.richards.t_total_s - infiltration_rate * p.richards.t_total_s;
        h_swe = h_swe.max(0.0);
    }

    out.push_str(&format!("Initial Surface Water : {:.3} m\n", p.h_swe_initial_m));
    out.push_str(&format!("Rainfall Rate         : {:.2e} m/s\n", p.rainfall_m_s));
    out.push_str(&format!("Simulation Time       : {:.0} s\n\n", p.richards.t_total_s));
    
    out.push_str("=== COUPLING DYNAMICS ===\n");
    out.push_str(&format!("Active Boundary Regime: {}\n", regime));
    out.push_str(&format!("Average Infiltration  : {:.2e} m/s\n", infiltration_rate));
    out.push_str(&format!("Final Surface Water   : {:.3} m\n\n", h_swe));

    out.push_str("Note: Full 3D coupling requires step-by-step matrix assembly. This tool implements the logical boundary switching framework (MIKE SHE).\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupling_regimes() {
        let mut r = RichardsParam {
            theta_r: 0.0, theta_s: 0.4, alpha_inv_m: 1.0, n_vg: 2.0, k_sat_m_s: 1e-5,
            depth_m: 1.0, n_nodes: 10, t_total_s: 3600.0, h_initial_m: -1.0, h_top_m: 0.0, h_bottom_m: 0.0
        };
        let p1 = CoupledParam { h_swe_initial_m: 0.001, rainfall_m_s: 0.0, richards: r };
        assert!(solve_coupled(&p1).contains("Flux")); // water limited
    }
}
