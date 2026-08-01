use reqwest::Client;

pub async fn solar(
    client: &Client,
    lat: f64,
    lon: f64,
    start: Option<String>,
    end: Option<String>,
) -> String {
    let start = start.unwrap_or_else(|| "20250101".into());
    let end = end.unwrap_or_else(|| "20251231".into());

    let url = format!(
        "https://power.larc.nasa.gov/api/temporal/monthly/point?parameters=ALLSKY_SFC_SW_DWN,CLRSKY_SFC_SW_DWN,ALLSKY_SFC_SW_DIFF,T2M&community=RE&longitude={}&latitude={}&start={}&end={}&format=JSON",
        lon, lat, &start[..6], &end[..6]
    );

    let mut out = format!(
        "=== NASA POWER Solar Irradiance ({:.4}, {:.4}) ===\n",
        lat, lon
    );
    out.push_str(&format!("Period: {} to {}\n", start, end));
    out.push_str("Source: NASA POWER (Prediction of Worldwide Energy Resources)\n\n");

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                if let Some(params) = v.get("properties").and_then(|p| p.get("parameter")) {
                    // GHI - Global Horizontal Irradiance
                    if let Some(ghi) = params.get("ALLSKY_SFC_SW_DWN").and_then(|g| g.as_object()) {
                        out.push_str("GHI (Global Horizontal Irradiance) kWh/m²/day:\n");
                        for (month, val) in ghi {
                            if let Some(v) = val.as_f64() {
                                if v > 0.0 {
                                    out.push_str(&format!("  {}: {:.2}\n", month, v));
                                }
                            }
                        }
                    }
                    // Temperature
                    if let Some(temp) = params.get("T2M").and_then(|t| t.as_object()) {
                        out.push_str("\nTemperature (°C):\n");
                        for (month, val) in temp {
                            if let Some(v) = val.as_f64() {
                                if v > -900.0 {
                                    out.push_str(&format!("  {}: {:.1}\n", month, v));
                                }
                            }
                        }
                    }
                    out.push_str("\nInterpretation:\n");
                    out.push_str("  GHI > 5.0 kWh/m²/day = Excellent solar potential\n");
                    out.push_str("  GHI 4.0-5.0 = Good\n");
                    out.push_str("  GHI 3.0-4.0 = Moderate\n");
                    out.push_str("  GHI < 3.0 = Low\n");
                    out.push_str(
                        "  Indonesia typically: 4.5-6.5 kWh/m²/day (varies by latitude/altitude)\n",
                    );
                } else {
                    out.push_str(&format!(
                        "Response: {}\n",
                        serde_json::to_string_pretty(&v)
                            .unwrap_or_default()
                            .chars()
                            .take(2000)
                            .collect::<String>()
                    ));
                }
            }
            Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}
