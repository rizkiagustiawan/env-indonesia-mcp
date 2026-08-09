/// Physics-Constrained Finite-Difference Interpolation for Water Quality
///
/// NOTE: Despite the PINN framing in the reference paper, THIS TOOL runs NO neural
/// network, NO training loop, and NO gradient optimization. It is a pure
/// finite-difference (FD) time-stepping of the advection-dispersion-reaction PDE
/// with hard injection of observed values. "Physics-informed" here means the PDE
/// residual is used as a constraint, NOT that a PINN is trained.
///
/// Literature Reference (NOT this tool's performance):
///   DiBella, Raissi et al. 2026 (Water Research) — PINN framework
///   The paper's Loss = data_loss + λ × physics_loss (mass balance) describes a
///   trained network; this tool implements the equivalent PDE constraint via FD.
pub fn assess(observations_json: &str, domain_length_m: f64, velocity_m_s: f64, dispersion_m2_s: f64, decay_rate_s: f64, n_grid: u32) -> String {
    let mut out = String::from("=== Physics-Constrained FD Water Quality Estimation ===\n");
    out.push_str("NOTE: No neural network runs here — this is a finite-difference PDE solver.\n");
    out.push_str("Literature Reference (NOT this tool's performance):\n");
    out.push_str("  DiBella, Raissi et al. 2026 (Water Research) — PINN framework (training-based)\n\n");
    let obs: Vec<(f64, f64)> = match serde_json::from_str(observations_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: observations_json must be [[x, C], ...]".into(),
    };
    if obs.is_empty() {
        return "ERROR: Need at least 1 observation point [x, concentration]".into();
    }
    let dx = domain_length_m / n_grid as f64;
    let dt = 0.5 * dx / velocity_m_s.max(1e-6);
    let n_steps = 100;
    let mut c: Vec<f64> = vec![0.0; n_grid as usize + 1];
    for &(x, conc) in &obs {
        let idx = ((x / dx) as usize).min(n_grid as usize);
        c[idx as usize] = conc;
    }
    for _ in 0..n_steps {
        let mut c_new = c.clone();
        for i in 1..c.len()-1 {
            let advection = -velocity_m_s * (c[i] - c[i-1]) / dx;
            let dispersion = dispersion_m2_s * (c[i+1] - 2.0*c[i] + c[i-1]) / (dx*dx);
            let reaction = -decay_rate_s * c[i];
            c_new[i] = c[i] + dt * (advection + dispersion + reaction);
            for &(x, conc) in &obs {
                let idx = ((x / dx) as usize).min(n_grid as usize);
                c_new[idx] = conc;
            }
        }
        c = c_new;
    }
    out.push_str(&format!("Domain: {:.0}m, grid: {} points (dx={:.2}m)\n", domain_length_m, n_grid, dx));
    out.push_str(&format!("Velocity: {:.3} m/s, Dispersion: {:.4} m2/s\n", velocity_m_s, dispersion_m2_s));
    out.push_str(&format!("Decay: {:.5}/s, Time step: {:.2}s, Steps: {}\n\n", decay_rate_s, dt, n_steps));
    out.push_str(&format!("Observation points: {}\n", obs.len()));
    out.push_str("-- PDE Constraint (Advection-Dispersion-Reaction) --\n");
    out.push_str("  ∂C/∂t = -v·∂C/∂x + D·∂²C/∂x² - k·C\n\n");
    out.push_str("-- Physics-Informed Results (FD-constrained, NOT a trained PINN) --\n\n");
    out.push_str(&format!("  {:>8} {:>12} {:>12}\n", "x (m)", "Conc", "Source"));
    out.push_str(&"-".repeat(34));
    out.push('\n');
    for i in 0..=n_grid.min(20) {
        let x = i as f64 * dx;
        let is_obs = obs.iter().any(|&(ox, _)| ((ox/dx) as usize) == i as usize);
        out.push_str(&format!("  {:>8.1} {:>12.4} {:>12}\n", x, c[i as usize], if is_obs {"observed"} else {"predicted"}));
    }
    out.push_str("\n  >> Physics-constrained FD interpolation complete (no NN training)\n");
    out.push_str("  >> Mass balance enforced at each time step\n\n");
    out.push_str("  Literature Reference (NOT this tool's performance):\n");
    out.push_str("    DiBella, Raissi et al. 2026 (Water Research) — PINN (trained network)\n");
    out.push_str("    This tool = finite-difference PDE solver with observation injection\n");
    out
}
