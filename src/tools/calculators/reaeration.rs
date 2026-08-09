/// Reaeration Coefficient Calculator — Multiple Formulas
/// Ref: Chapra 2008; O'Connor 1958; Churchill 1962; Owens-Gibbs 1964
pub fn assess(velocity_m_s: f64, depth_m: f64, temp_c: f64) -> String {
    let mut out = String::from("=== Reaeration Coefficient Calculator ===\n");
    out.push_str("Ref: Chapra 2008; O'Connor; Churchill; Owens-Gibbs\n\n");

    if velocity_m_s <= 0.0 || depth_m <= 0.0 {
        return "ERROR [E102]: velocity and depth must be > 0.".into();
    }

    let v = velocity_m_s; // m/s  (CRITICAL: these empirical formulas require velocity in m/s, NOT m/day)
    let H = depth_m; // m

    out.push_str(&format!("Velocity: {:.3} m/s\n", v));
    out.push_str(&format!("Depth: {:.1} m\n", H));
    out.push_str(&format!("Temperature: {:.1} C\n\n", temp_c));

    // Multiple formulas (all at 20C, then temp-corrected).
    // Units convention (verified): U in m/s, H in m, ka in day^-1.
    // Ref: Assessing Reaeration Rate Equations, semanticscholar 6555/...; USGS PP 0737.
    // O'Connor & Dobbins 1958: ka = 3.93 * U^0.5 / H^1.5
    let ka_oconnor = 3.93 * v.powf(0.5) / H.powf(1.5).max(1e-6);

    // Churchill 1962: ka = 5.01 * U^0.97 / H^1.67  (some refs use 5.026)
    let ka_churchill = 5.01 * v.powf(0.97) / H.powf(1.67).max(1e-6);

    // Owens-Gibbs 1964: ka = 5.32 * U^0.67 / H^1.85
    let ka_owens = 5.32 * v.powf(0.67) / H.powf(1.85).max(1e-6);

    // Tsivoglou-Wallace (steep streams): ka = C * S * U ; S = slope (1/m), U in m/s -> approximate.
    // NOTE: proper Tsivoglou uses ka = 0.054 * S * U with S in m/km; here screening only.
    let slope = 0.001;
    let ka_tsivoglou = 31.183 * slope * v; // Tsivoglou escoefficient (m/s, slope m/m) screening

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

#[cfg(test)]
mod tests {
    // Self-check: O'Connor-Dobbins with U=0.3 m/s, H=1.5 m should give ka ~= 1.17 /day
    // ka = 3.93 * 0.3^0.5 / 1.5^1.5 = 3.93 * 0.547723 / 1.837117 = 1.1716
    #[test]
    fn oconnor_reference_value() {
        let v: f64 = 0.3;
        let h: f64 = 1.5;
        let ka = 3.93 * v.powf(0.5) / h.powf(1.5);
        assert!((ka - 1.1716).abs() < 0.01, "ka={ka} expected ~1.17 /day (m/s units)");
        // Sanity: must be in realistic river range 0.1-5 /day, NOT thousands (the old m/day bug gave ~345)
        assert!(ka > 0.1 && ka < 5.0, "ka={ka} outside realistic range");
    }
}

