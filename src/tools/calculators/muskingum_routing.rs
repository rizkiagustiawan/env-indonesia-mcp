/// Muskingum Flood Routing
/// O₂ = C₀×I₂ + C₁×I₁ + C₂×O₁
/// Ref: McCarthy (1938), Chow (1959) Open-Channel Hydraulics

pub fn route(inflow_hydrograph: &[(f64, f64)], k_hours: f64, x: f64, dt_hours: f64) -> String {
    let mut out = String::from("=== Muskingum Flood Routing ===\n");
    out.push_str("Ref: McCarthy (1938), Chow (1959)\n\n");

    if inflow_hydrograph.len() < 2 { return "ERROR: Minimal 2 titik data inflow diperlukan.".into(); }
    if k_hours <= 0.0 { return "ERROR: K (storage constant) harus > 0.".into(); }
    if x < 0.0 || x > 0.5 { return "ERROR: x harus antara 0 dan 0.5.".into(); }
    if dt_hours <= 0.0 { return "ERROR: Δt harus > 0.".into(); }

    // Check stability: K should satisfy 2Kx < Δt < 2K(1-x)
    let lower = 2.0 * k_hours * x;
    let upper = 2.0 * k_hours * (1.0 - x);
    if dt_hours < lower || dt_hours > upper {
        out.push_str(&format!("⚠️ PERINGATAN: Δt = {:.2} jam di luar rentang stabil [{:.2}, {:.2}] jam\n\n", dt_hours, lower, upper));
    }

    // Muskingum coefficients
    let denom = k_hours - k_hours * x + 0.5 * dt_hours;
    let c0 = (-k_hours * x + 0.5 * dt_hours) / denom;
    let c1 = (k_hours * x + 0.5 * dt_hours) / denom;
    let c2 = (k_hours - k_hours * x - 0.5 * dt_hours) / denom;

    // Verify C0 + C1 + C2 = 1
    let c_sum = c0 + c1 + c2;

    out.push_str(&format!("Input:\n  Jumlah data inflow = {} titik\n  K = {:.2} jam\n  x = {:.2}\n  Δt = {:.2} jam\n\n", inflow_hydrograph.len(), k_hours, x, dt_hours));

    out.push_str("Koefisien Muskingum:\n");
    out.push_str(&format!("  C₀ = (-Kx + 0.5Δt) / (K-Kx+0.5Δt) = {:.4}\n", c0));
    out.push_str(&format!("  C₁ = (Kx + 0.5Δt) / (K-Kx+0.5Δt) = {:.4}\n", c1));
    out.push_str(&format!("  C₂ = (K-Kx - 0.5Δt) / (K-Kx+0.5Δt) = {:.4}\n", c2));
    out.push_str(&format!("  C₀+C₁+C₂ = {:.4} (harus = 1.0)\n\n", c_sum));

    // Route the hydrograph
    let mut outflow: Vec<(f64, f64)> = Vec::new();
    outflow.push(inflow_hydrograph[0]); // initial outflow = initial inflow

    for i in 1..inflow_hydrograph.len() {
        let i2 = inflow_hydrograph[i].1;
        let i1 = inflow_hydrograph[i - 1].1;
        let o1 = outflow[i - 1].1;
        let o2 = c0 * i2 + c1 * i1 + c2 * o1;
        outflow.push((inflow_hydrograph[i].0, o2.max(0.0)));
    }

    // Find peaks
    let peak_in = inflow_hydrograph.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
    let peak_out = outflow.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();

    let attenuation_pct = (1.0 - peak_out.1 / peak_in.1) * 100.0;
    let time_lag = peak_out.0 - peak_in.0;

    out.push_str("Hasil routing:\n");
    out.push_str(&format!("  {:>8} | {:>12} | {:>12}\n", "t (jam)", "Inflow", "Outflow"));
    for i in 0..inflow_hydrograph.len() {
        out.push_str(&format!("  {:>8.2} | {:>12.2} | {:>12.2}\n",
            inflow_hydrograph[i].0, inflow_hydrograph[i].1, outflow[i].1));
    }

    out.push_str(&format!("\nAnalisis puncak:\n  Inflow peak = {:.2} m³/s pada t = {:.2} jam\n  Outflow peak = {:.2} m³/s pada t = {:.2} jam\n",
        peak_in.1, peak_in.0, peak_out.1, peak_out.0));
    out.push_str(&format!("  Atenuasi puncak = {:.1}%\n", attenuation_pct));
    out.push_str(&format!("  Time lag = {:.2} jam\n", time_lag));

    out
}
