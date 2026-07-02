/// Water Balance Calculator
/// P = ET + Q + ΔS (Konservasi Massa Air)

pub fn calculate(p_mm: f64, et_mm: f64, q_mm: f64) -> String {
    let delta_s = p_mm - et_mm - q_mm;
    let mut out = String::from("=== Water Balance (Neraca Air) ===\n");
    out.push_str("Ref: Thornthwaite & Mather (1955)\n");
    out.push_str("Hukum: P = ET + Q + ΔS (Konservasi Massa)\n\n");
    out.push_str(&format!("Input:\n  P (presipitasi) = {:.1} mm\n  ET (evapotranspirasi) = {:.1} mm\n  Q (limpasan/runoff) = {:.1} mm\n\n", p_mm, et_mm, q_mm));
    out.push_str(&format!("ΔS (perubahan simpanan) = P - ET - Q = {:.1} mm\n\n", delta_s));

    if delta_s > 0.0 {
        out.push_str("ΔS > 0: SURPLUS — air tanah terisi, muka air tanah naik.\n");
    } else if delta_s < -10.0 {
        out.push_str("ΔS << 0: DEFISIT BESAR — cadangan air tanah menipis. Potensi kekeringan.\n");
    } else if delta_s < 0.0 {
        out.push_str("ΔS < 0: DEFISIT RINGAN — air tanah menurun.\n");
    } else {
        out.push_str("ΔS ≈ 0: SEIMBANG.\n");
    }

    // Validasi fisika
    if et_mm < 0.0 { out.push_str("\n⛔ ERROR FISIKA: ET tidak boleh negatif.\n"); }
    if q_mm < 0.0 { out.push_str("\n⛔ ERROR FISIKA: Runoff tidak boleh negatif.\n"); }
    if q_mm > p_mm { out.push_str("\n⛔ ERROR FISIKA: Runoff melebihi presipitasi. Melanggar konservasi massa.\n"); }
    out
}
