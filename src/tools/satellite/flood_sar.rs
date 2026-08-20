use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// Flood SAR Mapping — Sentinel-1 scene search.
///
/// IMPORTANT: This tool ONLY searches for Sentinel-1 GRD scenes (pre/post flood)
/// via Planetary Computer STAC. It does NOT run any flood-segmentation model.
/// Accuracy figures in the output are values REPORTED IN THE CITED PAPERS on
/// their own datasets, not results produced by or achievable through this tool.
///
/// Methodology (described, not all executed here): download S1 GRD pre/post
/// flood, VV threshold, change detection.
///
/// Verified benchmarks quoted in the output (see `src/citations.rs`):
///   Bonafilia et al. 2020 Sen1Floods11, DOI 10.1109/cvprw50498.2020.00113
///   Bai et al. 2021 S1+S2 fusion, DOI 10.3390/rs13112220
///   Aldiansyah et al. 2024 Kendari, DOI 10.23960/jgrs.ft.unila.205
///   Bereczky et al. 2022 CNN vs rule-based, DOI 10.1109/jstars.2022.3152127
///   Amitrano et al. 2024 review, DOI 10.3390/rs16040656
///
/// Previously this file quoted F1 96.1% (Siamese U-Net) and 98.1%
/// (TLE-FEDformer) attributed to citations that could not be located in
/// Crossref, OpenAlex or arXiv, and which exceeded every verified benchmark.
/// Those rows were removed; the tokens are recorded in `citations::UNVERIFIED`.
///
/// LIMITATION:
/// - 6-day revisit (not real-time)
/// - VV threshold -17dB is generic; adaptive Otsu better for Indonesia (rice fields, mangrove)
/// - False positive: rice paddies, shadows, wind-roughened water
/// - Cannot detect flooding under dense vegetation canopy (radar penetration limited)
/// - No radiometric terrain correction and no speckle filter are applied here;
///   on sloping terrain, radar shadow and layover can read as water. Correction
///   module: Vollrath, Mullissa & Reiche 2020, DOI 10.3390/rs12111867

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
    out.push_str("Method: S1 GRD VV threshold + change detection + DEMNAS mask\n");
    out.push_str("NOTE: This tool searches for scenes only — no DL model runs here.\n\n");

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
    out.push_str("BENCHMARK TERVERIFIKASI (bukan performa tool ini):\n");
    out.push_str("  Angka di bawah berasal dari paper yang dapat ditelusuri lewat DOI.\n");
    out.push_str("  Tool ini hanya mencari scene; model harus dijalankan terpisah.\n\n");
    out.push_str("  Metode / produk                          Skor                    Setting\n");
    out.push_str("  ------------------------------------     --------------------    -------\n");
    out.push_str("  Sen1Floods11 semi-supervised ensemble    IoU 0.7654              open area\n");
    out.push_str("    Paul & Ganju 2021, arXiv 2107.08369\n");
    out.push_str("  Sen1Floods11 fusi S1+S2                  mIoU 52.99%, OA 92.81%  open area\n");
    out.push_str("    Bai et al. 2021, DOI 10.3390/rs13112220\n");
    out.push_str("  Kendari, S1 + Otsu (area tergenang)      OA 95.81%, Kappa 0.86   open area\n");
    out.push_str("    Aldiansyah et al. 2024, DOI 10.23960/jgrs.ft.unila.205\n");
    out.push_str("  UFO, model tersegmentasi terlatih         mIoU 77.3               URBAN\n");
    out.push_str("    Mukherjee et al. 2026, arXiv 2604.23066\n");
    out.push_str("  Google Dynamic World, kelas air          IoU 48.1                URBAN\n");
    out.push_str("    Mukherjee et al. 2026, arXiv 2604.23066\n");
    out.push_str("  NASA IMPACT (Sentinel-1)                 IoU 44.1                URBAN\n");
    out.push_str("    Mukherjee et al. 2026, arXiv 2604.23066\n");
    out.push_str("\n");
    out.push_str("  Untuk AOI perkotaan Indonesia (Jakarta, Semarang, Surabaya), pakai\n");
    out.push_str("  batas URBAN: produk siap pakai hanya mencapai IoU 44-48, artinya\n");
    out.push_str("  sekitar separuh piksel air salah klasifikasi.\n\n");
    out.push_str("  Temuan pendukung:\n");
    out.push_str("  - Zhao, Xiong & Zhu 2024 (UrbanSARFloods, arXiv 2406.04111): 8.879 chip,\n");
    out.push_str("    807.500 km2, 20 kelas tutupan lahan, 5 benua. Weighted cross-entropy\n");
    out.push_str("    dan transfer learning TIDAK cukup mengatasi data tak seimbang;\n");
    out.push_str("    deteksi banjir urban tetap sulit.\n");
    out.push_str("  - Amitrano et al. 2024 (DOI 10.3390/rs16040656): SAR masih terbatas berat\n");
    out.push_str("    di area bervegetasi dan urban karena mekanisme hamburan kompleks.\n");
    out.push_str("  - Bereczky et al. 2022 (DOI 10.1109/jstars.2022.3152127): dual-pol VV+VH\n");
    out.push_str("    mengalahkan single-pol 5% IoU; augmentasi radiometrik membantu,\n");
    out.push_str("    augmentasi geometrik menurunkan performa.\n");
    out.push_str("\n");
    out.push_str("  Klaim F1 96-98% yang sebelumnya dicantumkan di sini (Siamese U-Net,\n");
    out.push_str("  TLE-FEDformer) DIHAPUS: sitasinya tidak dapat diverifikasi di Crossref,\n");
    out.push_str("  OpenAlex, maupun arXiv, dan angkanya melampaui setiap benchmark yang\n");
    out.push_str("  terverifikasi di atas. Lihat src/citations.rs (UNVERIFIED).\n");
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
