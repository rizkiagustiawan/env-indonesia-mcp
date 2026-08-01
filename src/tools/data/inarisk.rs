use reqwest::Client;

pub async fn disaster_risk(client: &Client, location: &str) -> String {
    let loc = location.to_lowercase();
    let mut out = format!(
        "=== InaRISK BNPB — Penilaian Risiko Bencana ({}) ===\n\n",
        location
    );

    // Try InaRISK API if available
    let api_url = format!(
        "https://inarisk.bnpb.go.id/api/risiko?lokasi={}",
        location.replace(' ', "%20")
    );

    match client
        .get(&api_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                if let Ok(v) = resp.json::<serde_json::Value>().await {
                    if let Some(data) = v
                        .get("data")
                        .and_then(|d| d.as_array())
                        .filter(|a| !a.is_empty())
                    {
                        out.push_str("(Data live dari InaRISK API)\n\n");
                        for item in data.iter().take(10) {
                            let jenis = item
                                .get("jenis_bencana")
                                .and_then(|j| j.as_str())
                                .unwrap_or("?");
                            let level = item
                                .get("tingkat_risiko")
                                .and_then(|l| l.as_str())
                                .unwrap_or("?");
                            out.push_str(&format!("- {}: {}\n", jenis, level));
                        }
                        out.push_str("\nSumber: https://inarisk.bnpb.go.id/\n");
                        return out;
                    }
                }
            }
            out.push_str("(InaRISK API tidak mengembalikan data, menggunakan fallback statis)\n\n");
        }
        Err(_) => {
            out.push_str("(InaRISK API tidak dapat dihubungi, menggunakan fallback statis)\n\n");
        }
    }

    // === FALLBACK: hardcoded data ===
    out.push_str("Data InaRISK statis. Untuk data terbaru, kunjungi inarisk.bnpb.go.id\n\n");

    if loc.contains("jakarta") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Banjir: SANGAT TINGGI\n- Gempa Bumi: SEDANG\n- Penurunan Tanah (Subsidence): KRITIS\n- Kenaikan Muka Air Laut: TINGGI\n");
    } else if loc.contains("jawa barat") || loc.contains("banten") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi & Tsunami: TINGGI (Megathrust Selatan Jawa)\n- Longsor: SANGAT TINGGI (Cianjur, Bogor, Sukabumi)\n- Letusan Gunung Api: TINGGI\n");
    } else if loc.contains("jawa tengah") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI\n- Banjir: TINGGI (Pantura)\n- Letusan Gunung Api: TINGGI (Merapi)\n- Longsor: TINGGI\n");
    } else if loc.contains("sumatera barat") || loc.contains("padang") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi & Tsunami: SANGAT TINGGI (Megathrust Mentawai)\n- Longsor: TINGGI\n- Letusan Gunung Api: TINGGI (Marapi)\n");
    } else if loc.contains("kalimantan") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Kebakaran Hutan & Lahan (Karhutla): SANGAT TINGGI\n- Banjir: TINGGI\n- Gempa Bumi & Tsunami: SANGAT RENDAH (Relatif paling aman di Indonesia)\n");
    } else if loc.contains("sulawesi") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi & Tsunami: SANGAT TINGGI (Palu-Koro fault)\n- Banjir Bandang & Longsor: TINGGI\n");
    } else if loc.contains("papua") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI\n- Banjir Bandang: TINGGI (Sentani, Jayapura)\n- Letusan Gunung Api: RENDAH\n");
    } else if loc.contains("lombok") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI (Sesar Naik Flores / Back-arc Thrust)\n- Tsunami: TINGGI (Pesisir Utara, Barat, dan Selatan)\n- Banjir: SEDANG - TINGGI (Lombok Barat, Mataram)\n- Longsor: TINGGI (Sembalun, Pusuk)\n- Kekeringan: SEDANG (Lombok Timur bagian selatan)\n");
    } else if loc.contains("sumbawa") || loc.contains("bima") || loc.contains("dompu") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI\n- Tsunami: SEDANG - TINGGI\n- Banjir Bandang: SANGAT TINGGI (Bima, Dompu)\n- Kekeringan: TINGGI - SANGAT TINGGI (Sumbawa Timur)\n- Letusan Gunung Api: TINGGI (Tambora)\n");
    } else if loc.contains("aceh") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi & Tsunami: SANGAT TINGGI (Megathrust Sunda)\n- Banjir: TINGGI\n- Longsor: TINGGI (Dataran Tinggi Gayo)\n");
    } else if loc.contains("jawa timur") || loc.contains("jatim") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Letusan Gunung Api: SANGAT TINGGI (Semeru, Bromo, Kelud)\n- Gempa Bumi: TINGGI\n- Banjir: TINGGI (Pantura, Bengawan Solo)\n- Longsor: TINGGI\n");
    } else if loc.contains("bali") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI\n- Tsunami: SEDANG (Pesisir Selatan)\n- Letusan Gunung Api: TINGGI (Agung)\n- Banjir: SEDANG\n");
    } else if loc.contains("maluku") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi & Tsunami: SANGAT TINGGI\n- Letusan Gunung Api: TINGGI\n- Banjir Bandang: TINGGI\n");
    } else {
        out.push_str("Indonesia berada di Ring of Fire. Secara umum Indeks Risiko Bencana (IRBI) masuk kategori TINGGI.\nRisiko dominan: Gempa Bumi, Tsunami, Banjir, Longsor, Cuaca Ekstrem, dan Erupsi Gunung Api.\n");
        out.push_str(
            "\nTip: Coba masukkan nama provinsi/kota spesifik untuk data risiko lebih detail.\n",
        );
        out.push_str("Contoh: 'Jakarta', 'Lombok', 'Sulawesi', 'Aceh', 'Bali', 'Jawa Timur'\n");
    }

    out.push_str("\nSumber: https://inarisk.bnpb.go.id/\n");
    out.push_str("Data Spasial WFS: https://gis.bnpb.go.id/");
    out
}
