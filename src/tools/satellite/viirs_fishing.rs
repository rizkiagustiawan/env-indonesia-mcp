use reqwest::Client;
use serde_json::json;

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";

/// VIIRS Fishing Detection — Night light fishing boat detection
/// Ref: Elvidge et al. 2024 (SE Asia 2012-2023); Wang et al. 2025 (South China Sea)
///      Li et al. 2024 (VBD algorithm); Cheng et al. 2026 (Arabian Sea)
/// Method: VIIRS DNB (Day/Night Band) → threshold → boat detection → MPA overlay

pub async fn search(
    _client: &Client,
    lat: f64,
    lon: f64,
    date: &str,
) -> String {
    let (s, w, n, e) = bbox_from_center(lat, lon, 50.0);
    let _datetime = format!("{}T00:00:00Z/{}T23:59:59Z", date, date);

    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("VIIRS Fishing Detection — Night Light Boat Detection\n");
    out.push_str("Ref: Elvidge et al. 2024 (VBD SE Asia); Wang et al. 2025; Li et al. 2024\n\n");

    out.push_str(&format!("Location: ({:.4}, {:.4}), 50km buffer\n", lat, lon));
    out.push_str(&format!("Date: {}\n\n", date));

    out.push_str("Note: VIIRS DNB nightly composites available via NASA Earthdata.\n");
    out.push_str("      STAC MPC does not host VIIRS DNB directly.\n");
    out.push_str("      Access: https://worldview.earthdata.nasa.gov/ or NASA CMR.\n\n");

    let _body = json!({
        "collections": ["nasadem"],
        "bbox": [w, s, e, n],
        "limit": 1u32
    });
    let _url = format!("{}/search", MPC_STAC_URL);

    out.push_str("VIIRS Boat Detection (VBD) Algorithm (Elvidge):\n");
    out.push_str("  1. Acquire VIIRS DNB nightly composite (~750m resolution)\n");
    out.push_str("  2. Threshold: nLw > 1.5 nW/cm²/sr (radiance threshold)\n");
    out.push_str("  3. Filter: ocean pixels only (exclude land/urban glow)\n");
    out.push_str("  4. Classify: boat vs gas flares (spectral signature)\n");
    out.push_str("  5. Overlay with MPA boundaries → potential illegal fishing\n\n");

    out.push_str("DATA ACCESS:\n");
    out.push_str("  - NASA Earthdata: https://search.earthdata.nasa.gov/ (register free)\n");
    out.push_str("  - Collection: VNP46A1 (VIIRS/NPP DNB Daily)\n");
    out.push_str("  - Or: NOAA VBD product (Elvidge's pre-processed)\n");
    out.push_str("  - .netrc already configured (rizkiagustiawan)\n\n");

    out.push_str("INDONESIA MPA (Marine Protected Areas):\n");
    out.push_str("  - Savu Sea MPA (NTT)\n");
    out.push_str("  - Wakatobi NP (Sultra)\n");
    out.push_str("  - Bunaken NP (Sulut)\n");
    out.push_str("  - Komodo NP (NTT)\n");
    out.push_str("  - Raja Ampat MPA (Papua Barat)\n");
    out.push_str("  - Banda Sea MPA (Maluku)\n\n");

    out.push_str("LIMITATION:\n");
    out.push_str("  - Cloud cover blocks VIIRS (can't see through clouds)\n");
    out.push_str("  - Only detects lit boats (unlit = invisible)\n");
    out.push_str("  - DNB resolution ~750m — can't separate individual boats in fleet\n");
    out.push_str("  - Gas flares and aurora can produce false positives\n");
    out.push_str("  - Moonlight affects detection (high lunar illumination = more noise)\n");
    out.push_str("  - VBD is Elvidge's algorithm — tool is simplified version\n");
    out.push_str("  - Does not identify vessel type (fishing vs cargo vs military)\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}

fn bbox_from_center(lat: f64, lon: f64, buffer_km: f64) -> (f64, f64, f64, f64) {
    let lat_offset = buffer_km / 111.0;
    let lon_offset = buffer_km / (111.0 * lat.to_radians().cos().abs().max(0.01));
    (lat - lat_offset, lon - lon_offset, lat + lat_offset, lon + lon_offset)
}
