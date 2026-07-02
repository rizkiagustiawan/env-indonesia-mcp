/// PROPER Scoring Engine (PermenLHK P.1/2021)
/// Program Penilaian Peringkat Kinerja Perusahaan dalam Pengelolaan LH

pub fn score(has_izin: bool, compliance_pct: f64, beyond_compliance: bool, community_dev: bool, circular_economy: bool) -> String {
    let mut out = String::from("=== PROPER Scoring Engine ===\n");
    out.push_str("Ref: PermenLHK No. P.1/2021\n\n");

    let (color, label, desc) = if !has_izin || compliance_pct < 50.0 {
        ("⬛", "HITAM", "Sengaja melakukan perbuatan atau kelalaian yang mengakibatkan pencemaran/kerusakan LH, serta pelanggaran serius terhadap peraturan.")
    } else if compliance_pct < 100.0 {
        ("🟥", "MERAH", "Upaya pengelolaan LH telah dilakukan tetapi belum sesuai persyaratan sebagaimana diatur dalam peraturan perundangan.")
    } else if compliance_pct >= 100.0 && !beyond_compliance {
        ("🔵", "BIRU", "Telah melakukan upaya pengelolaan LH sesuai dengan persyaratan peraturan perundangan (taat/compliance).")
    } else if beyond_compliance && !circular_economy {
        ("🟢", "HIJAU", "Telah melakukan pengelolaan LH lebih dari yang dipersyaratkan (beyond compliance) melalui implementasi SML, efisiensi energi, 3R, CSR.")
    } else {
        ("🥇", "EMAS", "Telah secara konsisten menunjukkan keunggulan lingkungan (environmental excellency) melalui implementasi produksi bersih, ekonomi sirkular, dan pengembangan masyarakat.")
    };

    out.push_str(&format!("Input:\n  Izin lingkungan: {}\n  Compliance: {:.0}%\n  Beyond compliance: {}\n  Community development: {}\n  Circular economy: {}\n\n", has_izin, compliance_pct, beyond_compliance, community_dev, circular_economy));
    out.push_str(&format!("Peringkat: {} {} — {}\n\n", color, label, desc));
    out.push_str("Skala PROPER:\n  ⬛ HITAM  — Pelanggaran serius\n  🟥 MERAH  — Belum taat\n  🔵 BIRU   — Taat (minimum requirement)\n  🟢 HIJAU  — Beyond compliance\n  🥇 EMAS   — Environmental excellency\n\n");
    out.push_str("Referensi: https://proper.menlhk.go.id/\n");
    out
}
