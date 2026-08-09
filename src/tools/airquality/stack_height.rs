/// Stack Height (GEP — Good Engineering Practice)
/// Ref: EPA 40 CFR 51.100; ASME
pub fn assess(building_height_m: f64, building_width_m: f64, building_length_m: f64, wind_direction_deg: f64) -> String {
    let mut out = String::from("=== Stack Height (GEP) ===\n");
    out.push_str("Ref: EPA 40 CFR 51.100; ASME\n\n");
    let H = building_height_m;
    // GEP = H + 1.5 * min(W, H) for across-wind dimension
    // Direction-specific: W_eff = W*sin(theta) + L*cos(theta)
    let theta = wind_direction_deg.to_radians();
    let w_eff = (building_width_m * theta.sin().abs() + building_length_m * theta.cos().abs()).min(building_width_m.max(building_length_m));
    let L = w_eff.min(H); // characteristic building dimension
    let gep = H + 1.5 * L;
    out.push_str(&format!("Building: H={:.1}m, W={:.1}m, L={:.1}m\n", H, building_width_m, building_length_m));
    out.push_str(&format!("Wind direction: {:.0} deg, W_eff={:.1}m\n\n", wind_direction_deg, w_eff));
    out.push_str(&format!("  GEP = H + 1.5*L = {:.1} + 1.5*{:.1} = {:.1} m\n\n", H, L, gep));
    out.push_str(&format!("  >> Minimum stack height (GEP): {:.1} m\n\n", gep));
    out.push_str("  Cavity zone: 0.5L downwind (recirculation)\n");
    out.push_str("  Wake region: 2-3L downwind (turbulence)\n");
    out.push_str("  Stack must be above wake to avoid downwash\n");
    out.push_str("\n  Ref: EPA 40 CFR 51.100; ASME\n");
    out
}
