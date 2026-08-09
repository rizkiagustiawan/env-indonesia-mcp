/// Air Dispersion Screening Model (Gaussian + Land-Use Correction)
/// NOTE: This is NOT a trained ML model. It is a Gaussian plume with an empirical
///   land-use correction factor. The R²>0.8 / 100-1000× faster claims below are
///   from the cited paper's ML surrogate, NOT this tool's performance.
/// Ref: ML surrogate concept — Nature s44407-025-00035-4 (2025)
pub fn assess(emission_g_s: f64, wind_speed_m_s: f64, wind_dir_deg: f64, mixing_height_m: f64, distance_m: f64, land_use: &str, receptor_height_m: f64) -> String {
    let mut out = String::from("=== Air Dispersion Screening (Gaussian + Land-Use Correction) ===\n");
    out.push_str("NOTE: This is a Gaussian plume model with empirical correction, NOT a trained ML surrogate.\n");
    out.push_str("Ref: ML surrogate concept — Nature s44407-025-00035-4 (2025)\n\n");
    let z0 = match land_use.to_lowercase().as_str() {
        s if s.contains("urban") || s.contains("kota") => 1.0,
        s if s.contains("forest") || s.contains("hutan") => 1.5,
        s if s.contains("agri") || s.contains("pertanian") => 0.2,
        s if s.contains("rural") || s.contains("pedesaan") => 0.1,
        s if s.contains("industrial") || s.contains("industri") => 1.2,
        _ => 0.5,
    };
    let sigma_y = 0.1 * distance_m * ((1.0_f64 + 0.3 * z0)).sqrt();
    let sigma_z = 0.07 * distance_m * ((1.0_f64 + 0.3 * z0)).sqrt();
    let wind_factor = (-distance_m / mixing_height_m.max(1e-6)).exp() * (wind_dir_deg.to_radians()).cos().abs();
    let e_weighted = emission_g_s * wind_factor;
    let stability_class = if wind_speed_m_s < 2.0 {"A-B (unstable)"} else if wind_speed_m_s < 5.0 {"C-D (neutral)"} else {"E-F (stable)"};
    let conc_ug_m3 = e_weighted / (2.0 * std::f64::consts::PI * sigma_y * sigma_z * wind_speed_m_s.max(0.5)) * 1e6;
    let correction_factor = 0.85 + 0.1 * (mixing_height_m / 500.0).min(2.0);
    let conc_corrected = conc_ug_m3 * correction_factor;
    out.push_str(&format!("Emission: {:.2} g/s, Wind: {:.1} m/s at {:.0}°\n", emission_g_s, wind_speed_m_s, wind_dir_deg));
    out.push_str(&format!("Mixing height: {:.0}m, Distance: {:.0}m\n", mixing_height_m, distance_m));
    out.push_str(&format!("Land use: {} (z0={:.1}m)\n\n", land_use, z0));
    out.push_str("-- Feature Engineering --\n\n");
    out.push_str(&format!("  Wind-weighted emission: {:.4} g/s\n", e_weighted));
    out.push_str(&format!("  σ_y: {:.1}m, σ_z: {:.1}m\n", sigma_y, sigma_z));
    out.push_str(&format!("  Stability: {}\n", stability_class));
    out.push_str(&format!("  Correction factor: {:.2}\n\n", correction_factor));
    out.push_str("-- Prediction --\n\n");
    out.push_str(&format!("  Base Gaussian: {:.2} µg/m3\n", conc_ug_m3));
    out.push_str(&format!("  >> Land-use corrected: {:.2} µg/m3\n\n", conc_corrected));
    out.push_str("-- STATUS KEPATUHAN --\n");
    out.push_str("  PP 22/2021 Lampiran VII (Udara Ambien):\n");
    out.push_str("  SO2 1jam:150 | NO2 1jam:200 | PM10 24jam:75 | PM2.5 24jam:55 µg/m3\n");
    out.push_str(&format!("  Measured: {:.2} µg/m3 → check parameter\n\n", conc_corrected));
    out.push_str("  -- Literature Reference (NOT this tool's performance) --\n");
    out.push_str("  ML surrogate (Nature s44407-025-00035-4): R²>0.8, 100-1000x faster than AERMOD\n");
    out.push_str("  NOTE: This tool is a screening Gaussian model, NOT the cited ML surrogate.\n");
    out.push_str("  For regulatory compliance: use AERMOD (EPA) or CALPUFF.\n");
    out
}
