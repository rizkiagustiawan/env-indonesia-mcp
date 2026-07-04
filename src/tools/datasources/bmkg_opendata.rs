use reqwest::Client;

/// BMKG Historical Climate Data
/// Source: dataonline.bmkg.go.id

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
        let available: Vec<String> = params.iter().map(|(p, d)| format!("  {} - {}", p, d)).collect();
        return format!(
            "Parameter '{}' tidak valid.\nParameter tersedia:\n{}",
            parameter, available.join("\n")
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

    match client.get(&url)
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
    let param_desc = params.iter().find(|(p, _)| *p == parameter).map(|(_, d)| *d).unwrap_or("N/A");

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
        result.push_str(&format!("Parameter: {}\nJumlah data: {}\n\n", parameter, records.len()));
        for record in records.iter().take(30) {
            let date = record.get("Tanggal").and_then(|v| v.as_str()).unwrap_or("N/A");
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
    let stations = vec![
        ("97120", "Stasiun Meteorologi Selaparang, Mataram, NTB (-8.53°, 116.08°)"),
        ("97230", "Stasiun Meteorologi Lombok Tengah (-8.73°, 116.28°)"),
        ("97330", "Stasiun Klimatologi Kediri, Lombok Barat (-8.65°, 116.12°)"),
        ("97410", "Stasiun Meteorologi Sultan M. Kaharuddin, Sumbawa (-8.49°, 117.41°)"),
        ("97510", "Stasiun Meteorologi Bima (-8.54°, 118.69°)"),
        ("96749", "Stasiun Meteorologi Juanda, Surabaya (-7.38°, 112.77°)"),
        ("96839", "Stasiun Meteorologi Ngurah Rai, Bali (-8.75°, 115.17°)"),
        ("97072", "Stasiun Meteorologi Soekarno-Hatta, Jakarta (-6.12°, 106.66°)"),
    ];

    for (id, info) in &stations {
        if *id == station_id {
            return info.to_string();
        }
    }

    format!("Stasiun '{}' tidak ditemukan dalam database referensi.", station_id)
}
