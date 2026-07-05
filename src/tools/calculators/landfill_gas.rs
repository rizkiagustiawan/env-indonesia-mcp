/// Landfill Gas (LFG) Estimator — Simplified LandGEM (EPA)
/// L = L0 × k × Mi × e^(-k×t)
/// Ref: EPA AP-42, LandGEM v3.02

pub fn calculate(waste_mass_ton: f64, years_open: u32, k: f64, l0: f64) -> String {
    let mut out = String::from("=== Landfill Gas (CH₄) Estimator ===\n");
    out.push_str("Ref: EPA LandGEM v3.02, AP-42 Chapter 2.4\n\n");

    if waste_mass_ton <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    // Default parameters for tropical wet climate (Indonesia)
    let k_val = if k > 0.0 { k } else { 0.7 }; // /yr, tropical wet (EPA default CAA: 0.05, tropis: 0.7)
    let l0_val = if l0 > 0.0 { l0 } else { 170.0 }; // m³/Mg, EPA default CAA conventional

    let annual_waste = waste_mass_ton / (years_open as f64);

    out.push_str(&format!("Input:\n  Total sampah = {:.0} ton\n  Tahun operasi = {} tahun\n  Sampah/tahun = {:.0} ton/tahun\n  k (decay rate) = {:.2} /tahun\n  L0 (gas potential) = {:.0} m³/Mg\n\n", waste_mass_ton, years_open, annual_waste, k_val, l0_val));

    // Total LFG generation estimate (simplified first-order decay)
    let mut total_ch4 = 0.0;
    out.push_str("Emisi CH₄ per tahun setelah penutupan:\n");
    for t in 1..=10 {
        let ch4 = k_val * l0_val * annual_waste * (-k_val * (t as f64)).exp();
        total_ch4 += ch4;
        out.push_str(&format!("  Tahun +{}: {:.0} m³ CH₄\n", t, ch4));
    }

    // Convert CH4 m³ to CO2e (1 m³ CH4 ≈ 0.717 kg, GWP = 28)
    let ch4_kg = total_ch4 * 0.717;
    let co2e_ton = ch4_kg * 28.0 / 1000.0;

    out.push_str(&format!("\nTotal CH₄ (10 tahun): {:.0} m³ ≈ {:.0} kg\n", total_ch4, ch4_kg));
    out.push_str(&format!("Setara CO₂e (GWP=28): {:.1} ton CO₂e\n", co2e_ton));
    out.push_str(&format!("Nilai karbon (NEK Rp465.000/tCO₂e): Rp {:.0}\n", co2e_ton * 465000.0));
    out
}
