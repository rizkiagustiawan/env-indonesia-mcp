/// Noise dB Calculator (Logarithmic addition + attenuation)
/// Ref: KepMenLH 48/1996, ISO 9613

pub fn add_sources(levels_db: &[f64]) -> String {
    if levels_db.is_empty() { return "ERROR: Masukkan minimal 1 sumber kebisingan.".into(); }

    let sum: f64 = levels_db.iter().map(|l| 10.0_f64.powf(l / 10.0)).sum();
    let total = 10.0 * sum.log10();

    let mut out = String::from("=== Noise dB Calculator ===\n");
    out.push_str("Ref: KepMenLH 48/1996, penjumlahan logaritmik\n\n");
    out.push_str("Sumber:\n");
    for (i, l) in levels_db.iter().enumerate() {
        out.push_str(&format!("  Sumber {}: {:.1} dB\n", i + 1, l));
    }
    out.push_str(&format!("\nLtotal = 10 × log₁₀(Σ10^(Li/10)) = {:.1} dB\n\n", total));

    let zone_check = |name: &str, limit: f64| -> String {
        if total > limit { format!("  {} ({}dB): MELEBIHI ⚠️\n", name, limit) }
        else { format!("  {} ({}dB): OK ✅\n", name, limit) }
    };
    out.push_str("Baku Mutu KepMenLH 48/1996:\n");
    out.push_str(&zone_check("Perumahan (siang)", 55.0));
    out.push_str(&zone_check("Perumahan (malam)", 45.0));
    out.push_str(&zone_check("Perkantoran", 65.0));
    out.push_str(&zone_check("Industri", 70.0));
    out
}

pub fn attenuation_distance(source_db: f64, distance_m: f64) -> String {
    if distance_m <= 0.0 { return "ERROR: Jarak harus > 0.".into(); }
    // Point source: -6 dB per doubling of distance (inverse square law)
    let ref_dist = 1.0; // 1 meter reference
    let atten = 20.0 * (distance_m / ref_dist).log10();
    let result = source_db - atten;

    format!("=== Atenuasi Kebisingan (Jarak) ===\nSumber: {:.1} dB @ 1m\nJarak: {:.1} m\nAtenuasi: -{:.1} dB (inverse square law)\nHasil: {:.1} dB\n", source_db, distance_m, atten, result.max(0.0))
}
