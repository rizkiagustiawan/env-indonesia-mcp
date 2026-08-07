/// Ensemble Kalman Filter (EnKF) — Data Assimilation
/// 2026 SOTA: Sun 2026 (IoT water quality ADAPT, beats EnKF),
///   Sahar 2026 (fault detection EnKF), Zhao 2026 (soil-water DA),
///   Sandu 2026 (atmospheric composition DA), Hammoud 2026 (RL+Bayesian)
/// Ref: Evensen 1994 (EnKF); Burgers et al. 1998 (analysis scheme)
/// EnKF: x_a = x_f + K(y - Hx_f), K = P_f H^T (H P_f H^T + R)^{-1}

pub fn assimilate(
    model_states_json: &str,
    observations_json: &str,
    ensemble_size: u32,
    noise_std: f64,
) -> String {
    let mut out = String::from("=== Ensemble Kalman Filter (EnKF) ===\n");
    out.push_str("Ref: Evensen 1994; Burgers et al. 1998\n");
    out.push_str("2026 SOTA: Sun 2026; Sahar 2026; Zhao 2026; Sandu 2026\n\n");

    let model_states: Vec<Vec<f64>> = match serde_json::from_str(model_states_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR [E102]: model_states_json: {}. Format: [[x1,x2,...],...] (ensemble of state vectors)", e),
    };
    let observations: Vec<f64> = match serde_json::from_str(observations_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR [E102]: observations_json: {}. Format: [y1, y2, ...]", e),
    };

    if model_states.is_empty() {
        return "ERROR: model_states_json kosong.".into();
    }

    let n_state = model_states[0].len();
    let n_obs = observations.len();
    let n_ens = model_states.len().min(ensemble_size as usize);

    out.push_str(&format!("State dimension: {}\n", n_state));
    out.push_str(&format!("Observation count: {}\n", n_obs));
    out.push_str(&format!("Ensemble size: {}\n", n_ens));
    out.push_str(&format!("Observation noise σ: {:.3}\n\n", noise_std));

    // ═══ Phase 1: Forecast — Ensemble statistics ═══
    out.push_str("── Phase 1: Forecast (prior) ──\n\n");

    let prior_mean: Vec<f64> = (0..n_state).map(|i| {
        model_states.iter().map(|s| s[i]).sum::<f64>() / n_ens as f64
    }).collect();

    let prior_var: Vec<f64> = (0..n_state).map(|i| {
        let m = prior_mean[i];
        model_states.iter().map(|s| (s[i] - m).powi(2)).sum::<f64>() / (n_ens - 1).max(1) as f64
    }).collect();

    let prior_std: Vec<f64> = prior_var.iter().map(|v| v.sqrt()).collect();

    out.push_str("  Prior mean + uncertainty:\n");
    for i in 0..n_state.min(10) {
        out.push_str(&format!("    x[{}] = {:>10.4} ± {:.4}\n", i, prior_mean[i], prior_std[i]));
    }
    if n_state > 10 {
        out.push_str(&format!("    ... ({} more)\n", n_state - 10));
    }

    // ═══ Phase 2: Update — Kalman gain ═══
    out.push_str("\n── Phase 2: Update (analysis) ──\n\n");

    // H = observation operator (assume identity if n_obs == n_state, else linear projection)
    // P_f H^T = cross-covariance between state and observations
    // For simplicity: H projects state to observation space

    let mut updated_states: Vec<Vec<f64>> = Vec::new();

    // Observation perturbations
    let obs_pert: Vec<Vec<f64>> = (0..n_ens).map(|_| {
        (0..n_obs).map(|_| {
            // Box-Muller for Gaussian noise
            let u1 = 1e-10 + rand_f64();
            let u2 = rand_f64();
            noise_std * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        }).collect()
    }).collect();

    let mut innovation_sum = 0.0f64;
    let mut total_k_gain = 0.0f64;

    for e in 0..n_ens {
        let mut state = model_states[e % n_ens].clone();

        // Project state to obs space (H = identity for matching indices)
        for (j, y_obs) in observations.iter().enumerate() {
            if j >= n_state { break; }

            let y_pred = state[j]; // H = identity
            let y_pert = obs_pert[e][j];
            let y = y_obs + y_pert;

            // Innovation (d = y - Hx)
            let innovation = y - y_pred;
            innovation_sum += innovation.abs();

            // Kalman gain (simplified: K = P_f / (P_f + R))
            let pf = prior_var[j];
            let r = noise_std * noise_std;
            let k = pf / (pf + r).max(1e-10);
            if j == 0 { total_k_gain = k; }

            // Analysis: x_a = x_f + K * d
            state[j] = y_pred + k * innovation;
        }

        updated_states.push(state);
    }

    // Posterior statistics
    let posterior_mean: Vec<f64> = (0..n_state).map(|i| {
        updated_states.iter().map(|s| s[i]).sum::<f64>() / n_ens as f64
    }).collect();

    let posterior_var: Vec<f64> = (0..n_state).map(|i| {
        let m = posterior_mean[i];
        updated_states.iter().map(|s| (s[i] - m).powi(2)).sum::<f64>() / (n_ens - 1).max(1) as f64
    }).collect();

    let posterior_std: Vec<f64> = posterior_var.iter().map(|v| v.sqrt()).collect();

    let mean_innovation = innovation_sum / n_ens as f64;

    out.push_str("  Posterior mean + uncertainty:\n");
    for i in 0..n_state.min(10) {
        let improvement = prior_std[i] - posterior_std[i];
        let pct = if prior_std[i] > 0.0 { (improvement / prior_std[i]) * 100.0 } else { 0.0 };
        out.push_str(&format!("    x[{}] = {:>10.4} ± {:.4}  (σ reduced {:.1}%)\n",
            i, posterior_mean[i], posterior_std[i], pct));
    }

    out.push_str(&format!("\n  Kalman gain (K): {:.4}", total_k_gain));
    out.push_str(&format!("  (high K = trust obs; low K = trust model)\n"));
    out.push_str(&format!("  Mean innovation |y - Hx|: {:.4}\n\n", mean_innovation));

    // ═══ Phase 3: Uncertainty reduction ═══
    out.push_str("── Phase 3: Uncertainty Reduction ──\n\n");

    let avg_prior = prior_std.iter().sum::<f64>() / prior_std.len() as f64;
    let avg_posterior = posterior_std.iter().sum::<f64>() / posterior_std.len() as f64;
    let reduction = if avg_prior > 0.0 {
        ((avg_prior - avg_posterior) / avg_prior) * 100.0
    } else { 0.0 };

    out.push_str(&format!("  Avg prior σ:     {:.4}\n", avg_prior));
    out.push_str(&format!("  Avg posterior σ: {:.4}\n", avg_posterior));
    out.push_str(&format!("  Uncertainty reduction: {:.1}%\n\n", reduction));

    if reduction > 50.0 {
        out.push_str("  🟢 Significant improvement — observations well integrated\n");
    } else if reduction > 20.0 {
        out.push_str("  🟡 Moderate improvement — some model-observation agreement\n");
    } else {
        out.push_str("  🟠 Low improvement — observations may be inconsistent or sparse\n");
    }

    // ═══ Summary ═══
    out.push_str("\n═══ EnKF SUMMARY ═══\n\n");
    out.push_str(&format!("  Ensemble: {}\n", n_ens));
    out.push_str(&format!("  States: {}  Obs: {}\n", n_state, n_obs));
    out.push_str(&format!("  Kalman gain: {:.4}\n", total_k_gain));
    out.push_str(&format!("  Innovation: {:.4}\n", mean_innovation));
    out.push_str(&format!("  σ reduction: {:.1}%\n", reduction));

    out.push_str("\n  Ref: Sun 2026 (ADAPT IoT water quality); Sahar 2026 (fault detection);\n");
    out.push_str("       Zhao 2026 (soil-water DA); Sandu 2026 (atmospheric composition)\n");

    // Honest limitation
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • Assumes Gaussian distributions (particle filter for non-Gaussian)\n");
    out.push_str("  • H = identity (simplified — real H may be nonlinear)\n");
    out.push_str("  • For production: use PDAF (Parallel Data Assimilation Framework)\n");
    out.push_str("  • ADAPT (Sun 2026) outperforms EnKF for water quality forecasting\n");

    // Return updated JSON for downstream use
    let updated_json = serde_json::to_string(&posterior_mean).unwrap_or_default();
    out.push_str(&format!("\n  Updated states JSON: {}\n", updated_json));

    out
}

fn rand_f64() -> f64 {
    // Simple PRNG (xorshift) — no external dependency
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(123456789);
    let mut x = SEED.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    SEED.store(x, Ordering::Relaxed);
    (x % 1000000) as f64 / 1000000.0
}
