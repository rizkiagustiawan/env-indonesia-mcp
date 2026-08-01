/// SCS-CN Runoff Calculator (USDA TR-55, 1986)
/// Q = (P - 0.2S)² / (P + 0.8S), S = (25400/CN) - 254

pub fn calculate(p_mm: f64, cn: f64) -> String {
    let mut out = String::from("=== SCS-CN Runoff Calculator ===\n");
    out.push_str("Ref: USDA TR-55 (1986)\n\n");

    if cn < 0.0 || cn > 100.0 {
        return format!("ERROR: CN ({}) harus 0-100.", cn);
    }
    if p_mm < 0.0 {
        return format!("ERROR [E102]: Parameter tidak boleh negatif. {}", p_mm);
    }
    if cn == 0.0 {
        return "CN=0: Infiltrasi sempurna. Q=0 mm.".into();
    }

    let s = (25400.0 / cn) - 254.0;
    let ia = 0.2 * s;

    let q = if p_mm <= ia {
        0.0
    } else {
        (p_mm - ia).powi(2) / (p_mm + 0.8 * s)
    };

    out.push_str(&format!(
        "Input:\n  P (curah hujan) = {:.1} mm\n  CN = {:.0}\n\n",
        p_mm, cn
    ));
    out.push_str(&format!(
        "Perhitungan:\n  S = (25400/CN) - 254 = {:.2} mm\n  Ia = 0.2 × S = {:.2} mm\n",
        s, ia
    ));
    out.push_str(&format!("  Q = (P - Ia)² / (P + 0.8S) = {:.2} mm\n\n", q));
    out.push_str(&format!(
        "Koefisien Limpasan (C) = Q/P = {:.3}\n",
        if p_mm > 0.0 { q / p_mm } else { 0.0 }
    ));
    out.push_str(&format!("Volume Infiltrasi = P - Q = {:.2} mm\n", p_mm - q));
    out
}
