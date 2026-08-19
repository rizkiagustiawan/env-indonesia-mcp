/// PFAS Electro-Nanofiltration Design (2026 SOTA)
///
/// IMPLEMENTS: Hua et al. 2026 "Enhanced removal of anionic PFAS by electrically
/// assisted nanofiltration" (J. Hazard. Mater. 504, 141395)
///
/// MODIFIED SDEM MODEL:
///   j_i = -P_i * dC_i/dx - z_i * P_i * C_i * dφ/dx
///   where dφ/dx is EXTERNALLY IMPOSED (not from zero-current)
///
/// KEY RESULTS (Hua 2026):
///   Optimal field: 11.1-13.3 V/cm
///   PFOA rejection: 90.4%
///   PFBS rejection: 83.9%
///   Energy: <1.92 kWh/m3
///   PFOA flux reduction: 75.2% (forward-aligned field)
///   Critical pressure: 0.8 MPa (beyond: concentration polarization)
///   PFBS electrophoretic mobility > PFOA
///
/// MECHANISM:
///   Forward-aligned field -> electrophoretic migration OPPOSES PFAS permeation
///   PFAS anions (z=-1) driven back toward feed by electric field
///   Solution-diffusion-electromigration decouples transport

pub fn assess(
    pfas_type: &str,
    feed_conc_ng_l: f64,
    membrane_type: &str,
    applied_voltage_v: f64,
    pressure_mpa: f64,
    flow_rate_lmh: f64,
    temperature_c: f64,
    treatment_goal_ng_l: f64,
) -> String {
    let mut out = String::from("=== PFAS Electro-Nanofiltration Design ===\n");
    out.push_str("Ref: Hua et al. 2026 (J. HazMat 504, 141395)\n");
    out.push_str("Model: Modified Solution-Diffusion-Electromigration (SDEM)\n\n");

    if feed_conc_ng_l <= 0.0 {
        return "ERROR [E102]: feed_conc must be > 0.".into();
    }
    if pressure_mpa <= 0.0 || pressure_mpa > 2.0 {
        return "ERROR [E102]: pressure 0.01-2.0 MPa.".into();
    }

    // ═══ Phase 1: SDEM Transport Model ═══
    out.push_str("-- Phase 1: Modified SDEM Transport --\n\n");
    out.push_str("j_i = -P_i*dC_i/dx - z_i*P_i*C_i*dφ/dx\n\n");
    out.push_str("Where:\n");
    out.push_str("  P_i = permeance (includes partition coefficient)\n");
    out.push_str("  z_i = charge number (PFAS anions: z = -1)\n");
    out.push_str("  dφ/dx = EXTERNALLY IMPOSED field (not zero-current)\n\n");

    let z_pfas = -1.0; // PFAS are anionic
    let membrane_thickness_nm = 25.0; // typical NF selective layer
    let membrane_thickness_m = membrane_thickness_nm * 1e-9;

    // Electric field
    let e_field_v_per_cm = applied_voltage_v / (membrane_thickness_m * 100.0);
    let e_field_v_per_m = e_field_v_per_cm * 100.0;

    out.push_str(&format!("PFAS: {} (z = {})\n", pfas_type, z_pfas));
    out.push_str(&format!("Membrane: {} ({:.0} nm selective layer)\n", membrane_type, membrane_thickness_nm));
    out.push_str(&format!("Applied voltage: {:.1} V\n", applied_voltage_v));
    out.push_str(&format!("Electric field: {:.1} V/cm ({:.0e} V/m)\n", e_field_v_per_cm, e_field_v_per_m));
    out.push_str(&format!("Pressure: {:.2} MPa\n", pressure_mpa));
    out.push_str(&format!("Flow rate: {:.0} LMH\n", flow_rate_lmh));
    out.push_str(&format!("Temperature: {:.1} C\n\n", temperature_c));

    // ═══ Phase 2: Rejection Prediction ═══
    out.push_str("-- Phase 2: Rejection Prediction (Hua 2026) --\n\n");

    // Optimal range: 11.1-13.3 V/cm
    let optimal_min = 11.1;
    let optimal_max = 13.3;

    let (base_rejection, _flux_reduction) = match pfas_type.to_lowercase().as_str() {
        s if s.contains("pfoa") => (0.904, 0.752),
        s if s.contains("pfbs") => (0.839, 0.65),
        s if s.contains("pfos") => (0.92, 0.78), // similar to PFOA but longer chain
        s if s.contains("pfba") => (0.70, 0.50), // short chain, harder
        s if s.contains("pfhxs") => (0.80, 0.60),
        s if s.contains("genx") => (0.85, 0.65),
        _ => (0.80, 0.60),
    };

    // Adjust for actual field vs optimal
    let field_ratio = if e_field_v_per_cm < optimal_min {
        e_field_v_per_cm / optimal_min // suboptimal
    } else if e_field_v_per_cm > optimal_max {
        // Beyond optimal: concentration polarization
        let excess = (e_field_v_per_cm - optimal_max) / optimal_max;
        1.0 - 0.1 * excess.min(0.5) // slight degradation
    } else {
        1.0 // in optimal range
    };

    // Pressure effect
    let pressure_ratio = if pressure_mpa > 0.8 {
        // Beyond critical: rejection collapse (chain-length dependent)
        let collapse = (pressure_mpa - 0.8) / 0.8 * 0.15;
        1.0 - collapse.min(0.3)
    } else {
        1.0 + 0.05 * (0.8 - pressure_mpa).max(0.0) // lower pressure slightly better
    };

    let predicted_rejection = (base_rejection * field_ratio * pressure_ratio).min(0.99).max(0.0);
    let permeate_conc = feed_conc_ng_l * (1.0 - predicted_rejection);

    out.push_str(&format!("Base rejection (Hua 2026): {:.1}%\n", base_rejection * 100.0));
    out.push_str(&format!("Field efficiency: {:.2} (optimal: {}-{} V/cm)\n", field_ratio, optimal_min, optimal_max));
    out.push_str(&format!("Pressure effect: {:.2} (critical: 0.8 MPa)\n", pressure_ratio));
    out.push_str(&format!(">> Predicted rejection: {:.1}%\n", predicted_rejection * 100.0));
    out.push_str(&format!(">> Permeate concentration: {:.2} ng/L\n\n", permeate_conc));

    // ═══ Phase 3: Energy Consumption ═══
    out.push_str("-- Phase 3: Energy Consumption --\n\n");

    // Energy: E = (V * I) / Q, simplified to field-based estimate
    // Hua 2026: <1.92 kWh/m3 at optimal conditions
    let base_energy = 1.5; // kWh/m3 at optimal field
    let energy = base_energy * (e_field_v_per_cm / 12.0).powi(2).min(3.0);

    out.push_str(&format!("Predicted energy: {:.2} kWh/m3 (Hua 2026: <1.92)\n", energy));
    out.push_str(&format!("Comparison:\n"));
    out.push_str("  GAC: 0.05-0.10 kWh/m3 (but needs regeneration)\n");
    out.push_str("  IEX: 0.10-0.20 kWh/m3 (but needs resin replacement)\n");
    out.push_str(&format!("  E-NF: {:.2} kWh/m3 (this design)\n", energy));
    out.push_str("  RO: 3-5 kWh/m3 (higher energy)\n\n");

    // ═══ Phase 4: Treatment Goal Compliance ═══
    out.push_str("-- Phase 4: Treatment Goal Compliance --\n\n");
    out.push_str(&format!("Feed: {:.2} ng/L -> Permeate: {:.2} ng/L\n", feed_conc_ng_l, permeate_conc));
    out.push_str(&format!("Goal: {:.2} ng/L\n\n", treatment_goal_ng_l));

    if permeate_conc <= treatment_goal_ng_l {
        out.push_str(">> PASS: permeate meets treatment goal\n\n");
    } else {
        let removal_needed = 1.0 - treatment_goal_ng_l / feed_conc_ng_l;
        out.push_str(&format!(">> FAIL: need {:.1}% removal, achieved {:.1}%\n",
            removal_needed * 100.0, predicted_rejection * 100.0));
        out.push_str("Options:\n");
        out.push_str("  1. Increase voltage (toward 13.3 V/cm optimal)\n");
        out.push_str("  2. Multi-stage E-NF (2-3 stages in series)\n");
        out.push_str("  3. Combine with GAC post-treatment\n");
        out.push_str("  4. Lower pressure below 0.8 MPa critical threshold\n\n");

        // Multi-stage calculation
        let n_stages_needed = ((treatment_goal_ng_l / feed_conc_ng_l).ln() /
            (1.0 - predicted_rejection).ln().max(-0.99)).ceil() as u32;
        out.push_str(&format!("  >> Multi-stage: {} stages needed for goal\n\n", n_stages_needed.max(1)));
    }

    // ═══ Phase 5: Comparison to Other Technologies ═══
    out.push_str("-- Phase 5: Technology Comparison (2026) --\n\n");
    out.push_str("Technology           PFOA Removal  Energy(kWh/m3)  Cost    Ref\n");
    out.push_str("----------           ------------  --------------  ----    ---\n");
    out.push_str("Electro-NF (E-NF)    90.4%         1.92            Medium  Hua 2026\n");
    out.push_str("GAC                  70-85%        0.10            Low     Jafarinejad 2025\n");
    out.push_str("Ion Exchange (IEX)  80-95%        0.20            Medium  Chen 2025\n");
    out.push_str("MOF (PCN-999)        >99%          0.05            High    Lee 2025\n");
    out.push_str("SCWO                 >99.99%       15-50           High    Prasetya 2025\n");
    out.push_str("Foam Fractionation   90-99%        0.01            Low     (AWI Langmuir)\n");
    out.push_str("Photocatalytic TiO2  80%           0.50            Low     McQueen 2025\n");
    out.push_str("Colloidal Carbon     >99.9%        N/A (in-situ)   Medium  Hatton 2025\n\n");

    // ═══ Indonesia Context ═══
    out.push_str("-- Indonesia Context --\n");
    out.push_str("  Indonesia belum punya baku mutu PFAS\n");
    out.push_str("  EPA MCL: 4 ng/L PFOA/PFOS (May 2025)\n");
    out.push_str("  WHO guideline: 100 ng/L PFOA (2022)\n");
    out.push_str("  PFAS sources: textile (Bandung), firefighting foam (airport/military)\n");
    out.push_str("  Permen LH 12/2025: tekstil effluent (PFAS not yet regulated)\n\n");

    // ═══ Limitations ═══
    out.push_str("-- Limitations (honest) --\n");
    out.push_str("  • Simplified SDEM (no actual finite-element membrane model)\n");
    out.push_str("  • Rejection values from Hua 2026 empirical (not computed from SDEM)\n");
    out.push_str("  • No concentration polarization modeling (critical >0.8 MPa)\n");
    out.push_str("  • No fouling dynamics (membrane aging)\n");
    out.push_str("  • Temperature effects simplified\n");
    out.push_str("  • Full SDEM needs numerical integration of differential equations\n");
    out.push_str("  • Ref: Hua 2026 (DOI:10.1016/j.jhazmat.2026.141395)\n");
    out.push_str("  • Ref: Yaroshchuk 2016 (OSTI) for fundamental SDEM\n");

    out
}
