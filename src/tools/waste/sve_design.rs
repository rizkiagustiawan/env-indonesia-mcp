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
    let mut out = String::from("=== Soil Vapor Extraction (SVE) Design ===\n");
    out.push_str("Ref: Staudinger et al. 1997; Lowe et al. 1999; Shi et al. 2022\n\n");

    if k_air_m2 <= 0.0 || screen_length_m <= 0.0 {
        return "ERROR [E102]: air permeability and screen length must be > 0.".into();
    }

    // Convert vacuum pressure
    let p_atm = 101.3; // kPa
    let p_well = p_atm - vacuum_pressure_kpa; // absolute pressure at well
    let mu_air = 1.81e-5; // Pa·s at 20°C
    let p_well_pa = p_well * 1000.0; // Pa
    let p_atm_pa = p_atm * 1000.0;

    out.push_str(&format!("Air permeability: {:.2e} m²\n", k_air_m2));
    out.push_str(&format!("Screen length: {:.1} m\n", screen_length_m));
    out.push_str(&format!("Vacuum: {:.1} kPa (well abs: {:.1} kPa)\n", vacuum_pressure_kpa, p_well));
    out.push_str(&format!("Porosity: {:.2}, Temp: {:.0}°C\n\n", soil_porosity, soil_temp_c));

    // ═══ Airflow Rate (Radial Flow) ═══
    out.push_str("── Airflow Rate (Radial Steady-State) ──\n\n");

    // Q = 2π × k_air × H × (P_w² - P_atm²) / (μ × P_atm × ln(R_w/R_i))
    // For initial estimate, assume R_i = 10 m, R_w = 0.05 m
    let r_well = 0.05; // well radius
    let r_influence = 10.0; // initial estimate

    let p_w_sq = p_well_pa * p_well_pa;
    let p_atm_sq = p_atm_pa * p_atm_pa;
    let ln_ratio = (r_influence as f64 / r_well as f64).ln();

    let q_air_m3_s = 2.0 * std::f64::consts::PI * k_air_m2 * screen_length_m
        * (p_w_sq - p_atm_sq).abs() / (mu_air * p_atm_pa * ln_ratio).max(1e-15);
    let q_air_m3_min = q_air_m3_s * 60.0;
    let q_air_m3_hr = q_air_m3_s * 3600.0;

    out.push_str(&format!("  Well radius: {:.2} m, Radius of influence: {:.0} m\n", r_well, r_influence));
    out.push_str(&format!("  ln(R_i/R_w) = {:.2}\n", ln_ratio));
    out.push_str(&format!("  ► Airflow rate: {:.2} m³/s ({:.0} m³/min, {:.0} m³/hr)\n\n", q_air_m3_s, q_air_m3_min, q_air_m3_hr));

    // ═══ Radius of Influence (Transient) ═══
    out.push_str("── Radius of Influence (Transient) ──\n\n");

    // R_i = sqrt(Q × t / (π × n × H))
    let t_sec = cleanup_time_target_days * 86400.0;
    let r_i_transient = (q_air_m3_s * t_sec / (std::f64::consts::PI * soil_porosity * screen_length_m).max(1e-6)).sqrt();

    out.push_str(&format!("  At t = {:.0} days: R_i = {:.1} m\n", cleanup_time_target_days, r_i_transient));
    out.push_str(&format!("  At t = 1 day: R_i = {:.1} m\n",
        (q_air_m3_s * 86400.0 / (std::f64::consts::PI * soil_porosity * screen_length_m).max(1e-6)).sqrt()));
    out.push_str(&format!("  At t = 30 days: R_i = {:.1} m\n\n",
        (q_air_m3_s * 30.0 * 86400.0 / (std::f64::consts::PI * soil_porosity * screen_length_m).max(1e-6)).sqrt()));

    // ═══ Vapor Concentration (Raoult's Law) ═══
    out.push_str("── Vapor Concentration (Raoult's Law) ──\n\n");

    // C_vapor = x_i × P_sat × MW / (R × T)
    let (mw, p_sat_kpa, _name) = match contaminant.to_lowercase().as_str() {
        "benzene" => (78.11, 10.0, "Benzene"),
        "toluene" => (92.14, 3.8, "Toluene"),
        "xylene" | "o-xylene" => (106.17, 1.1, "Xylene"),
        "tce" | "trichloroethylene" => (131.39, 9.6, "TCE"),
        "pce" | "perchloroethylene" => (165.83, 2.5, "PCE"),
        "gasoline" | "btx" => (100.0, 7.0, "Gasoline (avg)"),
        _ => (100.0, 5.0, "Unknown (assumed)"),
    };

    let temp_k = soil_temp_c + 273.15;
    let r_gas = 8.314; // J/(mol·K)
    let x_i = 1.0; // assume pure NAPL
    let c_vapor_mg_m3 = x_i * p_sat_kpa * 1000.0 * mw / (r_gas * temp_k) * 1000.0; // mg/m³

    out.push_str(&format!("  Contaminant: {} (MW={:.0}, P_sat={:.1} kPa)\n", contaminant, mw, p_sat_kpa));
    out.push_str(&format!("  Vapor concentration: {:.0} mg/m³ ({:.1} g/m³)\n\n", c_vapor_mg_m3, c_vapor_mg_m3 / 1000.0));

    // ═══ Mass Removal Rate ═══
    out.push_str("── Mass Removal Rate ──\n\n");

    let removal_rate_g_hr = q_air_m3_hr * c_vapor_mg_m3 / 1000.0;
    let removal_rate_kg_day = removal_rate_g_hr * 24.0 / 1000.0;

    out.push_str(&format!("  Removal rate: {:.1} g/hr ({:.2} kg/day)\n", removal_rate_g_hr, removal_rate_kg_day));

    // ═══ Cleanup Time ═══
    let cleanup_time_days = napl_mass_kg / removal_rate_kg_day.max(1e-10);

    out.push_str(&format!("  NAPL mass: {:.0} kg\n", napl_mass_kg));
    out.push_str(&format!("  ► Estimated cleanup time: {:.0} days ({:.1} years)\n\n", cleanup_time_days, cleanup_time_days / 365.0));

    // ═══ Number of Wells ═══
    out.push_str("── Multi-Well Design ──\n\n");

    let area_per_well = std::f64::consts::PI * r_i_transient * r_i_transient;
    let total_area = std::f64::consts::PI * r_i_transient * r_i_transient * 2.0; // assume 2× radius area
    let n_wells = (total_area / area_per_well).ceil().max(1.0) as u32;

    out.push_str(&format!("  Area per well (R_i={:.0}m): {:.0} m²\n", r_i_transient, area_per_well));
    out.push_str(&format!("  ► Recommended wells: {} (for area ~{:.0} m²)\n\n", n_wells, total_area));

    // ═══ Assessment ═══
    out.push_str("═══ SVE DESIGN ASSESSMENT ═══\n\n");

    if k_air_m2 < 1e-14 {
        out.push_str("  ⚠️ Low air permeability (<1e-14 m²). SVE may not be effective.\n");
        out.push_str("     Consider: soil fracturing, thermal enhancement, or alternative remediation.\n");
    } else if k_air_m2 > 1e-12 {
        out.push_str("  🟢 High air permeability. SVE should be effective.\n");
    } else {
        out.push_str("  🟡 Moderate permeability. SVE viable with sufficient wells.\n");
    }

    if cleanup_time_days > 365.0 {
        out.push_str("  ⚠️ Cleanup >1 year. Consider: thermal enhancement, more wells, or bioventing.\n");
    } else {
        out.push_str("  🟢 Cleanup time reasonable (<1 year).\n");
    }

    if removal_rate_kg_day < 0.01 {
        out.push_str("  ⚠️ Low removal rate (<10 g/day). Check vapor concentration and airflow.\n");
    }

    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • Assumes homogeneous soil, steady-state radial flow\n");
    out.push_str("  • Raoult's Law assumes pure NAPL (real: multi-component mixtures)\n");
    out.push_str("  • No desorption-limited mass transfer modeled\n");
    out.push_str("  • For design: pilot test + numerical model (e.g., T2VOC, MODFLOW-SURFACT)\n");

    out
}
