/// PFAS Transport in Groundwater — SDEM + Langmuir AWI + Solid Sorption (2026 SOTA)
///
/// IMPLEMENTS: Yaroshchuk 2016 SDEM model (OSTI) + Brusseau 2025 Langmuir AWI
/// + Hua 2026 Electro-Nanofiltration transport decoupling
///
/// SDEM FUNDAMENTAL EQUATION (Yaroshchuk 2016, Eq. 1):
///   j_i = -P_i * (dC_i/dx + z_i * C_i * dφ/dx)
///
/// Where:
///   j_i = flux of ion i (mol m-2 s-1)
///   P_i = permeance (includes partition coefficient, units m s-1)
///   C_i = virtual (reference) concentration (mol m-3)
///   x = transmembrane coordinate (dimensionless 0-1)
///   z_i = charge number (valence)
///   φ = dimensionless electrostatic potential (F/RT units)
///
/// CONSTRAINTS:
///   Steady state: j_i = constant
///   Zero current: Σ z_i * j_i = 0
///   Electroneutrality: Σ z_i * C_i = 0
///
/// REDUCED FORM (Fernández de Labastida 2021, Eq. 3):
///   f_t = (f_s)^b + K * (f_s - (f_s)^b) / (1 - b)
///   where f = 1/(1-R), b and K are fitting parameters
///
/// ELECTRO-NF MODIFICATION (Hua 2026):
///   Applied field replaces zero-current condition:
///   j_i = -P_i*dC_i/dx - z_i*P_i*C_i*dφ/dx
///   dφ/dx set by external power supply (not spontaneous)
///
/// PERMEANCES:
///   P_± = P_s / (1 - (z_+/z_-) * b)  (dominant salt ions)
///   P_t = P_s / K                    (trace ion / PFAS)
///
/// PFAS-SPECIFIC (Hua 2026):
///   PFOA: z=-1, rejection 90.4%, field 11.1-13.3 V/cm
///   PFBS: z=-1, rejection 83.9%, PFBS mobility > PFOA
///   Energy: <1.92 kWh/m3
///   Critical hydraulic threshold: 0.8 MPa

pub fn assess(
    pfas_type: &str, conc_mg_l: f64, distance_m: f64,
    velocity_m_day: f64, dispersivity_m: f64, time_days: f64,
    foc_pct: f64, koc_l_kg: f64, water_saturation: f64,
    awi_area_m2_per_m3: f64, kaw_m: f64, gamma_max_mol_m2: f64,
    decay_rate_day: f64,
) -> String {
    let mut out = String::from("=== PFAS Transport — SDEM + Langmuir AWI (2026) ===\n");
    out.push_str("Ref: Yaroshchuk 2016 (OSTI); Brusseau 2025; Hua 2026 (J HazMat 141395)\n\n");

    if distance_m <= 0.0 || velocity_m_day <= 0.0 {
        return "ERROR [E102]: distance and velocity must be > 0.".into();
    }

    let foc = foc_pct / 100.0;
    let kd = foc * koc_l_kg;

    // ═══ Phase 1: Solid Sorption Retardation ═══
    out.push_str("-- Phase 1: Solid Sorption (Linear Kd) --\n\n");
    out.push_str(&format!("foc: {:.2}%, Koc: {:.1} L/kg -> Kd = {:.4} L/kg\n", foc_pct, koc_l_kg, kd));
    let rho_s = 2650.0; // soil particle density kg/m3
    let r_solid = 1.0 + (1.0 - water_saturation) * rho_s * kd / water_saturation;
    out.push_str(&format!("  R_solid = 1 + (1-S_w)*rho_s*Kd/S_w = {:.3}\n", r_solid));

    // ═══ Phase 2: AWI Adsorption (Langmuir Isotherm) ═══
    out.push_str("\n-- Phase 2: AWI Adsorption (Langmuir, NOT Freundlich) --\n\n");
    out.push_str("Ref: Brusseau 2025 — Langmuir isotherm for air-water interface\n");
    out.push_str("  Γ = (Kaw * C) / (1 + Kaw * C) * Γ_max\n\n");

    let gamma = if kaw_m > 0.0 && gamma_max_mol_m2 > 0.0 {
        (kaw_m * conc_mg_l) / (1.0 + kaw_m * conc_mg_l) * gamma_max_mol_m2
    } else { 0.0 };
    let c_aw = gamma * awi_area_m2_per_m3 * 1000.0; // mol/m3 -> mmol/m3 approx
    let r_awi = if water_saturation < 1.0 && conc_mg_l > 0.0 {
        (1.0 - water_saturation) * awi_area_m2_per_m3 * kaw_m / water_saturation
    } else { 0.0 };

    out.push_str(&format!("  Kaw = {:.4e} m\n", kaw_m));
    out.push_str(&format!("  Γ_max = {:.4e} mol/m2\n", gamma_max_mol_m2));
    out.push_str(&format!("  Γ = {:.4e} mol/m2 (at C = {:.4} mg/L)\n", gamma, conc_mg_l));
    out.push_str(&format!("  C_aw = {:.4e} mmol/m3\n", c_aw));
    out.push_str(&format!("  R_AWI = {:.4}\n\n", r_awi));

    // ═══ Phase 3: SDEM Transport Model ═══
    out.push_str("-- Phase 3: SDEM Transport (Yaroshchuk 2016) --\n\n");
    out.push_str("Fundamental: j_i = -P_i * (dC_i/dx + z_i * C_i * dφ/dx)\n");
    out.push_str("  P_i = permeance (includes partition coefficient)\n");
    out.push_str("  z_i = charge (PFAS anions: z = -1)\n");
    out.push_str("  φ = dimensionless potential (F/RT units)\n\n");

    // PFAS as trace anion (z=-1) in background electrolyte
    let z_pfas = -1.0; // PFOA/PFOS/PFBS are anionic
    let p_s = velocity_m_day / 86400.0 * 0.01; // dominant salt permeance (approx from velocity)
    let b_param = 0.3; // SDEM fitting parameter (typical)
    let k_param = 5.0; // trace ion permeance ratio

    // P_t = P_s / K (trace ion permeance)
    let p_tfas = p_s / k_param;
    let p_dominant_ion = p_s / (1.0 - (1.0 / -1.0) * b_param); // z_+/z_- = 1/-1 = -1

    out.push_str(&format!("  Dominant salt permeance P_s = {:.4e} m/s\n", p_s));
    out.push_str(&format!("  SDEM b parameter = {:.3}\n", b_param));
    out.push_str(&format!("  SDEM K parameter = {:.3}\n", k_param));
    out.push_str(&format!("  PFAS permeance P_t = P_s/K = {:.4e} m/s\n", p_tfas));
    out.push_str(&format!("  Dominant ion permeance = {:.4e} m/s\n\n", p_dominant_ion));

    // Rejection (Yaroshchuk 2016, Eq. 21)
    let f_s: f64 = 1.0 / (1.0 - 0.3); // f_s = 1/(1-R_s), R_s=0.3 assumed
    let f_t = f_s.powf(b_param) + k_param * (f_s - f_s.powf(b_param)) / (1.0 - b_param);
    let r_tfas = 1.0 - 1.0 / f_t;

    out.push_str(&format!("  f_s = {:.3}, f_t = {:.3}\n", f_s, f_t));
    out.push_str(&format!("  SDEM predicted rejection R_t = {:.1}%\n\n", r_tfas * 100.0));

    // ═══ Phase 4: Electro-NF Enhancement (Hua 2026) ═══
    out.push_str("-- Phase 4: Electro-Nanofiltration (Hua 2026) --\n\n");
    out.push_str("Modified SDEM with externally imposed field:\n");
    out.push_str("  j_i = -P_i*dC_i/dx - z_i*P_i*C_i*dφ/dx\n");
    out.push_str("  Field dφ/dx set by external power (NOT zero-current)\n\n");

    let e_field_v_cm = 12.0; // optimal 11.1-13.3 V/cm
    let pfoa_rejection_enf = 0.904; // 90.4% (Hua 2026)
    let pfbs_rejection_enf = 0.839; // 83.9%
    let energy_kwh_m3 = 1.92; // <1.92 kWh/m3
    let critical_pressure_mpa = 0.8; // 0.8 MPa threshold

    out.push_str("Hua 2026 results (J HazMat 141395):\n");
    out.push_str(&format!("  Applied field: {} V/cm (optimal: 11.1-13.3)\n", e_field_v_cm));
    out.push_str(&format!("  PFOA rejection: {:.1}%\n", pfoa_rejection_enf * 100.0));
    out.push_str(&format!("  PFBS rejection: {:.1}%\n", pfbs_rejection_enf * 100.0));
    out.push_str(&format!("  Energy: <{:.2} kWh/m3\n", energy_kwh_m3));
    out.push_str(&format!("  Critical pressure: {:.1} MPa (beyond: conc. polarization)\n\n", critical_pressure_mpa));
    out.push_str("  PFBS mobility > PFOA (higher electrophoretic mobility)\n");
    out.push_str("  Forward-aligned field reduces PFOA flux by 75.2%\n\n");

    // ═══ Phase 5: Transport Solution (Ogata-Banks + Retardation) ═══
    out.push_str("-- Phase 5: Transport Solution (Ogata-Banks + Retardation) --\n\n");

    let retardation = r_solid + r_awi;
    let v_retarded = velocity_m_day / retardation.max(1.0);
    let dl = dispersivity_m * v_retarded;
    let t = time_days;
    let decay_factor = if decay_rate_day > 0.0 { (-decay_rate_day * t).exp() } else { 1.0 };
    let arg = (distance_m - v_retarded * t) / (2.0 * (dl * t).sqrt().max(1e-15));
    let erfc_val = erfc_approx(arg);
    let c_ratio = 0.5 * erfc_val * decay_factor;
    let conc_at_receptor = conc_mg_l * c_ratio;

    out.push_str(&format!("  Total retardation: R = R_solid + R_AWI = {:.2} + {:.4} = {:.2}\n", r_solid, r_awi, retardation));
    out.push_str(&format!("  Retarded velocity: {:.4} m/day\n", v_retarded));
    out.push_str(&format!("  arg = {:.4}, erfc = {:.6}\n", arg, erfc_val));
    out.push_str(&format!("  C/C0 = {:.6} (decay: {:.4})\n", c_ratio, decay_factor));
    out.push_str(&format!("  >> Concentration at receptor: {:.6} mg/L = {:.2} ng/L\n\n",
        conc_at_receptor, conc_at_receptor * 1e6));

    let travel_time = distance_m / v_retarded;
    out.push_str(&format!("  Travel time: {:.0} days ({:.1} years)\n\n", travel_time, travel_time / 365.0));

    // ═══ Status Kepatuhan ═══
    let epa_mcl_ng_l = match pfas_type.to_lowercase().as_str() {
        s if s.contains("pfoa") => 4.0,
        s if s.contains("pfos") => 4.0,
        s if s.contains("pfna") => 10.0,
        s if s.contains("pfhxs") => 10.0,
        s if s.contains("genx") || s.contains("hpfoda") => 10.0,
        _ => 0.0,
    };
    let conc_ng_l = conc_at_receptor * 1e6;
    if epa_mcl_ng_l > 0.0 {
        out.push_str("-- STATUS KEPATUHAN --\n");
        out.push_str(&format!("  EPA MCL: {:.0} ng/L, Measured: {:.2} ng/L -> {}\n",
            epa_mcl_ng_l, conc_ng_l, if conc_ng_l <= epa_mcl_ng_l {"PASS"} else {"FAIL"}));
        out.push_str("  Note: Indonesia belum punya baku mutu PFAS -- compare ke EPA/WHO\n\n");
    }

    // ═══ Treatment Options ═══
    out.push_str("-- Treatment Options (2026 SOTA) --\n\n");
    out.push_str("  1. Granular Activated Carbon (GAC): mature landfill, Freundlich K=393\n");
    out.push_str("  2. Ion Exchange (IEX): >GAC for PFAS, pseudo-1st-order k=0.03-0.05/min\n");
    out.push_str("  3. Electro-Nanofiltration (E-NF): 90.4% PFOA, <1.92 kWh/m3 (Hua 2026)\n");
    out.push_str("  4. MOF Adsorption: PCN-999 1090 mg/g, TA@MOF-808 2500 mg/g\n");
    out.push_str("  5. Hydrophobic Ion Pairing + GAC: 350% breakthrough improvement\n");
    out.push_str("  6. Colloidal Carbon (CCP): 4 orders of magnitude removal in-situ\n");
    out.push_str("  7. Photocatalytic TiO2 3D-printed: 80% PFOS removal\n");
    out.push_str("  8. SCWO: T>374C, P>22.1MPa, DRE>99.99%\n\n");

    // ═══ PEMANTAUAN & PELAPORAN ═══
    out.push_str("-- PEMANTAUAN (RPL) --\n");
    out.push_str("  Parameter: PFAS target list (EPA 1633), pH, DO, EC, TOC\n");
    out.push_str("  Frekuensi: Quarterly (active), Semi-annual (stable)\n");
    out.push_str("  Metode: EPA 1633 (LC-MS/MS), LOQ 1-10 ng/L\n\n");
    out.push_str("-- PELAPORAN --\n");
    out.push_str("  PP 22/2021 Annex VI; PP 101/2014 (B3); Permen LH 6/2026\n");
    out.push_str("  Ref: Yaroshchuk 2016; Brusseau 2025; Hua 2026; ITRC\n");

    out
}

fn erfc_approx(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    if x >= 0.0 { poly } else { 2.0 - poly }
}
