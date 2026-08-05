/// Gaussian Plume Dispersion Model
/// C(x,y,0) = (Q / (π u σy σz)) × exp(-y²/(2σy²)) × exp(-H²/(2σz²))
/// Ref: Turner (1970), AERMOD simplified

pub fn calculate(
    emission_rate_gs: f64,
    wind_speed_ms: f64,
    stack_height_m: f64,
    distance_m: f64,
    stability_class: &str,
) -> String {
    let mut out = String::from("=== Gaussian Plume Dispersion Model ===\n");
    out.push_str("Ref: Turner (1970), Pasquill-Gifford stability classes\n\n");

    if wind_speed_ms < 0.28 {
        return "ERROR FISIKA: Wind speed < 0.28 m/s. Model Gaussian TIDAK VALID (singularitas). Gunakan AERMOD calm-wind algorithm.".into();
    }
    if emission_rate_gs <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if distance_m <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    let x = distance_m;

    // Pasquill-Gifford dispersion coefficients (simplified Briggs formulas)
    let (sigma_y, sigma_z) = match stability_class.to_uppercase().as_str() {
        "A" => (0.22 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.20 * x),
        "B" => (0.16 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.12 * x),
        "C" => (
            0.11 * x * (1.0 + 0.0001 * x).powf(-0.5),
            0.08 * x * (1.0 + 0.0002 * x).powf(-0.5),
        ),
        "D" => (
            0.08 * x * (1.0 + 0.0001 * x).powf(-0.5),
            0.06 * x * (1.0 + 0.0015 * x).powf(-0.5),
        ),
        "E" => (
            0.06 * x * (1.0 + 0.0001 * x).powf(-0.5),
            0.03 * x * (1.0 + 0.0003 * x).powf(-1.0),
        ),
        "F" => (
            0.04 * x * (1.0 + 0.0001 * x).powf(-0.5),
            0.016 * x * (1.0 + 0.0003 * x).powf(-1.0),
        ),
        _ => return "ERROR: Stability class harus A-F (Pasquill-Gifford).".into(),
    };

    // Cap σz at typical mixing height (stability-dependent)
    let mixing_height = match stability_class.to_uppercase().as_str() {
        "A" | "B" => 1500.0,
        "C" => 1000.0,
        "D" => 800.0,
        "E" => 300.0,
        "F" => 100.0,
        _ => 1000.0,
    };
    let sigma_z = sigma_z.min(mixing_height);

    // Ground-level centerline concentration (y=0, z=0)
    let h = stack_height_m;
    let c_ground = (emission_rate_gs * 1e6)
        / (std::f64::consts::PI * wind_speed_ms * sigma_y * sigma_z)
        * (-h * h / (2.0 * sigma_z * sigma_z)).exp();

    out.push_str(&format!("Input:\n  Q (emisi) = {:.2} g/s\n  u (angin) = {:.2} m/s\n  H (tinggi cerobong) = {:.1} m\n  x (jarak) = {:.0} m\n  Stability = {} (Pasquill-Gifford)\n\n", emission_rate_gs, wind_speed_ms, stack_height_m, distance_m, stability_class));
    out.push_str(&format!(
        "Koefisien dispersi:\n  σy = {:.2} m\n  σz = {:.2} m\n\n",
        sigma_y, sigma_z
    ));
    out.push_str(&format!(
        "Konsentrasi ground-level (centerline):\n  C = {:.4} µg/m³\n\n",
        c_ground
    ));

    // Cek baku mutu
    out.push_str("Perbandingan baku mutu PP 22/2021 (24 jam):\n");
    out.push_str(&format!(
        "  PM2.5: {} (baku mutu 65 µg/m³)\n",
        if c_ground > 65.0 {
            "MELEBIHI ⚠️"
        } else {
            "OK ✅"
        }
    ));
    out.push_str(&format!(
        "  SO2: {} (baku mutu 75 µg/m³)\n",
        if c_ground > 75.0 {
            "MELEBIHI ⚠️"
        } else {
            "OK ✅"
        }
    ));
    out
}

/// Line Source Gaussian Model (untuk jalan raya / conveyor belt)
/// Ref: Caline4 (Caltrans), PermenLHK 15/2019
/// Menggunakan integrasi numerik titik-titik sumber sepanjang garis
pub fn line_source(
    emission_rate_g_per_m_s: f64,
    line_length_m: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    receptor_distance_m: f64,
    stability_class: &str,
) -> String {
    if wind_speed < 0.28 {
        return "ERROR: Kecepatan angin < 0.28 m/s. Model dispersi tidak valid (singularitas). Ref: US EPA AERMOD Guide.".into();
    }
    if emission_rate_g_per_m_s <= 0.0 {
        return "ERROR: Emission rate harus positif.".into();
    }

    let n_segments = 20;
    let seg_len = line_length_m / n_segments as f64;
    let wind_rad = wind_angle_deg.to_radians();
    let mut total_conc = 0.0;

    for i in 0..n_segments {
        let seg_center = (i as f64 + 0.5) * seg_len - line_length_m / 2.0;
        let x_eff = receptor_distance_m * wind_rad.cos() - seg_center * wind_rad.sin();
        let y_eff = receptor_distance_m * wind_rad.sin() + seg_center * wind_rad.cos();

        if x_eff <= 0.0 { continue; }

        let (sy, sz) = pasquill_gifford(x_eff, stability_class);
        let q_seg = emission_rate_g_per_m_s * seg_len;
        let conc = (q_seg / (2.0 * std::f64::consts::PI * wind_speed * sy * sz))
            * (-0.5 * (y_eff / sy).powi(2)).exp();
        total_conc += conc;
    }

    let total_ug = total_conc * 1e6;
    format!(
        "=== Line Source Gaussian Dispersion ===\nRef: Caline4, PermenLHK 15/2019\n\nEmission Rate: {:.4} g/m/s\nLine Length: {:.0} m ({} segments)\nWind Speed: {:.1} m/s | Angle: {:.0}°\nReceptor Distance: {:.0} m\nStability: {}\n\nKonsentrasi di reseptor: {:.2} µg/m³\n",
        emission_rate_g_per_m_s, line_length_m, n_segments,
        wind_speed, wind_angle_deg, receptor_distance_m, stability_class, total_ug
    )
}

/// Area Source Gaussian Model (untuk landfill, stockpile, kolam limbah)
/// Ref: US EPA AP-42, Virtual Point Source method
pub fn area_source(
    emission_rate_g_per_m2_s: f64,
    area_side_m: f64,
    wind_speed: f64,
    receptor_distance_m: f64,
    stability_class: &str,
) -> String {
    if wind_speed < 0.28 {
        return "ERROR: Kecepatan angin < 0.28 m/s. Model dispersi tidak valid.".into();
    }

    let virtual_distance = area_side_m / 4.3;
    let x_eff = receptor_distance_m + virtual_distance;
    let total_emission = emission_rate_g_per_m2_s * area_side_m * area_side_m;

    let (sy, sz) = pasquill_gifford(x_eff, stability_class);
    let conc = total_emission / (std::f64::consts::PI * wind_speed * sy * sz);
    let conc_ug = conc * 1e6;

    format!(
        "=== Area Source Gaussian Dispersion ===\nRef: US EPA AP-42, Virtual Point Source Method\n\nEmission Rate: {:.6} g/m²/s\nArea Side: {:.0} m (Area: {:.0} m²)\nVirtual Distance: {:.1} m\nWind Speed: {:.1} m/s\nReceptor Distance: {:.0} m\nStability: {}\n\nKonsentrasi di reseptor: {:.2} µg/m³\n",
        emission_rate_g_per_m2_s, area_side_m, area_side_m * area_side_m,
        virtual_distance, wind_speed, receptor_distance_m, stability_class, conc_ug
    )
}

fn pasquill_gifford(x: f64, stability: &str) -> (f64, f64) {
    match stability {
        "A" => (0.22 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.20 * x),
        "B" => (0.16 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.12 * x),
        "C" => (0.11 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.08 * x * (1.0 + 0.0002 * x).powf(-0.5)),
        "D" => (0.08 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.06 * x * (1.0 + 0.0015 * x).powf(-0.5)),
        "E" => (0.06 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.03 * x * (1.0 + 0.0003 * x).powf(-1.0)),
        "F" => (0.04 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.016 * x * (1.0 + 0.0003 * x).powf(-1.0)),
        _ => (0.08 * x * (1.0 + 0.0001 * x).powf(-0.5), 0.06 * x * (1.0 + 0.0015 * x).powf(-0.5)),
    }
}
