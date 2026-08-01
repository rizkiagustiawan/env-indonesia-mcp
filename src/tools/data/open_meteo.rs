use reqwest::Client;

pub async fn weather(client: &Client, lat: f64, lon: f64) -> String {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,precipitation,rain,wind_speed_10m,wind_direction_10m,weather_code&daily=temperature_2m_max,temperature_2m_min,precipitation_sum,rain_sum,weather_code&timezone=Asia/Makassar&forecast_days=7",
        lat, lon
    );

    let mut out = format!("=== Weather Forecast ({:.4}, {:.4}) ===\n", lat, lon);
    out.push_str("Source: Open-Meteo (ECMWF/GFS, free, no API key)\n\n");

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                // Current
                if let Some(c) = v.get("current") {
                    let temp = c
                        .get("temperature_2m")
                        .and_then(|t| t.as_f64())
                        .unwrap_or(0.0);
                    let rh = c
                        .get("relative_humidity_2m")
                        .and_then(|r| r.as_f64())
                        .unwrap_or(0.0);
                    let precip = c
                        .get("precipitation")
                        .and_then(|p| p.as_f64())
                        .unwrap_or(0.0);
                    let wind = c
                        .get("wind_speed_10m")
                        .and_then(|w| w.as_f64())
                        .unwrap_or(0.0);
                    let wcode = c.get("weather_code").and_then(|w| w.as_u64()).unwrap_or(0);

                    out.push_str(&format!("CURRENT: {:.1}°C | RH {:.0}% | Precip {:.1}mm | Wind {:.1}km/h | WMO:{}\n\n", 
                        temp, rh, precip, wind, wcode));
                }
                // Daily forecast
                if let Some(daily) = v.get("daily") {
                    out.push_str("7-DAY FORECAST:\n");
                    if let (Some(dates), Some(tmax), Some(tmin), Some(precip)) = (
                        daily.get("time").and_then(|t| t.as_array()),
                        daily.get("temperature_2m_max").and_then(|t| t.as_array()),
                        daily.get("temperature_2m_min").and_then(|t| t.as_array()),
                        daily.get("precipitation_sum").and_then(|p| p.as_array()),
                    ) {
                        for i in 0..dates.len().min(7) {
                            let date = dates[i].as_str().unwrap_or("?");
                            let mx = tmax[i].as_f64().unwrap_or(0.0);
                            let mn = tmin[i].as_f64().unwrap_or(0.0);
                            let pr = precip[i].as_f64().unwrap_or(0.0);
                            out.push_str(&format!(
                                "  {} | {:.0}-{:.0}°C | Rain: {:.1}mm\n",
                                date, mn, mx, pr
                            ));
                        }
                    }
                }
            }
            Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}
