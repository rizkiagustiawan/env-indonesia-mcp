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
        "SSP245" | "SSP2-4.5" | "SSP245" | "245" | "MODERATE" => "ssp245",
        "SSP585" | "SSP5-8.5" | "SSP585" | "585" | "WORST" => "ssp585",
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

    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Climate Projection — NEX-GDDP-CMIP6\n");
    out.push_str("NASA Earth Exchange Global Daily Downscaled Projections\n");
    out.push_str(&format!("Center: ({:.4}, {:.4})\n", lat, lon));
    out.push_str(&format!("Scenario: {} ({})\n", scenario_normalized, scenario_label(scenario_normalized)));
    out.push_str(&format!("Period: {}-{}\n", start_year, end_year));
    out.push_str(&format!("BBox: {:.2},{:.2},{:.2},{:.2} (50km buffer)\n\n", w, s, e, n));
    out.push_str("Ref: Thrasher et al. 2022 (NASA Ames); Eyring et al. 2016 (CMIP6)\n");
    out.push_str("Source: NASA Earth Exchange via Microsoft Planetary Computer\n\n");

    let body = json!({
        "collections": ["nasa-nex-gddp-cmip6"],
        "bbox": [w, s, e, n],
        "datetime": datetime_future,
        "limit": 10u32,
        "query": {
            "cmip6:scenario": {"eq": scenario_normalized}
        }
    });

    out.push_str(&format!("STAC search: nasa-nex-gddp-cmip6, scenario={}\n", scenario_normalized));
    out.push_str(&format!("Datetime: {}\n\n", datetime_future));

    let url = format!("{}/search", MPC_STAC_URL);
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

                    out.push_str(&format!("Matched: {} scenes\n", matched));

                    if let Some(features) = features {
                        if features.is_empty() {
                            out.push_str("No scenes found.\n");
                            return out;
                        }

                        let mut models: Vec<String> = Vec::new();
                        let mut scenarios_found: Vec<String> = Vec::new();

                        for (i, feat) in features.iter().take(10).enumerate() {
                            let id = feat.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                            let props = feat.get("properties").unwrap_or(&Value::Null);

                            let model = props.get("cmip6:model")
                                .and_then(|m| m.as_str())
                                .unwrap_or("?");
                            let scen = props.get("cmip6:scenario")
                                .and_then(|s| s.as_str())
                                .unwrap_or("?");
                            let var = props.get("cmip6:variable")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?");
                            let grid_label = props.get("cmip6:grid_label")
                                .and_then(|g| g.as_str())
                                .unwrap_or("?");
                            let member = props.get("cmip6:member")
                                .and_then(|m| m.as_str())
                                .unwrap_or("?");

                            if !models.contains(&model.to_string()) {
                                models.push(model.to_string());
                            }
                            if !scenarios_found.contains(&scen.to_string()) {
                                scenarios_found.push(scen.to_string());
                            }

                            out.push_str(&format!(
                                "{}. ID: {}\n   MODEL: {}\n   SCENARIO: {}\n   VARIABLE: {}\n   GRID: {} MEMBER: {}\n",
                                i + 1, id, model, scen, var, grid_label, member
                            ));

                            if let Some(assets) = feat.get("assets").and_then(|a| a.as_object()) {
                                for (key, val) in assets.iter() {
                                    if let Some(href) = val.get("href").and_then(|h| h.as_str()) {
                                        out.push_str(&format!("   {} URL: {}\n", key, href));
                                    }
                                }
                            }
                        }

                        out.push_str("\n═══════════════════════════════════════════════\n");
                        out.push_str("CLIMATE ANALYSIS PROTOCOL:\n");
                        out.push_str("\nVariables:\n");
                        out.push_str("  tasmax = daily maximum air temperature (°K → convert to °C: K - 273.15)\n");
                        out.push_str("  tasmin = daily minimum air temperature (°K → convert to °C: K - 273.15)\n");
                        out.push_str("  pr = daily precipitation (kg/m²/s → mm/day: × 86400)\n\n");

                        out.push_str("1. Download NetCDF for selected model/scenario/variable\n");
                        out.push_str("2. Extract time series for pixel nearest to (lat, lon)\n");
                        out.push_str("3. Compute baseline (1995-2014) mean for same season\n");
                        out.push_str("4. Compute anomaly: future_period_mean - baseline_mean\n");
                        out.push_str("5. Output: temperature change (°C), precipitation change (%)\n");
                        out.push_str("\n");

                        out.push_str(&format!("Models available: {} ({})\n", models.len(), models.join(", ")));
                        out.push_str(&format!("Scenarios: {}\n\n", scenarios_found.join(", ")));

                        out.push_str("LIMITATION:\n");
                        out.push_str("  - 25km resolution: 1 pixel per major city (not neighborhood-level)\n");
                        out.push_str("  - Multi-model ensemble recommended (not single model)\n");
                        out.push_str("  - CMIP6 model uncertainty: ±20% precipitation, ±1°C temperature\n");
                        out.push_str("  - Requires NetCDF processing (xarray or netcdf-reader)\n");
                        out.push_str("  - SSP5-8.5 is worst case (RCP8.5 equivalent, unlikely but precautionary)\n");
                        out.push_str("\n");

                        out.push_str("SCENARIO CONTEXT:\n");
                        out.push_str("  SSP2-4.5: Moderate. ~3°C global warming by 2100. Current policy trajectory.\n");
                        out.push_str("  SSP5-8.5: Worst case. ~4.5°C global warming by 2100. Fossil-fueled development.\n");
                        out.push_str("\n");

                        out.push_str("INDONESIA-SPECIFIC:\n");
                        out.push_str("  - Jakarta: projected +1.5-3°C by 2050 (SSP5-8.5)\n");
                        out.push_str("  - Extreme rainfall: +10-30% intensity increase\n");
                        out.push_str("  - Sea level rise: 0.3-1.0m by 2100 (combined with subsidence = worse)\n");
                        out.push_str("  - Dry season lengthening: +15-30 days in Java/Sumatera\n");
                    } else {
                        out.push_str("No features in response.\n");
                    }
                }
                Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
            }
        }
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }

    out.push_str("═══════════════════════════════════════════════\n");
    out
}

fn scenario_label(scenario: &str) -> &'static str {
    match scenario {
        "ssp245" => "Moderate — ~3°C warming by 2100",
        "ssp585" => "Worst case — ~4.5°C warming by 2100",
        _ => "Unknown",
    }
}

fn bbox_from_center(lat: f64, lon: f64, buffer_km: f64) -> (f64, f64, f64, f64) {
    let lat_offset = buffer_km / 111.0;
    let lon_offset = buffer_km / (111.0 * lat.to_radians().cos().abs().max(0.01));
    (lat - lat_offset, lon - lon_offset, lat + lat_offset, lon + lon_offset)
}
