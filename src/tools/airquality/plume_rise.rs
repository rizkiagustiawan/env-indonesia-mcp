/// Briggs Plume Rise Calculator
/// Ref: Briggs (1969, 1971, 1975), used in AERMOD

pub fn calculate(
    stack_height_m: f64,
    stack_diameter_m: f64,
    exit_velocity_ms: f64,
    exit_temp_k: f64,
    ambient_temp_k: f64,
    wind_speed_ms: f64,
) -> String {
    if stack_height_m <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if exit_velocity_ms <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if exit_temp_k <= 0.0 || ambient_temp_k <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0 Kelvin.".into();
    }
    if wind_speed_ms < 0.28 {
        return "ERROR: Wind speed < 0.28 m/s. Model tidak valid.".into();
    }

    let g = 9.81_f64;
    let ds = stack_diameter_m;

    // Buoyancy flux F (m⁴/s³)
    let f_buoy =
        g * exit_velocity_ms * ds * ds * (exit_temp_k - ambient_temp_k) / (4.0 * exit_temp_k);

    // Distance to final rise
    let x_star = if f_buoy < 55.0 {
        14.0 * f_buoy.powf(0.625)
    } else {
        34.0 * f_buoy.powf(0.4)
    };
    let xf = 3.5 * x_star;

    // Plume rise (unstable/neutral)
    let delta_h = 1.6 * f_buoy.powf(1.0 / 3.0) * xf.powf(2.0 / 3.0) / wind_speed_ms;

    let h_eff = stack_height_m + delta_h;

    let mut out = String::from(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Briggs Plume Rise\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    out.push_str("Ref: Briggs (1969-1975), AERMOD formulation\n\n");
    out.push_str(&format!("INPUT:\n  Stack height = {:.1} m\n  Stack diameter = {:.2} m\n  Exit velocity = {:.1} m/s\n  Exit temp = {:.0} K ({:.0}°C)\n  Ambient temp = {:.0} K ({:.0}°C)\n  Wind speed = {:.1} m/s\n\n",
        stack_height_m, stack_diameter_m, exit_velocity_ms, exit_temp_k, exit_temp_k - 273.15,
        ambient_temp_k, ambient_temp_k - 273.15, wind_speed_ms));
    out.push_str(&format!("HASIL:\n  Buoyancy flux (F) = {:.2} m⁴/s³\n  Distance to final rise (xf) = {:.0} m\n  Plume rise (Δh) = {:.1} m\n  Effective height (H_eff) = {:.1} m\n",
        f_buoy, xf, delta_h, h_eff));

    if delta_h > stack_height_m * 3.0 {
        out.push_str("\n⚠️ Plume rise sangat tinggi (>3x stack height). Cek input suhu gas.\n");
    }
    out
}
