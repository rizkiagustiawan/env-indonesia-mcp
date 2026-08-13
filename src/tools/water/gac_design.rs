/// Activated Carbon (GAC) Adsorption Design
/// Ref: Crittenden et al. 2012 (MWH "Water Treatment")
///   Freundlich isotherm; Bohart-Adams bed life

pub fn design(
    contaminant: &str,
    c_influent_mg_l: f64,
    c_target_mg_l: f64,
    flow_m3_day: f64,
    freundlich_k: f64,      // K (mg/g)(L/mg)^(1/n)
    freundlich_1_over_n: f64, // 1/n (dimensionless)
    ebct_min: f64,          // empty bed contact time (minutes)
) -> String {
    let mut out = String::from("=== Activated Carbon (GAC) Design ===\n");
    out.push_str("Ref: Crittenden et al. 2012 (MWH); Freundlich isotherm\n\n");

    if c_influent_mg_l <= 0.0 || flow_m3_day <= 0.0 || freundlich_k <= 0.0 {
        return "ERROR [E102]: parameters must be > 0.".into();
    }

    out.push_str(&format!("Contaminant: {}\n", contaminant));
    out.push_str(&format!("  C_influent: {:.2} mg/L → C_target: {:.4} mg/L\n", c_influent_mg_l, c_target_mg_l));
    out.push_str(&format!("  Flow: {:.1} m³/day ({:.0} L/hr)\n", flow_m3_day, flow_m3_day * 1000.0 / 24.0));
    out.push_str(&format!("  Freundlich: K={:.2}, 1/n={:.2}\n", freundlich_k, freundlich_1_over_n));
    out.push_str(&format!("  EBCT: {:.1} min\n\n", ebct_min));

    // ═══ Adsorption Capacity (Freundlich) ═══
    out.push_str("── Adsorption Capacity (Freundlich Isotherm) ──\n\n");

    // q = K × C^(1/n) where C = equilibrium concentration
    // Use target concentration as equilibrium (conservative)
    let c_eq = c_target_mg_l.max(0.001);
    let q_capacity = freundlich_k * c_eq.powf(freundlich_1_over_n);
    // Also at influent concentration
    let q_at_influent = freundlich_k * c_influent_mg_l.powf(freundlich_1_over_n);

    out.push_str(&format!("  q = K × C^(1/n)\n"));
    out.push_str(&format!("  At C_target: q = {:.2} × {:.4}^{:.2} = {:.2} mg/g\n", freundlich_k, c_eq, freundlich_1_over_n, q_capacity));
    out.push_str(&format!("  At C_influent: q = {:.2} mg/g (capacity at influent)\n\n", q_at_influent));
    out.push_str(&format!("  ► Design capacity (at target): {:.2} mg/g ({:.2} g/kg)\n\n", q_capacity, q_capacity));

    // ═══ Bed Volume ═══
    out.push_str("── Bed Volume & Dimensions ──\n\n");

    // EBCT = V_bed / Q
    let flow_l_min = flow_m3_day * 1000.0 / (24.0 * 60.0); // L/min
    let bed_volume_l = ebct_min * flow_l_min;
    let bed_volume_m3 = bed_volume_l / 1000.0;

    // Typical GAC bulk density: 450-550 kg/m³
    let bulk_density = 500.0; // kg/m³
    let carbon_mass_kg = bed_volume_m3 * bulk_density;

    out.push_str(&format!("  Flow: {:.1} L/min ({:.2} m³/day)\n", flow_l_min, flow_m3_day));
    out.push_str(&format!("  ► Bed volume: {:.2} m³ ({:.0} L)\n", bed_volume_m3, bed_volume_l));
    out.push_str(&format!("  ► Carbon mass: {:.0} kg ({:.1} tons)\n\n", carbon_mass_kg, carbon_mass_kg / 1000.0));

    // ═══ Bed Life (Bohart-Adams) ═══
    out.push_str("── Bed Life (Bohart-Adams) ──\n\n");

    // Simplified: throughput = q × ρ / C₀ (bed volumes to breakthrough)
    let bv_to_breakthrough = q_capacity * bulk_density / c_influent_mg_l; // BV
    let bed_life_days = bv_to_breakthrough * bed_volume_m3 / flow_m3_day;
    let bed_life_months = bed_life_days / 30.0;

    out.push_str(&format!("  Throughput to breakthrough: {:.0} bed volumes (BV)\n", bv_to_breakthrough));
    out.push_str(&format!("  ► Bed life: {:.0} days ({:.1} months)\n\n", bed_life_days, bed_life_months));

    // ═══ Carbon Usage Rate (CUR) ═══
    out.push_str("── Carbon Usage Rate (CUR) ──\n\n");

    // Q[m³/day]×ΔC[mg/L] = Q×ΔC×1000 [mg/day] ÷ q[mg/g] ÷ 1000 = Q×ΔC/q [kg/day]
    let cur_kg_day = flow_m3_day * (c_influent_mg_l - c_target_mg_l) / q_capacity.max(1e-10);
    let annual_carbon_kg = cur_kg_day * 365.0;

    out.push_str(&format!("  ► CUR: {:.2} kg/day ({:.0} kg/year)\n\n", cur_kg_day, annual_carbon_kg));

    // ═══ Hydraulic Loading Rate ═══
    // Assume bed cross-sectional area (0.5m diameter × 2m height)
    let bed_diameter = ((bed_volume_m3 / 3.0) * 4.0 / std::f64::consts::PI).sqrt().max(0.5); // assume L/D = 3
    let bed_height = bed_volume_m3 / (std::f64::consts::PI * bed_diameter * bed_diameter / 4.0).max(0.01);
    let bed_area_m2 = std::f64::consts::PI * bed_diameter * bed_diameter / 4.0;
    let hlr_m_hr = flow_m3_day / 24.0 / bed_area_m2.max(0.01);

    out.push_str("── Bed Geometry ──\n\n");
    out.push_str(&format!("  Bed diameter: {:.2} m\n", bed_diameter));
    out.push_str(&format!("  Bed height: {:.2} m (L/D = {:.1})\n", bed_height, bed_height / bed_diameter));
    out.push_str(&format!("  Hydraulic loading rate: {:.1} m/hr\n\n", hlr_m_hr));

    if hlr_m_hr > 15.0 {
        out.push_str("  ⚠️ HLR >15 m/hr — high head loss. Increase diameter.\n\n");
    } else {
        out.push_str("  🟢 HLR in range (5-15 m/hr typical)\n\n");
    }

    // ═══ Number of Vessels ═══
    let n_vessels = 2; // minimum 2 for continuous operation (1 operating + 1 standby)
    out.push_str(&format!("  ► Vessels: {} (1 operating + 1 standby)\n\n", n_vessels));

    // ═══ Summary ═══
    out.push_str("═══ GAC DESIGN SUMMARY ═══\n\n");
    out.push_str(&format!("  Adsorption capacity: {:.2} mg/g (at target)\n", q_capacity));
    out.push_str(&format!("  Bed volume: {:.2} m³, Carbon: {:.0} kg\n", bed_volume_m3, carbon_mass_kg));
    out.push_str(&format!("  Bed life: {:.1} months ({:.0} BV)\n", bed_life_months, bv_to_breakthrough));
    out.push_str(&format!("  CUR: {:.2} kg/day ({:.0} kg/year)\n", cur_kg_day, annual_carbon_kg));
    out.push_str(&format!("  Vessels: {} (Ø{:.2}m × {:.2}m)\n", n_vessels, bed_diameter, bed_height));

    out.push_str("\n  Ref: Crittenden et al. 2012 (MWH); Freundlich 1906\n");
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • Freundlich assumes single-solute (real: multi-component competition)\n");
    out.push_str("  • No mass transfer zone (MTZ) modeled — breakthrough is sharp\n");
    out.push_str("  • No thermal regeneration efficiency (typically 80-90%)\n");
    out.push_str("  • For design: RSSCT (rapid small-scale column test) recommended\n");

    out
}

#[cfg(test)]
mod tests {
    use super::design;

    #[test]
    fn carbon_usage_rate_units() {
        // Q=1000 m3/day, dC=100 mg/L, q=20 mg/g -> CUR = 1000*100/20 = 5000 kg/day
        let result = design("tce", 110.0, 10.0, 1000.0, 20.0, 0.0, 10.0);
        assert!(result.contains("CUR: 5000.00"), "CUR should be 5000 kg/day, got:\n{result}");
    }
}
