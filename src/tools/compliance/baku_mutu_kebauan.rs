/// Baku Mutu Kebauan
/// Ref: KepmenLH 50/1996

pub fn check(chemical: &str, concentration_ppm: f64) -> String {
    let c = chemical.to_lowercase();

    // Intensity scale mode
    if c == "skala" || c == "intensitas" || c == "intensity" {
        let skala = concentration_ppm as i32;
        if skala < 0 || skala > 5 {
            return format!("ERROR: Skala intensitas bau harus 0-5, diberikan: {}", skala);
        }
        let (label, status) = match skala {
            0 => ("Tidak Berbau", "Baik ✅"),
            1 => ("Sangat Lemah", "Baik ✅"),
            2 => ("Lemah / Sedang", "Sedang ⚠️"),
            3 => ("Kuat", "Melebihi Baku Mutu ❌"),
            4 => ("Sangat Kuat", "Melebihi Baku Mutu ❌"),
            5 => ("Sangat Kuat Sekali", "Melebihi Baku Mutu ❌"),
            _ => ("", ""),
        };

        let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Kebauan (Skala Intensitas)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        out.push_str("Ref: KepmenLH No. 50 Tahun 1996\n\n");
        out.push_str(&format!("Skala Intensitas: {} — {}\n\n", skala, label));
        out.push_str(&format!("Status: {}\n\n", status));
        out.push_str("Skala Intensitas Bau:\n");
        out.push_str("  0 = Tidak berbau\n");
        out.push_str("  1 = Sangat lemah (ambang deteksi)\n");
        out.push_str("  2 = Lemah (bau terdeteksi)\n");
        out.push_str("  3 = Kuat (bau mudah terdeteksi)\n");
        out.push_str("  4 = Sangat kuat (bau menyengat)\n");
        out.push_str("  5 = Sangat kuat sekali (tidak tertahankan)\n");
        out.push_str("\nBatas: Skala 0-1 (Baik), 2 (Sedang), 3-5 (Melebihi)\n");
        return out;
    }

    if concentration_ppm < 0.0 {
        return format!("ERROR: Konsentrasi ({:.4} ppm) tidak boleh negatif.", concentration_ppm);
    }

    // Chemical-specific limits in ppm
    let (limit, chem_name): (f64, &str) = match c.as_str() {
        "h2s" | "hydrogen_sulfide" => (0.02, "Hidrogen Sulfida (H₂S)"),
        "nh3" | "ammonia" | "amonia" => (2.0, "Amonia (NH₃)"),
        "methyl_mercaptan" | "ch3sh" | "metil_merkaptan" => (0.002, "Metil Merkaptan (CH₃SH)"),
        "styrene" | "stirena" => (0.1, "Stirena (C₈H₈)"),
        _ => return format!(
            "ERROR: Senyawa '{}' tidak ditemukan dalam KepmenLH 50/1996.\n\
             Senyawa valid: H2S, NH3, methyl_mercaptan, styrene\n\
             Untuk pengukuran intensitas bau, gunakan chemical='skala' dengan nilai 0-5.",
            chemical
        ),
    };

    let pct = (concentration_ppm / limit) * 100.0;
    let status = if concentration_ppm <= limit { "Memenuhi Baku Mutu ✅" } else { "Melebihi Baku Mutu ❌" };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Kebauan\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: KepmenLH No. 50 Tahun 1996\n\n");
    out.push_str(&format!("Senyawa     : {}\n", chem_name));
    out.push_str(&format!("Konsentrasi : {:.4} ppm\n", concentration_ppm));
    out.push_str(&format!("Baku Mutu   : {:.4} ppm\n", limit));
    out.push_str(&format!("Persentase  : {:.1}% dari baku mutu\n\n", pct));
    out.push_str(&format!("Status: {}\n\n", status));
    out.push_str("Daftar Baku Mutu Kebauan (KepmenLH 50/1996):\n");
    out.push_str("  H₂S             : 0.02 ppm\n");
    out.push_str("  NH₃              : 2.0 ppm\n");
    out.push_str("  Metil Merkaptan  : 0.002 ppm\n");
    out.push_str("  Stirena          : 0.1 ppm\n");
    out
}
