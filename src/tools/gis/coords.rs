pub fn transform(x: f64, y: f64, direction: &str) -> String {
    match direction {
        "wgs84_to_utm" => {
            // Simple UTM Zone 50S projection (NTB is in UTM 50S)
            let lon0 = 117.0; // central meridian UTM 50
            let k0 = 0.9996;
            let a = 6378137.0; // WGS84 semi-major axis
            let lat_rad = (y as f64).to_radians();
            let lon_rad = (x as f64).to_radians();
            let lon0_rad = (lon0 as f64).to_radians();
            let n = a / (1.0 - 0.00669437999014 * lat_rad.sin().powi(2)).sqrt();
            let t = lat_rad.tan();
            let c = 0.00673949674228 * lat_rad.cos().powi(2);
            let aa = (lon_rad - lon0_rad) * lat_rad.cos();
            let m = a * ((1.0 - 0.00669437999014/4.0 - 3.0*0.00669437999014_f64.powi(2)/64.0) * lat_rad
                - (3.0*0.00669437999014/8.0 + 3.0*0.00669437999014_f64.powi(2)/32.0) * (2.0*lat_rad).sin()
                + (15.0*0.00669437999014_f64.powi(2)/256.0) * (4.0*lat_rad).sin());
            let easting = k0 * n * (aa + (1.0-t*t+c)*aa.powi(3)/6.0) + 500000.0;
            let northing = k0 * (m + n * lat_rad.tan() * (aa*aa/2.0 + (5.0-t*t+9.0*c+4.0*c*c)*aa.powi(4)/24.0)) + 10000000.0; // Southern hemisphere
            format!("=== Coordinate Transform ===\nInput (WGS84): {:.6}°E, {:.6}°N\nOutput (UTM 50S): {:.2} E, {:.2} N\nEPSG: 32750\nZone: 50S (NTB)", x, y, easting, northing)
        }
        "utm_to_wgs84" => {
            format!("=== Coordinate Transform ===\nInput (UTM 50S): {:.2} E, {:.2} N\nNote: Full UTM→WGS84 inverse transform requires iterative solution.\nFor precise conversion, use QGIS or proj4 library.\nApproximate center NTB: 117.5°E, -8.65°N", x, y)
        }
        _ => format!("Unknown direction '{}'. Use 'wgs84_to_utm' or 'utm_to_wgs84'.", direction),
    }
}
