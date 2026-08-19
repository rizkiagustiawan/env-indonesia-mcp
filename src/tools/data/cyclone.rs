use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// Tropical Cyclone Track — ECMWF Trajectory Forecast
/// Ref: Yang et al. 2025/2026 (ECMWF benchmark); DeMaria et al. 2025
/// ECMWF STAC: type='tf' = tropical cyclone track trajectory
/// Track error: ~100-350km at 24h, ~200-600km at 120h (Yang 2026)

pub async fn search(
    client: &Client,
    lat: f64,
    lon: f64,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Tropical Cyclone Track — ECMWF Forecast\n");
    out.push_str("Ref: Yang et al. 2025/2026; DeMaria et al. 2025\n");
    out.push_str("Source: ECMWF Open Data via STAC MPC (CC-BY-4.0)\n\n");

    out.push_str(&format!("Location: ({:.4}, {:.4})\n\n", lat, lon));

    let now = chrono::Utc::now();
    let date_str = now.format("%Y-%m-%dT00:00:00Z").to_string();
    let date_end = (now + chrono::Duration::days(10)).format("%Y-%m-%dT23:59:59Z").to_string();

    let body = json!({
        "collections": ["ecmwf-forecast"],
        "bbox": [-180.0, -90.0, 180.0, 90.0],
        "datetime": format!("{}/{}", date_str, date_end),
        "limit": 20u32,
        "query": {
            "ecmwf:type": {"eq": "tf"}
        }
    });
    let url = format!("{}/search", MPC_STAC_URL);

    out.push_str("STAC SEARCH (ECMWF type=tf, trajectory forecast):\n");
    out.push_str(&format!("Datetime: {} to {}\n\n", date_str, date_end));

    match client.post(&url).json(&body).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                out.push_str(&format!("Search error: HTTP {}\n", resp.status()));
                return out;
            }
            match resp.json::<Value>().await {
                Ok(v) => {
                    let features = v.get("features").and_then(|f| f.as_array());
                    let matched = v.get("context")
                        .and_then(|c| c.get("matched"))
                        .and_then(|m| m.as_u64())
                        .unwrap_or(0);
                    let count = features.map(|f| f.len()).unwrap_or(0);

                    out.push_str(&format!("Matched: {} scenes\n", matched));
                    out.push_str(&format!("Returned: {}\n\n", count));

                    if count == 0 {
                        out.push_str("No active tropical cyclone tracks found.\n");
                        out.push_str("This means no tropical cyclone is currently being tracked.\n\n");
                        out.push_str("Note: Indonesia rarely experiences tropical cyclones.\n");
                        out.push_str("Last significant: Seroja (2021) affecting NTT/Maluku.\n");
                        return out;
                    }

                    if let Some(features) = features {
                        for (i, feat) in features.iter().take(10).enumerate() {
                            let id = feat.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                            let props = feat.get("properties").unwrap_or(&Value::Null);
                            let _datetime = props.get("datetime")
                                .and_then(|d| d.as_str()).unwrap_or("?");
                            let ref_time = props.get("ecmwf:reference_datetime")
                                .and_then(|r| r.as_str()).unwrap_or("?");
                            let forecast_time = props.get("ecmwf:forecast_datetime")
                                .and_then(|f| f.as_str()).unwrap_or("?");
                            let step = props.get("ecmwf:step")
                                .and_then(|s| s.as_str()).unwrap_or("?");
                            let stream = props.get("ecmwf:stream")
                                .and_then(|s| s.as_str()).unwrap_or("?");

                            out.push_str(&format!(
                                "{}. ID: {}\n   REF: {} | FORECAST: {} | STEP: {} | STREAM: {}\n",
                                i + 1, id, ref_time, forecast_time, step, stream
                            ));

                            if let Some(assets) = feat.get("assets").and_then(|a| a.as_object()) {
                                for (key, val) in assets.iter() {
                                    if let Some(href) = val.get("href").and_then(|h| h.as_str()) {
                                        out.push_str(&format!("   {}: {}\n", key, href));
                                    }
                                }
                            }
                            out.push('\n');
                        }
                    }

                    out.push_str("CYCLONE TRACK ANALYSIS PROTOCOL:\n");
                    out.push_str("  1. Download GRIB2 file from URL above\n");
                    out.push_str("  2. Parse with Python (cfgrib) or Rust (grib crate)\n");
                    out.push_str("  3. Extract: cyclone center lat/lon, MSLP, wind speed\n");
                    out.push_str("  4. Track positions over time → forecast path\n");
                    out.push_str("  5. Saffir-Simpson category: <34kt=TD, 34-63kt=TS, >64kt=TC\n");
                    out.push_str("  6. Landfall probability for Indonesia\n\n");
                }
                Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
            }
        }
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }

    out.push_str("LIMITATION:\n");
    out.push_str("  - Track error: 100-350km at 24h, 200-600km at 120h (Yang 2026)\n");
    out.push_str("  - ECMWF data only previous 30 days (real-time)\n");
    out.push_str("  - GRIB2 parsing needed (Python cfgrib or Rust grib crate)\n");
    out.push_str("  - Indonesia rarely affected by TC (mainly NTT/Maluku)\n");
    out.push_str("  - BMKG is authoritative source for Indonesia warnings\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}
