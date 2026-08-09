/// Indoor Air Quality Model
/// Ref: ASHRAE 62.1; EPA Indoor Air Quality
pub fn assess(emission_rate_mg_hr: f64, room_volume_m3: f64, ventilation_m3_hr: f64, outdoor_conc_mg_m3: f64, deposition_rate_hr: f64) -> String {
    let mut out = String::from("=== Indoor Air Quality Model ===\n");
    out.push_str("Ref: ASHRAE 62.1; EPA IAQ\n\n");
    let Q = ventilation_m3_hr; let V = room_volume_m3; let G = emission_rate_mg_hr;
    let C_out = outdoor_conc_mg_m3; let k = deposition_rate_hr;
    let ach = Q / V; // air changes per hour
    // Steady state: C = (G/V + Q*C_out/V) / (Q/V + k) = (G + Q*C_out) / (Q + k*V)
    let c_steady = (G + Q * C_out) / (Q + k * V).max(1e-6);
    out.push_str(&format!("Emission: {:.1} mg/hr, Room: {:.0} m3, Vent: {:.0} m3/hr\n", G, V, Q));
    out.push_str(&format!("ACH: {:.1}/hr, Outdoor C: {:.3} mg/m3\n\n", ach, C_out));
    out.push_str(&format!("  >> Steady-state C: {:.4} mg/m3 ({:.1} ug/m3)\n\n", c_steady, c_steady*1000.0));
    // Time to 95% steady state: t = -ln(0.05) / (ACH + k)
    let t95 = 3.0 / (ach + k).max(1e-6);
    out.push_str(&format!("  Time to 95% steady: {:.1} hr\n", t95));
    if ach < 0.3 { out.push_str("  [WARN] ACH < 0.3 — inadequate ventilation per ASHRAE\n"); }
    else if ach < 1.0 { out.push_str("  [WARN] ACH < 1.0 — minimum for residential\n"); }
    else { out.push_str("  [OK] ACH adequate\n"); }
    out.push_str("\n  Ref: ASHRAE 62.1; EPA IAQ\n");
    out
}
