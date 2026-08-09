use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// Flood SAR Mapping — Sentinel-1 VV change detection (2026 SOTA DL methods)
/// Methodology: download S1 GRD pre/post flood, VV threshold, change detection
/// Ref: Clement et al. 2025; Twele et al. 2016; Cian et al. 2018
/// 2026 SOTA: Siamese U-Net (Kacmaz 2026, F1=96%); TLE-FEDformer (Ahmadi 2026, 98.1%)
///   LightFloodNet (Kinalioglu 2026, 1.57M params); CMFS-UNet Mamba (Wei 2025)
///   FloodsNet (Wu 2025); RS-Mamba (Gierszewska 2026)
///
/// LIMITATION:
/// - 6-day revisit (not real-time)
/// - VV threshold -17dB is generic; adaptive Otsu better for Indonesia (rice fields, mangrove)
/// - False positive: rice paddies, shadows, wind-roughened water
/// - Cannot detect flooding under dense vegetation canopy (radar penetration limited)

pub async fn search_flood_scenes(
    client: &Client,
    lat: f64,
    lon: f64,
    buffer_km: f64,
    flood_date: &str,
) -> String {
    let (s, w, n, e) = bbox_from_center(lat, lon, buffer_km);

    let flood_dt = chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", flood_date))
        .unwrap_or_else(|_| chrono::Utc::now().into());
    let post_start = flood_dt - chrono::Duration::days(3);
    let post_end = flood_dt + chrono::Duration::days(3);
    let pre_start = flood_dt - chrono::Duration::days(18);
    let pre_end = flood_dt - chrono::Duration::days(6);

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
    out.push_str("SAR Flood Mapping — Sentinel-1 Change Detection\n");
    out.push_str(&format!("Center: ({:.4}, {:.4}), Buffer: {:.1} km\n", lat, lon, buffer_km));
    out.push_str(&format!("Flood event date: {}\n", flood_date));
    out.push_str(&format!("BBox: {:.4},{:.4},{:.4},{:.4} (W,S,E,N)\n\n", w, s, e, n));
    out.push_str("Ref: Twele et al. 2016; Cian et al. 2018; Clement et al. 2025\n");
    out.push_str("Method: S1 GRD VV threshold + change detection + DEMNAS mask\n\n");

    let pre_body = json!({
        "collections": ["sentinel-1-grd"],
        "bbox": [w, s, e, n],
        "datetime": datetime_pre,
        "limit": 5u32
    });
    let post_body = json!({
        "collections": ["sentinel-1-grd"],
        "bbox": [w, s, e, n],
        "datetime": datetime_post,
        "limit": 5u32
    });

    out.push_str(&format!("PRE-FLOOD window: {} (12 days before)\n", datetime_pre));
    let pre_count = search_and_format(client, &pre_body, "PRE-FLOOD", &mut out).await;

    out.push_str(&format!("\nPOST-FLOOD window: {} (3 days after)\n", datetime_post));
    let post_count = search_and_format(client, &post_body, "POST-FLOOD", &mut out).await;

    out.push_str("\n═══════════════════════════════════════════════\n");
    out.push_str("FLOOD ANALYSIS PROTOCOL:\n");
    out.push_str("1. Download PRE and POST VV GeoTIFF from URLs above\n");
    out.push_str("2. Apply speckle filter (Lee 5x5 or Refined Lee)\n");
    out.push_str("3. Convert dB to sigma0 (backscatter)\n");
    out.push_str("4. Threshold: water = VV < -17 dB (or Otsu adaptive)\n");
    out.push_str("5. Change detection: POST_water - PRE_water = FLOOD EXTENT\n");
    out.push_str("6. Mask permanent water (DEMNAS + JRC Global Surface Water)\n");
    out.push_str("7. Overlay OSM settlements within flood extent\n");
    out.push_str("\n");
    out.push_str("2026 SOTA DEEP LEARNING METHODS:\n");
    out.push_str("  Method              F1/IoU     Params    Ref\n");
    out.push_str("  ------              ------     ------    ---\n");
    out.push_str("  Siamese U-Net       F1=96.1%   -         Kacmaz 2026 (Earth 7(3))\n");
    out.push_str("  TLE-FEDformer       98.1%/97.4% -        Ahmadi 2026 (RS 18(6))\n");
    out.push_str("  LightFloodNet       IoU=0.54   1.57M     Kinalioglu 2026 (Tuzal)\n");
    out.push_str("  CMFS-UNet Mamba     mIoU=79.4% -         Wei 2025 (PIERS)\n");
    out.push_str("  FloodsNet           F1+1-2%    -         Wu 2025 (RS 17(16))\n");
    out.push_str("  RS-Mamba            mIoU=56.6% -         Gierszewska 2026 (JSTARS)\n");
    out.push_str("  RF VH/VV ratio      94% acc    -         Amer 2025 (RS 17(11))\n");
    out.push_str("  DAM-Net             IoU=93.2%  -         benchmark (S1GFloods)\n");
    out.push_str("\n");
    out.push_str("  Recommended: Siamese U-Net for emergency response (high recall)\n");
    out.push_str("  Recommended: TLE-FEDformer for accuracy (multi-sensor fusion)\n");
    out.push_str("  Recommended: LightFloodNet for edge deployment (1.57M params)\n");
    out.push_str("\n");

    if pre_count > 0 && post_count > 0 {
        out.push_str("✅ Both pre and post scenes available.\n");
        out.push_str("   Ready for flood extent mapping.\n\n");
        out.push_str("LIMITATION:\n");
        out.push_str("  - Sentinel-1 revisit: 6 days (not real-time)\n");
        out.push_str("  - VV threshold -17dB is generic — false positive in:\n");
        out.push_str("    * Rice paddies (similar low backscatter)\n");
        out.push_str("    * Mangrove/marsh areas (radar attenuation)\n");
        out.push_str("    * Wind-roughened water (increases backscatter)\n");
        out.push_str("  - Cannot detect flooding under dense canopy\n");
        out.push_str("  - DEMNAS 8m mask needed to exclude permanent water\n");
    } else if pre_count == 0 {
        out.push_str("⚠️ No pre-flood scene found in window.\n");
        out.push_str("   Widen the date range or check cloud-free period.\n");
    } else {
        out.push_str("⚠️ No post-flood scene found in window.\n");
        out.push_str("   Sentinel-1 may not have passed yet (6-day revisit).\n");
        out.push_str("   Check again in 1-6 days.\n");
    }

    out.push_str("═══════════════════════════════════════════════\n");
    out
}

async fn search_and_format(
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
                            let orbit_dir = props.get("sat:orbit_state")
                                .and_then(|o| o.as_str()).unwrap_or("?");

                            out.push_str(&format!(
                                "  {}. ID: {}\n     DATE: {}\n     ORBIT: {}\n",
                                i + 1, id, datetime, orbit_dir
                            ));

                            if let Some(assets) = feat.get("assets").and_then(|a| a.as_object()) {
                                let asset_keys: Vec<&str> = assets.keys().map(|k| k.as_str()).collect();
                                out.push_str(&format!("     ASSETS: {}\n", asset_keys.join(", ")));

                                for (key, val) in assets.iter() {
                                    let title = val.get("title").and_then(|t| t.as_str()).unwrap_or("");
                                    if key == "vv" || key == "data" || title.contains("VV") || title.contains("backscatter") {
                                        if let Some(href) = val.get("href").and_then(|h| h.as_str()) {
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
