/// Streeter-Phelps DO Sag Curve (1925)
/// D(t) = [(k1·L0)/(k2-k1)] × (e^(-k1t) - e^(-k2t)) + D0·e^(-k2t)

pub fn calculate(k1: f64, k2: f64, l0: f64, d0: f64, velocity_ms: f64, distance_km: f64) -> String {
    let mut out = String::from("=== Streeter-Phelps DO Sag Curve ===\n");
    out.push_str("Ref: Streeter & Phelps (1925), Ohio River Study\n\n");

    if k1 <= 0.0 || k2 <= 0.0 { return "ERROR: k1 dan k2 harus > 0.".into(); }
    if k2 <= k1 { return format!("ERROR: k2 ({:.3}) harus > k1 ({:.3}) agar sungai bisa self-purify.", k2, k1); }
    if l0 <= 0.0 { return "ERROR: BOD ultimate (L0) harus > 0.".into(); }

    // Waktu kritis
    let tc = (1.0 / (k2 - k1)) * ((k2 / k1) * (1.0 - d0 * (k2 - k1) / (k1 * l0))).ln();
    let dc = ((k1 * l0) / (k2 - k1)) * ((-k1 * tc).exp() - (-k2 * tc).exp()) + d0 * (-k2 * tc).exp();

    // Jarak kritis
    let xc = if velocity_ms > 0.0 { tc * velocity_ms * 86400.0 / 1000.0 } else { 0.0 }; // km

    out.push_str(&format!("Input:\n  k1 (deoxygenation rate) = {:.3} /hari\n  k2 (reaeration rate) = {:.3} /hari\n  L0 (BOD ultimate) = {:.2} mg/L\n  D0 (deficit awal) = {:.2} mg/L\n  Kecepatan sungai = {:.2} m/s\n\n", k1, k2, l0, d0, velocity_ms));

    out.push_str(&format!("Hasil:\n  Waktu kritis (tc) = {:.2} hari\n  Deficit kritis (Dc) = {:.2} mg/L\n  Jarak kritis = {:.2} km dari sumber pencemaran\n\n", tc, dc, xc));

    // Hitung DO di beberapa titik
    out.push_str("Profil DO Deficit sepanjang sungai:\n");
    out.push_str("  Jarak(km) | Waktu(hari) | Deficit(mg/L)\n");
    let steps = 10;
    let max_dist = if distance_km > 0.0 { distance_km } else { xc * 3.0 };
    for i in 0..=steps {
        let x = max_dist * (i as f64) / (steps as f64);
        let t = if velocity_ms > 0.0 { x * 1000.0 / (velocity_ms * 86400.0) } else { 0.0 };
        let d = ((k1 * l0) / (k2 - k1)) * ((-k1 * t).exp() - (-k2 * t).exp()) + d0 * (-k2 * t).exp();
        out.push_str(&format!("  {:.1} km | {:.2} hari | {:.2} mg/L\n", x, t, d.max(0.0)));
    }
    out
}
