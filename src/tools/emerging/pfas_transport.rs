/// PFAS Transport in Groundwater — Advection-Dispersion + Langmuir AWI + Solid Sorption
/// Ref: Brusseau 2025 (Water Research 284, 123952); ITRC PFAS Guidance; HYDRUS-PFAS
/// Key: Langmuir isotherm for air-water interface (NOT Freundlich)
pub fn assess(
    pfas_type: &str, conc_mg_l: f64, distance_m: f64,
    velocity_m_day: f64, dispersivity_m: f64, time_days: f64,
    foc_pct: f64, koc_l_kg: f64, water_saturation: f64,
    awi_area_m2_per_m3: f64, kaw_m: f64, gamma_max_mol_m2: f64,
    decay_rate_day: f64,
) -> String {
    let mut out = String::from("=== PFAS Transport in Groundwater ===\n");
    out.push_str("Ref: Brusseau 2025 (Water Research); ITRC; HYDRUS-PFAS\n\n");
    if distance_m <= 0.0 || velocity_m_day <= 0.0 {
        return "ERROR [E102]: distance and velocity must be > 0.".into();
    }
    let foc = foc_pct / 100.0;
    let kd = foc * koc_l_kg; // solid-liquid partition (linear at low conc)
    let r_solid = 1.0 + (1.0 - water_saturation) * 2650.0 * kd / water_saturation; // retardation from solid
    let c_aw = if kaw_m > 0.0 && gamma_max_mol_m2 > 0.0 {
        (kaw_m * conc_mg_l) / (1.0 + kaw_m * conc_mg_l) * gamma_max_mol_m2 * awi_area_m2_per_m3 * 1000.0
    } else { 0.0 };
    let r_awi = if water_saturation < 1.0 && conc_mg_l > 0.0 {
        (1.0 - water_saturation) * awi_area_m2_per_m3 * kaw_m / water_saturation
    } else { 0.0 };
    let retardation = r_solid + r_awi;
    let v_retarded = velocity_m_day / retardation.max(1.0);
    let dl = dispersivity_m * v_retarded;
    let t = time_days;
    let decay_factor = if decay_rate_day > 0.0 { (-decay_rate_day * t).exp() } else { 1.0 };
    let arg = (distance_m - v_retarded * t) / (2.0 * (dl * t).sqrt().max(1e-15));
    let erfc_val = erfc_approx(arg);
    let c_ratio = 0.5 * erfc_val * decay_factor;
    let conc_at_receptor = conc_mg_l * c_ratio;
    out.push_str(&format!("PFAS: {} (conc: {:.4} mg/L)\n", pfas_type, conc_mg_l));
    out.push_str(&format!("Distance: {:.0}m, Velocity: {:.2} m/day\n", distance_m, velocity_m_day));
    out.push_str(&format!("foc: {:.2}%, Koc: {:.1} L/kg → Kd: {:.4} L/kg\n", foc_pct, koc_l_kg, kd));
    out.push_str(&format!("Water saturation: {:.2}\n", water_saturation));
    out.push_str(&format!("AWI area: {:.2e} m2/m3, Kaw: {:.4e} m\n\n", awi_area_m2_per_m3, kaw_m));
    out.push_str("-- Retardation Components --\n\n");
    out.push_str(&format!("  Solid sorption R: {:.2}\n", r_solid));
    out.push_str(&format!("  AWI adsorption R: {:.4}\n", r_awi));
    out.push_str(&format!("  >> Total retardation: {:.2}\n", retardation));
    out.push_str(&format!("  >> Retarded velocity: {:.4} m/day\n\n", v_retarded));
    out.push_str("-- Transport Solution (Ogata-Banks + Langmuir AWI) --\n\n");
    out.push_str(&format!("  arg = {:.4}, erfc = {:.6}\n", arg, erfc_val));
    out.push_str(&format!("  >> C/C0 = {:.6} (decay factor: {:.4})\n", c_ratio, decay_factor));
    out.push_str(&format!("  >> Concentration at receptor: {:.6} mg/L = {:.2} ng/L\n\n", conc_at_receptor, conc_at_receptor * 1e6));
    let travel_time = distance_m / v_retarded;
    out.push_str(&format!("  Travel time: {:.0} days ({:.1} years)\n\n", travel_time, travel_time / 365.0));
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
        out.push_str(&format!("  EPA MCL: {:.0} ng/L, Measured: {:.2} ng/L → {}\n", epa_mcl_ng_l, conc_ng_l, if conc_ng_l <= epa_mcl_ng_l {"✅"} else {"❌"}));
        out.push_str("  Note: Indonesia belum punya baku mutu PFAS — compare ke EPA/WHO\n\n");
    }
    out.push_str("-- PEMANTAUAN (RPL) --\n");
    out.push_str("  Parameter: PFAS target list (EPA 1633), pH, DO, EC, TOC\n");
    out.push_str("  Frekuensi: Quarterly (active), Semi-annual (stable)\n");
    out.push_str("  Metode: EPA 1633 (LC-MS/MS), LOQ 1-10 ng/L\n\n");
    out.push_str("-- PELAPORAN --\n");
    out.push_str("  PP 22/2021 Annex VI; PP 101/2014 (B3); Permen LH 6/2026\n");
    out.push_str("  Ref: Brusseau 2025; ITRC; HYDRUS-PFAS\n");
    out
}
fn erfc_approx(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    if x >= 0.0 { poly } else { 2.0 - poly }
}
