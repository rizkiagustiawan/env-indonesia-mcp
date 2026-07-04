use reqwest::Client;

/// SiPongi Forest Fire Hotspots
/// Source: sipongi.menlhk.go.id

/// Get fire hotspot data for a province
/// Tries SiPongi API, falls back to descriptive information
pub async fn get_hotspots(client: &Client, province: &str) -> String {
    // Try SiPongi API endpoint
    let url = format!(
        "https://sipongi.menlhk.go.id/api/hotspot?provinsi={}",
        province.replace(' ', "%20")
    );

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
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                            return format_hotspot_response(&data, province);
                        }
                        return format!(
                            "Data hotspot dari SiPongi untuk '{}':\n{}\nSumber: sipongi.menlhk.go.id",
                            province, &body[..body.len().min(2000)]
                        );
                    }
                    Err(e) => {
                        return format!("Error membaca respons SiPongi: {}", e);
                    }
                }
            }
        }
        Err(_) => {}
    }

    // Fallback: provide reference information
    let fire_prone = get_fire_prone_info(province);

    format!(
        "══════════════════════════════════════════════\n\
         SiPongi - HOTSPOT KEBAKARAN HUTAN & LAHAN\n\
         Provinsi: {}\n\
         ══════════════════════════════════════════════\n\n\
         API SiPongi tidak dapat diakses saat ini.\n\n\
         INFORMASI WILAYAH:\n\
         {}\n\n\
         SUMBER DATA KEBAKARAN:\n\
         • sipongi.menlhk.go.id (SiPongi KLHK - resmi)\n\
         • firms.modaps.eosdis.nasa.gov (NASA FIRMS - global)\n\
         • earthdata.nasa.gov (VIIRS/MODIS)\n\n\
         TINGKAT KERAWANAN KEBAKARAN:\n\
         • Riau, Kalimantan Tengah, Kalimantan Barat: SANGAT TINGGI\n\
         • Sumatera Selatan, Jambi, Kalimantan Selatan: TINGGI\n\
         • Papua, Kalimantan Timur, Lampung: SEDANG\n\
         • NTB, NTT, Bali: RENDAH (musim kemarau waspada)\n\n\
         MUSIM KEBAKARAN: Juni - Oktober (puncak Agustus-September)\n\
         Ref: InPres 3/2020 tentang Penanggulangan Karhutla\n\
         ══════════════════════════════════════════════",
        province, fire_prone
    )
}

fn format_hotspot_response(data: &serde_json::Value, province: &str) -> String {
    let mut result = format!(
        "══════════════════════════════════════════════\n\
         SiPongi - HOTSPOT KEBAKARAN HUTAN & LAHAN\n\
         Provinsi: {}\n\
         Sumber: sipongi.menlhk.go.id\n\
         ══════════════════════════════════════════════\n\n",
        province
    );

    if let Some(hotspots) = data.get("data").and_then(|v| v.as_array()) {
        result.push_str(&format!("Total hotspot: {}\n\n", hotspots.len()));
        for (i, hs) in hotspots.iter().take(20).enumerate() {
            let lat = hs.get("latitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let lon = hs.get("longitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let confidence = hs.get("confidence").and_then(|v| v.as_str()).unwrap_or("N/A");
            let date = hs.get("acq_date").and_then(|v| v.as_str()).unwrap_or("N/A");
            let satellite = hs.get("satellite").and_then(|v| v.as_str()).unwrap_or("N/A");
            result.push_str(&format!(
                "{}. ({:.4}, {:.4}) | {} | Confidence: {} | Sat: {}\n",
                i + 1, lat, lon, date, confidence, satellite
            ));
        }
        if hotspots.len() > 20 {
            result.push_str(&format!("\n... dan {} hotspot lainnya\n", hotspots.len() - 20));
        }
    } else {
        result.push_str(&format!("Response: {}\n", data));
    }

    result.push_str("\n══════════════════════════════════════════════\n");
    result
}

fn get_fire_prone_info(province: &str) -> String {
    let query = province.to_lowercase();
    let info = vec![
        ("riau", "Riau: Wilayah lahan gambut luas. Risiko SANGAT TINGGI pada musim kemarau. Hotspot utama: Kabupaten Bengkalis, Rokan Hilir, Pelalawan, Siak."),
        ("kalimantan tengah", "Kalimantan Tengah: Lahan gambut dalam. Risiko SANGAT TINGGI. Hotspot utama: Pulang Pisau, Kapuas, Katingan, Kotawaringin Timur."),
        ("kalimantan barat", "Kalimantan Barat: Risiko TINGGI. Hotspot utama: Ketapang, Kayong Utara, Kubu Raya."),
        ("sumatera selatan", "Sumatera Selatan: Lahan gambut signifikan. Risiko TINGGI. Hotspot utama: OKI, Banyuasin, Musi Banyuasin."),
        ("jambi", "Jambi: Risiko TINGGI. Hotspot utama: Tanjung Jabung Barat, Tanjung Jabung Timur, Muaro Jambi."),
         ("ntb", "NTB: Risiko RENDAH-SEDANG. Kebakaran lahan kering pada musim kemarau. Area: hutan Gunung Rinjani, Sumbawa."),
        ("ntt", "NTT: Risiko SEDANG. Kebakaran savana dan padang rumput pada musim kemarau. Area: Timor, Sumba, Flores."),
        ("kalimantan selatan", "Kalimantan Selatan: Risiko TINGGI. Lahan gambut dan hutan rawa. Hotspot utama: Tanah Laut, Banjar, Barito Kuala."),
        ("kalimantan timur", "Kalimantan Timur: Risiko SEDANG-TINGGI. Hotspot utama: Kutai Kartanegara, Berau."),
        ("lampung", "Lampung: Risiko SEDANG. Pembukaan lahan dan perkebunan."),
        ("papua", "Papua: Risiko SEDANG. Deforestasi dan pembukaan lahan baru."),
    ];

    for (key, val) in &info {
        if query.contains(key) {
            return val.to_string();
        }
    }

    format!("Informasi spesifik untuk '{}' tidak tersedia dalam database referensi.", province)
}
