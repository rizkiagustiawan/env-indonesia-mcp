/// Buffer Capacity — Carbonate System
/// Ref: Stumm & Morgan 1996; Morel & Hering 1993
pub fn assess(alkalinity_mg_l_caco3: f64, ph: f64, temp_c: f64) -> String {
    let mut out = String::from("=== Buffer Capacity (Carbonate System) ===\n");
    out.push_str("Ref: Stumm & Morgan 1996; Morel & Hering 1993\n\n");

    let alk = alkalinity_mg_l_caco3 / 50000.0; // eq/L (MW CaCO3=100, valence=2)
    let kw = 1e-14; // water dissociation
    let ka1 = 4.45e-7 * 10.0_f64.powf(-0.002 * (temp_c - 25.0)); // H2CO3
    let ka2 = 4.69e-11 * 10.0_f64.powf(-0.002 * (temp_c - 25.0)); // HCO3
    let h = 10.0_f64.powf(-ph);
    let oh = kw / h;

    // Species distribution
    let alpha0 = h.powi(2) / (h.powi(2) + ka1 * h + ka1 * ka2); // H2CO3
    let alpha1 = ka1 * h / (h.powi(2) + ka1 * h + ka1 * ka2);   // HCO3
    let alpha2 = ka1 * ka2 / (h.powi(2) + ka1 * h + ka1 * ka2); // CO3

    let ct = alk / (alpha1 + 2.0 * alpha2).max(1e-15); // total carbonate

    // Buffer intensity (Stumm & Morgan)
    let beta = 2.303 * (h + oh + ct * (alpha0 * alpha1 + 4.0 * alpha1 * alpha2 + alpha0 * alpha2));

    out.push_str(&format!("Alkalinity: {:.1} mg/L CaCO3 ({:.4} eq/L)\n", alkalinity_mg_l_caco3, alk));
    out.push_str(&format!("pH: {:.1}, Temp: {:.0}C\n\n", ph, temp_c));
    out.push_str("-- Carbonate Speciation --\n\n");
    out.push_str(&format!("  H2CO3 (alpha0): {:.4} ({:.1}%)\n", alpha0, alpha0*100.0));
    out.push_str(&format!("  HCO3- (alpha1): {:.4} ({:.1}%)\n", alpha1, alpha1*100.0));
    out.push_str(&format!("  CO3^2- (alpha2): {:.4} ({:.1}%)\n", alpha2, alpha2*100.0));
    out.push_str(&format!("  Total CT: {:.4} mol/L\n\n", ct));
    out.push_str(&format!("  >> Buffer intensity beta: {:.4} mol/L/pH unit\n\n", beta));

    if beta > 0.001 { out.push_str("  [OK] Well buffered (>1e-3)\n"); }
    else if beta > 0.0001 { out.push_str("  Moderate buffering\n"); }
    else { out.push_str("  [WARN] Poorly buffered — pH sensitive to acid input\n"); }

    out.push_str("\n  Ref: Stumm & Morgan 1996; Morel & Hering 1993\n");
    out
}
