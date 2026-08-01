/// Baku Mutu Getaran
/// Ref: KepmenLH 49/1996

pub fn check(zone: &str, vibration_mm_s: f64) -> String {
    let z = zone.to_lowercase();

    if vibration_mm_s < 0.0 {
        return format!(
            "ERROR [E102]: Parameter tidak boleh negatif. {}",
            vibration_mm_s
        );
    }

    // KepmenLH 49/1996 limits in mm/s
    let (limit, zone_desc): (f64, &str) = match z.as_str() {
        "pemukiman" | "perumahan" | "residential" => (2.0, "Pemukiman / Perumahan"),
        "kantor" | "perkantoran" | "office" => (5.0, "Perkantoran"),
        "industri" | "industrial" => (10.0, "Kawasan Industri"),
        "rumah_sakit" | "hospital" => (1.0, "Rumah Sakit & Fasilitas Kesehatan"),
        _ => {
            return format!(
                "ERROR: Zona '{}' tidak ditemukan dalam KepmenLH 49/1996.\n\
             Zona valid: pemukiman, kantor, industri, rumah_sakit",
                zone
            )
        }
    };

    let pct = (vibration_mm_s / limit) * 100.0;
    let status = if vibration_mm_s <= limit {
        "Memenuhi Baku Mutu ✅"
    } else {
        "Melebihi Baku Mutu ❌"
    };

    let mut out = String::from(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Getaran\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    out.push_str("Ref: KepmenLH No. 49 Tahun 1996\n\n");
    out.push_str(&format!("Zona        : {} ({})\n", zone, zone_desc));
    out.push_str(&format!("Terukur     : {:.2} mm/s\n", vibration_mm_s));
    out.push_str(&format!("Baku Mutu   : {:.2} mm/s\n", limit));
    out.push_str(&format!("Persentase  : {:.1}% dari baku mutu\n\n", pct));
    out.push_str(&format!("Status: {}\n\n", status));
    out.push_str("Daftar Baku Mutu Getaran (KepmenLH 49/1996):\n");
    out.push_str("  Pemukiman    : 2.0 mm/s\n");
    out.push_str("  Kantor       : 5.0 mm/s\n");
    out.push_str("  Industri     : 10.0 mm/s\n");
    out.push_str("  Rumah Sakit  : 1.0 mm/s\n");
    out
}
