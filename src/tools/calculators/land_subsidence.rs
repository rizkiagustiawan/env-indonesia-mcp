/// Land Subsidence Calculator (Terzaghi 1D Consolidation)
/// Ref: Terzaghi (1943), relevant untuk Jakarta/Semarang/Pekalongan

pub fn calculate(clay_thickness_m: f64, delta_stress_kpa: f64, cc: f64, e0: f64, sigma0_kpa: f64) -> String {
    if clay_thickness_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if delta_stress_kpa <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if e0 <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if sigma0_kpa <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    // Terzaghi 1D primary consolidation
    let settlement_m = (cc * clay_thickness_m / (1.0 + e0)) * ((sigma0_kpa + delta_stress_kpa) / sigma0_kpa).log10();
    let settlement_cm = settlement_m * 100.0;

    let mut out = format!("=== Land Subsidence (Terzaghi 1D) ===\nRef: Terzaghi (1943)\n\nH (tebal lempung) = {:.1} m\nΔσ (tambahan tegangan) = {:.1} kPa\nCc (compression index) = {:.3}\ne0 (angka pori) = {:.2}\nσ'0 (overburden) = {:.1} kPa\n\n", clay_thickness_m, delta_stress_kpa, cc, e0, sigma0_kpa);
    out.push_str(&format!("Sc = Cc×H/(1+e0) × log[(σ'0+Δσ)/σ'0]\nSc = {:.4} m = {:.1} cm\n\n", settlement_m, settlement_cm));
    if settlement_cm > 10.0 { out.push_str("⚠️ Penurunan > 10 cm: Risiko kerusakan infrastruktur.\n"); }
    if settlement_cm > 50.0 { out.push_str("⛔ Penurunan > 50 cm: Zona KRITIS (Jakarta-level subsidence).\n"); }
    out
}
