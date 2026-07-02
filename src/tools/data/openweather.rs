use reqwest::Client;

pub async fn air_pollution(client: &Client, lat: f64, lon: f64) -> String {
    let api_key = std::env::var("OPENWEATHER_API_KEY").unwrap_or_default();
    
    let url = if !api_key.is_empty() {
        // Use OpenWeatherMap if key is available
        format!("http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}", lat, lon, api_key)
    } else {
        // Fallback to free Open-Meteo API
        format!(
            "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&current=pm10,pm2_5,carbon_monoxide,nitrogen_dioxide,sulphur_dioxide,ozone,uv_index,us_aqi&timezone=Asia/Makassar",
            lat, lon
        )
    };

    let mut out = format!("=== Air Quality — ({:.4}, {:.4}) ===\n", lat, lon);
    
    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                if let Some(current) = v.get("current") {
                    let aqi = current.get("us_aqi").and_then(|a| a.as_f64()).unwrap_or(0.0);
                    let pm25 = current.get("pm2_5").and_then(|p| p.as_f64()).unwrap_or(0.0);
                    let pm10 = current.get("pm10").and_then(|p| p.as_f64()).unwrap_or(0.0);
                    let no2 = current.get("nitrogen_dioxide").and_then(|n| n.as_f64()).unwrap_or(0.0);
                    let so2 = current.get("sulphur_dioxide").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    let o3 = current.get("ozone").and_then(|o| o.as_f64()).unwrap_or(0.0);
                    let co = current.get("carbon_monoxide").and_then(|c| c.as_f64()).unwrap_or(0.0);
                    let uv = current.get("uv_index").and_then(|u| u.as_f64()).unwrap_or(0.0);
                    
                    let category = match aqi as u32 {
                        0..=50 => "BAIK (Good)",
                        51..=100 => "SEDANG (Moderate)",
                        101..=150 => "TIDAK SEHAT untuk Sensitif (Unhealthy for Sensitive)",
                        151..=200 => "TIDAK SEHAT (Unhealthy)",
                        201..=300 => "SANGAT TIDAK SEHAT (Very Unhealthy)",
                        _ => "BERBAHAYA (Hazardous)",
                    };

                    out.push_str(&format!("US AQI: {:.0} — {}\n\n", aqi, category));
                    out.push_str("Pollutant Concentrations:\n");
                    out.push_str(&format!("  PM2.5:  {:.1} µg/m³\n", pm25));
                    out.push_str(&format!("  PM10:   {:.1} µg/m³\n", pm10));
                    out.push_str(&format!("  NO₂:    {:.1} µg/m³\n", no2));
                    out.push_str(&format!("  SO₂:    {:.1} µg/m³\n", so2));
                    out.push_str(&format!("  O₃:     {:.1} µg/m³\n", o3));
                    out.push_str(&format!("  CO:     {:.1} µg/m³\n", co));
                    out.push_str(&format!("  UV Index: {:.1}\n", uv));
                    out.push_str("\nSource: Open-Meteo Air Quality API (CAMS)\n");
                } else {
                    out.push_str("No current data available\n");
                }
            }
            Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}
