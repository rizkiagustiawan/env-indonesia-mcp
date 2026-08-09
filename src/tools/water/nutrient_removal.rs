/// Nutrient Removal (Nitrification + Denitrification)
/// Ref: Grady et al. 2011; Metcalf & Eddy 2004; Liu 2025
pub fn assess(
    influent_tkn_mg_l: f64,
    influent_no3_mg_l: f64,
    target_tn_mg_l: f64,
    srt_days: f64,
    temp_c: f64,
    do_mg_l: f64,
    mlss_mg_l: f64,
) -> String {
    let mut out = String::from("=== Nutrient Removal (Nitrification/Denitrification) ===\n");
    out.push_str("Ref: Grady et al. 2011; Metcalf & Eddy 2004\n\n");

    // Nitrification kinetics (AOB + NOB)
    let mu_aob_20 = 0.8; // day-1 at 20C
    let mu_nob_20 = 0.9; // day-1
    let theta: f64 = 1.072; // temp correction
    let mu_aob = mu_aob_20 * theta.powf(temp_c - 20.0);
    let mu_nob = mu_nob_20 * theta.powf(temp_c - 20.0);
    let srt_min_aob = 1.0 / mu_aob.max(1e-6);
    let srt_min_nob = 1.0 / mu_nob.max(1e-6);

    // DO limitation (Monod)
    let ko_do = 0.5; // mg/L half-saturation for DO
    let do_factor = do_mg_l / (ko_do + do_mg_l).max(1e-6);

    // Nitrification achievable?
    let nitrification_ok = srt_days > srt_min_aob && do_mg_l > 1.0;

    // Denitrification
    let k_dn_20 = 0.1; // day-1 (specific denitrification rate, g NO3-N/g VSS/day)
    let k_dn = k_dn_20 * 1.072_f64.powf(temp_c - 20.0);
    let no3_removed = k_dn * mlss_mg_l * 0.8 * do_factor; // mg/L/day approx

    out.push_str(&format!("Influent: TKN={:.1} mg/L, NO3={:.1} mg/L\n", influent_tkn_mg_l, influent_no3_mg_l));
    out.push_str(&format!("Target TN: {:.1} mg/L\n", target_tn_mg_l));
    out.push_str(&format!("SRT: {:.1} days, Temp: {:.1}C, DO: {:.1} mg/L\n\n", srt_days, temp_c, do_mg_l));

    out.push_str("-- Nitrification Kinetics --\n\n");
    out.push_str(&format!("  AOB: mu={:.3}/day, SRT_min={:.1}d (at {:.0}C)\n", mu_aob, srt_min_aob, temp_c));
    out.push_str(&format!("  NOB: mu={:.3}/day, SRT_min={:.1}d\n", mu_nob, srt_min_nob));
    out.push_str(&format!("  DO factor: {:.3} (Ko={:.1})\n\n", do_factor, ko_do));

    if nitrification_ok {
        let no3_produced = influent_tkn_mg_l; // all TKN -> NO3 (assuming complete)
        out.push_str(&format!("  [OK] Nitrification: TKN {:.1} -> NO3 {:.1} mg/L\n", influent_tkn_mg_l, no3_produced));
    } else {
        out.push_str("  [WARN] SRT < SRT_min or DO < 1 — nitrification INHIBITED\n");
        out.push_str("  TKN passes through without oxidation\n\n");
    }

    out.push_str("-- Denitrification --\n\n");
    out.push_str(&format!("  k_dn ({:.0}C): {:.4}/day\n", temp_c, k_dn));
    out.push_str(&format!("  NO3 removal rate: ~{:.1} mg N/L/day\n", no3_removed));

    let no3_after_dn = (influent_no3_mg_l + if nitrification_ok { influent_tkn_mg_l } else { 0.0 } - no3_removed).max(0.0);
    let tn_effluent = no3_after_dn + if !nitrification_ok { influent_tkn_mg_l } else { 0.0 };

    out.push_str(&format!("  >> NO3 after DN: {:.1} mg/L\n", no3_after_dn));
    out.push_str(&format!("  >> TN effluent: {:.1} mg/L (target: {:.1})\n\n", tn_effluent, target_tn_mg_l));

    if tn_effluent <= target_tn_mg_l {
        out.push_str("  [OK] Meets TN target\n");
    } else {
        out.push_str("  [WARN] TN exceeds target. Options: increase SRT, add external C (methanol), or anoxic zone.\n");
    }

    out.push_str("\n  Ref: Grady et al. 2011; Metcalf & Eddy 2004; Liu 2025\n");
    out
}
