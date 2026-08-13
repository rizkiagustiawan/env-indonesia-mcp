/// Struvite Precipitation Design
/// Ref: Bhuiyan et al. 2007, 2008; Doyle & Parsons 2002; Wu et al. 2022
pub fn assess(
    mg_mg_l: f64, nh4_mg_l: f64, po4_mg_l: f64,
    ph: f64, temp_c: f64,
) -> String {
    let mut out = String::from("=== Struvite Precipitation ===\n");
    out.push_str("Ref: Bhuiyan et al. 2007; Doyle & Parsons 2002\n\n");

    // Ksp struvite = 10^-13.26 (pKsp = 13.26)
    let pksp = 13.26;
    let ksp = 10.0_f64.powf(-pksp);

    // Convert mg/L to mol/L
    let mg_mol = mg_mg_l / 24.31 / 1000.0; // Mg MW=24.31
    let nh4_mol = nh4_mg_l / 18.04 / 1000.0; // NH4 MW=18.04
    let po4_mol = po4_mg_l / 94.97 / 1000.0; // PO4 MW=94.97

    // IAP (Ion Activity Product) with pH speciation:
    // Ksp = 10^-13.26 is activity-based for Mg²⁺ + NH4⁺ + PO4³⁻.
    // Must use free-ion fractions, not total concentrations (PO4 is mostly HPO4²⁻ at pH 7-9).
    let h = 10.0_f64.powf(-ph);
    let ka1 = 10.0_f64.powf(-2.15);
    let ka2 = 10.0_f64.powf(-7.20);
    let ka3 = 10.0_f64.powf(-12.35);
    let h2 = h * h;
    let frac_po4 = ka1 * ka2 * ka3 / (h2 * h + ka1 * h2 + ka1 * ka2 * h + ka1 * ka2 * ka3);
    let ka_nh4 = 10.0_f64.powf(-9.25);
    let frac_nh4 = h / (h + ka_nh4);
    let iap = mg_mol * (nh4_mol * frac_nh4) * (po4_mol * frac_po4);

    // Supersaturation ratio
    let omega = iap / ksp;

    out.push_str(&format!("Mg: {:.1} mg/L ({:.4e} M)\n", mg_mg_l, mg_mol));
    out.push_str(&format!("NH4: {:.1} mg/L ({:.4e} M)\n", nh4_mg_l, nh4_mol));
    out.push_str(&format!("PO4: {:.1} mg/L ({:.4e} M)\n", po4_mg_l, po4_mol));
    out.push_str(&format!("pH: {:.1}, Temp: {:.1}C\n\n", ph, temp_c));

    out.push_str("-- Supersaturation --\n\n");
    out.push_str(&format!("  Ksp (struvite): {:.2e}\n", ksp));
    out.push_str(&format!("  IAP: {:.4e}\n", iap));
    out.push_str(&format!("  >> Omega (IAP/Ksp): {:.2}\n\n", omega));

    if omega > 1.0 {
        out.push_str("  [OK] Supersaturated (Omega > 1) — precipitation will occur\n");
    } else {
        out.push_str("  [WARN] Undersaturated — no spontaneous precipitation\n");
        out.push_str("  Add Mg or raise pH to achieve supersaturation\n");
    }

    // Optimal conditions
    out.push_str("\n-- Optimal Conditions --\n");
    out.push_str("  pH: 8.5-9.5 (higher pH = more NH3, less competition)\n");
    out.push_str("  Mg:N:P = 1:1:1 (stoichiometric). Excess Mg improves recovery.\n");
    out.push_str("  Common Mg sources: MgCl2 (47% Mg), MgO (60% Mg), bittern\n\n");

    // Recovery potential
    let limiting = mg_mol.min(nh4_mol).min(po4_mol);
    let struvite_mg_l = limiting * 245.4 * 1000.0; // MgNH4PO4·6H2O MW=245.4
    out.push_str(&format!("  >> Max struvite recoverable: {:.1} mg/L ({:.2} kg/m3)\n", struvite_mg_l, struvite_mg_l/1000.0));

    out.push_str("\n  Ref: Bhuiyan et al. 2007; Doyle & Parsons 2002; Wu et al. 2022\n");
    out
}

#[cfg(test)]
mod tests {
    use super::assess;

    #[test]
    fn supersaturation_depends_on_ph() {
        // 1 mM each Mg/NH4/PO4. pH 7: PO4 mostly HPO4²⁻ → undersaturated;
        // pH 10: PO4³⁻ fraction much higher → supersaturated. Must be pH-dependent.
        let low = assess(24.31, 18.04, 94.97, 7.0, 25.0);
        let high = assess(24.31, 18.04, 94.97, 10.0, 25.0);
        assert!(!low.contains("Supersaturated"), "pH 7 should be undersaturated:\n{low}");
        assert!(high.contains("Supersaturated"), "pH 10 should be supersaturated:\n{high}");
    }
}
