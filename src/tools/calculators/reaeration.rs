/// Reaeration Coefficient Calculator — Multiple Formulas
/// Ref: Chapra 2008; O'Connor 1958; Churchill 1962; Owens-Gibbs 1964
pub fn assess(velocity_m_s: f64, depth_m: f64, temp_c: f64) -> String {
    let mut out = String::from("=== Reaeration Coefficient Calculator ===\n");
    out.push_str("Ref: Chapra 2008; O'Connor; Churchill; Owens-Gibbs\n\n");

    if velocity_m_s <= 0.0 || depth_m <= 0.0 {
        return "ERROR [E102]: velocity and depth must be > 0.".into();
    }

    let v = velocity_m_s; // m/s
    let H = depth_m; // m
    let v_m_day = v * 86400.0; // m/day

    out.push_str(&format!("Velocity: {:.2} m/s ({:.0} m/day)\n", v, v_m_day));
    out.push_str(&format!("Depth: {:.1} m\n", H));
    out.push_str(&format!("Temperature: {:.1} C\n\n", temp_c));

    // Multiple formulas (all at 20C, then temp-corrected)
    // O'Connor & Dobbins 1958: ka = 3.93 * v^0.5 / H^1.5
    let ka_oconnor = 3.93 * v_m_day.powf(0.5) / H.powf(1.5).max(1e-6);

    // Churchill 1962: ka = 5.01 * v^0.97 / H^1.67
    let ka_churchill = 5.01 * v_m_day.powf(0.97) / H.powf(1.67).max(1e-6);

    // Owens-Gibbs 1964: ka = 5.32 * v^0.67 / H^1.85
    let ka_owens = 5.32 * v_m_day.powf(0.67) / H.powf(1.85).max(1e-6);

    // Tsivoglou (for steep streams): ka = 1.0 * S * v / H (S = slope, assume 0.001)
    let slope = 0.001;
    let ka_tsivoglou = 1.0 * slope * v_m_day / H.max(1e-6);

    // Temperature correction: ka(T) = ka(20) * 1.024^(T-20)
    let theta: f64 = 1.024;
    let temp_factor = theta.powf(temp_c - 20.0);

    out.push_str("-- Reaeration Coefficients (ka at 20C, day-1) --\n\n");
    out.push_str(&format!("  O'Connor-Dobbins: {:.3} -> {:.3} (at {:.0}C)\n", ka_oconnor, ka_oconnor * temp_factor, temp_c));
    out.push_str(&format!("  Churchill:        {:.3} -> {:.3}\n", ka_churchill, ka_churchill * temp_factor));
    out.push_str(&format!("  Owens-Gibbs:      {:.3} -> {:.3}\n", ka_owens, ka_owens * temp_factor));
    out.push_str(&format!("  Tsivoglou (S={:.3}): {:.3} -> {:.3}\n\n", slope, ka_tsivoglou, ka_tsivoglou * temp_factor));

    // Average
    let ka_avg = (ka_oconnor + ka_churchill + ka_owens) / 3.0;
    let ka_avg_t = ka_avg * temp_factor;
    out.push_str(&format!("  >> Average (3 formulas): {:.3} day-1 (at {:.0}C: {:.3})\n\n", ka_avg, temp_c, ka_avg_t));

    // Classification
    out.push_str("-- Classification --\n");
    if ka_avg_t < 0.1 { out.push_str("  Slow (pools, lakes): ka < 0.1/day\n"); }
    else if ka_avg_t < 0.5 { out.push_str("  Moderate (slow rivers): 0.1-0.5/day\n"); }
    else if ka_avg_t < 2.0 { out.push_str("  Fast (moderate rivers): 0.5-2.0/day\n"); }
    else { out.push_str("  Very fast (rapids): >2.0/day\n"); }

    out.push_str("\n  Ref: Chapra 2008; O'Connor 1958; Churchill 1962\n");
    out
}
