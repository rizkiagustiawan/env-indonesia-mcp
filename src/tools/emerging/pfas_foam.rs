/// PFAS Foam Fractionation Design (2026 SOTA)
///
/// IMPLEMENTES: ITRC PFAS Guidance Section 18 (2025-2026)
/// + Hatton et al. 2025 "Colloidal Carbon in-situ" (J HazMat 140292)
///   (foam fractionation related AWI mechanisms)
/// + Burns et al. 2021 + We et al. 2024
///
/// KEY MECHANISM: Langmuir isotherm at Air-Water Interface (AWI)
///   PFAS are surfactants -> concentrate at AWI
///   Foam bubbles create enormous AWI area
///   Long-chain PFAS (>C8) adsorb more strongly (higher Kaw)
///   Short-chain PFAS (<C6) less amenable to foam
///
/// PERFORMANCE (2025-2026):
///   HRT: 25-60 min (ITRC)
///   CF (concentration factor): 10-1,000,000x
///   Long-chain removal: 85-99%
///   Short-chain removal: 40-70% (with co-surfactant)
///   Co-surfactant (CTAC): boosts short-chain removal
///
/// DOWNSTREAM:
///   Foamate (concentrated PFAS) -> SCWO or electrochemical destruction
///   Volume reduction: 95-99% -> economical for destruction

pub fn assess(
    pfas_type: &str, conc_ug_l: f64, volume_m3: f64,
    gas_flow_lpm: f64, column_height_m: f64, column_diameter_m: f64,
    hrt_min: f64, n_stages: u32, co_surfactant: bool,
) -> String {
    let mut out = String::from("=== PFAS Foam Fractionation (2026) ===\n");
    out.push_str("Ref: ITRC 18 (2025); Hatton 2025 (J HazMat); Burns 2021; We 2024\n\n");

    if conc_ug_l <= 0.0 || volume_m3 <= 0.0 {
        return "ERROR [E102]: conc and volume must be > 0.".into();
    }

    let is_long_chain = pfas_type.to_lowercase().contains("pfoa")
        || pfas_type.to_lowercase().contains("pfos")
        || pfas_type.to_lowercase().contains("c8");
    let is_short_chain = pfas_type.to_lowercase().contains("pfba")
        || pfas_type.to_lowercase().contains("pfbs")
        || pfas_type.to_lowercase().contains("pfhxa");

    // ═══ Phase 1: Langmuir AWI Mechanism ═══
    out.push_str("-- Phase 1: Langmuir AWI Mechanism --\n\n");
    out.push_str("PFAS are surfactants -> adsorb at Air-Water Interface (AWI)\n");
    out.push_str("  Langmuir: Gamma = (Kaw * C) / (1 + Kaw * C) * Gamma_max\n");
    out.push_str("  Foam bubbles create enormous AWI area\n");
    out.push_str("  Long-chain PFAS: higher Kaw -> faster removal\n");
    out.push_str("  Short-chain PFAS: lower Kaw -> needs co-surfactant\n\n");

    out.push_str(&format!("PFAS: {} ({})\n", pfas_type,
        if is_long_chain {"Long-chain C>=8 -- fast removal"}
        else if is_short_chain {"Short-chain C<8 -- slower removal"}
        else {"Medium-chain"}));
    out.push_str(&format!("Conc: {:.1} ug/L, Volume: {:.1} m3\n", conc_ug_l, volume_m3));
    out.push_str(&format!("Column: {:.1}m H x {:.2}m D\n", column_height_m, column_diameter_m));
    out.push_str(&format!("Gas: {:.0} L/min, HRT: {:.0} min, Stages: {}\n", gas_flow_lpm, hrt_min, n_stages));
    out.push_str(&format!("Co-surfactant: {}\n\n", if co_surfactant {"Yes (boosts short-chain)"} else {"No"}));

    // ═══ Phase 2: Performance Estimation ═══
    out.push_str("-- Phase 2: Performance (Empirical, ITRC 2025) --\n\n");

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

    out.push_str(&format!("  Expected removal: {:.0}%\n", removal_pct));
    out.push_str(&format!("  Concentration factor: {:.0}x\n", cf_estimate));
    out.push_str(&format!("  Foamate volume: {:.4} m3 ({:.1} L)\n", foamate_volume_m3, foamate_volume_m3 * 1000.0));
    out.push_str(&format!("  Foamate conc: {:.1} ug/L (concentrated {}x)\n\n", foamate_conc, cf_estimate as u32));

    // ═══ Phase 3: Effluent ═══
    out.push_str("-- Phase 3: Effluent Quality --\n\n");
    out.push_str(&format!("  >> Effluent conc: {:.2} ug/L ({:.0} ng/L)\n\n", effluent_conc, effluent_conc * 1000.0));

    // ═══ Phase 4: Downstream Treatment ═══
    out.push_str("-- Phase 4: Downstream Treatment --\n\n");
    out.push_str("Foamate (concentrated PFAS) requires DESTRUCTION:\n");
    out.push_str("  1. SCWO: T>374C, DRE>99.99% (volume reduction makes this economical)\n");
    out.push_str("  2. Electrochemical oxidation: BDD/Ti4O7 electrodes\n");
    out.push_str("  3. UV+sulfite: reductive defluorination\n");
    out.push_str("  4. Plasma treatment (APPJ/EDP)\n\n");
    out.push_str(&format!("Volume reduction: {:.0}% -> only {:.1} L needs destruction\n\n",
        (1.0 - foamate_pct) * 100.0, foamate_volume_m3 * 1000.0));

    // ═══ Phase 5: 2026 Treatment Alternatives ═══
    out.push_str("-- Phase 5: 2026 Treatment Alternatives --\n\n");
    out.push_str("Technology             Removal    CF/x     Energy    Ref\n");
    out.push_str("----------             -------    ----     ------    ---\n");
    out.push_str("Foam fractionation     85-99%     10-1M    0.01      ITRC 2025\n");
    out.push_str("Colloidal Carbon (CCP) >99.9%     N/A      N/A       Hatton 2025\n");
    out.push_str("GAC                    70-85%     N/A      0.10      Jafarinejad 2025\n");
    out.push_str("IEX                    80-95%     N/A      0.20      Chen 2025\n");
    out.push_str("Electro-NF             90.4%      N/A      1.92      Hua 2026\n");
    out.push_str("MOF (PCN-999)          >99%       N/A      0.05      Lee 2025\n");
    out.push_str("HIP+GAC pretreat       +350% BV   N/A      0.10      Zhang 2025\n\n");

    // ═══ Status Kepatuhan ═══
    out.push_str("-- STATUS KEPATUHAN --\n\n");
    let epa_mcl = 4.0;
    let eff_ng = effluent_conc * 1000.0;
    out.push_str(&format!("EPA MCL: 4 ng/L -> Effluent: {:.0} ng/L -> {}\n\n",
        eff_ng, if eff_ng <= epa_mcl {"PASS"} else {"FAIL -- add downstream treatment"}));

    // ═══ PEMANTAUAN ═══
    out.push_str("-- PEMANTAUAN --\n");
    out.push_str("  Parameter: PFAS influent/effluent/foamate, foam stability, pH\n");
    out.push_str("  Metode: EPA 1633 (LC-MS/MS)\n");
    out.push_str("  Frekuensi: Quarterly (operational), Monthly (startup)\n\n");

    // ═══ Limitations ═══
    out.push_str("-- Limitations (honest) --\n");
    out.push_str("  • Removal % is empirical (not from Langmuir kinetic model)\n");
    out.push_str("  • No foam drainage/coalescence dynamics\n");
    out.push_str("  • No temperature/salinity effects on foam stability\n");
    out.push_str("  • Short-chain PFAS poorly modeled (needs co-surfactant data)\n");
    out.push_str("  • Ref: ITRC 18 (2025); Hatton 2025 (J HazMat 140292)\n");

    out
}
