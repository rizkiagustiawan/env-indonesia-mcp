/// Chlorine Demand & CT Concept
/// Ref: Crittenden 2012 (MWH); EPA 40 CFR 141.72
pub fn assess(
    free_chlorine_mg_l: f64,
    contact_time_min: f64,
    target_log_removal: f64,
    contaminant: &str,
    temp_c: f64,
    ph: f64,
) -> String {
    let mut out = String::from("=== Chlorine Demand & CT Concept ===\n");
    out.push_str("Ref: Crittenden 2012 (MWH); EPA 40 CFR 141.72\n\n");

    let ct = free_chlorine_mg_l * contact_time_min; // mg·min/L

    // EPA CT tables (simplified) for Giardia and Virus
    let ct_giardia_3log = match (temp_c as i32, ph) {
        (t, p) if t <= 5 && p <= 7.5 => 181.0,
        (t, p) if t <= 5 => 256.0,
        (t, p) if t <= 10 && p <= 7.5 => 128.0,
        (t, p) if t <= 10 => 183.0,
        (t, p) if t <= 15 && p <= 7.5 => 91.0,
        (t, p) if t <= 15 => 131.0,
        (t, p) if t <= 20 && p <= 7.5 => 65.0,
        (t, p) if t <= 20 => 93.0,
        (t, p) if t <= 25 && p <= 7.5 => 46.0,
        _ => 67.0,
    };

    let ct_virus_4log = match temp_c as i32 {
        t if t <= 5 => 12.0,
        t if t <= 10 => 8.0,
        t if t <= 15 => 6.0,
        t if t <= 20 => 4.0,
        _ => 3.0,
    };

    let giardia_removal = if ct > 0.0 { 3.0 * ct / ct_giardia_3log } else { 0.0 };
    let virus_removal = if ct > 0.0 { 4.0 * ct / ct_virus_4log } else { 0.0 };

    out.push_str(&format!("Free Cl2: {:.1} mg/L, Contact: {:.1} min\n", free_chlorine_mg_l, contact_time_min));
    out.push_str(&format!(">> CT = {:.1} mg*min/L\n\n", ct));
    out.push_str(&format!("Temp: {:.0}C, pH: {:.1}\n\n", temp_c, ph));

    out.push_str("-- EPA SWTR Compliance --\n\n");
    out.push_str(&format!("  Giardia (3-log required): CT_needed={:.0}, achieved_log={:.1}\n", ct_giardia_3log, giardia_removal));
    out.push_str(&format!("  Virus (4-log required): CT_needed={:.0}, achieved_log={:.1}\n\n", ct_virus_4log, virus_removal));

    if giardia_removal >= 3.0 { out.push_str("  [OK] Giardia 3-log achieved\n"); }
    else { out.push_str(&format!("  [WARN] Giardia: only {:.1}-log. Need CT={:.0}\n", giardia_removal, ct_giardia_3log)); }

    if virus_removal >= 4.0 { out.push_str("  [OK] Virus 4-log achieved\n\n"); }
    else { out.push_str(&format!("  [WARN] Virus: only {:.1}-log. Need CT={:.0}\n\n", virus_removal, ct_virus_4log)); }

    // Breakpoint chlorination
    out.push_str("-- Breakpoint Chlorination --\n");
    out.push_str("  Cl2:NH3-N weight ratio for breakpoint = 7.6:1 (theoretical) to 10:1 (practical)\n");
    out.push_str("  Before breakpoint: monochloramine (NH2Cl) formed\n");
    out.push_str("  At breakpoint (7.6:1): free chlorine residual appears\n\n");

    out.push_str("  Ref: Crittenden 2012 (MWH); EPA SWTR\n");
    out
}
