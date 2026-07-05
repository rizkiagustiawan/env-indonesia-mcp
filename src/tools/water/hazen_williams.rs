/// Hazen-Williams: hf = (10.67 × L × Q^1.852) / (C^1.852 × D^4.87)
/// Ref: Hazen & Williams (1903), Water Supply Engineering

pub fn calculate(q_m3s: f64, length_m: f64, diameter_m: f64, c_coeff: f64, include_minor_losses: bool) -> String {
    let mut out = String::from("=== Hazen-Williams Head Loss ===\n");
    out.push_str("Ref: Hazen & Williams (1903)\n\n");

    if q_m3s <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if length_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if diameter_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if c_coeff <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    let pi = std::f64::consts::PI;

    // Head loss (Hazen-Williams)
    let hf = (10.67 * length_m * q_m3s.powf(1.852)) / (c_coeff.powf(1.852) * diameter_m.powf(4.87));

    // Velocity
    let area = pi * (diameter_m / 2.0).powi(2);
    let velocity = q_m3s / area;

    // Reynolds number estimate (assume water at 20°C, ν = 1.004e-6 m²/s)
    let nu = 1.004e-6;
    let re = velocity * diameter_m / nu;

    // Minor losses (typically 10-20% of friction loss)
    let minor_loss = if include_minor_losses { hf * 0.15 } else { 0.0 };
    let total_loss = hf + minor_loss;

    out.push_str(&format!("Input:\n  Q = {:.4} m³/s ({:.2} L/s)\n  L = {:.1} m\n  D = {:.3} m ({:.0} mm)\n  C (Hazen-Williams) = {:.0}\n  Minor losses = {}\n\n",
        q_m3s, q_m3s * 1000.0, length_m, diameter_m, diameter_m * 1000.0, c_coeff,
        if include_minor_losses { "Ya (15% friction)" } else { "Tidak" }));

    out.push_str("Hasil:\n");
    out.push_str(&format!("  Head loss (hf) = {:.3} m\n", hf));
    if include_minor_losses {
        out.push_str(&format!("  Minor losses = {:.3} m\n", minor_loss));
        out.push_str(&format!("  Total head loss = {:.3} m\n", total_loss));
    }
    out.push_str(&format!("  Gradient hidraulik = {:.4} m/m\n", hf / length_m));
    out.push_str(&format!("  Kecepatan aliran (v) = {:.2} m/s\n", velocity));
    out.push_str(&format!("  Estimasi Re = {:.0}\n\n", re));

    // Velocity warnings
    if velocity > 3.0 {
        out.push_str("⚠️ PERINGATAN: Kecepatan > 3 m/s — risiko water hammer & erosi pipa!\n");
    } else if velocity < 0.6 {
        out.push_str("⚠️ PERINGATAN: Kecepatan < 0.6 m/s — risiko sedimentasi dalam pipa!\n");
    } else {
        out.push_str("✅ Kecepatan dalam rentang optimal (0.6 - 3.0 m/s)\n");
    }

    // C coefficient table
    out.push_str("\nKoefisien C Hazen-Williams:\n");
    out.push_str("  PVC             : C = 150\n");
    out.push_str("  PE (polyethylene): C = 140\n");
    out.push_str("  Baja baru       : C = 120\n");
    out.push_str("  Baja lama       : C = 100\n");
    out.push_str("  Besi tuang      : C = 100\n");
    out.push_str("  Beton           : C = 110\n");
    out.push_str("  Asbestos cement  : C = 140\n");

    out
}
