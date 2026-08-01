use reqwest::Client;

/// Mengambil data kualitas udara ground-truth (stasiun fisik) dari WAQI (World Air Quality Index)
/// WAQI mengumpulkan data dari KLHK, US Embassy, dan kontributor independen.

pub async fn get_air_quality(client: &Client, lat: f64, lon: f64) -> String {
    let api_key = std::env::var("WAQI_API_KEY").unwrap_or_else(|_| "demo".to_string());

    // WAQI Geo-localized API Endpoint
    let url = format!(
        "https://api.waqi.info/feed/geo:{};{}/?token={}",
        lat, lon, api_key
    );

    let mut out = format!("=== Ground Sensor Air Quality (WAQI) ===\n");
    out.push_str("Source: World Air Quality Index Project (waqi.info)\n");

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                if let Some(status) = v.get("status").and_then(|s| s.as_str()) {
                    if status == "ok" {
                        if let Some(data) = v.get("data") {
                            let aqi = data.get("aqi").and_then(|a| a.as_u64()).unwrap_or(0);
                            let stasiun = data
                                .get("city")
                                .and_then(|c| c.get("name"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("Unknown Station");
                            let waktu = data
                                .get("time")
                                .and_then(|t| t.get("s"))
                                .and_then(|s| s.as_str())
                                .unwrap_or("Unknown Time");

                            out.push_str(&format!("\nStasiun Terdekat: {}\n", stasiun));
                            out.push_str(&format!("Waktu Update: {}\n\n", waktu));

                            let kriteria = if aqi <= 50 {
                                "Baik (Good)"
                            } else if aqi <= 100 {
                                "Sedang (Moderate)"
                            } else if aqi <= 150 {
                                "Tidak Sehat bagi Sensitif (Unhealthy for Sensitive Groups)"
                            } else if aqi <= 200 {
                                "Tidak Sehat (Unhealthy)"
                            } else if aqi <= 300 {
                                "Sangat Tidak Sehat (Very Unhealthy)"
                            } else {
                                "Berbahaya (Hazardous)"
                            };

                            out.push_str(&format!("AQI (US EPA): {} - {}\n", aqi, kriteria));

                            if let Some(iaqi) = data.get("iaqi") {
                                out.push_str("\nDetail Polutan (Indeks):\n");
                                if let Some(pm25) = iaqi
                                    .get("pm25")
                                    .and_then(|p| p.get("v"))
                                    .and_then(|v| v.as_f64())
                                {
                                    out.push_str(&format!("  PM2.5: {:.1}\n", pm25));
                                }
                                if let Some(pm10) = iaqi
                                    .get("pm10")
                                    .and_then(|p| p.get("v"))
                                    .and_then(|v| v.as_f64())
                                {
                                    out.push_str(&format!("  PM10: {:.1}\n", pm10));
                                }
                                if let Some(no2) = iaqi
                                    .get("no2")
                                    .and_then(|p| p.get("v"))
                                    .and_then(|v| v.as_f64())
                                {
                                    out.push_str(&format!("  NO2: {:.1}\n", no2));
                                }
                                if let Some(so2) = iaqi
                                    .get("so2")
                                    .and_then(|p| p.get("v"))
                                    .and_then(|v| v.as_f64())
                                {
                                    out.push_str(&format!("  SO2: {:.1}\n", so2));
                                }
                                if let Some(o3) = iaqi
                                    .get("o3")
                                    .and_then(|p| p.get("v"))
                                    .and_then(|v| v.as_f64())
                                {
                                    out.push_str(&format!("  O3: {:.1}\n", o3));
                                }
                                if let Some(co) = iaqi
                                    .get("co")
                                    .and_then(|p| p.get("v"))
                                    .and_then(|v| v.as_f64())
                                {
                                    out.push_str(&format!("  CO: {:.1}\n", co));
                                }
                            }

                            out.push_str("\nCatatan: Data berasal dari stasiun ground-truth. Sangat berguna sebagai validasi jika citra satelit (TROPOMI) terhalang awan tebal.");
                        }
                    } else {
                        let msg = v
                            .get("data")
                            .and_then(|d| d.as_str())
                            .unwrap_or("Unknown error");
                        out.push_str(&format!("API mengembalikan error: {}\nJika ini adalah error kuota/akses, pastikan Anda mendaftarkan WAQI_API_KEY di environment.", msg));
                    }
                }
            }
            Err(e) => out.push_str(&format!("Gagal parse JSON dari WAQI: {}", e)),
        },
        Err(e) => out.push_str(&format!("Gagal menghubungi WAQI API: {}", e)),
    }

    out
}
