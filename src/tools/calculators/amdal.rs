/// Leopold Matrix for AMDAL / EIA
/// Ref: Leopold et al. (1971) USGS Circular 645

pub fn score(impacts: &[(String, String, i32, u32)]) -> String {
    // impacts: Vec<(kegiatan, komponen_lh, magnitude[-10..10], importance[1..10])>
    let mut out = String::from("=== Leopold Matrix — AMDAL/EIA ===\n");
    out.push_str("Ref: Leopold et al. (1971), PP 22/2021, UU 32/2009\n\n");

    if impacts.is_empty() { return "ERROR: Masukkan minimal 1 dampak (kegiatan, komponen, magnitude, importance).".into(); }

    let mut total_positive = 0i64;
    let mut total_negative = 0i64;
    let mut count_significant = 0;

    out.push_str("MATRIKS DAMPAK:\n");
    out.push_str(&format!("{:<30} {:<25} {:>5} {:>5} {:>7}\n", "Kegiatan", "Komponen LH", "M", "I", "M×I"));
    out.push_str(&"-".repeat(75));
    out.push_str("\n");

    for (kegiatan, komponen, mag, imp) in impacts {
        if *mag < -10 || *mag > 10 { out.push_str(&format!("⚠️ Magnitude {} di luar rentang (-10 s.d. +10)\n", mag)); continue; }
        if *imp < 1 || *imp > 10 { out.push_str(&format!("⚠️ Importance {} di luar rentang (1-10)\n", imp)); continue; }

        let score = (*mag as i64) * (*imp as i64);
        if score > 0 { total_positive += score; } else { total_negative += score; }
        if score.abs() >= 30 { count_significant += 1; }

        let indicator = if score > 0 { "➕" } else if score < 0 { "➖" } else { "⚪" };
        out.push_str(&format!("{:<30} {:<25} {:>5} {:>5} {:>5} {}\n", 
            &kegiatan[..kegiatan.len().min(29)], &komponen[..komponen.len().min(24)], mag, imp, score, indicator));
    }

    out.push_str(&format!("\nRINGKASAN:\n  Dampak positif total: +{}\n  Dampak negatif total: {}\n  Dampak signifikan (|M×I| ≥ 30): {}\n  Net impact: {}\n", 
        total_positive, total_negative, count_significant, total_positive + total_negative));

    if total_positive + total_negative < -50 {
        out.push_str("\n⚠️ Net impact sangat negatif. Proyek ini memerlukan mitigasi serius atau redesain.\n");
    }
    out
}
