use reqwest::Client;

/// BPS domain IDs for Indonesian provinces
fn province_domain_id(province: &str) -> Option<(&'static str, &'static str)> {
    let p = province.to_lowercase();
    // Returns (domain_id, province_name)
    match () {
        _ if p.contains("aceh") => Some(("1100", "Aceh")),
        _ if p.contains("sumatera utara") || p.contains("sumut") => {
            Some(("1200", "Sumatera Utara"))
        }
        _ if p.contains("sumatera barat") || p.contains("sumbar") => {
            Some(("1300", "Sumatera Barat"))
        }
        _ if p.contains("riau") && !p.contains("kepulauan") => Some(("1400", "Riau")),
        _ if p.contains("jambi") => Some(("1500", "Jambi")),
        _ if p.contains("sumatera selatan") || p.contains("sumsel") => {
            Some(("1600", "Sumatera Selatan"))
        }
        _ if p.contains("bengkulu") => Some(("1700", "Bengkulu")),
        _ if p.contains("lampung") => Some(("1800", "Lampung")),
        _ if p.contains("bangka") => Some(("1900", "Kep. Bangka Belitung")),
        _ if p.contains("kepulauan riau") || p.contains("kepri") => {
            Some(("2100", "Kepulauan Riau"))
        }
        _ if p.contains("jakarta") || p.contains("dki") => Some(("3100", "DKI Jakarta")),
        _ if p.contains("jawa barat") || p.contains("jabar") => Some(("3200", "Jawa Barat")),
        _ if p.contains("jawa tengah") || p.contains("jateng") => Some(("3300", "Jawa Tengah")),
        _ if p.contains("yogyakarta") || p.contains("diy") => Some(("3400", "DI Yogyakarta")),
        _ if p.contains("jawa timur") || p.contains("jatim") => Some(("3500", "Jawa Timur")),
        _ if p.contains("banten") => Some(("3600", "Banten")),
        _ if p.contains("bali") => Some(("5100", "Bali")),
        _ if p.contains("ntb") || p.contains("nusa tenggara barat") => {
            Some(("5200", "Nusa Tenggara Barat"))
        }
        _ if p.contains("ntt") || p.contains("nusa tenggara timur") => {
            Some(("5300", "Nusa Tenggara Timur"))
        }
        _ if p.contains("kalimantan barat") || p.contains("kalbar") => {
            Some(("6100", "Kalimantan Barat"))
        }
        _ if p.contains("kalimantan tengah") || p.contains("kalteng") => {
            Some(("6200", "Kalimantan Tengah"))
        }
        _ if p.contains("kalimantan selatan") || p.contains("kalsel") => {
            Some(("6300", "Kalimantan Selatan"))
        }
        _ if p.contains("kalimantan timur") || p.contains("kaltim") => {
            Some(("6400", "Kalimantan Timur"))
        }
        _ if p.contains("kalimantan utara") || p.contains("kaltara") => {
            Some(("6500", "Kalimantan Utara"))
        }
        _ if p.contains("sulawesi utara") || p.contains("sulut") => {
            Some(("7100", "Sulawesi Utara"))
        }
        _ if p.contains("sulawesi tengah") || p.contains("sulteng") => {
            Some(("7200", "Sulawesi Tengah"))
        }
        _ if p.contains("sulawesi selatan") || p.contains("sulsel") => {
            Some(("7300", "Sulawesi Selatan"))
        }
        _ if p.contains("sulawesi tenggara") || p.contains("sultra") => {
            Some(("7400", "Sulawesi Tenggara"))
        }
        _ if p.contains("gorontalo") => Some(("7500", "Gorontalo")),
        _ if p.contains("sulawesi barat") || p.contains("sulbar") => {
            Some(("7600", "Sulawesi Barat"))
        }
        _ if p.contains("maluku") && !p.contains("utara") => Some(("8100", "Maluku")),
        _ if p.contains("maluku utara") => Some(("8200", "Maluku Utara")),
        // Papua provinces — order matters: check specific names before generic "papua"
        _ if p.contains("papua barat daya") => Some(("9600", "Papua Barat Daya")),
        _ if p.contains("papua barat") => Some(("9100", "Papua Barat")),
        _ if p.contains("papua selatan") => Some(("9300", "Papua Selatan")),
        _ if p.contains("papua tengah") => Some(("9400", "Papua Tengah")),
        _ if p.contains("papua pegunungan") => Some(("9500", "Papua Pegunungan")),
        _ if p.contains("papua") => Some(("9200", "Papua")),
        _ => None,
    }
}

pub async fn statistics(client: &Client, keyword: &str) -> String {
    // Extract province from keyword if present, default to national
    let keyword_lower = keyword.to_lowercase();

    // Check for known province names in keyword
    let province_info = province_domain_id(&keyword_lower);
    let domain_id = province_info.map(|(id, _)| id).unwrap_or("0000"); // 0000 = nasional
    let province_name = province_info
        .map(|(_, name)| name)
        .unwrap_or("Indonesia (Nasional)");

    let api_key = std::env::var("BPS_API_KEY").unwrap_or_default();

    let mut out = format!(
        "=== BPS Statistik Lingkungan — {} (Query: {}) ===\n\n",
        province_name, keyword
    );

    // Try real BPS API if key is available
    if !api_key.is_empty() {
        // Try environment-related var IDs
        // 69 = Luas Kawasan Hutan, 160 = Produksi Sampah, etc.
        let env_var_ids = ["69", "160", "1714", "1715"];

        let mut got_data = false;
        for var_id in &env_var_ids {
            let url = format!(
                "https://webapi.bps.go.id/v1/api/list/model/data/domain/{}/var/{}/key/{}/",
                domain_id, var_id, api_key
            );

            match client
                .get(&url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => {
                    if let Ok(v) = resp.json::<serde_json::Value>().await {
                        if v.get("status").and_then(|s| s.as_str()) == Some("OK") {
                            if let Some(data) = v.get("data").and_then(|d| d.as_array()) {
                                let var_name = v
                                    .get("var")
                                    .and_then(|vr| vr.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|item| item.get("label"))
                                    .and_then(|l| l.as_str())
                                    .unwrap_or("Data");
                                out.push_str(&format!("[API] {}: ", var_name));
                                for (i, d) in data.iter().take(5).enumerate() {
                                    let val = d
                                        .get("data_content")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-");
                                    let year =
                                        d.get("tahun").and_then(|y| y.as_str()).unwrap_or("?");
                                    if i > 0 {
                                        out.push_str(", ");
                                    }
                                    out.push_str(&format!("{}={}", year, val));
                                }
                                out.push('\n');
                                got_data = true;
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }

        if got_data {
            out.push_str(&format!(
                "\nSumber: BPS API (webapi.bps.go.id), domain={}\n",
                domain_id
            ));
            return out;
        }
        out.push_str(
            "(BPS API tidak mengembalikan data yang relevan, menggunakan data fallback)\n\n",
        );
    } else {
        out.push_str("(BPS_API_KEY tidak diset — menggunakan data fallback statis)\n\n");
    }

    // === FALLBACK: hardcoded data ===
    out.push_str(&format!(
        "Data Referensi Utama (BPS 2023-2025, {}):\n",
        province_name
    ));

    let l = keyword.to_lowercase();
    if l.contains("hutan") || l.contains("forest") {
        out.push_str("- Luas Kawasan Hutan Indonesia: ~120.5 Juta Ha (KLHK 2023)\n");
        out.push_str("- Hutan Konservasi: ~27.4 Juta Ha\n");
        out.push_str("- Hutan Lindung: ~29.7 Juta Ha\n");
        out.push_str("- Hutan Produksi: ~68.8 Juta Ha\n");
        out.push_str("- Deforestasi 2021-2022: ~104 ribu Ha (terendah dalam dekade)\n");
    } else if l.contains("sampah") || l.contains("waste") {
        out.push_str("- Timbulan Sampah Nasional: ~68.5 Juta Ton/Tahun (SIPSN 2023)\n");
        out.push_str("- Sampah Terkelola: ~41.25%\n");
        out.push_str("- Target 2025: 30% pengurangan, 70% penanganan (Perpres 97/2017)\n");
    } else if l.contains("air") || l.contains("water") {
        out.push_str("- Rumah Tangga dengan Akses Air Minum Layak (Nasional): ~91.05%\n");
        out.push_str("- Akses Sanitasi Layak: ~79.45%\n");
        out.push_str("- Sungai dengan Status Tercemar Berat: ~59% (dari 97 sungai utama)\n");
    } else {
        out.push_str("- PDRB Indonesia didominasi sektor Industri, Pertanian, dan Perdagangan.\n");
        out.push_str("- Sektor Pertanian rentan terhadap perubahan iklim (El Nino/La Nina).\n");
        out.push_str("- Emisi GRK Nasional: ~1.2 Gt CO2e (2022, termasuk FOLU)\n");
    }

    out.push_str(
        "\nUntuk integrasi API live, daftarkan API Key di https://webapi.bps.go.id/developer/\n",
    );
    out.push_str("Lalu atur environment variable BPS_API_KEY.\n");
    out.push_str(&format!(
        "Domain ID untuk {}: {}\n",
        province_name, domain_id
    ));
    out
}
