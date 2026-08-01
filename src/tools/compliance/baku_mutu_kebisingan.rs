/// Baku Mutu Kebisingan
/// Ref: KepmenLH 48/1996

pub fn check(zone: &str, measured_db: f64) -> String {
    let z = zone.to_lowercase();

    if measured_db < 0.0 {
        return format!(
            "ERROR [E102]: Parameter tidak boleh negatif. {}",
            measured_db
        );
    }

    // KepmenLH 48/1996 limits in dBA
    let (limit, zone_desc): (f64, &str) = match z.as_str() {
        "perumahan" | "pemukiman" | "residential" => (55.0, "Perumahan & Pemukiman"),
        "perdagangan" | "komersial" | "commercial" => (70.0, "Perdagangan & Jasa"),
        "perkantoran" | "office" => (65.0, "Perkantoran"),
        "industri" | "industrial" => (70.0, "Kawasan Industri"),
        "rumah_sakit" | "hospital" => (55.0, "Rumah Sakit & Sejenisnya"),
        "sekolah" | "school" => (55.0, "Sekolah & Sejenisnya"),
        "ibadah" | "tempat_ibadah" | "worship" => (55.0, "Tempat Ibadah"),
        "ruang_terbuka_hijau" | "rth" | "taman" => (50.0, "Ruang Terbuka Hijau"),
        _ => {
            return format!(
                "ERROR: Zona '{}' tidak ditemukan dalam KepmenLH 48/1996.\n\
             Zona valid: perumahan, perdagangan, perkantoran, industri, rumah_sakit,\n\
             sekolah, ibadah, ruang_terbuka_hijau",
                zone
            )
        }
    };

    let pct = (measured_db / limit) * 100.0;
    let selisih = measured_db - limit;
    let status = if measured_db <= limit {
        "Memenuhi Baku Mutu ✅"
    } else {
        "Melebihi Baku Mutu ❌"
    };

    let mut out = String::from(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Kebisingan\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    out.push_str("Ref: KepmenLH No. 48 Tahun 1996\n\n");
    out.push_str(&format!("Zona        : {} ({})\n", zone, zone_desc));
    out.push_str(&format!("Terukur     : {:.1} dBA\n", measured_db));
    out.push_str(&format!("Baku Mutu   : {:.1} dBA\n", limit));
    out.push_str(&format!("Persentase  : {:.1}% dari baku mutu\n", pct));
    out.push_str(&format!("Selisih     : {:+.1} dBA\n\n", selisih));
    out.push_str(&format!("Status: {}\n\n", status));
    out.push_str("Daftar Baku Mutu Kebisingan (KepmenLH 48/1996):\n");
    out.push_str("  Perumahan           : 55 dBA\n");
    out.push_str("  Perdagangan & Jasa  : 70 dBA\n");
    out.push_str("  Perkantoran         : 65 dBA\n");
    out.push_str("  Industri            : 70 dBA\n");
    out.push_str("  Rumah Sakit         : 55 dBA\n");
    out.push_str("  Sekolah             : 55 dBA\n");
    out.push_str("  Tempat Ibadah       : 55 dBA\n");
    out.push_str("  Ruang Terbuka Hijau : 50 dBA\n");
    out
}
