/// Ocean Acidification Calculator
/// Ref: Zeebe & Wolf-Gladrow (2001), Millero (2007)

pub fn calculate(ph: f64, pco2_uatm: f64, temp_c: f64, salinity_psu: f64) -> String {
    if ph < 7.0 || ph > 9.0 { return format!("ERROR: pH laut {} di luar rentang normal (7.0-9.0).", ph); }
    if pco2_uatm <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    // Simplified aragonite saturation state
    let ca_mmol = 10.3; // typical seawater Ca²⁺
    let co3_umol = 200.0 * 10.0_f64.powf(ph - 8.1); // very simplified
    let ksp_arag = 6.65e-7 * (1.0 + 0.02 * (temp_c - 25.0)); // temp corrected
    let omega_arag = (ca_mmol * 1e-3 * co3_umol * 1e-6) / ksp_arag;

    let status = if omega_arag > 3.5 { "Sehat (supersaturated)" } else if omega_arag > 1.0 { "Marginal (coral stress mulai)" } else { "KRITIS (undersaturated — coral dissolution)" };

    let mut out = format!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Ocean Acidification\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nRef: Zeebe & Wolf-Gladrow (2001)\n⚠️ Model DISEDERHANAKAN. Untuk akurasi riset gunakan CO2SYS package.\n\npH = {:.2}\npCO₂ = {:.0} µatm\nSuhu = {:.1}°C\nSalinitas = {:.1} PSU\n\nΩ aragonite ≈ {:.2}\nStatus: {}\n\n", ph, pco2_uatm, temp_c, salinity_psu, omega_arag, status);
    out.push_str("Pre-industrial pH: ~8.18, Current: ~8.05\nΩ < 1: Coral TIDAK bisa membentuk cangkang CaCO₃\nΩ > 3.5: Optimal untuk pertumbuhan coral\n");
    if omega_arag < 3.5 { out.push_str("\n⚠️ Coral reef Indonesia (termasuk terumbu karang Indonesia) terancam.\n"); }
    out
}
