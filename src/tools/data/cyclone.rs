use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// Tropical Cyclone Track — ECMWF Trajectory Forecast
/// Ref: Yang et al. 2025/2026 (ECMWF benchmark); DeMaria et al. 2025
/// ECMWF STAC: type='tf' = tropical cyclone track trajectory
/// Track error: ~100-350km at 24h, ~200-600km at 120h (Yang 2026)

pub async fn search(
    client: &Client,
    _lat: f64,
    _lon: f64,
) -> String {
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

    let mut results = vec![];

    match client.post(&url).json(&body).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return json!([{"status": "validation_failed", "error": format!("Search error: HTTP {}", resp.status())}]).to_string();
            }
            match resp.json::<Value>().await {
                Ok(v) => {
                    let features = v.get("features").and_then(|f| f.as_array());
                    let matched = v.get("context")
                        .and_then(|c| c.get("matched"))
                        .and_then(|m| m.as_f64())
                        .unwrap_or(0.0);
                        
                    let count = features.map(|f| f.len()).unwrap_or(0);

                    let mut res = ScientificResult::new("active_cyclone_tracks", matched, "count")
                        .with_status(ResultStatus::Valid)
                        .with_provenance(Provenance::new("database", "ECMWF_Forecast", "2026-08-19T00:00:00Z"))
                        .with_claim(Claim::new("search_period", &format!("{} to {}", date_str, date_end)));

                    if count == 0 {
                        res = res.with_claim(Claim::new("status", "No active tropical cyclone tracks found"));
                        res = res.with_claim(Claim::new("observation", "Indonesia rarely experiences tropical cyclones. Last significant: Seroja (2021)."));
                    } else if let Some(features) = features {
                        for (i, feat) in features.iter().take(10).enumerate() {
                            let props = feat.get("properties").unwrap_or(&Value::Null);
                            let ref_time = props.get("ecmwf:reference_datetime")
                                .and_then(|r| r.as_str()).unwrap_or("?");
                            
                            res = res.with_claim(Claim::new(&format!("cyclone_track_{}", i), &format!("Ref_Time: {}", ref_time)));
                        }
                    }
                    
                    res = res.with_claim(Claim::new("limitation", "Track error: 100-350km at 24h, 200-600km at 120h (Yang 2026). GRIB2 parsing required for actual coordinates."));
                    results.push(res);
                }
                Err(e) => return json!([{"status": "validation_failed", "error": format!("Parse error: {}", e)}]).to_string(),
            }
        }
        Err(e) => return json!([{"status": "validation_failed", "error": format!("Connection error: {}", e)}]).to_string(),
    }

    let json_array: Vec<serde_json::Value> = results.iter()
        .map(|r| serde_json::from_str(&r.clone().emit_validated()).unwrap())
        .collect();

    json!(json_array).to_string()
}
