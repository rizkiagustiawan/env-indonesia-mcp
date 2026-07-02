use reqwest::Client;

pub async fn deforestation_alerts(client: &Client) -> String {
    let mut out = String::from("=== Global Forest Watch — NTB Deforestation Data ===\n\n");

    // GFW dashboard data for NTB (province code 17 in GFW)
    let url = "https://data-api.globalforestwatch.org/dataset/gfw_integrated_alerts/latest";
    
    match client.get(url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    out.push_str(&format!("API Response: {}\n", serde_json::to_string_pretty(&v).unwrap_or_default().chars().take(1500).collect::<String>()));
                } else {
                    out.push_str(&format!("Raw: {}\n", &body[..body.len().min(1000)]));
                }
            }
            Err(e) => out.push_str(&format!("Error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }

    out.push_str("\n--- NTB Forest Statistics (Reference Data) ---\n");
    out.push_str("Province: Nusa Tenggara Barat (NTB)\n");
    out.push_str("Total forest area: ~1.1 million hectares\n");
    out.push_str("Key protected areas:\n");
    out.push_str("  - Taman Nasional Gunung Rinjani (41,330 ha)\n");
    out.push_str("  - Taman Nasional Tambora (71,645 ha)\n");
    out.push_str("  - Hutan Lindung Pelangan (various)\n");
    out.push_str("\nDashboard: https://www.globalforestwatch.org/dashboards/country/IDN/17/\n");
    out.push_str("Weekly GLAD alerts available at dashboard link above.\n");
    out
}
