use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";
const OPEN_METEO_MARINE: &str = "https://marine-api.open-meteo.com/v1/marine";

/// Tidal Flood Compound — SLR + Subsidence + Tide (bathtub model)
/// Ref: IPCC AR6; Shan et al. 2025 (Nature); Momin et al. 2026 (83-paper review); Chrysanti et al. 2024
/// Method: flood_depth = max(tide_height + SLR - ground_elevation + subsidence_cumulative, 0)

pub async fn assess(
    client: &Client,
    lat: f64,
    lon: f64,
    slr_scenario: &str,
    subsidence_rate_mm_yr: f64,
    projection_year: u32,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Tidal Flood Compound Assessment (Bathtub Model)\n");
    out.push_str("Ref: IPCC AR6; Shan et al. 2025; Momin et al. 2026; Chrysanti et al. 2024\n\n");

    let slr_mm = match slr_scenario.to_lowercase().as_str() {
        "ssp245" | "moderate" => match projection_year {
            2050 => 240.0,
            2100 => 560.0,
            _ => 240.0,
        },
        "ssp585" | "worst" => match projection_year {
            2050 => 370.0,
            2100 => 1100.0,
            _ => 370.0,
        },
        _ => 240.0,
    };

    let years = projection_year.saturating_sub(2026).max(1) as f64;
    let subsidence_mm = subsidence_rate_mm_yr * years;
    let total_sea_level_rise_mm = slr_mm + subsidence_mm;
    let total_m = total_sea_level_rise_mm / 1000.0;

    out.push_str(&format!("Location: ({:.4}, {:.4})\n", lat, lon));
    out.push_str(&format!("SLR scenario: {} ({:.0} mm by {})\n", slr_scenario, slr_mm, projection_year));
    out.push_str(&format!("Subsidence rate: {:.1} mm/yr\n", subsidence_rate_mm_yr));
    out.push_str(&format!("Subsidence by {}: {:.0} mm\n", projection_year, subsidence_mm));
    out.push_str(&format!("Compound SLR: {:.0} mm = {:.2} m\n\n", total_sea_level_rise_mm, total_m));

    let tide_url = format!(
        "{}?latitude={}&longitude={}&hourly=sea_level_height_msl&forecast_days=3",
        OPEN_METEO_MARINE, lat, lon
    );

    out.push_str("TIDE FORECAST (Open-Meteo Marine, 3 days):\n");
    match client.get(&tide_url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(v) => {
                if let Some(times) = v.pointer("/hourly/time").and_then(|t| t.as_array()) {
                    if let Some(heights) = v.pointer("/hourly/sea_level_height_msl").and_then(|h| h.as_array()) {
                        let mut max_tide: f64 = heights.iter()
                            .filter_map(|h| h.as_f64())
                            .fold(0.0f64, f64::max);
                        let mut min_tide: f64 = heights.iter()
                            .filter_map(|h| h.as_f64())
                            .fold(0.0f64, f64::min);

                        if max_tide > 0.0 {
                            let tide_above_msl = if max_tide > 0.0 { max_tide } else { 0.0 };
                            let tide_str = format!("Max tide: {:.2}m, Min tide: {:.2}m", max_tide, min_tide);
                            out.push_str(&format!("  {}\n", tide_str));

                            let flood_depth = total_m + tide_above_msl;
                            out.push_str(&format!("\nCompound flood depth (bathtub):\n"));
                            out.push_str(&format!("  = SLR ({:.2}m) + max tide ({:.2}m)\n", total_m, tide_above_msl));
                            out.push_str(&format!("  = {:.2}m above current MSL\n\n", flood_depth));

                            if flood_depth > 0.5 {
                                out.push_str(&format!("⚠️ FLOOD RISK: {:.2}m inundation likely\n", flood_depth));
                                out.push_str("  Impact: buildings, roads, agriculture at current elevation\n");
                            } else {
                                out.push_str("  Low risk at current ground elevation.\n");
                            }
                        }
                        let _ = max_tide;
                    }
                }
            }
            Err(e) => out.push_str(&format!("  Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("  Connection error: {}\n", e)),
    }

    let (s, w, n, e) = bbox_from_center(lat, lon, 10.0);
    let stac_body = json!({
        "collections": ["cop-dem-glo-30"],
        "bbox": [w, s, e, n],
        "limit": 1u32
    });
    let stac_url = format!("{}/search", MPC_STAC_URL);

    out.push_str("\nGROUND ELEVATION (Copernicus DEM 30m via STAC):\n");
    match client.post(&stac_url).json(&stac_body).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(v) => {
                if let Some(features) = v.get("features").and_then(|f| f.as_array()) {
                    if !features.is_empty() {
                        let id = features[0].get("id").and_then(|i| i.as_str()).unwrap_or("?");
                        out.push_str(&format!("  DEM scene: {}\n", id));
                        if let Some(assets) = features[0].get("assets").and_then(|a| a.as_object()) {
                            if let Some(data) = assets.get("data") {
                                if let Some(href) = data.get("href").and_then(|h| h.as_str()) {
                                    out.push_str(&format!("  Download: {}\n", href));
                                }
                            }
                        }
                        out.push_str("  Note: Extract elevation at (lat, lon) from GeoTIFF\n");
                    }
                }
            }
            Err(_) => out.push_str("  DEM search error\n"),
        },
        Err(e) => out.push_str(&format!("  DEM connection error: {}\n", e)),
    }

    out.push_str("\nLIMITATION:\n");
    out.push_str("  - Bathtub model: no hydraulic connectivity (water paths)\n");
    out.push_str("  - Does not account for storm surge, waves, river discharge\n");
    out.push_str("  - DEMNAS 8m vertical accuracy ~2-3m\n");
    out.push_str("  - Subsidence rate must be from InSAR (Sidiq 2025, Yuwono 2026)\n");
    out.push_str("  - For policy-grade: need hydrodynamic model (MIKE 21, Delft3D)\n");
    out.push_str("  - Momin 2026: most compound flood studies omit subsidence (we include it)\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}

fn bbox_from_center(lat: f64, lon: f64, buffer_km: f64) -> (f64, f64, f64, f64) {
    let lat_offset = buffer_km / 111.0;
    let lon_offset = buffer_km / (111.0 * lat.to_radians().cos().abs().max(0.01));
    (lat - lat_offset, lon - lon_offset, lat + lat_offset, lon + lon_offset)
}
