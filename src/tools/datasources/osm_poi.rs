use reqwest::Client;

/// Query nearby POI from OpenStreetMap Overpass API
/// Critical for AMDAL: identifying schools, hospitals, settlements near project

/// Map POI type to Overpass query tags
fn poi_to_overpass_tags(poi_type: &str) -> (&'static str, &'static str) {
    match poi_type.to_lowercase().as_str() {
        "hospital" | "rumahsakit" | "rs" => ("amenity", "hospital"),
        "school" | "sekolah" => ("amenity", "school"),
        "residential" | "permukiman" | "perumahan" => ("landuse", "residential"),
        "worship" | "ibadah" | "masjid" => ("amenity", "place_of_worship"),
        "market" | "pasar" => ("amenity", "marketplace"),
        "river" | "sungai" => ("waterway", "river"),
        "forest" | "hutan" => ("landuse", "forest"),
        "university" | "universitas" => ("amenity", "university"),
        "kindergarten" | "tk" | "paud" => ("amenity", "kindergarten"),
        "clinic" | "klinik" | "puskesmas" => ("amenity", "clinic"),
        "fuel" | "spbu" => ("amenity", "fuel"),
        "police" | "polisi" => ("amenity", "police"),
        "fire_station" | "damkar" => ("amenity", "fire_station"),
        "cemetery" | "pemakaman" => ("landuse", "cemetery"),
        "industrial" | "industri" => ("landuse", "industrial"),
        "farmland" | "pertanian" | "sawah" => ("landuse", "farmland"),
        _ => ("amenity", poi_type.to_lowercase().leak()),
    }
}

/// Calculate distance between two coordinates using Haversine formula
fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6_371_000.0; // Earth radius in meters
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}

/// Query POI near a coordinate from OpenStreetMap Overpass API
pub async fn query_poi(client: &Client, lat: f64, lon: f64, radius_m: f64, poi_type: &str) -> String {
    let (tag_key, tag_value) = poi_to_overpass_tags(poi_type);

    // Build Overpass API query
    let query = format!(
        "[out:json][timeout:25];\n\
         (\n\
           node[\"{}\"=\"{}\"](around:{},{},{});\n\
           way[\"{}\"=\"{}\"](around:{},{},{});\n\
         );\n\
         out center body;\n\
         >;out skel qt;",
        tag_key, tag_value, radius_m, lat, lon,
        tag_key, tag_value, radius_m, lat, lon
    );

    let url = "https://overpass-api.de/api/interpreter";

    match client.post(url)
        .body(format!("data={}", query))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(body) => {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                            return format_poi_response(&data, lat, lon, radius_m, poi_type);
                        }
                        return format!("Error parsing Overpass response: {}", &body[..body.len().min(500)]);
                    }
                    Err(e) => return format!("Error reading Overpass response: {}", e),
                }
            } else {
                return format!("Overpass API error: HTTP {}", response.status());
            }
        }
        Err(e) => {
            return format!(
                "══════════════════════════════════════════════\n\
                 OSM POI QUERY - ERROR\n\
                 ══════════════════════════════════════════════\n\n\
                 Gagal mengakses Overpass API: {}\n\n\
                 Query yang digunakan:\n\
                 Tipe: {} ({}={})\n\
                 Koordinat: {}, {}\n\
                 Radius: {} m\n\n\
                 ALTERNATIF:\n\
                 • Coba lagi beberapa saat kemudian\n\
                 • Gunakan overpass-turbo.eu untuk query manual\n\
                 • Gunakan Google Maps atau OpenStreetMap langsung\n\
                 ══════════════════════════════════════════════",
                e, poi_type, tag_key, tag_value, lat, lon, radius_m
            );
        }
    }
}

fn format_poi_response(data: &serde_json::Value, center_lat: f64, center_lon: f64, radius_m: f64, poi_type: &str) -> String {
    let elements = match data.get("elements").and_then(|v| v.as_array()) {
        Some(e) => e,
        None => return "Tidak ada data ditemukan dari Overpass API.".to_string(),
    };

    // Extract POIs with name and location
    let mut pois: Vec<(String, f64, f64, f64)> = Vec::new(); // (name, lat, lon, distance)

    for elem in elements {
        let elem_type = elem.get("type").and_then(|v| v.as_str()).unwrap_or("");

        let (plat, plon) = if elem_type == "node" {
            let la = elem.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let lo = elem.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
            (la, lo)
        } else if elem_type == "way" {
            // Use center coordinates for ways
            let center = elem.get("center");
            let la = center.and_then(|c| c.get("lat")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let lo = center.and_then(|c| c.get("lon")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            (la, lo)
        } else {
            continue;
        };

        if plat == 0.0 && plon == 0.0 {
            continue;
        }

        let name = elem.get("tags")
            .and_then(|t| t.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("(tanpa nama)");

        let dist = haversine_m(center_lat, center_lon, plat, plon);
        pois.push((name.to_string(), plat, plon, dist));
    }

    // Sort by distance
    pois.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
    // Dedup by name
    pois.dedup_by(|a, b| a.0 == b.0);

    let mut result = format!(
        "══════════════════════════════════════════════\n\
         OSM POI - {} (radius {} m)\n\
         Koordinat pusat: {}, {}\n\
         Sumber: OpenStreetMap Overpass API\n\
         ══════════════════════════════════════════════\n\n\
         Ditemukan: {} lokasi\n\n",
        poi_type.to_uppercase(), radius_m, center_lat, center_lon, pois.len()
    );

    if pois.is_empty() {
        result.push_str("Tidak ditemukan POI dalam radius yang ditentukan.\n");
        result.push_str("Coba perbesar radius atau ubah tipe POI.\n");
    } else {
        for (i, (name, plat, plon, dist)) in pois.iter().take(50).enumerate() {
            let dist_str = if *dist < 1000.0 {
                format!("{:.0} m", dist)
            } else {
                format!("{:.2} km", dist / 1000.0)
            };
            result.push_str(&format!(
                "{:>3}. {} | Jarak: {} | ({:.5}, {:.5})\n",
                i + 1, name, dist_str, plat, plon
            ));
        }
        if pois.len() > 50 {
            result.push_str(&format!("\n... dan {} lokasi lainnya\n", pois.len() - 50));
        }
    }

    // AMDAL relevance note
    result.push_str(&format!(
        "\n──────────────────────────────────────────────\n\
         RELEVANSI AMDAL:\n\
         Keberadaan {} dalam radius {} m dari rencana kegiatan\n\
         perlu dipertimbangkan dalam:\n\
         • Pelingkupan dampak (KA-ANDAL)\n\
         • Identifikasi reseptor sensitif\n\
         • Penentuan batas wilayah studi\n\
         • Rencana pengelolaan & pemantauan (RKL-RPL)\n\
         ══════════════════════════════════════════════\n",
        poi_type, radius_m
    ));

    result
}
