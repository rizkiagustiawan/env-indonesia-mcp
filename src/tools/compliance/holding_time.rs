/// Holding Time & Preservation Checker
/// Ref: EPA 40 CFR 136 Table II; Standard Methods 23rd Ed
pub fn assess(parameter: &str, sample_matrix: &str, days_since_sampling: f64, preserved: bool, temp_c: f64) -> String {
    let mut out = String::from("=== Holding Time & Preservation Checker ===\n");
    out.push_str("Ref: EPA 40 CFR 136 Table II\n\n");

    let (holding_time_days, preservation, method) = match parameter.to_lowercase().as_str() {
        "bod" | "bod5" => (2.0, "Cool 4C, dark", "BOD5"),
        "cod" => (28.0, "H2SO4 to pH<2, cool 4C", "COD"),
        "toc" => (28.0, "H2SO4 to pH<2, cool 4C", "TOC"),
        "nh3" | "nh3-n" | "ammonia" => (28.0, "H2SO4 to pH<2, cool 4C", "Ammonia"),
        "no3" | "no3-n" | "nitrate" => (28.0, "H2SO4 to pH<2, cool 4C", "Nitrate"),
        "tkn" => (28.0, "H2SO4 to pH<2, cool 4C", "TKN"),
        "total_p" | "tp" | "phosphorus" => (28.0, "H2SO4 to pH<2, cool 4C", "Total P"),
        "metals" | "metal" | "heavy_metals" => (180.0, "HNO3 to pH<2", "Metals (dissolved: filter first)"),
        "hg" | "mercury" => (28.0, "HNO3 to pH<2", "Mercury"),
        "cr6" | "cr(vi)" | "hexavalent_chromium" => (1.0, "NaOH, cool 4C", "Cr(VI)"),
        "cyanide" | "cn" => (14.0, "NaOH to pH>12, cool 4C", "Cyanide"),
        "phenol" | "phenols" => (28.0, "H2SO4 to pH<2, cool 4C", "Phenols"),
        "oil_grease" | "og" => (28.0, "HCl to pH<2, cool 4C", "Oil & Grease"),
        "tph" => (28.0, "HCl to pH<2, cool 4C", "TPH"),
        "voc" | "voc_epa_8260" => (14.0, "HCl to pH<2, cool 4C, no headspace", "VOC (8260)"),
        "svoc" | "epa_8270" => (7.0, "Cool 4C, dark", "SVOC (8270)"),
        "pesticides" | "epa_8081" => (7.0, "Cool 4C, dark", "Pesticides"),
        "pcb" | "epa_8082" => (7.0, "Cool 4C, dark", "PCBs"),
        "coliform" | "fecal_coliform" | "e_coli" => (0.13, "Cool 4C, dark (30hr max)", "Coliform"),
        "ph_field" | "ph" => (0.0, "Analyze in field", "pH (field)"),
        "do" | "dissolved_oxygen" => (0.0, "Analyze in field", "DO (field)"),
        "residual_cl2" | "chlorine_residual" => (0.0, "Analyze in field", "Residual Cl2"),
        "tss" | "total_suspended_solids" => (7.0, "Cool 4C, dark", "TSS"),
        "tds" | "total_dissolved_solids" => (7.0, "Cool 4C, dark", "TDS"),
        _ => (7.0, "Cool 4C, dark (default)", "Unknown - verify method"),
    };

    let expired = days_since_sampling > holding_time_days && holding_time_days > 0.0;
    let matrix_note = if sample_matrix.to_lowercase().contains("soil") || sample_matrix.to_lowercase().contains("solid") {
        " (soil/solid: holding times may differ - check method)"
    } else { "" };

    out.push_str(&format!("Parameter: {} ({})\n", parameter, method));
    out.push_str(&format!("Matrix: {}{}\n", sample_matrix, matrix_note));
    out.push_str(&format!("Days since sampling: {:.1}\n", days_since_sampling));
    out.push_str(&format!("Preserved: {}, Temp: {:.1}C\n\n", if preserved { "Yes" } else { "No" }, temp_c));

    out.push_str("-- EPA Requirements --\n\n");
    out.push_str(&format!("  Holding time: {:.0} days ({:.0} hours)\n", holding_time_days, holding_time_days*24.0));
    out.push_str(&format!("  Preservation: {}\n", preservation));
    out.push_str(&format!("  Storage: cool 4C, dark (unless noted)\n\n"));

    if holding_time_days == 0.0 {
        out.push_str("  [CRITICAL] Must be analyzed in field — no holding time!\n");
    } else if expired {
        out.push_str(&format!("  [REJECT] Sample EXPIRED ({:.1} > {:.0} days). Results invalid.\n", days_since_sampling, holding_time_days));
        out.push_str("  Re-sampling required.\n");
    } else {
        let remaining = holding_time_days - days_since_sampling;
        out.push_str(&format!("  [OK] Within holding time ({:.1} days remaining)\n", remaining));
    }

    if !preserved && holding_time_days > 0.0 {
        out.push_str("  [WARN] Not preserved — holding time significantly reduced\n");
    }
    if temp_c > 6.0 && holding_time_days > 0.0 {
        out.push_str("  [WARN] Storage temp > 6C — bacterial growth may alter results\n");
    }

    out.push_str("\n  Ref: EPA 40 CFR 136 Table II; Standard Methods 23rd Ed\n");
    out
}
