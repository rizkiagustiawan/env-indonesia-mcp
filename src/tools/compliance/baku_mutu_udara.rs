/// Baku Mutu Udara Ambien
/// Ref: PP 41/1999

pub fn check(parameter: &str, concentration: f64, averaging_time: &str) -> String {
    let p = parameter.to_uppercase();
    let t = averaging_time.to_lowercase();

    // PP 41/1999 limits in µg/m³ (except opacity in %)
    let limit: Option<f64> = match (p.as_str(), t.as_str()) {
        // SO2
        ("SO2", "1_hour" | "1h")   => Some(900.0),
        ("SO2", "24_hour" | "24h") => Some(365.0),
        ("SO2", "annual" | "1_year" | "tahunan") => Some(60.0),
        // CO (µg/m³)
        ("CO", "1_hour" | "1h")   => Some(30000.0),
        ("CO", "8_hour" | "8h")   => Some(10000.0),
        // NO2
        ("NO2", "1_hour" | "1h")   => Some(400.0),
        ("NO2", "24_hour" | "24h") => Some(150.0),
        ("NO2", "annual" | "1_year" | "tahunan") => Some(100.0),
        // O3
        ("O3", "1_hour" | "1h")   => Some(235.0),
        ("O3", "annual" | "1_year" | "tahunan") => Some(50.0),
        // PM10
        ("PM10", "24_hour" | "24h") => Some(150.0),
        ("PM10", "annual" | "1_year" | "tahunan") => Some(50.0),
        // PM2.5
        ("PM2.5" | "PM25", "24_hour" | "24h") => Some(65.0),
        ("PM2.5" | "PM25", "annual" | "1_year" | "tahunan") => Some(15.0),
        // Pb (Timbal)
        ("PB", "24_hour" | "24h") => Some(2.0),
        // TSP (Debu)
        ("TSP", "24_hour" | "24h") => Some(230.0),
        ("TSP", "annual" | "1_year" | "tahunan") => Some(90.0),
        // HC (Hidrokarbon)
        ("HC", "3_hour" | "3h") => Some(160.0),
        _ => None,
    };

    if concentration < 0.0 {
        return format!("ERROR: Konsentrasi ({}) tidak boleh negatif.", concentration);
    }

    let limit = match limit {
        Some(v) => v,
        None => return format!(
            "ERROR: Kombinasi parameter '{}' dan waktu pengukuran '{}' tidak ditemukan dalam PP 41/1999.\n\
             Parameter valid: SO2, CO, NO2, O3, PM10, PM2.5, Pb, TSP, HC\n\
             Waktu valid: 1_hour, 3_hour, 8_hour, 24_hour, annual",
            parameter, averaging_time
        ),
    };

    let pct = (concentration / limit) * 100.0;
    let status = if concentration <= limit { "Memenuhi Baku Mutu ✅" } else { "Melebihi Baku Mutu ❌" };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Udara Ambien\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PP No. 41 Tahun 1999\n\n");
    out.push_str(&format!("Parameter   : {}\n", parameter));
    out.push_str(&format!("Waktu Ukur  : {}\n", averaging_time));
    out.push_str(&format!("Konsentrasi : {:.2} µg/m³\n", concentration));
    out.push_str(&format!("Baku Mutu   : {:.2} µg/m³\n", limit));
    out.push_str(&format!("Persentase  : {:.1}% dari baku mutu\n\n", pct));
    out.push_str(&format!("Status: {}\n", status));

    out.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Catatan Regulasi:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("1. Nilai baku mutu di atas mengacu PP 41/1999 yang masih berlaku\n");
    out.push_str("   secara hukum untuk udara ambien (outdoor).\n");
    out.push_str("2. PP 22/2021 adalah regulasi perlindungan lingkungan hidup\n");
    out.push_str("   secara umum dan TIDAK menggantikan nilai baku mutu udara\n");
    out.push_str("   ambien PP 41/1999.\n");
    out.push_str("3. Permenkes No. 2 Tahun 2023 menetapkan nilai baku mutu\n");
    out.push_str("   berbeda (lebih ketat) untuk konteks kualitas udara DALAM\n");
    out.push_str("   RUANGAN (indoor). Jangan gunakan nilai PP 41/1999 untuk\n");
    out.push_str("   penilaian kualitas udara dalam ruangan.\n");
    out.push_str("4. Untuk Analisis Risiko Kesehatan Lingkungan (ARKL), gunakan\n");
    out.push_str("   acuan Pedoman ARKL Kemenkes Tahun 2012.\n");

    out
}
