use reqwest::Client;

pub async fn deforestation_alerts(client: &Client) -> String {
    let mut out = String::from("=== Global Forest Watch — Indonesia Deforestation Data ===\n\n");

    // GFW dashboard data for NTB (province code 17 in GFW)
    let url = "https://data-api.globalforestwatch.org/dataset/gfw_integrated_alerts/latest";

    match client.get(url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    out.push_str(&format!(
                        "API Response: {}\n",
                        serde_json::to_string_pretty(&v)
                            .unwrap_or_default()
                            .chars()
                            .take(1500)
                            .collect::<String>()
                    ));
                } else {
                    out.push_str(&format!("Raw: {}\n", &body[..body.len().min(1000)]));
                }
            }
            Err(e) => out.push_str(&format!("Error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }

    out.push_str("\n--- Indonesia Forest Statistics (Reference Data, BUKAN live) ---\n");
    out.push_str("Catatan: endpoint di atas mengembalikan metadata dataset GFW, bukan data alert deforestasi aktual.\n");
    out.push_str("Untuk alert terkini gunakan GFW analysis API (data-api.globalforestwatch.org) atau dashboard resmi.\n");
    out.push_str("Country: Indonesia\n");
    out.push_str("Total forest area (2000): ~160 million hectares\n");
    out.push_str("Tree cover loss 2001-2024: ~32 million hectares\n");
    out.push_str("Primary driver: palm oil plantation expansion (~70%)\n");
    out.push_str("Top deforestation provinces:\n");
    out.push_str("  - Riau\n");
    out.push_str("  - Kalimantan Tengah\n");
    out.push_str("  - Kalimantan Barat\n");
    out.push_str("  - Papua\n");
    out.push_str("\nDashboard: https://www.globalforestwatch.org/dashboards/country/IDN/\n");
    out.push_str("Weekly GLAD alerts available at dashboard link above.\n");
    out
}
