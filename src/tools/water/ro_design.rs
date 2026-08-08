/// Reverse Osmosis (RO) Membrane Design
/// Ref: Crittenden et al. 2012 (MWH "Water Treatment")
///   Biesheuvel et al. 2023; Kim et al. 2022

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

    // Convert salinity to mol/L (assume NaCl, MW=58.44)
    let mw_nacl = 58.44;
    let c_feed_mol = feed_salinity_mg_l / 1000.0 / mw_nacl; // mol/L = mol/dm³ = mol/m³ × 1000
    let c_feed_mol_m3 = c_feed_mol * 1000.0; // mol/m³

    out.push_str(&format!("Feed salinity: {:.0} mg/L ({:.4} mol/m³ NaCl)\n", feed_salinity_mg_l, c_feed_mol_m3));
    out.push_str(&format!("Target permeate: {:.0} mg/L\n", target_permeate_mg_l));
    out.push_str(&format!("Feed pressure: {:.1} bar ({:.0} kPa)\n", feed_pressure_bar, feed_pressure_bar * 100.0));
    out.push_str(&format!("Membrane: A={:.2} LMH/bar, B={:.2} LMH\n", A, B));
    out.push_str(&format!("Feed flow: {:.1} m³/day\n\n", feed_flow_m3_day));

    // ═══ Osmotic Pressure (van't Hoff) ═══
    out.push_str("── Osmotic Pressure ──\n\n");

    // π = i × C × R × T (for NaCl: i=2)
    let i = 2.0; // van't Hoff factor
    let pi_feed_pa = i * c_feed_mol_m3 * r_gas * temp_k;
    let pi_feed_bar = pi_feed_pa / 1e5;

    out.push_str(&format!("  van't Hoff: π = i×C×R×T = {} × {:.4} × {} × {:.1}\n", i, c_feed_mol_m3, r_gas, temp_k));
    out.push_str(&format!("  ► Feed osmotic pressure: {:.2} bar ({:.0} kPa)\n\n", pi_feed_bar, pi_feed_pa / 1000.0));

    // ═══ Net Driving Pressure ═══
    let delta_P = feed_pressure_bar; // applied pressure
    let net_pressure = delta_P - pi_feed_bar;

    out.push_str("── Net Driving Pressure ──\n\n");
    out.push_str(&format!("  ΔP = {:.1} bar, Δπ = {:.2} bar\n", delta_P, pi_feed_bar));
    out.push_str(&format!("  ► Net pressure (ΔP - Δπ): {:.2} bar\n\n", net_pressure));

    if net_pressure <= 0.0 {
        return format!("{}ERROR: Net pressure ≤ 0. Feed pressure must exceed osmotic pressure.\nIncrease feed pressure to >{:.1} bar.\n", out, pi_feed_bar);
    }

    // ═══ Water Flux ═══
    out.push_str("── Water Flux ──\n\n");

    // J_w = A × (ΔP - Δπ) (LMH)
    let j_water = A * net_pressure;
    let j_water_m_s = j_water / 3600.0 / 1000.0; // LMH → m/s

    out.push_str(&format!("  J_w = A × (ΔP - Δπ) = {:.2} × {:.2}\n", A, net_pressure));
    out.push_str(&format!("  ► Water flux: {:.2} LMH (L/m²/hr) ({:.2e} m/s)\n\n", j_water, j_water_m_s));

    // ═══ Salt Flux ═══
    // J_s = B × (C_feed - C_permeate)
    // Assume initial C_permeate ≈ 0 for first iteration
    let c_feed_g_m3 = feed_salinity_mg_l; // mg/L = g/m³
    let j_salt = B * (c_feed_g_m3 as f64); // LMH × g/m³ → need proper units
    // Actually: J_s = B × ΔC where B is in LMH and ΔC in g/L
    let delta_c_g_l = feed_salinity_mg_l / 1000.0; // g/L
    let salt_flux_g_m2_h = B * delta_c_g_l;

    out.push_str("── Salt Flux ──\n\n");
    out.push_str(&format!("  ΔC = {:.3} g/L\n", delta_c_g_l));
    out.push_str(&format!("  ► Salt flux: {:.2} g/m²/hr\n\n", salt_flux_g_m2_h));

    // ═══ Permeate Concentration ═══
    // C_permeate = J_s / J_w (g/L)
    let c_permeate_g_l = salt_flux_g_m2_h / j_water.max(1e-10);
    let c_permeate_mg_l = c_permeate_g_l * 1000.0;

    out.push_str("── Permeate Quality ──\n\n");
    out.push_str(&format!("  ► Permeate concentration: {:.1} mg/L (target: {:.0})\n\n", c_permeate_mg_l, target_permeate_mg_l));

    // ═══ Salt Rejection ═══
    let salt_rejection = (1.0 - c_permeate_mg_l / feed_salinity_mg_l) * 100.0;
    out.push_str(&format!("  ► Salt rejection: {:.1}%\n\n", salt_rejection));

    // ═══ Recovery Rate ═══
    // For single element: typical recovery 10-15%
    // For system: depends on staging
    let recovery_single = 0.12; // 12% per element
    let permeate_flow_m3_day = feed_flow_m3_day * recovery_single;
    let brine_flow_m3_day = feed_flow_m3_day - permeate_flow_m3_day;

    out.push_str("── Recovery & Flow ──\n\n");
    out.push_str(&format!("  Single-element recovery: {:.0}%\n", recovery_single * 100.0));
    out.push_str(&format!("  ► Permeate flow: {:.2} m³/day\n", permeate_flow_m3_day));
    out.push_str(&format!("  ► Brine flow: {:.2} m³/day\n\n", brine_flow_m3_day));

    // Brine concentration (mass balance)
    let c_brine_mg_l = (feed_flow_m3_day * feed_salinity_mg_l - permeate_flow_m3_day * c_permeate_mg_l) / brine_flow_m3_day.max(0.001);
    out.push_str(&format!("  ► Brine concentration: {:.0} mg/L\n\n", c_brine_mg_l));

    // ═══ Membrane Area ═══
    let membrane_area_m2 = (permeate_flow_m3_day * 1000.0) / (j_water * 24.0).max(1e-10); // L/day → m²

    out.push_str("── Membrane Area ──\n\n");
    out.push_str(&format!("  ► Required membrane area: {:.1} m²\n\n", membrane_area_m2));

    // Number of 8-inch elements (each ~37 m²)
    let element_area = 37.0; // m² per 8" element
    let n_elements = (membrane_area_m2 / element_area).ceil() as u32;
    let n_pressure_vessels = (n_elements as f64 / 6.0).ceil() as u32; // 6 elements per vessel

    out.push_str(&format!("  Elements (8\", 37m² each): {}\n", n_elements));
    out.push_str(&format!("  Pressure vessels (6 elem/vessel): {}\n\n", n_pressure_vessels));

    // ═══ Energy Consumption ═══
    // E = ΔP × Q_feed / (R × η_pump)
    let pump_efficiency = 0.85;
    let energy_kwh_m3 = feed_pressure_bar * 100.0 * feed_flow_m3_day / (recovery_single * feed_flow_m3_day * pump_efficiency) / 3.6e6 * feed_flow_m3_day;
    // Simplified: E = P / (Q_perm × η)
    let energy_kwh_m3_corrected = feed_pressure_bar * 100.0 / (recovery_single * pump_efficiency) / 36.0; // bar→kPa, kPa×m³/s→kW, /3.6→kWh/m³
    let daily_energy = energy_kwh_m3_corrected * permeate_flow_m3_day;

    out.push_str("── Energy Consumption ──\n\n");
    out.push_str(&format!("  Pump efficiency: {:.0}%\n", pump_efficiency * 100.0));
    out.push_str(&format!("  ► Specific energy: {:.2} kWh/m³ permeate\n", energy_kwh_m3_corrected));
    out.push_str(&format!("  ► Daily energy: {:.1} kWh/day\n\n", daily_energy));

    // ═══ Summary ═══
    out.push_str("═══ RO DESIGN SUMMARY ═══\n\n");
    out.push_str(&format!("  Net pressure: {:.2} bar\n", net_pressure));
    out.push_str(&format!("  Water flux: {:.2} LMH\n", j_water));
    out.push_str(&format!("  Permeate: {:.1} mg/L (rejection {:.1}%)\n", c_permeate_mg_l, salt_rejection));
    out.push_str(&format!("  Recovery: {:.0}%\n", recovery_single * 100.0));
    out.push_str(&format!("  Membrane area: {:.1} m² ({} elements)\n", membrane_area_m2, n_elements));
    out.push_str(&format!("  Energy: {:.2} kWh/m³\n", energy_kwh_m3_corrected));

    if c_permeate_mg_l <= target_permeate_mg_l {
        out.push_str("\n  ✅ Design meets permeate quality target.\n");
    } else {
        out.push_str("\n  ⚠️ Permeate exceeds target. Consider 2-pass RO or higher pressure.\n");
    }

    out.push_str("\n  Ref: Crittenden et al. 2012 (MWH); Biesheuvel et al. 2023\n");
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • No concentration polarization (CP) effect modeled\n");
    out.push_str("  • No membrane fouling/recovery decline\n");
    out.push_str("  • van't Hoff assumes ideal (real: non-ideal for high salinity)\n");
    out.push_str("  • For design: use ROSA/DOW WLM software\n");

    out
}
