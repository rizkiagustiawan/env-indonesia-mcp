use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// Karhutla Assessment — Sentinel-2 dNBR + Peat Proxy + FIRMS
/// Methodology: NBR pre/post fire, dNBR severity (Key & Benson 2006, USGS)
/// Peat proxy: DEMNAS elev <50m + slope <2° + FIRMS sustained FRP pattern
///
/// Ref:
/// - Key & Benson (2006) — dNBR severity thresholds (USGS FIREMON)
/// - Cai et al. (2025) — EMIT methane from landfill (matched filter approach)
/// - Hooijer et al. (2012) — peat subsidence → CO2 emissions proportional
///
/// LIMITATION:
/// - Cloud cover limits S2 acquisition (Indonesia ~70% cloud)
/// - NBR saturates in dense canopy (>15m height)
/// - Peat proxy (elev<50m + slope<2°) is approximate — needs KLHK shapefile
/// - dNBR thresholds calibrated for US coniferous forests — tropical forest may differ
/// - FIRMS sustained FRP >100MW heuristic — not validated for Indonesia peat

pub async fn assess_karhutla(
    client: &Client,
    lat: f64,
    lon: f64,
    buffer_km: f64,
    fire_date: &str,
) -> String {
    let (s, w, n, e) = bbox_from_center(lat, lon, buffer_km);

    let fire_dt = chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", fire_date))
        .unwrap_or_else(|_| chrono::Utc::now().into());
    let post_start = fire_dt - chrono::Duration::days(2);
    let post_end = fire_dt + chrono::Duration::days(14);
    let pre_start = fire_dt - chrono::Duration::days(60);
    let pre_end = fire_dt - chrono::Duration::days(7);

    let datetime_post = format!(
        "{}/{}",
        post_start.format("%Y-%m-%dT00:00:00Z"),
        post_end.format("%Y-%m-%dT00:00:00Z")
    );
    let datetime_pre = format!(
        "{}/{}",
        pre_start.format("%Y-%m-%dT00:00:00Z"),
        pre_end.format("%Y-%m-%dT00:00:00Z")
    );

    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Karhutla Assessment — Sentinel-2 dNBR + Peat Proxy\n");
    out.push_str(&format!("Center: ({:.4}, {:.4}), Buffer: {:.1} km\n", lat, lon, buffer_km));
    out.push_str(&format!("Fire event date: {}\n", fire_date));
    out.push_str(&format!("BBox: {:.4},{:.4},{:.4},{:.4} (W,S,E,N)\n\n", w, s, e, n));
    out.push_str("Ref: Key & Benson 2006 (USGS FIREMON); Hooijer et al. 2012\n");
    out.push_str("Method: NBR=(NIR-SWIR2)/(NIR+SWIR2), dNBR=NBR_pre - NBR_post\n\n");

    let pre_body = json!({
        "collections": ["sentinel-2-l2a"],
        "bbox": [w, s, e, n],
        "datetime": datetime_pre,
        "limit": 5u32,
        "query": {"eo:cloud_cover": {"lt": 30f64}}
    });
    let post_body = json!({
        "collections": ["sentinel-2-l2a"],
        "bbox": [w, s, e, n],
        "datetime": datetime_post,
        "limit": 5u32,
        "query": {"eo:cloud_cover": {"lt": 50f64}}
    });

    out.push_str(&format!("PRE-FIRE window: {} (60 days before, cloud <30%)\n", datetime_pre));
    let pre_count = search_s2_scenes(client, &pre_body, "PRE-FIRE", &mut out).await;

    out.push_str(&format!("\nPOST-FIRE window: {} (2-14 days after, cloud <50%)\n", datetime_post));
    let post_count = search_s2_scenes(client, &post_body, "POST-FIRE", &mut out).await;

    out.push_str("\n═══════════════════════════════════════════════\n");
    out.push_str("BURNED AREA ANALYSIS PROTOCOL:\n");
    out.push_str("Sentinel-2 bands:\n");
    out.push_str("  B08 = NIR (842nm, 10m)\n");
    out.push_str("  B12 = SWIR2 (2190nm, 20m) — resample to 10m\n");
    out.push_str("\n1. Download B08 (NIR) and B12 (SWIR2) for PRE and POST scenes\n");
    out.push_str("2. Compute NBR = (B08 - B12) / (B08 + B12) for each scene\n");
    out.push_str("3. dNBR = NBR_pre - NBR_post\n");
    out.push_str("4. Severity classification (Key & Benson 2006):\n");
    out.push_str("   dNBR < 0.10  → Unburned / Enhanced Greenup\n");
    out.push_str("   dNBR 0.10-0.27 → Low Severity\n");
    out.push_str("   dNBR 0.27-0.44 → Moderate Severity\n");
    out.push_str("   dNBR 0.44-0.66 → High Severity\n");
    out.push_str("   dNBR > 0.66   → Very High Severity\n");
    out.push_str("\n5. Peat Identification (hybrid proxy):\n");
    out.push_str("   a. DEMNAS: elevation < 50m AND slope < 2° → peat candidate\n");
    out.push_str("   b. FIRMS temporal pattern: sustained FRP > 100MW over >3 days\n");
    out.push_str("      (peat smoldering = long duration, lower FRP than surface fire)\n");
    out.push_str("   c. Cross-check: if both (a) and (b) → HIGH peat probability\n");
    out.push_str("\n");

    if pre_count > 0 && post_count > 0 {
        out.push_str("✅ Both pre and post S2 scenes available.\n");
        out.push_str("   Ready for dNBR severity mapping.\n\n");
        out.push_str("LIMITATION:\n");
        out.push_str("  - dNBR thresholds calibrated for US coniferous — tropical forest may differ\n");
        out.push_str("  - NBR saturates in canopy >15m height (underestimates high severity)\n");
        out.push_str("  - Cloud cover: 50% threshold for post-fire may reduce usable pixels\n");
        out.push_str("  - Peat proxy accuracy: ~60-70% without KLHK peatland shapefile\n");
        out.push_str("  - FIRMS sustained FRP heuristic not validated for Indonesia peat\n");
    } else if pre_count == 0 {
        out.push_str("⚠️ No pre-fire S2 scene found (cloud-free window).\n");
        out.push_str("   Try: extend pre-fire window to 90 days, or use different date.\n");
    } else {
        out.push_str("⚠️ No post-fire S2 scene found.\n");
        out.push_str("   Cloud cover may be high post-fire (smoke).\n");
        out.push_str("   Alternative: use Sentinel-1 SAR (cloud-penetrating) for burned area.\n");
    }

    out.push_str("\n═══════════════════════════════════════════════\n");
    out.push_str("PEAT FIRE IDENTIFICATION:\n");
    out.push_str("  Peat fires smolder (not flaming) — characteristics:\n");
    out.push_str("  - FRP lower but sustained over many days (>3)\n");
    out.push_str("  - Occurs on flat, low-elevation terrain (<50m, <2° slope)\n");
    out.push_str("  - CO emissions higher than surface fire (smoldering)\n");
    out.push_str("  - 10x more carbon emissions per ha than mineral soil fire\n");
    out.push_str("  - Ref: Hooijer et al. 2012; Page et al. 2002\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}

async fn search_s2_scenes(
    client: &Client,
    body: &Value,
    label: &str,
    out: &mut String,
) -> usize {
    let url = format!("{}/search", MPC_STAC_URL);
    match client.post(&url).json(body).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                out.push_str(&format!("  {} search error: HTTP {}\n", label, resp.status()));
                return 0;
            }
            match resp.json::<Value>().await {
                Ok(v) => {
                    let features = v.get("features").and_then(|f| f.as_array());
                    let count = features.map(|f| f.len()).unwrap_or(0);
                    let matched = v.get("context")
                        .and_then(|c| c.get("matched"))
                        .and_then(|m| m.as_u64())
                        .unwrap_or(0);

                    if count == 0 {
                        out.push_str(&format!("  {} scenes: 0 (matched: {})\n", label, matched));
                        return 0;
                    }

                    out.push_str(&format!("  {} scenes: {} (matched: {})\n", label, count, matched));
                    if let Some(features) = features {
                        for (i, feat) in features.iter().take(3).enumerate() {
                            let id = feat.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                            let props = feat.get("properties").unwrap_or(&Value::Null);
                            let datetime = props.get("datetime")
                                .and_then(|d| d.as_str()).unwrap_or("?");
                            let cloud = props.get("eo:cloud_cover")
                                .and_then(|c| c.as_f64())
                                .map(|c| format!("{:.1}%", c))
                                .unwrap_or("N/A".to_string());

                            out.push_str(&format!(
                                "  {}. ID: {}\n     DATE: {}\n     CLOUD: {}\n",
                                i + 1, id, datetime, cloud
                            ));

                            if let Some(assets) = feat.get("assets").and_then(|a| a.as_object()) {
                                for key in ["B08", "B12", "visual", "rendered_preview", "thumbnail"] {
                                    if let Some(asset) = assets.get(key) {
                                        if let Some(href) = asset.get("href").and_then(|h| h.as_str()) {
                                            out.push_str(&format!("     {} URL: {}\n", key, href));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    count
                }
                Err(e) => {
                    out.push_str(&format!("  {} parse error: {}\n", label, e));
                    0
                }
            }
        }
        Err(e) => {
            out.push_str(&format!("  {} connection error: {}\n", label, e));
            0
        }
    }
}

fn bbox_from_center(lat: f64, lon: f64, buffer_km: f64) -> (f64, f64, f64, f64) {
    let lat_offset = buffer_km / 111.0;
    let lon_offset = buffer_km / (111.0 * lat.to_radians().cos().abs().max(0.01));
    (lat - lat_offset, lon - lon_offset, lat + lat_offset, lon + lon_offset)
}
