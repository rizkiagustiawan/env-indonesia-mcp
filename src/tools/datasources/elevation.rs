use reqwest::Client;

/// Elevation Profile between two points
/// Source: Open-Elevation API

/// Interpolate n points along a line between two coordinates
fn interpolate_points(lat1: f64, lon1: f64, lat2: f64, lon2: f64, n: u32) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    for i in 0..n {
        let t = i as f64 / (n - 1).max(1) as f64;
        let lat = lat1 + t * (lat2 - lat1);
        let lon = lon1 + t * (lon2 - lon1);
        points.push((lat, lon));
    }
    points
}

/// Calculate Haversine distance in meters
fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6_371_000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}

/// Get elevation profile along a line between two points
pub async fn profile(
    client: &Client,
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
    num_points: u32,
) -> String {
    let n = num_points.max(2).min(100);
    let points = interpolate_points(lat1, lon1, lat2, lon2, n);

    // Build locations string for Open-Elevation API
    let locations: Vec<String> = points
        .iter()
        .map(|(lat, lon)| format!("{},{}", lat, lon))
        .collect();
    let locations_str = locations.join("|");

    let url = format!(
        "https://api.open-elevation.com/api/v1/lookup?locations={}",
        locations_str
    );

    let elevations: Vec<f64>;

    match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(body) => {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
                                elevations = results
                                    .iter()
                                    .map(|r| {
                                        r.get("elevation").and_then(|v| v.as_f64()).unwrap_or(0.0)
                                    })
                                    .collect();
                            } else {
                                return format!(
                                    "Error: Unexpected response format from Open-Elevation: {}",
                                    &body[..body.len().min(500)]
                                );
                            }
                        } else {
                            return format!(
                                "Error parsing Open-Elevation response: {}",
                                &body[..body.len().min(500)]
                            );
                        }
                    }
                    Err(e) => return format!("Error reading Open-Elevation response: {}", e),
                }
            } else {
                return format!("Open-Elevation API error: HTTP {}", response.status());
            }
        }
        Err(e) => {
            return format!(
                "══════════════════════════════════════════════\n\
                 ELEVATION PROFILE - ERROR\n\
                 ══════════════════════════════════════════════\n\n\
                 Gagal mengakses Open-Elevation API: {}\n\n\
                 Titik awal : ({}, {})\n\
                 Titik akhir: ({}, {})\n\
                 Jumlah sampel: {}\n\n\
                 ALTERNATIF:\n\
                 • Coba lagi beberapa saat kemudian\n\
                 • Gunakan SRTM tool untuk data elevasi DEM\n\
                 • Gunakan Google Earth Pro untuk profil elevasi\n\
                 ══════════════════════════════════════════════",
                e, lat1, lon1, lat2, lon2, n
            );
        }
    }

    if elevations.len() < 2 {
        return "Error: Insufficient elevation data returned.".to_string();
    }

    // Calculate statistics
    let max_elev = elevations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_elev = elevations.iter().cloned().fold(f64::INFINITY, f64::min);
    let avg_elev = elevations.iter().sum::<f64>() / elevations.len() as f64;

    let total_distance = haversine_m(lat1, lon1, lat2, lon2);

    // Calculate total ascent and descent
    let mut total_climb = 0.0_f64;
    let mut total_descent = 0.0_f64;
    for i in 1..elevations.len() {
        let diff = elevations[i] - elevations[i - 1];
        if diff > 0.0 {
            total_climb += diff;
        } else {
            total_descent += diff.abs();
        }
    }

    // Calculate average slope
    let elev_diff = elevations.last().unwrap() - elevations.first().unwrap();
    let avg_slope = if total_distance > 0.0 {
        (elev_diff / total_distance * 100.0).abs()
    } else {
        0.0
    };

    // Build result
    let mut result = format!(
        "══════════════════════════════════════════════\n\
         PROFIL ELEVASI\n\
         Sumber: Open-Elevation API (SRTM)\n\
         ══════════════════════════════════════════════\n\n\
         LINTASAN:\n\
         • Titik awal  : ({:.5}, {:.5})\n\
         • Titik akhir  : ({:.5}, {:.5})\n\
         • Jarak total  : {:.0} m ({:.2} km)\n\
         • Jumlah sampel: {}\n\n\
         STATISTIK ELEVASI:\n\
         • Elevasi maksimum  : {:.1} m dpl\n\
         • Elevasi minimum   : {:.1} m dpl\n\
         • Elevasi rata-rata : {:.1} m dpl\n\
         • Total pendakian   : {:.1} m\n\
         • Total penurunan   : {:.1} m\n\
         • Selisih elevasi   : {:.1} m\n\
         • Kemiringan rata-rata: {:.2}%\n\n\
         DATA PROFIL:\n",
        lat1,
        lon1,
        lat2,
        lon2,
        total_distance,
        total_distance / 1000.0,
        n,
        max_elev,
        min_elev,
        avg_elev,
        total_climb,
        total_descent,
        elev_diff,
        avg_slope
    );

    result.push_str("  Jarak(m)   | Elevasi(m) | Lat        | Lon\n");
    result.push_str("  -----------|------------|------------|------------\n");

    for (i, ((lat, lon), elev)) in points.iter().zip(elevations.iter()).enumerate() {
        let dist = if i == 0 {
            0.0
        } else {
            haversine_m(lat1, lon1, *lat, *lon)
        };
        result.push_str(&format!(
            "  {:>9.1} | {:>10.1} | {:>10.5} | {:>10.5}\n",
            dist, elev, lat, lon
        ));
    }

    // Slope classification for AMDAL
    let slope_class = if avg_slope < 2.0 {
        "Datar (0-2%)"
    } else if avg_slope < 8.0 {
        "Landai (2-8%)"
    } else if avg_slope < 15.0 {
        "Agak curam (8-15%)"
    } else if avg_slope < 25.0 {
        "Curam (15-25%)"
    } else if avg_slope < 45.0 {
        "Sangat curam (25-45%)"
    } else {
        "Terjal (>45%)"
    };

    result.push_str(&format!(
        "\n  KLASIFIKASI KEMIRINGAN: {}\n\
         \n  RELEVANSI AMDAL:\n\
         • Kemiringan mempengaruhi potensi erosi dan longsor\n\
         • Area dengan kemiringan >40% termasuk kawasan rawan bencana\n\
         • Ref: Permen PU 22/2007 tentang Penataan Ruang Kawasan Rawan Bencana\n\
         ══════════════════════════════════════════════\n",
        slope_class
    ));

    result
}
