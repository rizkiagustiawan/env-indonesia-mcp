/// Gaussian Plume Dispersion Model
/// C(x,y,0) = (Q / (π u σy σz)) × exp(-y²/(2σy²)) × exp(-H²/(2σz²))
/// Ref: Turner (1970), AERMOD simplified

pub fn calculate(emission_rate_gs: f64, wind_speed_ms: f64, stack_height_m: f64, distance_m: f64, stability_class: &str) -> String {
    let mut out = String::from("=== Gaussian Plume Dispersion Model ===\n");
    out.push_str("Ref: Turner (1970), Pasquill-Gifford stability classes\n\n");

    if wind_speed_ms < 0.28 {
        return "ERROR FISIKA: Wind speed < 0.28 m/s. Model Gaussian TIDAK VALID (singularitas). Gunakan AERMOD calm-wind algorithm.".into();
    }
    if emission_rate_gs <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if distance_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    let x = distance_m;

    // Pasquill-Gifford dispersion coefficients (simplified Briggs formulas)
    let (sigma_y, sigma_z) = match stability_class.to_uppercase().as_str() {
        "A" => (0.22 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.20 * x),
        "B" => (0.16 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.12 * x),
        "C" => (0.11 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.08 * x * (1.0 + 0.0002 * x).powf(-0.5)),
        "D" => (0.08 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.06 * x * (1.0 + 0.0015 * x).powf(-0.5)),
        "E" => (0.06 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.03 * x * (1.0 + 0.0003 * x).powf(-1.0)),
        "F" => (0.04 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.016 * x * (1.0 + 0.0003 * x).powf(-1.0)),
        _ => return "ERROR: Stability class harus A-F (Pasquill-Gifford).".into(),
    };

    // Ground-level centerline concentration (y=0, z=0)
    let h = stack_height_m;
    let c_ground = (emission_rate_gs * 1e6) / (std::f64::consts::PI * wind_speed_ms * sigma_y * sigma_z)
        * (-h * h / (2.0 * sigma_z * sigma_z)).exp();

    out.push_str(&format!("Input:\n  Q (emisi) = {:.2} g/s\n  u (angin) = {:.2} m/s\n  H (tinggi cerobong) = {:.1} m\n  x (jarak) = {:.0} m\n  Stability = {} (Pasquill-Gifford)\n\n", emission_rate_gs, wind_speed_ms, stack_height_m, distance_m, stability_class));
    out.push_str(&format!("Koefisien dispersi:\n  σy = {:.2} m\n  σz = {:.2} m\n\n", sigma_y, sigma_z));
    out.push_str(&format!("Konsentrasi ground-level (centerline):\n  C = {:.4} µg/m³\n\n", c_ground));

    // Cek baku mutu
    out.push_str("Perbandingan baku mutu PP 22/2021 (24 jam):\n");
    out.push_str(&format!("  PM2.5: {} (baku mutu 65 µg/m³)\n", if c_ground > 65.0 { "MELEBIHI ⚠️" } else { "OK ✅" }));
    out.push_str(&format!("  SO2: {} (baku mutu 75 µg/m³)\n", if c_ground > 75.0 { "MELEBIHI ⚠️" } else { "OK ✅" }));
    out
}
