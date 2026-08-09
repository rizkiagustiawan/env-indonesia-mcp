/// PFAS Supercritical Water Oxidation (SCWO) Design
/// Ref: Aquarden Technologies Whitepaper v2 (Feb 2026); Krause et al. 2022; 374Water 2026
/// Critical point: T > 374°C, P > 22.1 MPa; DRE > 99.99%
pub fn assess(
    pfas_conc_ppb: f64, feed_flow_m3_day: f64, cod_g_l: f64,
    target_temp_c: f64, target_pressure_mpa: f64,
    residence_time_s: f64,
) -> String {
    let mut out = String::from("=== PFAS Supercritical Water Oxidation (SCWO) Design ===\n");
    out.push_str("Ref: Aquarden 2026; Krause 2022; 374Water 2026\n\n");
    let t_critical = 374.0;
    let p_critical = 22.1;
    let is_supercritical = target_temp_c > t_critical && target_pressure_mpa > p_critical;
    let cp_water = 4.18; // kJ/kg/°C
    let delta_t = target_temp_c - 25.0;
    let energy_heating_kj_kg = cp_water * delta_t;
    let energy_vap_kj_kg = 0.0; // above critical, no latent heat
    let total_energy_kj_kg = energy_heating_kj_kg + energy_vap_kj_kg;
    let energy_kwh_m3 = total_energy_kj_kg * 1000.0 / 3600.0;
    let total_energy_kwh = energy_kwh_m3 * feed_flow_m3_day;
    let autothermal = cod_g_l >= 120.0;
    let reactor_volume_m3 = feed_flow_m3_day * residence_time_s / 86400.0;
    let pfas_mass_g_day = pfas_conc_ppb * feed_flow_m3_day * 1000.0 / 1e6;
    let theoretical_f_mg = pfas_mass_g_day * 8.0 * 19.0 / 450.0 * 1000.0;
    let expected_dre = if residence_time_s >= 10.0 { 99.99 } else if residence_time_s >= 5.0 { 99.9 } else { 99.0 };
    let effluent_pfas_ppb = pfas_conc_ppb * (1.0 - expected_dre / 100.0);
    out.push_str(&format!("PFAS feed: {:.0} ppb ({:.0} ng/L), flow: {:.1} m3/day\n", pfas_conc_ppb, pfas_conc_ppb, feed_flow_m3_day));
    out.push_str(&format!("COD: {:.0} g/L, Temp: {:.0}°C, Pressure: {:.1} MPa\n", cod_g_l, target_temp_c, target_pressure_mpa));
    out.push_str(&format!("Residence time: {:.0} seconds\n\n", residence_time_s));
    out.push_str("-- Supercritical Status --\n\n");
    out.push_str(&format!("  Critical point: T > {:.0}°C, P > {:.1} MPa\n", t_critical, p_critical));
    out.push_str(&format!("  Operating: T={:.0}°C, P={:.1} MPa\n", target_temp_c, target_pressure_mpa));
    out.push_str(&format!("  >> Supercritical: {}\n\n", if is_supercritical {"✅ YES"} else {"❌ NO — must exceed critical point"}));
    out.push_str("-- Reactor Design --\n\n");
    out.push_str(&format!("  Reactor volume: {:.4} m3\n", reactor_volume_m3));
    out.push_str(&format!("  Energy for heating: {:.1} kWh/m3\n", energy_kwh_m3));
    out.push_str(&format!("  Total energy: {:.0} kWh/day\n", total_energy_kwh));
    out.push_str(&format!("  Autothermal (COD ≥ 120 g/L): {}\n\n", if autothermal {"✅ YES — no external heat needed"} else {"❌ NO — needs external heat"}));
    out.push_str("-- Destruction Performance --\n\n");
    out.push_str(&format!("  Expected DRE: >{:.4}%\n", expected_dre));
    out.push_str(&format!("  Effluent PFAS: {:.4} ppb ({:.2} ng/L)\n", effluent_pfas_ppb, effluent_pfas_ppb));
    out.push_str(&format!("  PFAS mass destroyed: {:.2} g/day\n", pfas_mass_g_day * (expected_dre / 100.0)));
    out.push_str(&format!("  Fluoride produced: {:.1} mg/day → neutralize to CaF2\n\n", theoretical_f_mg));
    out.push_str("-- Commercial Systems --\n");
    out.push_str("  Aquarden SuperOx (Denmark/ArianeGroup)\n");
    out.push_str("  Battelle PFAS Annihilator (US)\n");
    out.push_str("  374Water (US)\n\n");
    out.push_str("-- STATUS KEPATUHAN --\n");
    out.push_str(&format!("  EPA MCL: 4 ng/L → Effluent: {:.2} ng/L → {}\n\n", effluent_pfas_ppb, if effluent_pfas_ppb <= 4.0 {"✅"} else {"❌"}));
    out.push_str("-- PEMANTAUAN --\n");
    out.push_str("  Parameter: PFAS target list, TOF (Total Organic Fluorine), F-\n");
    out.push_str("  Metode: EPA 1633 (PFAS), ion chromatography (F-), TOF analyzer\n");
    out.push_str("  Note: Monitor TOF to detect hidden PFAS precursors\n");
    out.push_str("  Ref: Aquarden 2026; Krause 2022\n");
    out
}
