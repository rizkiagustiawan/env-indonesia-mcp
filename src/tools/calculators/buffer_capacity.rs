/// Buffer Capacity — Carbonate System
/// Ref: Stumm & Morgan 1996; Morel & Hering 1993
pub fn assess(alkalinity_mg_l_caco3: f64, ph: f64, temp_c: f64) -> String {
    let mut out = String::from("=== Buffer Capacity (Carbonate System) ===\n");
    out.push_str("Ref: Stumm & Morgan 1996; Morel & Hering 1993\n\n");

    let alk = alkalinity_mg_l_caco3 / 50000.0; // eq/L (MW CaCO3=100, valence=2)
    let kw = 1e-14; // water dissociation at 25C (simplified)
    // van't Hoff equation: Ka(T) = Ka(T_ref) * exp(-(dH/R) * (1/T - 1/T_ref))
    // BUG FIX: was 10^(-0.002*dT) crude linearization. Now uses proper thermodynamic dH.
    // dH1 (H2CO3) = -7.3 kJ/mol, dH2 (HCO3-) = -14.6 kJ/mol (Stumm & Morgan Table)
    let t_ref = 298.15_f64; // 25C in Kelvin
    let t_k = temp_c + 273.15;
    let r = 8.314e-3; // kJ/(mol·K)
    let ka1_25 = 4.45e-7_f64;
    let ka2_25 = 4.69e-11_f64;
    let ka1 = ka1_25 * (-(-7.3 / r) * (1.0 / t_k - 1.0 / t_ref)).exp();
    let ka2 = ka2_25 * (-(-14.6 / r) * (1.0 / t_k - 1.0 / t_ref)).exp();
    let h = 10.0_f64.powf(-ph);
    let oh = kw / h;

    // Species distribution
    let alpha0 = h.powi(2) / (h.powi(2) + ka1 * h + ka1 * ka2); // H2CO3
    let alpha1 = ka1 * h / (h.powi(2) + ka1 * h + ka1 * ka2);   // HCO3
    let alpha2 = ka1 * ka2 / (h.powi(2) + ka1 * h + ka1 * ka2); // CO3

    let ct = alk / (alpha1 + 2.0 * alpha2).max(1e-15); // total carbonate

    // Buffer intensity (Stumm & Morgan)
    let beta = 2.303 * (h + oh + ct * (alpha0 * alpha1 + 4.0 * alpha0 * alpha2 + alpha1 * alpha2));

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

#[cfg(test)]
mod tests {
    // Self-check: van't Hoff Ka at 25C must equal Ka(25) reference (factor = exp(0) = 1)
    #[test]
    fn vant_hoff_at_reference_temp() {
        let t_ref = 298.15_f64; let t_k = 298.15_f64; let r = 8.314e-3_f64;
        let factor = (-(-7.3 / r) * (1.0 / t_k - 1.0 / t_ref)).exp();
        assert!((factor - 1.0).abs() < 1e-10, "van't Hoff factor at T_ref must be 1.0");
    }

    // At higher temp (35C), Ka1 should increase slightly (dH negative -> exothermic dissolution
    // actually means Ka decreases with T; but the magnitude should be small, not the old -0.002/dT linear)
    #[test]
    fn vant_hoff_sensible_change() {
        let t_ref = 298.15_f64; let r = 8.314e-3_f64;
        let f35 = (-(-7.3 / r) * (1.0 / 308.15 - 1.0 / t_ref)).exp();
        // Change should be modest (few %), not 10x
        assert!(f35 > 0.5 && f35 < 2.0, "van't Hoff factor at 35C={f35} should be near 1");
    }

    // At pH ~ pKa2 (10.3), alpha2 is large. The 4x coefficient must sit on alpha0*alpha2,
    // NOT alpha1*alpha2. Correct beta ≈ 0.0012; swapped cross-terms give ≈ 0.0036.
    #[test]
    fn buffer_intensity_cross_terms_correct_order() {
        let result = super::assess(100.0, 10.3, 25.0);
        assert!(result.contains("beta: 0.0012"), "buffer intensity cross-terms swapped:\n{result}");
    }
}

