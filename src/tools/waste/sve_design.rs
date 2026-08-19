use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Soil Vapor Extraction (SVE) Design
/// Ref: Staudinger et al. 1997 "Simplified approach for SVE design"
///   Lowe et al. 1999 "SVE using RF Heating"
///   Shi et al. 2022; Zhao & Zytner 2013

pub fn design(
    k_air_m2: f64,          // air permeability (m²)
    screen_length_m: f64,   // well screen length
    vacuum_pressure_kpa: f64, // applied vacuum (below atmospheric)
    contaminant: &str,
    napl_mass_kg: f64,      // NAPL mass in soil
    soil_porosity: f64,
    soil_temp_c: f64,
    cleanup_time_target_days: f64,
) -> String {
    if k_air_m2 <= 0.0 || screen_length_m <= 0.0 {
        return json!({"error": "E102", "message": "Air permeability and screen length must be > 0"}).to_string();
    }

    // Convert vacuum pressure
    let p_atm = 101.3; // kPa
    let p_well = p_atm - vacuum_pressure_kpa; // absolute pressure at well
    let mu_air = 1.81e-5; // Pa·s at 20°C
    let p_well_pa = p_well * 1000.0; // Pa
    let p_atm_pa = p_atm * 1000.0;

    let r_well = 0.05_f64; // well radius
    let r_influence = 10.0_f64; // initial estimate

    let p_w_sq = p_well_pa * p_well_pa;
    let p_atm_sq = p_atm_pa * p_atm_pa;
    let ln_ratio = (r_influence / r_well).ln();

    let q_air_m3_s = std::f64::consts::PI * k_air_m2 * screen_length_m
        * (p_w_sq - p_atm_sq).abs() / (mu_air * p_atm_pa * ln_ratio).max(1e-15);
    let q_air_m3_hr = q_air_m3_s * 3600.0;

    // Radius of Influence (Transient)
    let t_sec = cleanup_time_target_days * 86400.0;
    let r_i_transient = (q_air_m3_s * t_sec / (std::f64::consts::PI * soil_porosity * screen_length_m).max(1e-6)).sqrt();

    // Vapor Concentration (Raoult's Law + Antoine Equation)
    let (mw, ant_a, ant_b, ant_c, _name) = match contaminant.to_lowercase().as_str() {
        "benzene" => (78.11, 6.90565, 1211.033, 220.79, "Benzene"),
        "toluene" => (92.14, 6.95464, 1344.800, 219.482, "Toluene"),
        "xylene" | "o-xylene" => (106.17, 6.99052, 1453.430, 215.307, "Xylene"),
        "tce" | "trichloroethylene" => (131.39, 6.5183, 1018.6, 192.7, "TCE"),
        "pce" | "perchloroethylene" => (165.83, 6.98807, 1386.2, 207.4, "PCE"),
        "gasoline" | "btx" => (100.0, 6.90000, 1200.0, 220.0, "Gasoline (avg)"),
        _ => (100.0, 6.90000, 1200.0, 220.0, "Unknown (assumed)"),
    };

    let log_p = ant_a - ant_b / (ant_c + soil_temp_c);
    let p_sat_mmhg = 10.0_f64.powf(log_p);
    let p_sat_kpa = p_sat_mmhg * 0.133322; // mmHg to kPa

    let temp_k = soil_temp_c + 273.15;
    let r_gas = 8.314; // J/(mol·K)
    let x_i = 1.0; // assume pure NAPL
    let c_vapor_mg_m3 = x_i * p_sat_kpa * 1000.0 * mw / (r_gas * temp_k) * 1000.0; // mg/m3

    // Mass Removal Rate
    let removal_rate_g_hr = q_air_m3_hr * c_vapor_mg_m3 / 1000.0;
    let removal_rate_kg_day = removal_rate_g_hr * 24.0 / 1000.0;

    let cleanup_time_days = napl_mass_kg / removal_rate_kg_day.max(1e-10);

    let area_per_well = std::f64::consts::PI * r_i_transient * r_i_transient;
    let total_area = std::f64::consts::PI * r_i_transient * r_i_transient * 2.0; // assume 2× radius area
    let n_wells = (total_area / area_per_well).ceil().max(1.0) as f64;

    let status = if k_air_m2 < 1e-14 {
        ResultStatus::ValidWithAssumptions
    } else {
        ResultStatus::ScreeningOnly
    };

    let res_q_air = ScientificResult::new("airflow_rate", q_air_m3_s, "m3/s")
        .with_status(status.clone())
        .with_provenance(Provenance::new("calculation", "RadialSteadyState_Staudinger", "2026-08-19T00:00:00Z"));

    let res_r_influence = ScientificResult::new("radius_of_influence_transient", r_i_transient, "m")
        .with_status(status.clone())
        .with_provenance(Provenance::new("calculation", "Transient_Staudinger", "2026-08-19T00:00:00Z"));

    let res_removal_rate = ScientificResult::new("mass_removal_rate", removal_rate_kg_day, "kg/day")
        .with_status(status.clone())
        .with_provenance(Provenance::new("calculation", "Raoult_Antoine", "2026-08-19T00:00:00Z"));

    let res_cleanup_time = ScientificResult::new("estimated_cleanup_time", cleanup_time_days, "days")
        .with_status(status.clone())
        .with_provenance(Provenance::new("calculation", "Raoult_Antoine", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("limitation", "Assumes homogeneous soil, steady-state radial flow, and pure NAPL. No desorption-limited mass transfer modeled."));

    let res_n_wells = ScientificResult::new("recommended_wells", n_wells, "count")
        .with_status(status.clone())
        .with_provenance(Provenance::new("calculation", "Area_Ratio", "2026-08-19T00:00:00Z"));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_q_air.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_r_influence.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_removal_rate.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_cleanup_time.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_n_wells.emit_validated()).unwrap()
    ]).to_string()
}
