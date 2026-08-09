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

    // IAP (Ion Activity Product)
    let iap = mg_mol * nh4_mol * po4_mol;

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
