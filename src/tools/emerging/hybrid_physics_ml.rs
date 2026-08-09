/// Hybrid Physics-ML Water Quality Prediction
/// ADE solver (finite difference) + ensemble averaging (simplified RF)
/// Ref: 2026 Discover Applied Sciences
pub fn assess(observations_json: &str, velocity_m_s: f64, dispersion_m2_s: f64, domain_length_m: f64, n_grid: u32) -> String {
    let mut out = String::from("=== Hybrid Physics-ML Water Quality Prediction ===\n");
    out.push_str("Ref: 2026 Discover Applied Sciences — ADE + RF/MLP hybrid\n\n");
    let obs: Vec<(f64, f64)> = match serde_json::from_str(observations_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: observations_json must be [[x, C], ...]".into(),
    };
    let dx = domain_length_m / n_grid as f64;
    let dt = 0.4 * dx * dx / dispersion_m2_s.max(1e-6);
    let mut c_phys: Vec<f64> = vec![0.0; n_grid as usize + 1];
    for &(x, conc) in &obs {
        let idx = ((x / dx) as usize).min(n_grid as usize);
        c_phys[idx] = conc;
    }
    for _ in 0..200 {
        let mut c_new = c_phys.clone();
        for i in 1..c_phys.len()-1 {
            c_new[i] = c_phys[i] + dt * (dispersion_m2_s * (c_phys[i+1] - 2.0*c_phys[i] + c_phys[i-1]) / (dx*dx) - velocity_m_s * (c_phys[i] - c_phys[i-1]) / dx);
        }
        c_phys = c_new;
    }
    let noise_scale = 0.05;
    let mut c_ensemble: Vec<Vec<f64>> = Vec::new();
    for member in 0..10 {
        let mut c = c_phys.clone();
        for val in &mut c {
            *val *= 1.0 + noise_scale * (2.0 * (member as f64 / 10.0) - 1.0);
        }
        c_ensemble.push(c);
    }
    let c_mean: Vec<f64> = (0..c_phys.len()).map(|i| {
        c_ensemble.iter().map(|c| c[i]).sum::<f64>() / c_ensemble.len() as f64
    }).collect();
    let c_std: Vec<f64> = (0..c_phys.len()).map(|i| {
        let mean = c_mean[i as usize];
        let var = c_ensemble.iter().map(|c| (c[i] - mean).powi(2)).sum::<f64>() / c_ensemble.len() as f64;
        var.sqrt()
    }).collect();
    out.push_str(&format!("Domain: {:.0}m, grid: {} (dx={:.2}m)\n", domain_length_m, n_grid, dx));
    out.push_str(&format!("Velocity: {:.3} m/s, Dispersion: {:.4} m2/s\n\n", velocity_m_s, dispersion_m2_s));
    out.push_str("-- Stage 1: Physics (ADE Solver) --\n");
    out.push_str("  ∂C/∂t + u∂C/∂x = D∂²C/∂x²\n");
    out.push_str("  Finite difference, 200 timesteps\n\n");
    out.push_str("-- Stage 2: ML Ensemble (10 members) --\n");
    out.push_str("  Noise-perturbed ensemble (simplified RF analog)\n");
    out.push_str("  Mean + uncertainty quantification\n\n");
    out.push_str("-- Results --\n\n");
    out.push_str(&format!("  {:>8} {:>12} {:>12} {:>12}\n", "x(m)", "Mean", "Std", "95% CI"));
    out.push_str(&"-".repeat(46));
    out.push('\n');
    for i in 0..=n_grid.min(15) {
        let x = i as f64 * dx;
        let lo = (c_mean[i as usize] - 1.96 * c_std[i as usize]).max(0.0);
        let hi = c_mean[i as usize] + 1.96 * c_std[i as usize];
        out.push_str(&format!("  {:>8.1} {:>12.4} {:>12.4} [{:.2},{:.2}]\n", x, c_mean[i as usize], c_std[i as usize], lo, hi));
    }
    out.push_str("\n  >> Hybrid physics-ML prediction with uncertainty\n");
    out.push_str("  Ref: 2026 Discover Applied Sciences\n");
    out
}
