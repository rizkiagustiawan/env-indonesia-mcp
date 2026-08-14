/// Baku Mutu Udara Ambien Nasional
/// Ref: PP 22/2021 Lampiran VII (menggantikan PP 41/1999)

pub fn check(parameter: &str, concentration: f64, averaging_time: &str) -> String {
    let p = parameter.to_uppercase();
    let t = averaging_time.to_lowercase();

    // PP 22/2021 limits in µg/m³
    let limit: Option<f64> = match (p.as_str(), t.as_str()) {
        // SO2
        ("SO2", "1_hour" | "1h") => Some(150.0),
        ("SO2", "24_hour" | "24h") => Some(75.0),
        ("SO2", "annual" | "1_year" | "tahunan") => Some(45.0),
        // CO (µg/m³)
        ("CO", "1_hour" | "1h") => Some(10000.0),
        ("CO", "8_hour" | "8h") => Some(4000.0),
        // NO2
        ("NO2", "1_hour" | "1h") => Some(200.0),
        ("NO2", "24_hour" | "24h") => Some(65.0),
        ("NO2", "annual" | "1_year" | "tahunan") => Some(30.0),
        // O3
        ("O3", "1_hour" | "1h") => Some(150.0),
        ("O3", "8_hour" | "8h") => Some(100.0),
        ("O3", "annual" | "1_year" | "tahunan") => Some(35.0), // Some regions specify this, default PP22/2021
        // PM10
        ("PM10", "24_hour" | "24h") => Some(75.0),
        ("PM10", "annual" | "1_year" | "tahunan") => Some(40.0),
        // PM2.5
        ("PM2.5" | "PM25", "24_hour" | "24h") => Some(55.0),
        ("PM2.5" | "PM25", "annual" | "1_year" | "tahunan") => Some(15.0),
        // Pb (Timbal)
        ("PB", "24_hour" | "24h") => Some(2.0),
        // TSP (Debu)
        ("TSP", "24_hour" | "24h") => Some(230.0),
        // NMHC (Non Methane Hydrocarbons)
        ("NMHC" | "HC", "3_hour" | "3h") => Some(160.0),
        _ => None,
    };

    if concentration < 0.0 {
        return format!(
            "ERROR [E102]: Parameter tidak boleh negatif. {}",
            concentration
        );
    }

    let limit = match limit {
        Some(v) => v,
        None => return format!(
            "ERROR: Kombinasi parameter '{}' dan waktu pengukuran '{}' tidak ditemukan dalam PP 22/2021.\n\
             Parameter valid: SO2, CO, NO2, O3, PM10, PM2.5, Pb, TSP, NMHC\n\
             Waktu valid: 1_hour, 3_hour, 8_hour, 24_hour, annual",
            parameter, averaging_time
        ),
    };

    let pct = (concentration / limit) * 100.0;
    let status = if concentration <= limit {
        "Memenuhi Baku Mutu ✅"
    } else {
        "Melebihi Baku Mutu ❌"
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Udara Ambien\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PP No. 22 Tahun 2021 Lampiran VII\n\n");
    out.push_str(&format!("Parameter   : {}\n", parameter));
    out.push_str(&format!("Waktu Ukur  : {}\n", averaging_time));
    out.push_str(&format!("Konsentrasi : {:.2} µg/m³\n", concentration));
    out.push_str(&format!("Baku Mutu   : {:.2} µg/m³\n", limit));
    out.push_str(&format!("Persentase  : {:.1}% dari baku mutu\n\n", pct));
    out.push_str(&format!("Status: {}\n", status));

    out.push_str(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Catatan Regulasi:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    out.push_str("1. Nilai baku mutu mengacu pada PP 22/2021 yang telah mencabut PP 41/1999.\n");
    out.push_str("   Batas PP 22/2021 jauh lebih ketat (cth: PM2.5 turun dari 65 ke 55 µg/m³).\n");
    out.push_str("2. Permenkes No. 2 Tahun 2023 menetapkan nilai baku mutu\n");
    out.push_str("   untuk kualitas udara DALAM RUANGAN (indoor).\n");
    
    out
}
