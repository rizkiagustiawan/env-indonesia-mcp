use std::process::Command;

/// Transform coordinates between any CRS using pyproj
pub fn transform(x: f64, y: f64, from_epsg: &str, to_epsg: &str) -> String {
    // Call pyproj via Python one-liner
    let code = format!(
        "from pyproj import Transformer; t = Transformer.from_crs('{}', '{}', always_xy=True); x,y = t.transform({}, {}); print(f'{{x:.6f}},{{y:.6f}}')",
        from_epsg, to_epsg, x, y
    );
    match Command::new("python3").arg("-c").arg(&code).output() {
        Ok(o) => {
            let result = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if result.contains(',') {
                let parts: Vec<&str> = result.split(',').collect();
                // UTM zone only meaningful when source CRS is geographic WGS84 (x=lon, y=lat)
                let zone_info = if from_epsg.to_uppercase().contains("4326") {
                    format!("\n\nUTM Zone Info: {}", utm_zone_from_lon(y, x))
                } else {
                    String::new()
                };
                format!("=== Coordinate Transform ===\nInput: ({:.6}, {:.6}) [{}]\nOutput: ({}, {}) [{}]{}\n",
                    x, y, from_epsg, parts[0], parts[1], to_epsg, zone_info)
            } else {
                format!(
                    "ERROR: Transform failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                )
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

/// Auto-detect UTM zone for Indonesia coordinates
pub fn utm_zone_from_lon(lat: f64, lon: f64) -> String {
    let zone = ((lon + 180.0) / 6.0).floor() as i32 + 1;
    // BUG FIX: hemisphere is determined by LATITUDE not longitude.
    // Old code used lon>=0 -> "N" which is wrong (Indonesia lon 95-141 is all >=0 but mostly S).
    let hemisphere = if lat >= 0.0 { "N" } else { "S" };
    let epsg = if lat >= 0.0 { 32600 + zone } else { 32700 + zone };
    format!("UTM Zone {}{} (EPSG:{})", zone, hemisphere, epsg)
}

/// Auto UTM transform: WGS84 lat/lon -> appropriate UTM zone
pub fn wgs84_to_utm_auto(lat: f64, lon: f64) -> String {
    let zone = ((lon + 180.0) / 6.0).floor() as i32 + 1;
    let epsg = if lat >= 0.0 {
        32600 + zone
    } else {
        32700 + zone
    };
    let to_crs = format!("EPSG:{}", epsg);
    transform(lon, lat, "EPSG:4326", &to_crs)
}

/// UTM -> WGS84 (auto-detect zone from EPSG code)
pub fn utm_to_wgs84(easting: f64, northing: f64, epsg: &str) -> String {
    transform(easting, northing, epsg, "EPSG:4326")
}

/// Indonesia TM-3 zones (EPSG:23830-23845)
pub fn get_tm3_zone(lon: f64) -> String {
    // TM-3 zones: EPSG 23830 = CM 93°E, 23831 = 96°E, ..., 23845 = 138°E
    let cm = ((lon - 88.5) / 3.0).floor() as i32 * 3 + 90;
    let zone_num = (cm - 93) / 3;
    let epsg = 23830 + zone_num;
    format!("Indonesia TM-3: CM {}°E (EPSG:{})", cm, epsg)
}
