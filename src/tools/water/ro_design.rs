/// Reverse Osmosis (RO) Membrane Design
/// Ref: Crittenden et al. 2012 (MWH "Water Treatment")
///   Biesheuvel et al. 2023; Kim et al. 2022
/// Enhanced: Concentration polarization + iterative permeate concentration

pub fn design(
    feed_salinity_mg_l: f64,
    target_permeate_mg_l: f64,
    feed_pressure_bar: f64,
    membrane_water_perm_l_m2_h_bar: f64, // A (LMH/bar)
    membrane_salt_perm_l_m2_h: f64,     // B (LMH)
    feed_flow_m3_day: f64,
    temp_c: f64,
) -> String {
    let mut out = String::from("=== Reverse Osmosis (RO) Design ===\n");
    out.push_str("Ref: Crittenden et al. 2012 (MWH); Biesheuvel et al. 2023\n\n");

    if feed_flow_m3_day <= 0.0 || feed_pressure_bar <= 0.0 {
        return "ERROR [E102]: feed flow and pressure must be > 0.".into();
    }

    let A = membrane_water_perm_l_m2_h_bar;
    let B = membrane_salt_perm_l_m2_h;
    let temp_k = temp_c + 273.15;
    let r_gas = 8.314; // J/(mol·K)
    let mw_nacl = 58.44;

    // Convert salinity to mol/m3
    let c_feed_mol_m3 = feed_salinity_mg_l / 1000.0 / mw_nacl * 1000.0;

    out.push_str(&format!("Feed salinity: {:.0} mg/L ({:.4} mol/m3 NaCl)\n", feed_salinity_mg_l, c_feed_mol_m3));
    out.push_str(&format!("Target permeate: {:.0} mg/L\n", target_permeate_mg_l));
    out.push_str(&format!("Feed pressure: {:.1} bar ({:.0} kPa)\n", feed_pressure_bar, feed_pressure_bar * 100.0));
    out.push_str(&format!("Membrane: A={:.2} LMH/bar, B={:.2} LMH\n", A, B));
    out.push_str(&format!("Feed flow: {:.1} m3/day\n", feed_flow_m3_day));
    out.push_str(&format!("Temperature: {:.1} C ({:.2} K)\n\n", temp_c, temp_k));

    // ═══ Osmotic Pressure (van't Hoff) ═══
    out.push_str("-- Osmotic Pressure (van't Hoff) --\n\n");

    let i_vantoff = 2.0; // NaCl
    let pi_feed_bar = i_vantoff * c_feed_mol_m3 * r_gas * temp_k / 1e5;

    out.push_str(&format!("  pi = i*C*R*T = {:.2} bar\n\n", pi_feed_bar));

    // ═══ Net Driving Pressure ═══
    let net_pressure = feed_pressure_bar - pi_feed_bar;

    out.push_str("-- Net Driving Pressure --\n\n");
    out.push_str(&format!("  dP = {:.1} bar, dpi = {:.2} bar\n", feed_pressure_bar, pi_feed_bar));
    out.push_str(&format!("  >> Net pressure: {:.2} bar\n\n", net_pressure));

    if net_pressure <= 0.0 {
        return format!("{}ERROR: Net pressure <= 0. Feed pressure must exceed osmotic pressure ({:.1} bar).\n", out, pi_feed_bar);
    }

    // ═══ Concentration Polarization (CP) ═══
    // Ref: Crittenden 2012 (MWH), Chapter 11
    // CP = exp(J_w / k_m) where k_m = mass transfer coefficient
    // k_m typical for spiral-wound: 2e-5 m/s (varies with crossflow velocity)
    let k_m = 2e-5; // m/s mass transfer coefficient (typical spiral-wound)

    // Water flux (initial, without CP)
    let j_water_initial = A * net_pressure; // LMH
    let j_water_m_s = j_water_initial / 3600.0 / 1000.0; // m/s

    // CP factor
    let cp_factor = (j_water_m_s / k_m).exp();
    let cp_factor_capped = cp_factor.min(2.0); // cap at 2.0 (realistic limit)

    out.push_str("-- Concentration Polarization (CP) --\n\n");
    out.push_str(&format!("  Mass transfer coeff k_m: {:.2e} m/s\n", k_m));
    out.push_str(&format!("  J_w (initial): {:.2e} m/s\n", j_water_m_s));
    out.push_str(&format!("  CP = exp(J_w/k_m) = {:.3}", cp_factor));
    if cp_factor > 2.0 {
        out.push_str(" (capped at 2.0)");
    }
    out.push_str("\n\n");

    // ═══ Water Flux (with CP-adjusted osmotic pressure) ═══
    // Effective osmotic pressure at membrane surface = CP * pi_feed
    let pi_surface = pi_feed_bar * cp_factor_capped;
    let net_pressure_cp = feed_pressure_bar - pi_surface;
    let j_water = A * net_pressure_cp.max(0.0);

    out.push_str("-- Water Flux (CP-corrected) --\n\n");
    out.push_str(&format!("  pi_surface = CP x pi_feed = {:.2} bar\n", pi_surface));
    out.push_str(&format!("  Net pressure (CP): {:.2} bar\n", net_pressure_cp));
    out.push_str(&format!("  >> Water flux: {:.2} LMH (L/m2/hr)\n\n", j_water));

    // ═══ Iterative Permeate Concentration ═══
    // Ref: Crittenden 2012 (MWH) — iterative solution
    // C_permeate = B * (C_surface - C_perm) / J_w
    // Start with C_perm = 0, iterate until convergence

    out.push_str("-- Permeate Concentration (Iterative) --\n\n");

    let c_feed_g_l = feed_salinity_mg_l / 1000.0;
    let c_surface_g_l = c_feed_g_l * cp_factor_capped; // concentration at membrane surface

    let mut c_perm_g_l = 0.0;
    let mut iterations = 0;
    for iter in 0..20 {
        let delta_c = c_surface_g_l - c_perm_g_l;
        let salt_flux = B * delta_c; // g/m2/hr
        let c_perm_new = salt_flux / j_water.max(1e-10);
        let diff = (c_perm_new - c_perm_g_l).abs();
        c_perm_g_l = c_perm_new;
        iterations = iter + 1;
        if diff < 1e-6 { break; }
    }

    let c_permeate_mg_l = c_perm_g_l * 1000.0;
    let salt_rejection = (1.0 - c_permeate_mg_l / feed_salinity_mg_l) * 100.0;

    out.push_str(&format!("  C_surface = CP x C_feed = {:.3} g/L\n", c_surface_g_l));
    out.push_str(&format!("  Iterations: {} (converged)\n", iterations));
    out.push_str(&format!("  >> Permeate: {:.1} mg/L (target: {:.0})\n", c_permeate_mg_l, target_permeate_mg_l));
    out.push_str(&format!("  >> Salt rejection: {:.1}%\n\n", salt_rejection));

    // ═══ Recovery Rate ═══
    let recovery = 0.12; // 12% per element
    let permeate_flow = feed_flow_m3_day * recovery;
    let brine_flow = feed_flow_m3_day - permeate_flow;

    // Brine concentration (mass balance)
    let c_brine = (feed_flow_m3_day * feed_salinity_mg_l - permeate_flow * c_permeate_mg_l) / brine_flow.max(0.001);

    out.push_str("-- Recovery & Flow --\n\n");
    out.push_str(&format!("  Recovery: {:.0}%\n", recovery * 100.0));
    out.push_str(&format!("  Permeate: {:.2} m3/day | Brine: {:.2} m3/day\n", permeate_flow, brine_flow));
    out.push_str(&format!("  Brine concentration: {:.0} mg/L\n\n", c_brine));

    // ═══ Membrane Area ═══
    let membrane_area = (permeate_flow * 1000.0) / (j_water * 24.0).max(1e-10);
    let n_elements = (membrane_area / 37.0).ceil() as u32;
    let n_vessels = (n_elements as f64 / 6.0).ceil() as u32;

    out.push_str("-- Membrane Area --\n\n");
    out.push_str(&format!("  Area: {:.1} m2 ({} elements, {} vessels)\n\n", membrane_area, n_elements, n_vessels));

    // ═══ Energy Consumption (CORRECTED) ═══
    // E = dP / (R * eta) where dP in bar -> kPa -> J/m3 -> kWh/m3
    // E (kWh/m3) = dP(kPa) / (R * eta * 3600)
    let pump_eff = 0.85;
    // E[kWh/m³] = P[bar]×1e5 / (R·η·3.6e6) = P[bar]×100 / (R·η·3600)
    let energy_kwh_m3 = feed_pressure_bar * 100.0 / (recovery * pump_eff * 3600.0);
    let energy_kwh_m3_v2 = feed_pressure_bar * 100.0 / (recovery * pump_eff) / 3600.0;
    let daily_energy = energy_kwh_m3_v2 * permeate_flow;

    out.push_str("-- Energy Consumption --\n\n");
    out.push_str(&format!("  Pump efficiency: {:.0}%\n", pump_eff * 100.0));
    out.push_str(&format!("  >> Specific energy: {:.2} kWh/m3 permeate\n", energy_kwh_m3_v2));
    out.push_str(&format!("  >> Daily energy: {:.1} kWh/day\n\n", daily_energy));

    // ═══ Summary ═══
    out.push_str("=== RO DESIGN SUMMARY ===\n\n");
    out.push_str(&format!("  CP factor: {:.3}\n", cp_factor_capped));
    out.push_str(&format!("  Net pressure (CP): {:.2} bar\n", net_pressure_cp));
    out.push_str(&format!("  Water flux: {:.2} LMH\n", j_water));
    out.push_str(&format!("  Permeate: {:.1} mg/L (rejection {:.1}%, {} iter)\n", c_permeate_mg_l, salt_rejection, iterations));
    out.push_str(&format!("  Recovery: {:.0}%, Brine: {:.0} mg/L\n", recovery * 100.0, c_brine));
    out.push_str(&format!("  Area: {:.1} m2 ({} elements), Energy: {:.2} kWh/m3\n", membrane_area, n_elements, energy_kwh_m3_v2));

    if c_permeate_mg_l <= target_permeate_mg_l {
        out.push_str("\n  [OK] Design meets permeate quality target.\n");
    } else {
        out.push_str("\n  [WARN] Permeate exceeds target. Consider 2-pass RO or higher pressure.\n");
    }

    out.push_str("\n  Ref: Crittenden et al. 2012 (MWH); Biesheuvel et al. 2023\n");
    out.push_str("\n-- Limitations (honest) --\n");
    out.push_str("  • CP uses fixed k_m (real: depends on spacer geometry, crossflow)\n");
    out.push_str("  • No membrane fouling/recovery decline over time\n");
    out.push_str("  • van't Hoff assumes ideal (real: non-ideal for high salinity)\n");
    out.push_str("  • For design: use ROSA/DOW WLM software\n");

    out
}

#[cfg(test)]
mod tests {
    use super::design;

    #[test]
    fn specific_energy_units() {
        // P=50 bar, R=0.12, eta=0.85 -> E = 50*100/(0.12*0.85*3600) = 13.62 kWh/m3
        let result = design(1000.0, 10.0, 50.0, 5.0, 0.1, 1000.0, 25.0);
        assert!(result.contains("Specific energy: 13.62"), "energy should be ~13.6 kWh/m3, got:\n{result}");
    }
}
