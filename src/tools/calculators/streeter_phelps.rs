/// Streeter-Phelps DO Sag Curve (1925)
/// D(t) = [(k1·L0)/(k2-k1)] × (e^(-k1t) - e^(-k2t)) + D0·e^(-k2t)
/// Supports optional temperature correction (Metcalf & Eddy / O'Connor & Dobbins)

pub fn calculate(k1: f64, k2: f64, l0: f64, d0: f64, velocity_ms: f64, distance_km: f64, temp_c: Option<f64>) -> String {
    let mut out = String::from("=== Streeter-Phelps DO Sag Curve ===\n");
    out.push_str("Ref: Streeter & Phelps (1925), Ohio River Study\n\n");

    if k1 <= 0.0 || k2 <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if (k2 - k1).abs() < 1e-10 { return format!("ERROR: k2 ({:.3}) = k1 ({:.3}). Division by zero.", k2, k1); }
    if l0 <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    // Temperature correction if provided
    let (k1_eff, k2_eff) = match temp_c {
        Some(t) => {
            let k1_t = k1 * 1.047_f64.powf(t - 20.0);
            let k2_t = k2 * 1.024_f64.powf(t - 20.0);
            out.push_str(&format!("Koreksi suhu: T = {:.1}°C\n", t));
            out.push_str(&format!("  k1(20°C) = {:.4} → k1(T) = {:.4} (θ=1.047, Metcalf & Eddy)\n", k1, k1_t));
            out.push_str(&format!("  k2(20°C) = {:.4} → k2(T) = {:.4} (θ=1.024, O'Connor & Dobbins)\n\n", k2, k2_t));
            (k1_t, k2_t)
        },
        None => (k1, k2),
    };

    // Check if k2 < k1 after correction
    let k2_lt_k1 = k2_eff < k1_eff;
    if k2_lt_k1 {
        out.push_str("⚠️ k2 < k1: reaerasi lebih lambat dari deoksigenasi. DO deficit terus meningkat (sungai tidak dapat self-purify pada kondisi ini).\n\n");
    }

    // Waktu kritis — general formula works for both k2>k1 and k2<k1
    let ln_arg = (k2_eff / k1_eff) * (1.0 - d0 * (k2_eff - k1_eff) / (k1_eff * l0));
    let tc = if ln_arg > 0.0 {
        (1.0 / (k2_eff - k1_eff)) * ln_arg.ln()
    } else {
        // No critical point found; deficit keeps increasing
        -1.0
    };

    let dc = if tc > 0.0 {
        ((k1_eff * l0) / (k2_eff - k1_eff)) * ((-k1_eff * tc).exp() - (-k2_eff * tc).exp()) + d0 * (-k2_eff * tc).exp()
    } else {
        -1.0
    };

    // Jarak kritis
    let xc = if tc > 0.0 && velocity_ms > 0.0 { tc * velocity_ms * 86400.0 / 1000.0 } else { 0.0 }; // km

    out.push_str(&format!("Input:\n  k1 (deoxygenation rate) = {:.4} /hari\n  k2 (reaeration rate) = {:.4} /hari\n  L0 (BOD ultimate) = {:.2} mg/L\n  D0 (deficit awal) = {:.2} mg/L\n  Kecepatan sungai = {:.2} m/s\n\n", k1_eff, k2_eff, l0, d0, velocity_ms));

    if tc > 0.0 {
        out.push_str(&format!("Hasil:\n  Waktu kritis (tc) = {:.2} hari\n  Deficit kritis (Dc) = {:.2} mg/L\n  Jarak kritis = {:.2} km dari sumber pencemaran\n\n", tc, dc, xc));
    } else {
        out.push_str("Hasil:\n  Tidak ada titik kritis — deficit terus meningkat sepanjang sungai.\n\n");
    }

    // Hitung DO di beberapa titik
    out.push_str("Profil DO Deficit sepanjang sungai:\n");
    out.push_str("  Jarak(km) | Waktu(hari) | Deficit(mg/L)\n");
    let steps = 10;
    let max_dist = if distance_km > 0.0 { distance_km } else if tc > 0.0 { xc * 3.0 } else { 50.0 };
    for i in 0..=steps {
        let x = max_dist * (i as f64) / (steps as f64);
        let t = if velocity_ms > 0.0 { x * 1000.0 / (velocity_ms * 86400.0) } else { 0.0 };
        let d = ((k1_eff * l0) / (k2_eff - k1_eff)) * ((-k1_eff * t).exp() - (-k2_eff * t).exp()) + d0 * (-k2_eff * t).exp();
        out.push_str(&format!("  {:.1} km | {:.2} hari | {:.2} mg/L\n", x, t, d.max(0.0)));
    }
    out
}
