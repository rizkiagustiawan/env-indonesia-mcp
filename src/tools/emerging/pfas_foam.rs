/// PFAS Foam Fractionation Design
/// Ref: ITRC PFAS Guidance §18 (2025-2026); We et al. 2024; Burns et al. 2021
/// HRT 25-60 min; CF 10-1,000,000x; Long-chain PFAS removed faster
pub fn assess(
    pfas_type: &str, conc_ug_l: f64, volume_m3: f64,
    gas_flow_lpm: f64, column_height_m: f64, column_diameter_m: f64,
    hrt_min: f64, n_stages: u32, co_surfactant: bool,
) -> String {
    let mut out = String::from("=== PFAS Foam Fractionation Design ===\n");
    out.push_str("Ref: ITRC §18 (2025); We et al. 2024; Burns et al. 2021\n\n");
    let is_long_chain = pfas_type.to_lowercase().contains("pfoa") || pfas_type.to_lowercase().contains("pfos") || pfas_type.to_lowercase().contains("c8");
    let (base_removal_pct, cf_estimate) = match (is_long_chain, co_surfactant) {
        (true, false) => (85.0, 100.0),
        (true, true) => (95.0, 500.0),
        (false, false) => (40.0, 10.0),
        (false, true) => (70.0, 50.0),
    };
    let removal_pct = base_removal_pct * (0.8 + 0.1 * (n_stages as f64 - 1.0)).min(1.2);
    let foamate_pct = 1.0 / cf_estimate;
    let foamate_volume_m3 = volume_m3 * foamate_pct;
    let foamate_conc = conc_ug_l * cf_estimate;
    let effluent_conc = conc_ug_l * (1.0 - removal_pct / 100.0);
    let glr = gas_flow_lpm / (volume_m3 * 1000.0 / (hrt_min * 60.0 / 1e6)).max(1e-6);
    let bubble_surface_area = gas_flow_lpm / 60.0 * 6.0 / (0.002 * column_height_m * 3.14159 * (column_diameter_m / 2.0).powi(2)).max(1e-6);
    out.push_str(&format!("PFAS: {} (conc: {:.1} ug/L), Volume: {:.1} m3\n", pfas_type, conc_ug_l, volume_m3));
    out.push_str(&format!("Column: {:.1}m H x {:.2}m D\n", column_height_m, column_diameter_m));
    out.push_str(&format!("Gas flow: {:.0} L/min, HRT: {:.0} min, Stages: {}\n", gas_flow_lpm, hrt_min, n_stages));
    out.push_str(&format!("Co-surfactant: {}\n\n", if co_surfactant {"Yes"} else {"No"}));
    out.push_str("-- Performance (Empirical) --\n\n");
    out.push_str(&format!("  Chain type: {}\n", if is_long_chain {"Long (C≥8) — fast removal"} else {"Short (C<8) — slower removal"}));
    out.push_str(&format!("  Expected removal: {:.0}%\n", removal_pct));
    out.push_str(&format!("  Concentration factor: {:.0}x\n", cf_estimate));
    out.push_str(&format!("  GLR (gas-to-liquid ratio): {:.2}\n", glr));
    out.push_str(&format!("  Foamate volume: {:.4} m3 ({:.1} L)\n", foamate_volume_m3, foamate_volume_m3 * 1000.0));
    out.push_str(&format!("  Foamate conc: {:.1} ug/L\n\n", foamate_conc));
    out.push_str("-- Effluent --\n\n");
    out.push_str(&format!("  >> Effluent conc: {:.2} ug/L ({:.0} ng/L)\n\n", effluent_conc, effluent_conc * 1000.0));
    out.push_str("-- Downstream Treatment --\n");
    out.push_str("  Foamate requires destruction: SCWO or electrochemical oxidation\n");
    out.push_str("  Volume reduction: 95-99% → economical for destruction\n\n");
    out.push_str("-- STATUS KEPATUHAN --\n");
    let epa_mcl = 4.0; // ng/L for PFOA/PFOS
    out.push_str(&format!("  EPA MCL: 4 ng/L → Effluent: {:.0} ng/L → {}\n\n", effluent_conc * 1000.0, if effluent_conc * 1000.0 <= epa_mcl {"✅"} else {"❌ — add downstream treatment"}));
    out.push_str("-- PEMANTAUAN --\n");
    out.push_str("  Parameter: PFAS influent/effluent/foamate, foam stability, pH\n");
    out.push_str("  Metode: EPA 1633 (LC-MS/MS)\n");
    out.push_str("  Ref: ITRC §18; We et al. 2024; Burns 2021\n");
    out
}
