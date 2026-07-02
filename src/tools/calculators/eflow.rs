/// Environmental Flow (Tennant Method)
/// Ref: Tennant (1976) Montana Method
/// ⚠️ Hanya screening awal. Tidak valid sebagai rekomendasi final untuk sungai tropis Indonesia.

pub fn calculate(maf_m3s: f64) -> String {
    if maf_m3s <= 0.0 { return "ERROR: Mean Annual Flow harus > 0.".into(); }

    let mut out = String::from("=== Environmental Flow (Tennant Method) ===\n");
    out.push_str("Ref: Tennant (1976)\n");
    out.push_str("⚠️ DISCLAIMER: Metode ini dikembangkan untuk sungai Montana (temperate).\n");
    out.push_str("   Hanya cocok sebagai SCREENING AWAL untuk Indonesia.\n");
    out.push_str("   Untuk rekomendasi final: gunakan DRIFT, BBM, atau ELOHA.\n\n");
    out.push_str(&format!("Mean Annual Flow (MAF) = {:.2} m³/s\n\n", maf_m3s));
    out.push_str("Rekomendasi Environmental Flow:\n");

    let levels = [
        (10.0, "Minimum survival (severely degraded)"),
        (20.0, "Poor / minimum"),
        (30.0, "Good / fair"),
        (40.0, "Good"),
        (50.0, "Excellent"),
        (60.0, "Excellent to outstanding"),
    ];
    for (pct, desc) in &levels {
        out.push_str(&format!("  {:>3.0}% MAF = {:.2} m³/s — {}\n", pct, maf_m3s * pct / 100.0, desc));
    }
    out
}
