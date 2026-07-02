/// UV Index Calculator
/// Ref: WHO/WMO UV Index standard

pub fn calculate(solar_zenith_deg: f64, altitude_m: f64, ozone_du: f64, cloud_cover_pct: f64) -> String {
    if solar_zenith_deg < 0.0 || solar_zenith_deg > 90.0 { return "ERROR: Solar zenith 0-90°.".into(); }

    // Simplified UV model
    let cos_z = solar_zenith_deg.to_radians().cos().max(0.01);
    let base_uvi = 12.0 * cos_z; // Clear sky equatorial max ~12
    let ozone_factor = 300.0 / ozone_du.max(100.0); // Ozone thinning increases UV
    let altitude_factor = 1.0 + altitude_m * 0.0001; // +10% per 1000m
    let cloud_factor = 1.0 - (cloud_cover_pct / 100.0 * 0.75); // Clouds reduce ~75%
    let uvi = base_uvi * ozone_factor * altitude_factor * cloud_factor;

    let cat = if uvi < 3.0 { "Rendah" } else if uvi < 6.0 { "Sedang" } else if uvi < 8.0 { "Tinggi" } else if uvi < 11.0 { "Sangat Tinggi" } else { "Ekstrem" };

    let mut out = format!("=== UV Index ===\nRef: WHO/WMO\n\nSolar zenith: {:.1}°\nAltitude: {:.0} m\nOzone: {:.0} DU\nCloud: {:.0}%\n\nUV Index: {:.1}\nKategori: {}\n\n", solar_zenith_deg, altitude_m, ozone_du, cloud_cover_pct, uvi, cat);
    if uvi >= 8.0 { out.push_str("⚠️ Gunakan tabir surya SPF30+, topi, kacamata. Hindari paparan 10:00-15:00.\n"); }
    out.push_str("Indonesia tropis: UV Index 10-14 (Ekstrem) umum saat siang.\n");
    out
}
