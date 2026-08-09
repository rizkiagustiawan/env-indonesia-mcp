/// Nano-Treatment Design — MOF/Nanomaterial for PFAS/Heavy Metal Removal
/// Ref: PMC Review 2025; ACS Chem Mater 2021; C&EN 2025
/// MOF data: PCN-999 1090 mg/g, TA@MOF-808 2500 mg/g, NU-1000 400-620 mg/g
pub fn assess(
    contaminant: &str, conc_mg_l: f64, volume_m3: f64,
    nanomaterial: &str, dose_g: f64, contact_time_min: f64,
) -> String {
    let mut out = String::from("=== Nano-Treatment Design (MOF/Nanomaterial) ===\n");
    out.push_str("Ref: PMC 2025; ACS Chem Mater 2021; C&EN 2025\n\n");
    let (qmax_mg_g, kl_mg_l, k2_g_mg_hr, regenerant) = match nanomaterial.to_lowercase().as_str() {
        s if s.contains("pcn-999") || s.contains("pcn999") => (1090.0, 0.5, 1.38e-3, "MeOH"),
        s if s.contains("ta@mof-808") || s.contains("mof-808") || s.contains("mof808") => (2500.0, 0.3, 1.5e-3, "MeOH (70°C vacuum)"),
        s if s.contains("nu-1000") || s.contains("nu1000") => (600.0, 0.4, 1.2e-3, "MeOH + NH4OAc"),
        s if s.contains("mil-101") || s.contains("mil101") => (400.0, 0.3, 1.0e-3, "MeOH + NaNO3"),
        s if s.contains("uiO-66") || s.contains("uio66") => (300.0, 0.25, 0.8e-3, "MeOH-NaCl"),
        s if s.contains("nZVI") || s.contains("zero_valent") => (200.0, 0.5, 2.0e-3, "Not regenerable"),
        s if s.contains("tio2") || s.contains("titanium_dioxide") => (150.0, 0.4, 1.5e-3, "UV + H2O2 regeneration"),
        _ => (500.0, 0.3, 1.0e-3, "MeOH"),
    };
    let q_e = (qmax_mg_g * kl_mg_l * conc_mg_l) / (1.0 + kl_mg_l * conc_mg_l);
    let k2 = k2_g_mg_hr;
    let t_hr = contact_time_min / 60.0;
    let q_t = if q_e > 0.0 && k2 > 0.0 {
        (q_e * q_e * k2 * t_hr) / (1.0 + q_e * k2 * t_hr)
    } else { 0.0 };
    let removal_pct = if q_e > 0.0 { (q_t / q_e * 100.0).min(100.0) } else { 0.0 };
    let total_removed_mg = q_t * dose_g;
    let effluent_conc = conc_mg_l - (total_removed_mg / (volume_m3 * 1000.0)).max(0.0);
    let total_mass_mg = conc_mg_l * volume_m3 * 1000.0;
    let removal_eff = if total_mass_mg > 0.0 { (total_removed_mg / total_mass_mg * 100.0).min(100.0) } else { 0.0 };
    out.push_str(&format!("Contaminant: {} (conc: {:.2} mg/L, vol: {:.1} m3)\n", contaminant, conc_mg_l, volume_m3));
    out.push_str(&format!("Nanomaterial: {} (qmax: {:.0} mg/g, KL: {:.1}, k2: {:.2e})\n", nanomaterial, qmax_mg_g, kl_mg_l, k2));
    out.push_str(&format!("Dose: {:.0} g, Contact time: {:.0} min\n\n", dose_g, contact_time_min));
    out.push_str("-- Langmuir Isotherm --\n\n");
    out.push_str(&format!("  q_e = (qmax × KL × C) / (1 + KL × C) = {:.1} mg/g\n", q_e));
    out.push_str(&format!("  Capacity utilization: {:.0}%\n\n", q_e / qmax_mg_g * 100.0));
    out.push_str("-- Pseudo-2nd-Order Kinetics --\n\n");
    out.push_str(&format!("  q_t = {:.1} mg/g at t={:.0} min\n", q_t, contact_time_min));
    out.push_str(&format!("  Equilibrium progress: {:.0}%\n\n", removal_pct));
    out.push_str("-- Performance --\n\n");
    out.push_str(&format!("  Total removed: {:.0} mg\n", total_removed_mg));
    out.push_str(&format!("  Removal efficiency: {:.1}%\n", removal_eff));
    out.push_str(&format!("  >> Effluent: {:.4} mg/L ({:.2} ng/L)\n\n", effluent_conc, effluent_conc * 1e6));
    out.push_str(&format!("  Regeneration: {}\n", regenerant));
    out.push_str("  Reusability: 3-8 cycles at 80-100% capacity\n\n");
    out.push_str("-- STATUS KEPATUHAN --\n");
    if contaminant.to_lowercase().contains("pfoa") || contaminant.to_lowercase().contains("pfos") {
        out.push_str(&format!("  EPA MCL: 4 ng/L → Effluent: {:.2} ng/L → {}\n", effluent_conc * 1e6, if effluent_conc * 1e6 <= 4.0 {"✅"} else {"❌"}));
    } else if contaminant.to_lowercase().contains("pb") || contaminant.to_lowercase().contains("cd") {
        out.push_str(&format!("  PP 22/2021: check baku_mutu_air_permukaan\n"));
    } else {
        out.push_str("  Check applicable baku mutu\n");
    }
    out.push_str("\n-- PEMANTAUAN --\n");
    out.push_str("  Parameter: contaminant + nanoparticle release (TSS)\n");
    out.push_str("  Note: Monitor for nanoparticle release to environment\n");
    out.push_str("  Ref: PMC 2025; ACS Chem Mater 2021\n");
    out
}
