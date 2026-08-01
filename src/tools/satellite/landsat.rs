use reqwest::Client;

pub async fn search(client: &Client, lat: f64, lon: f64, days: u32) -> String {
    let bbox = format!("{},{},{},{}", lon - 0.5, lat - 0.5, lon + 0.5, lat + 0.5);
    let end_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let start_date = (chrono::Utc::now() - chrono::Duration::days(days as i64))
        .format("%Y-%m-%d")
        .to_string();

    let url = format!(
        "https://landsatlook.usgs.gov/stac-server/search?collections=landsat-c2l2-sr&bbox={}&datetime={}/{}&limit=5",
        bbox, start_date, end_date
    );

    match client.get(&url).send().await {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(v) => {
                    let mut out = format!("=== USGS Landsat STAC Search ===\nLocation: ({}, {})\nPeriod: {} to {}\n\n", lat, lon, start_date, end_date);
                    if let Some(matched) = v.get("numberMatched").and_then(|m| m.as_u64()) {
                        out.push_str(&format!("Total scenes matched: {}\n\n", matched));
                    }
                    if let Some(features) = v.get("features").and_then(|f| f.as_array()) {
                        for (i, f) in features.iter().enumerate() {
                            let id = f.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                            let dt = f
                                .get("properties")
                                .and_then(|p| p.get("datetime"))
                                .and_then(|d| d.as_str())
                                .unwrap_or("?");
                            let cloud = f
                                .get("properties")
                                .and_then(|p| p.get("eo:cloud_cover"))
                                .and_then(|c| c.as_f64());
                            out.push_str(&format!(
                                "{}. {} | {} | Cloud: {:.0}%\n",
                                i + 1,
                                id,
                                &dt[..10.min(dt.len())],
                                cloud.unwrap_or(0.0)
                            ));
                        }
                    }
                    out.push_str("\nSource: USGS LandsatLook STAC API (free, no auth)\n");
                    out
                }
                Err(e) => format!("ERROR parsing STAC response: {}", e),
            }
        }
        Err(e) => format!("ERROR connecting to USGS STAC: {}", e),
    }
}
