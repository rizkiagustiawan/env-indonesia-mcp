use reqwest::Client;

/// BMKG Historical Climate Data
/// Source: dataonline.bmkg.go.id

/// Station register, shipped in-repo as the single source of truth.
///
/// Previously this module carried its own hardcoded station table, which had
/// drifted from `resources/bmkg_stations_indonesia.json` — Stasiun Meteorologi
/// Bima appeared as `97510` here and `97240` there. Two codes for one station
/// means at least one lookup was wrong, so the table is no longer duplicated.
const STATION_REGISTER: &str = include_str!("../../../resources/bmkg_stations_indonesia.json");

/// Station codes that appeared in the old hardcoded table but are absent from
/// the register. They are NOT silently mapped to a register entry: we cannot
/// verify which code is authoritative without the BMKG station catalogue, and
/// guessing would attach real climate data to the wrong station.
const UNVERIFIED_LEGACY_CODES: &[(&str, &str)] = &[
    ("97510", "Stasiun Meteorologi Bima"),
    ("97120", "Stasiun Meteorologi Selaparang, Mataram"),
    ("97230", "Stasiun Meteorologi Lombok Tengah"),
    ("97330", "Stasiun Klimatologi Kediri, Lombok Barat"),
    ("97410", "Stasiun Meteorologi Sultan M. Kaharuddin, Sumbawa"),
    ("96749", "Stasiun Meteorologi Juanda, Surabaya"),
    ("96839", "Stasiun Meteorologi Ngurah Rai, Bali"),
    ("97072", "Stasiun Meteorologi Soekarno-Hatta, Jakarta"),
];

/// Valid climate parameters
fn valid_parameters() -> Vec<(&'static str, &'static str)> {
    vec![
        ("rainfall", "Curah Hujan (mm)"),
        ("temperature", "Suhu Udara (°C)"),
        ("humidity", "Kelembapan Relatif (%)"),
        ("wind", "Kecepatan Angin (m/s)"),
        ("sunshine", "Lama Penyinaran Matahari (jam)"),
    ]
}

/// Get climate data from BMKG Open Data
pub async fn get_climate_data(client: &Client, station_id: &str, parameter: &str) -> String {
    let params = valid_parameters();
    let param_valid = params.iter().any(|(p, _)| *p == parameter.to_lowercase());
    if !param_valid {
        let available: Vec<String> = params
            .iter()
            .map(|(p, d)| format!("  {} - {}", p, d))
            .collect();
        return format!(
            "Parameter '{}' tidak valid.\nParameter tersedia:\n{}",
            parameter,
            available.join("\n")
        );
    }

    // Map parameter to BMKG API parameter code
    let param_code = match parameter.to_lowercase().as_str() {
        "rainfall" => "RR",
        "temperature" => "TT",
        "humidity" => "RH",
        "wind" => "FF",
        "sunshine" => "SS",
        _ => "RR",
    };

    let url = format!(
        "https://dataonline.bmkg.go.id/akses_data/suhu_kelembapan_angin/qc/{}?parameter={}",
        station_id, param_code
    );

    match client
        .get(&url)
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(body) => {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                            return format_climate_response(&data, station_id, parameter);
                        }
                        return format!(
                            "Data iklim BMKG untuk stasiun '{}':\n{}\nSumber: dataonline.bmkg.go.id",
                            station_id, &body[..body.len().min(2000)]
                        );
                    }
                    Err(e) => {
                        return format!("Error membaca respons BMKG: {}", e);
                    }
                }
            }
        }
        Err(_) => {}
    }

    // Fallback: provide reference station information
    let station_info = get_station_info(station_id);
    let param_desc = params
        .iter()
        .find(|(p, _)| *p == parameter)
        .map(|(_, d)| *d)
        .unwrap_or("N/A");

    format!(
        "══════════════════════════════════════════════\n\
         BMKG - DATA IKLIM HISTORIS\n\
         Stasiun: {}\n\
         Parameter: {}\n\
         ══════════════════════════════════════════════\n\n\
         API BMKG tidak dapat diakses saat ini.\n\n\
         INFORMASI STASIUN:\n\
         {}\n\n\
         CARA AKSES DATA BMKG:\n\
         1. Kunjungi dataonline.bmkg.go.id\n\
         2. Registrasi akun (gratis)\n\
         3. Pilih stasiun dan parameter\n\
         4. Download data dalam format CSV/JSON\n\n\
         STASIUN BMKG CONTOH:\n\
         • 97072 - Stasiun Meteorologi Soekarno-Hatta (Jakarta)\n\
         • 96749 - Stasiun Meteorologi Juanda (Surabaya)\n\
         • 96839 - Stasiun Meteorologi Ngurah Rai (Bali)\n\
         • 97120 - Stasiun Meteorologi Selaparang (Mataram)\n\
         • 97410 - Stasiun Meteorologi Sultan Muhammad Kaharuddin (Sumbawa)\n\
         • 96011 - Stasiun Meteorologi Polonia (Medan)\n\
         • 97180 - Stasiun Meteorologi Sultan Hasanuddin (Makassar)\n\n\
         FORMAT STASIUN ID: 5 digit kode WMO\n\
         ══════════════════════════════════════════════",
        station_id, param_desc, station_info
    )
}

fn format_climate_response(data: &serde_json::Value, station_id: &str, parameter: &str) -> String {
    let mut result = format!(
        "══════════════════════════════════════════════\n\
         BMKG - DATA IKLIM HISTORIS\n\
         Stasiun: {}\n\
         Sumber: dataonline.bmkg.go.id\n\
         ══════════════════════════════════════════════\n\n",
        station_id
    );

    if let Some(records) = data.as_array() {
        result.push_str(&format!(
            "Parameter: {}\nJumlah data: {}\n\n",
            parameter,
            records.len()
        ));
        for record in records.iter().take(30) {
            let date = record
                .get("Tanggal")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A");
            let value = record.get("Nilai").and_then(|v| v.as_f64()).unwrap_or(0.0);
            result.push_str(&format!("{}: {:.1}\n", date, value));
        }
        if records.len() > 30 {
            result.push_str(&format!("\n... dan {} data lainnya\n", records.len() - 30));
        }
    } else if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            result.push_str(&format!("{}: {}\n", key, value));
        }
    }

    result.push_str("\n══════════════════════════════════════════════\n");
    result
}

fn get_station_info(station_id: &str) -> String {
    if let Ok(register) = serde_json::from_str::<serde_json::Value>(STATION_REGISTER) {
        if let Some(stations) = register.get("stations").and_then(|s| s.as_array()) {
            let matches: Vec<&serde_json::Value> = stations
                .iter()
                .filter(|s| s.get("code").and_then(|c| c.as_str()) == Some(station_id))
                .collect();

            if matches.len() > 1 {
                // Duplicate codes exist in the register (e.g. 97600 Merauke is
                // listed under both Papua and Papua Selatan after the 2022
                // provincial split). Report the conflict instead of picking one.
                let mut out = format!(
                    "KONFLIK: kode stasiun '{}' terdaftar {} kali dengan metadata berbeda:\n",
                    station_id,
                    matches.len()
                );
                for m in &matches {
                    out.push_str(&format!(
                        "  - {} ({:.4}, {:.4}) prov. {}\n",
                        m.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
                        m.get("lat").and_then(|v| v.as_f64()).unwrap_or(f64::NAN),
                        m.get("lon").and_then(|v| v.as_f64()).unwrap_or(f64::NAN),
                        m.get("province").and_then(|v| v.as_str()).unwrap_or("?"),
                    ));
                }
                out.push_str(
                    "  Metadata tidak dipilih otomatis: verifikasi ke katalog stasiun BMKG.\n",
                );
                return out;
            }

            if let Some(s) = matches.first() {
                return format!(
                    "{} ({:.4}°, {:.4}°) prov. {} [sumber: resources/bmkg_stations_indonesia.json]",
                    s.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
                    s.get("lat").and_then(|v| v.as_f64()).unwrap_or(f64::NAN),
                    s.get("lon").and_then(|v| v.as_f64()).unwrap_or(f64::NAN),
                    s.get("province").and_then(|v| v.as_str()).unwrap_or("?"),
                );
            }
        }
    }

    if let Some((_, name)) = UNVERIFIED_LEGACY_CODES
        .iter()
        .find(|(code, _)| *code == station_id)
    {
        return format!(
            "Kode '{}' berasal dari tabel lama dan TIDAK ada di register stasiun \
             (resources/bmkg_stations_indonesia.json). Nama yang pernah dilekatkan: '{}'. \
             Kode ini tidak dipetakan otomatis ke entri register karena kami tidak dapat \
             memverifikasi mana yang otoritatif — untuk Bima, tabel lama memakai 97510 \
             sedangkan register memakai 97240. Verifikasi ke katalog BMKG sebelum dipakai.",
            station_id, name
        );
    }

    format!(
        "Stasiun '{}' tidak ditemukan dalam register (resources/bmkg_stations_indonesia.json).",
        station_id
    )
}

#[cfg(test)]
mod station_register_tests {
    use super::*;

    #[test]
    fn register_parses_and_is_non_empty() {
        let v: serde_json::Value = serde_json::from_str(STATION_REGISTER).expect("register JSON");
        let stations = v.get("stations").and_then(|s| s.as_array()).expect("stations array");
        assert!(!stations.is_empty());
    }

    #[test]
    fn bima_resolves_from_register_not_legacy_code() {
        let info = get_station_info("97240");
        assert!(info.contains("Bima"), "unexpected: {info}");
        assert!(info.contains("bmkg_stations_indonesia.json"));
    }

    #[test]
    fn legacy_bima_code_is_flagged_not_silently_mapped() {
        let info = get_station_info("97510");
        assert!(info.contains("tabel lama"), "unexpected: {info}");
        assert!(info.contains("97240"), "should name the register code: {info}");
    }

    #[test]
    fn duplicate_register_code_reports_conflict() {
        // 97600 is listed twice (Papua / Papua Selatan) with differing coords.
        let info = get_station_info("97600");
        assert!(info.starts_with("KONFLIK"), "unexpected: {info}");
    }
}
