/// First-Order Kinetics: C(t) = C₀ × exp(-k×t)
/// Half-life t½ = ln(2)/k
/// Ref: Tchobanoglous et al. (2003), Environmental Engineering

pub fn calculate(c0: f64, k: f64, t: f64, time_unit: &str) -> String {
    let mut out = String::from("=== Kinetika Orde Pertama ===\n");
    out.push_str("Ref: Tchobanoglous et al. (2003)\n\n");

    if c0 <= 0.0 { return "ERROR: Konsentrasi awal (C₀) harus > 0.".into(); }
    if k <= 0.0 { return "ERROR: Konstanta laju (k) harus > 0.".into(); }
    if t < 0.0 { return "ERROR: Waktu (t) tidak boleh negatif.".into(); }

    let unit = match time_unit.to_lowercase().as_str() {
        "s" | "detik" => "detik",
        "min" | "menit" => "menit",
        "hr" | "jam" => "jam",
        "day" | "hari" => "hari",
        _ => return format!("ERROR: Satuan waktu '{}' tidak dikenali. Pilihan: s, min, hr, day.", time_unit),
    };

    // C(t)
    let ct = c0 * (-k * t).exp();

    // Half-life
    let t_half = (2.0_f64).ln() / k;

    // 90% removal (t90): C(t90) = 0.1 × C₀ → t90 = ln(10)/k
    let t90 = (10.0_f64).ln() / k;

    // 99% removal (t99): C(t99) = 0.01 × C₀ → t99 = ln(100)/k
    let t99 = (100.0_f64).ln() / k;

    // Time to reach a low target
    let removal_pct = (1.0 - ct / c0) * 100.0;

    out.push_str(&format!("Input:\n  C₀ = {:.2} mg/L\n  k = {:.4} /{}\n  t = {:.2} {}\n\n", c0, k, unit, t, unit));

    out.push_str(&format!("C(t) = C₀ × exp(-k×t) = {:.2} × exp(-{:.4} × {:.2})\n", c0, k, t));
    out.push_str(&format!("C({:.2}) = {:.4} mg/L\n", t, ct));
    out.push_str(&format!("Removal = {:.2}%\n\n", removal_pct));

    out.push_str("Waktu karakteristik:\n");
    out.push_str(&format!("  t½ (half-life) = {:.2} {}\n", t_half, unit));
    out.push_str(&format!("  t₉₀ (90% removal) = {:.2} {}\n", t90, unit));
    out.push_str(&format!("  t₉₉ (99% removal) = {:.2} {}\n\n", t99, unit));

    // Decay profile
    out.push_str("Profil peluruhan:\n");
    out.push_str(&format!("  {:>8} | {:>12} | {:>8}\n", "Waktu", "Konsentrasi", "Removal"));
    let steps = [0.0, 0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0];
    for &mult in &steps {
        let ti = t_half * mult;
        let ci = c0 * (-k * ti).exp();
        let rem = (1.0 - ci / c0) * 100.0;
        out.push_str(&format!("  {:>8.2} | {:>12.4} | {:>7.2}%\n", ti, ci, rem));
    }

    out.push_str("\nAplikasi tipikal:\n");
    out.push_str("  BOD decay: k = 0.1-0.4 /hari\n");
    out.push_str("  Patogen die-off: k = 0.5-2.0 /hari\n");
    out.push_str("  Kontaminan degradasi: bervariasi\n");

    out
}
