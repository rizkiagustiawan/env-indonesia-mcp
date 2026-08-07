use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// GPM IMERG Rainfall — 30-min precipitation (0.1°, ~10km)
/// Ref: Watters et al. 2025 (NASA GPM team); Setiyowati et al. 2025; Lufira et al. 2026
/// WARNING: IMERG underestimate -41% in tropics (Watters 2025)
/// Hourly correlation r=0.10 (Setiyowati 2025) — unreliable for flood real-time

pub async fn query(
    client: &Client,
    lat: f64,
    lon: f64,
    date: &str,
) -> String {
    let (s, w, n, e) = bbox_from_center(lat, lon, 15.0);
    let start = format!("{}T00:00:00Z", date);
    let end = format!("{}T23:59:59Z", date);

    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("GPM IMERG Rainfall — 30-min Precipitation\n");
    out.push_str("Ref: Watters et al. 2025 (NASA GPM); Setiyowati 2025; Lufira 2026\n\n");

    out.push_str(&format!("Location: ({:.4}, {:.4})\n", lat, lon));
    out.push_str(&format!("Date: {}\n\n", date));

    let body = json!({
        "collections": ["gpm-imerg-hhr"],
        "bbox": [w, s, e, n],
        "datetime": format!("{}/{}", start, end),
        "limit": 5u32
    });
    let url = format!("{}/search", MPC_STAC_URL);

    out.push_str("STAC SEARCH:\n");
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
                        out.push_str("No IMERG scenes found. Try different date or wider bbox.\n");
                        return out;
                    }

                    if let Some(features) = features {
                        for (i, feat) in features.iter().take(5).enumerate() {
                            let id = feat.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                            let props = feat.get("properties").unwrap_or(&Value::Null);
                            let datetime = props.get("datetime")
                                .and_then(|d| d.as_str()).unwrap_or("?");

                            out.push_str(&format!("{}. ID: {}\n   DATE: {}\n", i + 1, id, datetime));

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
                }
                Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
            }
        }
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }

    out.push_str("PRECIPITATION ANALYSIS PROTOCOL:\n");
    out.push_str("  1. Download IMERG NetCDF/HDF5 from URL above\n");
    out.push_str("  2. Extract precipitation at nearest pixel to (lat, lon)\n");
    out.push_str("  3. Convert units: kg/m²/s → mm/hr (× 3600)\n");
    out.push_str("  4. Compute: cumulative (mm/day), intensity (mm/hr), duration\n\n");

    out.push_str("⚠️ CRITICAL LIMITATION (Watters et al. 2025, NASA GPM team):\n");
    out.push_str("  - IMERG underestimate heavy rain by 41% in tropics\n");
    out.push_str("  - Hourly correlation r=0.10 (Setiyowati 2025) — very poor\n");
    out.push_str("  - Daily correlation r=0.24 (moderate)\n");
    out.push_str("  - Monthly correlation r=0.84 (good for seasonal)\n");
    out.push_str("  → USE for monthly/seasonal, NOT for flood real-time\n");
    out.push_str("  → For flood: use Sentinel-1 SAR or BMKG telemetri\n\n");

    out.push_str("BIAS CORRECTION (Lufira 2026):\n");
    out.push_str("  - Linear Scaling (LS) best: NSE=0.87, R=0.92\n");
    out.push_str("  - Requires BMKG ground station data for calibration\n");
    out.push_str("  - Without correction: add +24% bias correction factor (monthly)\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}

fn bbox_from_center(lat: f64, lon: f64, buffer_km: f64) -> (f64, f64, f64, f64) {
    let lat_offset = buffer_km / 111.0;
    let lon_offset = buffer_km / (111.0 * lat.to_radians().cos().abs().max(0.01));
    (lat - lat_offset, lon - lon_offset, lat + lat_offset, lon + lon_offset)
}
