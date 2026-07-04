use reqwest::Client;

/// Real-time ISPU (Indeks Standar Pencemar Udara) from KLHK stations
/// Source: ispu.menlhk.go.id

/// ISPU category thresholds per PermenLH No. 14/2020
fn ispu_category(value: f64) -> (&'static str, &'static str) {
    if value <= 50.0 {
        ("BAIK", "Tingkat kualitas udara yang tidak memberikan efek bagi kesehatan manusia atau hewan dan tidak berpengaruh pada tumbuhan, bangunan, ataupun nilai estetika.")
    } else if value <= 100.0 {
        ("SEDANG", "Tingkat kualitas udara yang tidak berpengaruh pada kesehatan manusia ataupun hewan tetapi berpengaruh pada tumbuhan yang sensitif dan nilai estetika.")
    } else if value <= 200.0 {
        ("TIDAK SEHAT", "Tingkat kualitas udara yang bersifat merugikan pada manusia ataupun kelompok hewan yang sensitif atau bisa menimbulkan kerusakan pada tumbuhan ataupun nilai estetika.")
    } else if value <= 300.0 {
        ("SANGAT TIDAK SEHAT", "Tingkat kualitas udara yang dapat merugikan kesehatan pada sejumlah segmen populasi yang terpapar.")
    } else {
        ("BERBAHAYA", "Tingkat kualitas udara yang berbahaya yang secara umum dapat merugikan kesehatan yang serius pada populasi.")
    }
}

/// Get ISPU data for a city
/// Tries KLHK ISPU API, falls back to informative message with cached reference data
pub async fn get_ispu(client: &Client, kota: &str) -> String {
    // Try KLHK ISPU API endpoint
    let url = format!("https://ispu.menlhk.go.id/api/ispu?kota={}", kota);

    match client.get(&url)
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(body) => {
                        // Try to parse KLHK response
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                            return format_ispu_response(&data, kota);
                        }
                        // If parsing fails, return raw data
                        return format!(
                            "Data ISPU dari KLHK untuk '{}':\n{}\n\nSumber: ispu.menlhk.go.id",
                            kota, &body[..body.len().min(2000)]
                        );
                    }
                    Err(e) => {
                        return format!("Error membaca respons KLHK: {}\nGunakan referensi manual.", e);
                    }
                }
            }
            // Non-success status, fall through to fallback
        }
        Err(_) => {
            // Connection error, fall through to fallback
        }
    }

    // Fallback: provide reference data and instructions
    let reference_stations = get_reference_stations(kota);

    format!(
        "══════════════════════════════════════════════\n\
         ISPU - INDEKS STANDAR PENCEMAR UDARA\n\
         Kota: {}\n\
         ══════════════════════════════════════════════\n\n\
         API KLHK tidak dapat diakses saat ini.\n\n\
         STASIUN PEMANTAU TERDEKAT:\n\
         {}\n\n\
         KATEGORI ISPU (PermenLH No. 14/2020):\n\
         • 0-50    : BAIK (hijau)\n\
         • 51-100  : SEDANG (biru)\n\
         • 101-200 : TIDAK SEHAT (kuning)\n\
         • 201-300 : SANGAT TIDAK SEHAT (merah)\n\
         • >300    : BERBAHAYA (hitam)\n\n\
         PARAMETER: PM2.5, PM10, SO₂, CO, O₃, NO₂\n\n\
         SUMBER DATA ALTERNATIF:\n\
         • ispu.menlhk.go.id (KLHK resmi)\n\
         • aqicn.org/city/{} (WAQI)\n\
         • iku.menlhk.go.id (IKU KLHK)\n\
         ══════════════════════════════════════════════",
        kota, reference_stations, kota.to_lowercase().replace(' ', "-")
    )
}

fn format_ispu_response(data: &serde_json::Value, kota: &str) -> String {
    let mut result = format!(
        "══════════════════════════════════════════════\n\
         ISPU - INDEKS STANDAR PENCEMAR UDARA\n\
         Kota: {}\n\
         Sumber: ispu.menlhk.go.id\n\
         ══════════════════════════════════════════════\n\n",
        kota
    );

    if let Some(records) = data.as_array() {
        for record in records {
            let stasiun = record.get("stasiun").and_then(|v| v.as_str()).unwrap_or("N/A");
            let pm25 = record.get("pm25").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let pm10 = record.get("pm10").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let so2 = record.get("so2").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let co = record.get("co").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let o3 = record.get("o3").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let no2 = record.get("no2").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let max_val = pm25.max(pm10).max(so2).max(co).max(o3).max(no2);
            let (category, desc) = ispu_category(max_val);

            result.push_str(&format!(
                "Stasiun: {}\n\
                 • PM2.5 : {:.0} | PM10: {:.0}\n\
                 • SO₂   : {:.0} | CO  : {:.0}\n\
                 • O₃    : {:.0} | NO₂ : {:.0}\n\
                 • ISPU   : {:.0} - {}\n\
                 • {}\n\n",
                stasiun, pm25, pm10, so2, co, o3, no2, max_val, category, desc
            ));
        }
    } else {
        result.push_str(&format!("Data: {}\n", data));
    }

    result.push_str("══════════════════════════════════════════════\n");
    result
}

fn get_reference_stations(kota: &str) -> String {
    let query = kota.to_lowercase();
    let stations = vec![
        ("jakarta", "DKI1 (Bundaran HI), DKI2 (Kelapa Gading), DKI3 (Jagakarsa), DKI4 (Lubang Buaya), DKI5 (Kebon Jeruk)"),
        ("surabaya", "SUB1 (Wonorejo), SUB2 (Kebonsari), SUB3 (Taman Prestasi)"),
        ("bandung", "BDG1 (Cisarua), BDG2 (Gedebage)"),
        ("semarang", "SMG1 (Pedurungan)"),
        ("medan", "MDN1 (Medan Kota)"),
        ("makassar", "MKS1 (Makassar Kota)"),
        ("denpasar", "DPS1 (Denpasar)"),
        ("mataram", "MTR1 (Mataram)"),
        ("palembang", "PLB1 (Palembang Kota)"),
        ("balikpapan", "BPN1 (Balikpapan Kota)"),
    ];

    for (city, info) in &stations {
        if query.contains(city) {
            return info.to_string();
        }
    }

    format!("Stasiun untuk '{}' tidak ditemukan dalam database referensi.\nKunjungi ispu.menlhk.go.id untuk daftar lengkap.", kota)
}
