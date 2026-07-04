/// Acid Mine Drainage (AMD) Calculator
/// Ref: PermenLH 113/2003, Acid Base Accounting (ABA)

pub fn calculate(sulfur_pct: f64, anc_kg_h2so4_t: f64, nag_ph: Option<f64>) -> String {
    if sulfur_pct < 0.0 { return "ERROR: Persentase Sulfur tidak boleh negatif.".into(); }

    let mpa = sulfur_pct * 30.6; // kg H2SO4/ton
    let napp = mpa - anc_kg_h2so4_t;

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Acid Mine Drainage (AMD)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Acid Base Accounting (ABA)\n\n");

    out.push_str(&format!("INPUT:\n  Total Sulfur (S) = {:.2} %\n  ANC = {:.2} kg H₂SO₄/ton\n", sulfur_pct, anc_kg_h2so4_t));
    if let Some(ph) = nag_ph { out.push_str(&format!("  NAG pH = {:.2}\n", ph)); }

    out.push_str(&format!("\nHASIL:\n  MPA (Maximum Potential Acidity) = {:.2} kg H₂SO₄/ton\n  NAPP (Net Acid Producing Potential) = {:.2} kg H₂SO₄/ton\n\n", mpa, napp));

    let status = if napp > 0.0 {
        if let Some(ph) = nag_ph {
            if ph < 4.5 { "PAF (Potentially Acid Forming) - BAHAYA TINGGI" }
            else { "Uncertain (PAF berdasarkan NAPP, tapi NAG pH >= 4.5)" }
        } else {
            "PAF (Potentially Acid Forming)"
        }
    } else if napp < 0.0 {
        if let Some(ph) = nag_ph {
            if ph >= 4.5 { "NAF (Non-Acid Forming) - AMAN" }
            else { "Uncertain (NAF berdasarkan NAPP, tapi NAG pH < 4.5)" }
        } else {
            "NAF (Non-Acid Forming)"
        }
    } else {
        "Uncertain (Perlu uji kinetik)"
    };

    out.push_str(&format!("Klasifikasi: {}\n", status));

    if napp > 0.0 {
        out.push_str("\n⚠️ REKOMENDASI MITIGASI:\n  1. Enkapsulasi dengan material NAF (dry cover).\n  2. Penempatan di bawah muka air tanah (wet cover).\n  3. Pengolahan aktif dengan kapur (CaCO3) di settling pond.\n");
    }

    out
}
