/// PFAS Supercritical Water Oxidation (SCWO) Design (2026 SOTA)
///
/// IMPLEMENTES: Prasetya et al. 2025 "Thermal treatment for PFAS" (J HazMat 138969)
/// + Aquarden 2026 + 374Water 2026 + Battelle PFAS Annihilator
///
/// KEY 2025-2026 ADVANCES:
///   - Bond cleavage order: C-S > C-C > C-F (Prasetya 2025)
///   - SCWO: T>374C, P>22.1MPa, DRE>99.99%
///   - Autothermal at COD>=120 g/L
///   - MAT (microwave-assisted thermal): rapid PFAS degradation
///   - EDP (electrical discharge plasma): in-situ radical generation
///   - APPJ (atmospheric pressure plasma jet): portable treatment
///
/// RADICAL MECHANISMS (Prasetya 2025):
///   *OH, *O, *H radicals attack PFAS
///   C-S bonds cleave first (thiols, sulfonates)
///   C-C bonds cleave next (backbone)
///   C-F bonds cleave last (strongest, 485 kJ/mol)
///   Defluorination rate = measure of mineralization
///
/// COMMERCIAL SYSTEMS (2026):
///   Aquarden SuperOx (Denmark) — full-scale operational
///   Battelle PFAS Annihilator (US) — mobile units
///   374Water (US) — sludge + AFFF concentrate
///   Cole-Parmer (2026) — bench-scale SCWO

pub fn assess(
    pfas_conc_ppb: f64, feed_flow_m3_day: f64, cod_g_l: f64,
    target_temp_c: f64, target_pressure_mpa: f64,
    residence_time_s: f64,
) -> String {
    let mut out = String::from("=== PFAS SCWO Design (2026 SOTA) ===\n");
    out.push_str("Ref: Prasetya 2025 (J HazMat 138969); Aquarden 2026; 374Water 2026\n\n");

    if pfas_conc_ppb < 0.0 || feed_flow_m3_day <= 0.0 {
        return "ERROR [E102]: pfas_conc >= 0, flow > 0.".into();
    }

    let t_critical = 374.0;
    let p_critical = 22.1;
    let is_supercritical = target_temp_c > t_critical && target_pressure_mpa > p_critical;

    // ═══ Phase 1: Supercritical State Check ═══
    out.push_str("-- Phase 1: Supercritical State --\n\n");
    out.push_str(&format!("Critical point: T > {}C, P > {} MPa\n", t_critical as u32, p_critical));
    out.push_str(&format!("Operating: T={}C, P={:.1} MPa\n", target_temp_c as u32, target_pressure_mpa));
    out.push_str(&format!(">> Supercritical: {}\n\n",
        if is_supercritical {"YES"} else {"NO -- must exceed critical point"}));

    if !is_supercritical {
        out.push_str("WARNING: Below critical point. Subcritical oxidation possible\n");
        out.push_str("but DRE will be lower (90-99% vs >99.99%)\n\n");
    }

    // ═══ Phase 2: Bond Cleavage Mechanism (Prasetya 2025) ═══
    out.push_str("-- Phase 2: Bond Cleavage Mechanism (Prasetya 2025) --\n\n");
    out.push_str("Thermal degradation order:\n");
    out.push_str("  1. C-S bonds (thiols, sulfonates) -- cleave FIRST\n");
    out.push_str("  2. C-C bonds (backbone) -- cleave next\n");
    out.push_str("  3. C-F bonds (485 kJ/mol) -- cleave LAST (hardest)\n\n");
    out.push_str("Radical attack: *OH > *O > *H\n");
    out.push_str("  *OH: hydroxyl radical (strongest oxidant)\n");
    out.push_str("  *O: atomic oxygen\n");
    out.push_str("  *H: hydrogen radical (reductive pathway)\n\n");
    out.push_str("Defluorination rate = true mineralization indicator\n");
    out.push_str("Incomplete byproducts: shorter-chain PFCAs (still PFAS!)\n\n");

    // ═══ Phase 3: Reactor Design ═══
    out.push_str("-- Phase 3: Reactor Design --\n\n");

    let cp_water = 4.18;
    let delta_t = target_temp_c - 25.0;
    let energy_heating_kj_kg = cp_water * delta_t;
    let energy_kwh_m3 = energy_heating_kj_kg * 1000.0 / 3600.0;
    let total_energy_kwh = energy_kwh_m3 * feed_flow_m3_day;
    let autothermal = cod_g_l >= 120.0;
    let reactor_volume_m3 = feed_flow_m3_day * residence_time_s / 86400.0;
    let pfas_mass_g_day = pfas_conc_ppb * feed_flow_m3_day * 1000.0 / 1e6;
    let theoretical_f_mg = pfas_mass_g_day * 8.0 * 19.0 / 450.0 * 1000.0;

    out.push_str(&format!("Reactor volume: {:.4} m3\n", reactor_volume_m3));
    out.push_str(&format!("Energy for heating: {:.1} kWh/m3\n", energy_kwh_m3));
    out.push_str(&format!("Total energy: {:.0} kWh/day\n", total_energy_kwh));
    out.push_str(&format!("Autothermal (COD >= 120 g/L): {}\n\n",
        if autothermal {"YES -- no external heat needed"} else {"NO -- needs external heat"}));

    // ═══ Phase 4: Destruction Performance ═══
    out.push_str("-- Phase 4: Destruction Performance --\n\n");

    let expected_dre = if is_supercritical {
        if residence_time_s >= 10.0 { 99.99 } else if residence_time_s >= 5.0 { 99.9 } else { 99.0 }
    } else {
        if residence_time_s >= 30.0 { 99.0 } else { 90.0 }
    };

    let effluent_pfas_ppb = pfas_conc_ppb * (1.0 - expected_dre / 100.0);

    out.push_str(&format!("Expected DRE: >{:.4}%\n", expected_dre));
    out.push_str(&format!("Effluent PFAS: {:.4} ppb ({:.2} ng/L)\n", effluent_pfas_ppb, effluent_pfas_ppb));
    out.push_str(&format!("PFAS mass destroyed: {:.2} g/day\n", pfas_mass_g_day * (expected_dre / 100.0)));
    out.push_str(&format!("Fluoride produced: {:.1} mg/day -> neutralize to CaF2\n\n", theoretical_f_mg));

    // ═══ Phase 5: Alternative Thermal Methods (Prasetya 2025) ═══
    out.push_str("-- Phase 5: Alternative Thermal Methods (2025-2026) --\n\n");
    out.push_str("Method                  Temp        DRE      Energy     Ref\n");
    out.push_str("------                  ----        ---      ------     ---\n");
    out.push_str("SCWO                    >374C       >99.99%  15-50      Aquarden 2026\n");
    out.push_str("MAT (microwave)         100-300C    90-99%   5-20       Prasetya 2025\n");
    out.push_str("EDP (plasma discharge)  ambient     80-95%   10-30      Prasetya 2025\n");
    out.push_str("APPJ (plasma jet)       ambient     70-90%   5-15       Prasetya 2025\n");
    out.push_str("GAC+HIP pretreat        ambient     23-96%   0.1        Zhang 2025 (EST)\n");
    out.push_str("Electro-NF              ambient     90.4%    1.92       Hua 2026\n\n");

    // ═══ Phase 6: Commercial Systems ═══
    out.push_str("-- Phase 6: Commercial Systems (2026) --\n\n");
    out.push_str("  Aquarden SuperOx (Denmark): full-scale, AFFF + landfill leachate\n");
    out.push_str("  Battelle PFAS Annihilator (US): mobile, skid-mounted\n");
    out.push_str("  374Water (US): sludge + AFFF concentrate, Oasis system\n");
    out.push_str("  Cole-Parmer: bench-scale SCWO for lab research\n\n");

    // ═══ Status Kepatuhan ═══
    out.push_str("-- STATUS KEPATUHAN --\n\n");
    out.push_str(&format!("EPA MCL: 4 ng/L -> Effluent: {:.2} ng/L -> {}\n\n",
        effluent_pfas_ppb, if effluent_pfas_ppb <= 4.0 {"PASS"} else {"FAIL"}));

    // ═══ PEMANTAUAN ═══
    out.push_str("-- PEMANTAUAN --\n");
    out.push_str("  Parameter: PFAS target list, TOF (Total Organic Fluorine), F-\n");
    out.push_str("  Metode: EPA 1633 (PFAS), ion chromatography (F-), TOF analyzer\n");
    out.push_str("  Note: Monitor TOF to detect hidden PFAS precursors\n");
    out.push_str("  Incomplete byproducts: short-chain PFCAs (still regulated!)\n\n");

    // ═══ Limitations ═══
    out.push_str("-- Limitations (honest) --\n");
    out.push_str("  • Simplified energy (no heat recovery modeling)\n");
    out.push_str("  • DRE is empirical estimate (not from kinetic model)\n");
    out.push_str("  • No byproduct speciation (short-chain PFCAs)\n");
    out.push_str("  • No corrosion/material compatibility analysis\n");
    out.push_str("  • Ref: Prasetya 2025 (DOI:10.1016/j.jhazmat.2025.138969)\n");
    out.push_str("  • Ref: Aquarden 2026; 374Water 2026\n");

    out
}
