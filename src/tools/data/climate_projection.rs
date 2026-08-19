use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// Climate Projection — NEX-GDDP-CMIP6 downscaled (25km, bias-corrected)
/// Source: NASA Earth Exchange Global Daily Downscaled Projections (CMIP6)
/// Variables: tasmax (max temp), tasmin (min temp), pr (precipitation)
/// Scenarios: SSP2-4.5 (moderate), SSP5-8.5 (worst case)
///
/// Ref:
/// - NEX-GDDP-CMIP6: Thrasher et al. 2022 (NASA Ames Research Center)
/// - CMIP6: Eyring et al. 2016 (GMDD)
///
/// LIMITATION:
/// - 25km resolution: 1 pixel per major Indonesian city
/// - Multi-model ensemble recommended (not single model)
/// - CMIP6 model uncertainty range is significant
/// - Need NetCDF processing for actual data extraction

pub async fn search_climate_projection(
    client: &Client,
    lat: f64,
    lon: f64,
    scenario: &str,
    period: &str,
) -> String {
    let (s, w, n, e) = bbox_from_center(lat, lon, 50.0);

    let scenario_normalized = match scenario.to_uppercase().as_str() {
        "SSP245" | "SSP2-4.5" | "245" | "MODERATE" => "ssp245",
        "SSP585" | "SSP5-8.5" | "585" | "WORST" => "ssp585",
        _ => "ssp585",
    };

    let (start_year, end_year) = match period.to_lowercase().as_str() {
        "2030" | "2030s" => (2030, 2039),
        "2050" | "2050s" => (2050, 2059),
        "2080" | "2080s" | "2100" => (2080, 2100),
        _ => (2050, 2059),
    };

    let datetime_future = format!(
        "{}-01-01T00:00:00Z/{}-12-31T23:59:59Z",
        start_year, end_year
    );

    let body = json!({
        "collections": ["nasa-nex-gddp-cmip6"],
        "bbox": [w, s, e, n],
        "datetime": datetime_future,
        "limit": 10u32,
        "query": {
            "cmip6:scenario": {"eq": scenario_normalized}
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

                    let mut res = ScientificResult::new("cmip6_dataset_matched", matched, "count")
                        .with_status(ResultStatus::Valid)
                        .with_provenance(Provenance::new("database", "MPC_NEX_GDDP_CMIP6", "2026-08-19T00:00:00Z"))
                        .with_claim(Claim::new("scenario", scenario_normalized))
                        .with_claim(Claim::new("period", &format!("{}-{}", start_year, end_year)));

                    if let Some(features) = features {
                        if !features.is_empty() {
                            let mut models: Vec<String> = Vec::new();
                            for (i, feat) in features.iter().take(10).enumerate() {
                                let props = feat.get("properties").unwrap_or(&Value::Null);
                                let model = props.get("cmip6:model").and_then(|m| m.as_str()).unwrap_or("?");
                                let var = props.get("cmip6:variable").and_then(|v| v.as_str()).unwrap_or("?");
                                
                                if !models.contains(&model.to_string()) {
                                    models.push(model.to_string());
                                }
                                
                                res = res.with_claim(Claim::new(&format!("dataset_{}_{}", i, var), model));
                            }
                            res = res.with_claim(Claim::new("models_found", &models.join(", ")));
                        } else {
                            res = res.with_claim(Claim::new("warning", "No datasets found in this bbox/period."));
                        }
                    }
                    
                    res = res.with_claim(Claim::new("limitation", "25km resolution. Requires NetCDF parsing for actual data extraction."));
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

fn bbox_from_center(lat: f64, lon: f64, buffer_km: f64) -> (f64, f64, f64, f64) {
    let lat_offset = buffer_km / 111.0;
    let lon_offset = buffer_km / (111.0 * lat.to_radians().cos().abs().max(0.01));
    (lat - lat_offset, lon - lon_offset, lat + lat_offset, lon + lon_offset)
}
