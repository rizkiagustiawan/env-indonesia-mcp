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

    match client
        .get(&url)
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
         • Aceh, Sumatera Utara, Kalimantan Utara: SEDANG-TINGGI\n\
         • Papua, Kalimantan Timur, Lampung, Sulawesi Tengah: SEDANG\n\
         • Sumatera Barat, Kepulauan Riau, Bangka Belitung, Maluku: SEDANG\n\
         • NTB, NTT, Bengkulu, Gorontalo, Sulawesi Utara: RENDAH-SEDANG\n\
         • Banten, Bali: RENDAH (musim kemarau waspada)\n\n\
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
            let confidence = hs
                .get("confidence")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A");
            let date = hs.get("acq_date").and_then(|v| v.as_str()).unwrap_or("N/A");
            let satellite = hs
                .get("satellite")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A");
            result.push_str(&format!(
                "{}. ({:.4}, {:.4}) | {} | Confidence: {} | Sat: {}\n",
                i + 1,
                lat,
                lon,
                date,
                confidence,
                satellite
            ));
        }
        if hotspots.len() > 20 {
            result.push_str(&format!(
                "\n... dan {} hotspot lainnya\n",
                hotspots.len() - 20
            ));
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
        ("papua tengah", "Papua Tengah: Risiko RENDAH-SEDANG. Kebakaran padang rumput dataran tinggi. Area: lembah Baliem, Jayawijaya. Puncak: Jul-Okt."),
        ("papua selatan", "Papua Selatan: Risiko SEDANG. Kebakaran savana dan pembukaan lahan pertanian (transmigrasi). Hotspot utama: Merauke, Boven Digoel. Puncak: Ags-Nov."),
        ("papua", "Papua: Risiko SEDANG. Deforestasi dan pembukaan lahan baru."),
        ("aceh", "Aceh: Risiko SEDANG-TINGGI. Kebakaran gambut di dataran rendah pantai timur. Hotspot utama: Aceh Timur, Nagan Raya, Aceh Barat. Puncak: Feb-Mar, Jun-Sep. Sumber: data hotspot KLHK."),
        ("sumatera utara", "Sumatera Utara: Risiko SEDANG-TINGGI. Kebakaran gambut di pantai timur dan perkebunan sawit. Hotspot utama: Labuhanbatu, Serdang Bedagai, Asahan, Langkat. Puncak: Feb-Apr, Jun-Sep."),
        ("sumatera barat", "Sumatera Barat: Risiko SEDANG. Kebakaran hutan di dataran tinggi saat kemarau. Area: kawasan hutan Solok, Pasaman, Sijunjung. Puncak: Jul-Okt."),
        ("bengkulu", "Bengkulu: Risiko RENDAH-SEDANG. Kebakaran hutan dan lahan saat musim kemarau. Area: Bengkulu Utara, Seluma. Puncak: Ags-Okt."),
        ("kepulauan riau", "Kepulauan Riau: Risiko SEDANG. Kebakaran gambut di Pulau Bintan dan sekitarnya. Hotspot utama: Bintan, Karimun, Lingga. Puncak: Feb-Mar, Jun-Ags."),
        ("bangka belitung", "Bangka Belitung: Risiko SEDANG. Kebakaran lahan bekas tambang dan hutan sekunder. Area: Bangka Tengah, Belitung Timur. Puncak: Jul-Okt."),
        ("banten", "Banten: Risiko RENDAH. Kebakaran pinggiran perkotaan dan zona penyangga Ujung Kulon. Area: Pandeglang, Lebak. Puncak: Ags-Okt."),
        ("kalimantan utara", "Kalimantan Utara: Risiko SEDANG-TINGGI. Kebakaran hutan di area perbatasan Malaysia, terkait aktivitas logging. Hotspot utama: Malinau, Nunukan, Bulungan. Puncak: Jul-Okt."),
        ("sulawesi selatan", "Sulawesi Selatan: Risiko SEDANG. Kebakaran savana dan padang rumput saat kemarau. Area: Bone, Wajo, Luwu. Puncak: Ags-Nov."),
        ("sulawesi tengah", "Sulawesi Tengah: Risiko SEDANG. Kebakaran hutan, terutama pasca pembukaan lahan setelah gempa 2018. Hotspot utama: Sigi, Donggala, Morowali. Puncak: Ags-Nov."),
        ("sulawesi tenggara", "Sulawesi Tenggara: Risiko SEDANG. Kebakaran savana dan hutan sekunder. Area: Konawe, Kolaka, Bombana. Puncak: Ags-Nov."),
        ("sulawesi utara", "Sulawesi Utara: Risiko RENDAH-SEDANG. Kebakaran hutan di sekitar Manado dan Minahasa. Area: Minahasa Selatan, Bolaang Mongondow. Puncak: Ags-Okt."),
        ("gorontalo", "Gorontalo: Risiko RENDAH-SEDANG. Kebakaran padang rumput musiman. Area: Gorontalo Utara, Pohuwato. Puncak: Ags-Nov."),
        ("maluku", "Maluku: Risiko SEDANG. Kebakaran musim kemarau di Maluku Tenggara. Area: Maluku Tenggara Barat, Kepulauan Aru. Puncak: Ags-Nov."),
    ];

    for (key, val) in &info {
        if query.contains(key) {
            return val.to_string();
        }
    }

    format!(
        "Informasi spesifik untuk '{}' tidak tersedia dalam database referensi.",
        province
    )
}
