/// PFAS Risk Screening — EPA MCL + WHO Guidelines
/// Ref: EPA 2024 (confirmed May 2025); WHO 2024
/// PFOA/PFOS: 4 ppt; PFNA/PFHxS/GenX: 10 ppt; Hazard Index = 1
pub fn assess(pfas_type: &str, conc_ng_l: f64, water_source: &str) -> String {
    let mut out = String::from("=== PFAS Risk Screening ===\n");
    out.push_str("Ref: EPA MCL 2024 (confirmed 2025); WHO 2024\n\n");
    let (limit_ng_l, source, health_effect) = match pfas_type.to_lowercase().as_str() {
        s if s.contains("pfoa") => (4.0, "EPA MCL", "Cancer, immune, developmental"),
        s if s.contains("pfos") => (4.0, "EPA MCL", "Cancer, immune, thyroid"),
        s if s.contains("pfna") => (10.0, "EPA MCL", "Liver, developmental"),
        s if s.contains("pfhxs") => (10.0, "EPA MCL", "Thyroid, immune"),
        s if s.contains("genx") || s.contains("hpfoda") => (10.0, "EPA MCL", "Liver, kidney"),
        s if s.contains("pfbs") => (0.0, "EPA HI component", "Thyroid, kidney"),
        s if s.contains("pfhxa") => (0.0, "EPA HI component", "Liver"),
        s if s.contains("pfba") => (0.0, "Not regulated", "Limited data"),
        _ => (0.0, "Not regulated", "Unknown"),
    };
    out.push_str(&format!("PFAS: {} (conc: {:.1} ng/L)\n", pfas_type, conc_ng_l));
    out.push_str(&format!("Water source: {}\n\n", water_source));
    out.push_str("-- Regulatory Limits --\n\n");
    out.push_str("  EPA MCL (2024, confirmed May 2025):\n");
    out.push_str("    PFOA: 4 ng/L | PFOS: 4 ng/L\n");
    out.push_str("    PFNA: 10 | PFHxS: 10 | GenX: 10 ng/L\n");
    out.push_str("    Hazard Index: ≤1 (mixture)\n");
    out.push_str("  WHO (2024): PFOA 100 ng/L, PFOS 200 ng/L\n");
    out.push_str("  Indonesia: BELUM ADA baku mutu PFAS spesifik\n\n");
    if limit_ng_l > 0.0 {
        let ratio = conc_ng_l / limit_ng_l;
        out.push_str("-- STATUS KEPATUHAN --\n\n");
        out.push_str(&format!("  {} limit: {:.0} ng/L\n", source, limit_ng_l));
        out.push_str(&format!("  Measured: {:.1} ng/L\n", conc_ng_l));
        out.push_str(&format!("  Ratio: {:.2}\n", ratio));
        out.push_str(if ratio <= 1.0 {"  ✅ MEMENUHI\n\n"} else {"  ❌ MELEBIHI — action required!\n\n"});
    } else {
        out.push_str("-- STATUS: Not individually regulated --\n");
        out.push_str("  Check Hazard Index if in mixture\n\n");
    }
    out.push_str(&format!("-- Health Effects --\n  {}\n\n", health_effect));
    out.push_str("-- RECOMMENDATION --\n");
    if conc_ng_l > limit_ng_l && limit_ng_l > 0.0 {
        out.push_str("  1. Treatment: GAC, IX resin, or foam fractionation\n");
        out.push_str("  2. Destruction: SCWO or electrochemical oxidation\n");
        out.push_str("  3. Source investigation\n");
        out.push_str("  4. Alternative water supply if acute risk\n");
    } else {
        out.push_str("  Monitor — annual testing recommended\n");
    }
    out.push_str("\n-- PEMANTAUAN --\n");
    out.push_str("  Parameter: PFAS target list (EPA 1633)\n");
    out.push_str("  Frekuensi: Annual (drinking water), Quarterly (contaminated site)\n");
    out.push_str("  Metode: EPA 1633 (LC-MS/MS), LOQ 1-10 ng/L\n");
    out.push_str("  Ref: EPA MCL 2024; WHO 2024\n");
    out
}
