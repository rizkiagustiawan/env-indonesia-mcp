/// AOP (Advanced Oxidation Process) Design
/// Ref: Glaze & Kang 1989; Beltran 2003; Oppenlander 2003
pub fn assess(
    contaminant: &str,
    initial_conc_mg_l: f64,
    target_conc_mg_l: f64,
    process_type: &str,     // "ozone", "uv_h2o2", "fenton", "uv_ozone"
    k_oh_m: f64,            // rate constant with OH radical (M-1 s-1)
    oh_conc_m: f64,         // OH radical concentration (M)
    contact_time_min: f64,
) -> String {
    let mut out = String::from("=== AOP (Advanced Oxidation Process) Design ===\n");
    out.push_str("Ref: Glaze & Kang 1989; Beltran 2003\n\n");

    let k = if k_oh_m > 0.0 { k_oh_m } else {
        match contaminant.to_lowercase().as_str() {
            "benzene" => 7.8e9, "toluene" => 5.1e9, "phenol" => 6.6e9,
            "tce" => 4.2e9, "pce" => 2.8e9, "atrazine" => 2.6e9,
            "mbte" => 1.6e9, "dye" => 1.0e9, _ => 1.0e9,
        }
    };
    let oh = if oh_conc_m > 0.0 { oh_conc_m } else {
        match process_type.to_lowercase().as_str() {
            "ozone" => 1e-12, "uv_h2o2" => 1e-10, "fenton" => 1e-9, "uv_ozone" => 1e-10, _ => 1e-11,
        }
    };

    let t_sec = contact_time_min * 60.0;
    let k_obs = k * oh; // pseudo-first-order rate (s-1)
    let c_ratio = (-k_obs * t_sec).exp();
    let c_final = initial_conc_mg_l * c_ratio;
    let removal = (1.0 - c_ratio) * 100.0;

    out.push_str(&format!("Contaminant: {} (k_OH = {:.1e} M-1 s-1)\n", contaminant, k));
    out.push_str(&format!("Process: {} ([OH] = {:.1e} M)\n", process_type, oh));
    out.push_str(&format!("C0: {:.2} mg/L -> target: {:.4} mg/L\n", initial_conc_mg_l, target_conc_mg_l));
    out.push_str(&format!("Contact time: {:.1} min ({:.0} s)\n\n", contact_time_min, t_sec));

    out.push_str("-- Kinetics --\n\n");
    out.push_str(&format!("  k_obs = k_OH * [OH] = {:.2e} s-1\n", k_obs));
    out.push_str(&format!("  C/C0 = exp(-k_obs*t) = {:.6}\n", c_ratio));
    out.push_str(&format!("  >> C_final = {:.4} mg/L\n", c_final));
    out.push_str(&format!("  >> Removal: {:.2}%\n\n", removal));

    // Required time for target
    let t_required = if target_conc_mg_l < initial_conc_mg_l && k_obs > 0.0 {
        (initial_conc_mg_l / target_conc_mg_l).ln() / k_obs / 60.0
    } else { 0.0 };

    out.push_str(&format!("  Required time for target: {:.1} min\n\n", t_required));

    if c_final <= target_conc_mg_l {
        out.push_str("  [OK] Target achieved within contact time\n");
    } else {
        out.push_str(&format!("  [WARN] Target NOT achieved. Need {:.1} min (current: {:.1})\n", t_required, contact_time_min));
    }

    // Process-specific notes
    out.push_str("\n-- Process Notes --\n");
    match process_type.to_lowercase().as_str() {
        "ozone" => out.push_str("  O3: pH > 9 for OH generation. Dose: 2-5 mg O3/mg COD.\n"),
        "uv_h2o2" => out.push_str("  UV/H2O2: H2O2 dose 5-50 mg/L. UV 254nm. pH 3-8 optimal.\n"),
        "fenton" => out.push_str("  Fenton: Fe2+ 0.1-1 mM, H2O2 1-10 mM, pH 2.5-3.5.\n"),
        "uv_ozone" => out.push_str("  UV/O3: Synergistic. UV 254nm decomposes O3 -> 2 OH.\n"),
        _ => {}
    }

    out.push_str("\n  Ref: Glaze & Kang 1989; Beltran 2003; Oppenlander 2003\n");
    out
}
