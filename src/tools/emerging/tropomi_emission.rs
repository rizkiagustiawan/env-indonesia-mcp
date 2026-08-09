/// TROPOMI Satellite Emission Monitoring
/// Ref: ESA/Copernicus Sentinel-5P; Remote Sensing of Environment 2026
/// Formula: E = (ΔVCD × A × U) / τ
pub fn assess(facility_lat: f64, facility_lon: f64, pollutant: &str, vcd_molec_cm2: f64, background_vcd: f64, wind_speed_ms: f64, area_m2: f64) -> String {
    let mut out = String::from("=== TROPOMI Satellite Emission Monitoring ===\n");
    out.push_str("Ref: ESA Sentinel-5P; Remote Sensing of Environment 2026\n\n");
    let lifetime_s = match pollutant.to_uppercase().as_str() {
        "NO2" => 4.0 * 3600.0,
        "SO2" => 1.0 * 86400.0,
        "CH4" => 9.0 * 365.0 * 86400.0,
        "CO" => 30.0 * 86400.0,
        _ => 4.0 * 3600.0,
    };
    let delta_vcd = vcd_molec_cm2 - background_vcd;
    let avogadro = 6.022e23;
    let mol_mass = match pollutant.to_uppercase().as_str() {
        "NO2" => 46.0, "SO2" => 64.0, "CH4" => 16.0, "CO" => 28.0, _ => 46.0,
    };
    let emission_mol_s = if lifetime_s > 0.0 {
        delta_vcd * 1e4 * area_m2 * wind_speed_ms / (lifetime_s * avogadro)
    } else { 0.0 };
    let emission_kg_hr = emission_mol_s * mol_mass * 3.6;
    out.push_str(&format!("Facility: ({:.4}, {:.4})\n", facility_lat, facility_lon));
    out.push_str(&format!("Pollutant: {}, VCD: {:.2e} molec/cm2\n", pollutant, vcd_molec_cm2));
    out.push_str(&format!("Background: {:.2e}, ΔVCD: {:.2e}\n", background_vcd, delta_vcd));
    out.push_str(&format!("Wind: {:.1} m/s, Area: {:.0} m2\n\n", wind_speed_ms, area_m2));
    out.push_str("-- Emission Quantification --\n\n");
    out.push_str("  Formula: E = (ΔVCD × A × U) / τ\n");
    out.push_str(&format!("  Lifetime τ: {:.1e} s\n", lifetime_s));
    out.push_str(&format!("  >> Top-down emission: {:.2} kg/hr\n\n", emission_kg_hr));
    let detection_limit = match pollutant.to_uppercase().as_str() {
        "NO2" => 0.5, "SO2" => 1.0, "CH4" => 10.0, _ => 1.0,
    };
    out.push_str("-- Detection Sensitivity --\n");
    out.push_str(&format!("  Detection limit: ~{:.1} tonnes/hr\n", detection_limit));
    out.push_str(&format!("  Detected: {:.2} kg/hr → {}\n\n", emission_kg_hr, if emission_kg_hr > detection_limit * 1000.0 {"✅ DETECTED"} else {"⚠️ Below detection"}));
    out.push_str("-- STATUS KEPATUHAN --\n");
    out.push_str("  PP 22/2021 Lampiran VII (Udara Ambien):\n");
    out.push_str("  Compare to reported emissions (bottom-up inventory)\n\n");
    out.push_str("-- DATA SOURCE --\n");
    out.push_str("  Sentinel-5P TROPOMI Level 2\n");
    out.push_str("  Spatial: 5.5×3.5 km, Daily revisit\n");
    out.push_str("  Oversampling: ~1 km achievable\n");
    out.push_str("  Ref: ESA; Remote Sensing of Environment 2026\n");
    out
}
