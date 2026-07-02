use reqwest::Client;

pub async fn emissions(client: &Client, sector: Option<String>) -> String {
    let mut out = String::from("=== Climate TRACE — Indonesia GHG Emissions ===\n");
    out.push_str("Source: climatetrace.org\n\n");

    let url = "https://api.climatetrace.org/v6/country/emissions?since=2023&to=2024&countries=IDN";
    
    match client.get(url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    out.push_str(&serde_json::to_string_pretty(&v).unwrap_or_default().chars().take(3000).collect::<String>());
                } else {
                    out.push_str(&body[..body.len().min(2000)]);
                }
            }
            Err(e) => out.push_str(&format!("Error: {}\n", e)),
        },
        Err(e) => {
            out.push_str(&format!("API connection error: {}\n", e));
            // Provide reference data
            out.push_str("\n--- Indonesia GHG Reference Data ---\n");
            out.push_str("Total emissions (2023 est): ~2,100 MtCO2e\n");
            out.push_str("By sector:\n");
            out.push_str("  Land Use & Forestry: ~900 MtCO2e (43%)\n");
            out.push_str("  Energy: ~650 MtCO2e (31%)\n");
            out.push_str("  Agriculture: ~250 MtCO2e (12%)\n");
            out.push_str("  Waste: ~150 MtCO2e (7%)\n");
            out.push_str("  Industry: ~150 MtCO2e (7%)\n");
            if let Some(s) = &sector {
                out.push_str(&format!("\nFilter: {} — use Climate TRACE dashboard for detailed facility-level data\n", s));
            }
        }
    }
    out
}
