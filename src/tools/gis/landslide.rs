use reqwest::Client;
use serde_json::{json, Value};

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// Landslide Susceptibility — Frequency Ratio (FR) + DEM + Rainfall
/// Ref: Tirsyayu et al. 2025 (Sulsel FR AUC=0.81); Gnagne et al. 2025 (Ivory Coast);
///      Akhil et al. 2025 (Wayanad FR AUC=0.896); Akbar et al. 2025 (Japan FR+LR)
/// Method: FR = (landslide pixels in class / total landslides) / (pixels in class / total pixels)
/// Without inventory: heuristic (slope >25° + rainfall >100mm/day = high)

pub async fn assess(
    client: &Client,
    lat: f64,
    lon: f64,
    buffer_km: f64,
    rainfall_mm: f64,
) -> String {
    let (s, w, n, e) = bbox_from_center(lat, lon, buffer_km);
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Landslide Susceptibility (Frequency Ratio Method)\n");
    out.push_str("Ref: Tirsyayu 2025 (AUC=0.81); Gnagne 2025; Akhil 2025 (AUC=0.896); Akbar 2025\n\n");

    out.push_str(&format!("Location: ({:.4}, {:.4}), Buffer: {:.1} km\n", lat, lon, buffer_km));
    out.push_str(&format!("Rainfall (24h): {:.1} mm\n\n", rainfall_mm));

    let dem_body = json!({
        "collections": ["cop-dem-glo-30"],
        "bbox": [w, s, e, n],
        "limit": 3u32
    });
    let stac_url = format!("{}/search", MPC_STAC_URL);

    out.push_str("DEM DATA (Copernicus DEM 30m via STAC MPC):\n");
    match client.post(&stac_url).json(&dem_body).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(v) => {
                if let Some(features) = v.get("features").and_then(|f| f.as_array()) {
                    for (i, feat) in features.iter().take(3).enumerate() {
                        let id = feat.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                        out.push_str(&format!("  {}. DEM scene: {}\n", i + 1, id));
                        if let Some(assets) = feat.get("assets").and_then(|a| a.as_object()) {
                            if let Some(data) = assets.get("data") {
                                if let Some(href) = data.get("href").and_then(|h| h.as_str()) {
                                    out.push_str(&format!("     URL: {}\n", href));
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => out.push_str("  DEM search error\n"),
        },
        Err(e) => out.push_str(&format!("  DEM connection error: {}\n", e)),
    }

    out.push_str("\nSUSCEPTIBILITY FACTORS:\n");
    out.push_str("  1. Slope (from DEM: slope >25° = high risk)\n");
    out.push_str("  2. Curvature (concave = water accumulation)\n");
    out.push_str("  3. Aspect (SE, S, NW slopes = higher)\n");
    out.push_str("  4. TWI (Topographic Wetness Index)\n");
    out.push_str("  5. Rainfall intensity (>100mm/24h = trigger)\n");
    out.push_str("  6. Land cover (bare = high, forest = low)\n");
    out.push_str("  7. Distance to drainage network\n\n");

    let slope_risk = if rainfall_mm > 100.0 { "HIGH" }
        else if rainfall_mm > 50.0 { "MODERATE" }
        else { "LOW" };

    out.push_str("HEURISTIC ASSESSMENT (no inventory data):\n");
    out.push_str(&format!("  Rainfall: {:.1}mm → trigger risk: {}\n", rainfall_mm, slope_risk));
    out.push_str(&format!("  Slope threshold: >25° = high susceptibility\n"));
    out.push_str(&format!("  Combined risk: {} (rainfall) × DEM slope (to be extracted)\n\n", slope_risk));

    out.push_str("FREQUENCY RATIO METHOD (data-driven):\n");
    out.push_str("  FR_i = (N_landslide_in_class_i / N_total_landslide) / (N_pixel_in_class_i / N_total_pixel)\n");
    out.push_str("  FR > 1 = higher susceptibility\n");
    out.push_str("  Requires: landslide inventory (historical occurrence points)\n");
    out.push_str("  Source: IRBI/BNPB disaster database, or local government\n\n");

    out.push_str("VALIDATION (from papers):\n");
    out.push_str("  Tirsyayu 2025 (Sulsel): AUC = 0.81\n");
    out.push_str("  Gnagne 2025 (Ivory Coast): AUC = 0.80+\n");
    out.push_str("  Akhil 2025 (Wayanad): AUC = 0.896\n");
    out.push_str("  Mittamidi 2026 (Mizoram): AUC = 0.85\n");
    out.push_str("  → FR is state-of-the-art globally (AUC 0.80-0.90)\n\n");

    out.push_str("PROCESSING PROTOCOL:\n");
    out.push_str("  1. Download DEM GeoTIFF from URL above\n");
    out.push_str("  2. Compute slope (gdaldem slope)\n");
    out.push_str("  3. Compute curvature, TWI, aspect\n");
    out.push_str("  4. If inventory available: compute FR per class\n");
    out.push_str("  5. If no inventory: use heuristic (slope + rainfall threshold)\n");
    out.push_str("  6. Output: susceptibility class (very low to very high)\n\n");

    out.push_str("LIMITATION:\n");
    out.push_str("  - Without landslide inventory, heuristic only (slope + rainfall)\n");
    out.push_str("  - TRIGRS (Sugianti 2026) is physics-based but needs soil parameters\n");
    out.push_str("  - 30m DEM may miss small-scale topographic controls\n");
    out.push_str("  - Rainfall is point estimate, not spatially distributed\n");
    out.push_str("  - Land cover change (deforestation) not included in heuristic\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}

fn bbox_from_center(lat: f64, lon: f64, buffer_km: f64) -> (f64, f64, f64, f64) {
    let lat_offset = buffer_km / 111.0;
    let lon_offset = buffer_km / (111.0 * lat.to_radians().cos().abs().max(0.01));
    (lat - lat_offset, lon - lon_offset, lat + lat_offset, lon + lon_offset)
}
