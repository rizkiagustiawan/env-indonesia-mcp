use reqwest::Client;
use crate::ntb;

pub async fn weather(client: &Client, location: &str) -> String {
    let loc_lower = location.to_lowercase();
    let adm4 = match loc_lower.as_str() {
        "mataram" | "kota mataram" => ntb::MATARAM_ADM4,
        "bima" | "kota bima" => ntb::BIMA_ADM4,
        "sumbawa" => ntb::SUMBAWA_ADM4,
        "dompu" => ntb::DOMPU_ADM4,
        "lombok_barat" | "lombok barat" => ntb::LOMBOK_BARAT_ADM4,
        other => other, // assume adm4 code passed directly
    };

    let url = format!("https://api.bmkg.go.id/publik/prakiraan-cuaca?adm4={}", adm4);
    match client.get(&url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => {
                // Parse JSON and extract key info
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    let mut out = String::from("=== BMKG Weather Forecast NTB ===\n");
                    if let Some(data) = v.get("data").and_then(|d| d.as_array()) {
                        for loc in data.iter().take(1) {
                            if let Some(name) = loc.get("lokasi").and_then(|l| l.get("desa")).and_then(|d| d.as_str()) {
                                out.push_str(&format!("Location: {}\n", name));
                            }
                            if let Some(cuaca) = loc.get("cuaca").and_then(|c| c.as_array()) {
                                for day in cuaca.iter().take(3) {
                                    if let Some(hours) = day.as_array() {
                                        for h in hours.iter().take(4) {
                                            let time = h.get("local_datetime").and_then(|t| t.as_str()).unwrap_or("?");
                                            let desc = h.get("weather_desc").and_then(|w| w.as_str()).unwrap_or("?");
                                            let temp = h.get("t").and_then(|t| t.as_f64()).unwrap_or(0.0);
                                            let humidity = h.get("hu").and_then(|h| h.as_f64()).unwrap_or(0.0);
                                            let wind_spd = h.get("ws").and_then(|w| w.as_f64()).unwrap_or(0.0);
                                            out.push_str(&format!("  {} | {}°C | {}% RH | {:.0} km/h | {}\n",
                                                time, temp, humidity, wind_spd, desc));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    out
                } else {
                    format!("Raw BMKG response (parse failed): {}", &body[..body.len().min(2000)])
                }
            }
            Err(e) => format!("Error reading BMKG response: {}", e),
        },
        Err(e) => format!("Error calling BMKG API: {}", e),
    }
}

pub async fn earthquake(client: &Client) -> String {
    let url = "https://data.bmkg.go.id/DataMKG/TEWS/gempaterkini.json";
    match client.get(url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                let mut out = String::from("=== BMKG Latest Earthquakes ===\n");
                if let Some(gempa) = v.get("Infogempa").and_then(|i| i.get("gempa")).and_then(|g| g.as_array()) {
                    for g in gempa.iter().take(10) {
                        let tanggal = g.get("Tanggal").and_then(|t| t.as_str()).unwrap_or("?");
                        let jam = g.get("Jam").and_then(|j| j.as_str()).unwrap_or("?");
                        let mag = g.get("Magnitude").and_then(|m| m.as_str()).unwrap_or("?");
                        let depth = g.get("Kedalaman").and_then(|d| d.as_str()).unwrap_or("?");
                        let wilayah = g.get("Wilayah").and_then(|w| w.as_str()).unwrap_or("?");
                        let coords = g.get("Coordinates").and_then(|c| c.as_str()).unwrap_or("?");
                        out.push_str(&format!("{} {} | M{} | {} | {} | {}\n",
                            tanggal, jam, mag, depth, wilayah, coords));
                    }
                }
                out
            }
            Err(e) => format!("Error parsing BMKG earthquake data: {}", e),
        },
        Err(e) => format!("Error calling BMKG earthquake API: {}", e),
    }
}
