/// IDF Curve (Intensity-Duration-Frequency) — Mononobe + Gumbel
/// Ref: Mononobe (standar Indonesia), Gumbel (Chow 1951)

pub fn mononobe(r24_mm: f64, duration_hours: f64) -> String {
    if r24_mm <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if duration_hours <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    let i = (r24_mm / 24.0) * (24.0 / duration_hours).powf(2.0 / 3.0);

    let mut out = String::from("=== IDF Curve — Metode Mononobe ===\n");
    out.push_str("⚠️ Hanya untuk desain pendahuluan. Untuk desain final: fitting data lokal.\n\n");
    out.push_str(&format!("INPUT:\n  R24 (hujan maks 24 jam) = {:.1} mm\n  Durasi = {:.2} jam\n\n", r24_mm, duration_hours));
    out.push_str(&format!("I = (R24/24) × (24/t)^(2/3) = {:.2} mm/jam\n\n", i));

    out.push_str("Tabel Intensitas untuk berbagai durasi:\n");
    for t in [0.25_f64, 0.5, 1.0, 2.0, 3.0, 6.0, 12.0, 24.0] {
        let it = (r24_mm / 24.0) * (24.0 / t).powf(2.0 / 3.0);
        out.push_str(&format!("  t = {:.2} jam → I = {:.1} mm/jam\n", t, it));
    }
    out
}

pub fn gumbel_return(mean: f64, std_dev: f64, return_period: f64) -> String {
    if return_period <= 1.0 { return "ERROR [E102]: Parameter harus > 1 tahun.".into(); }
    if std_dev <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    let k = -(6.0_f64.sqrt() / std::f64::consts::PI) * (0.5772 + (return_period / (return_period - 1.0)).ln().ln());
    let xt = mean + k * std_dev;

    format!("=== Gumbel Frequency Analysis ===\nMean = {:.1} mm, Std = {:.1} mm\nReturn Period = {:.0} tahun\nK = {:.4}\nX(T) = {:.1} mm\n", mean, std_dev, return_period, k, xt)
}
