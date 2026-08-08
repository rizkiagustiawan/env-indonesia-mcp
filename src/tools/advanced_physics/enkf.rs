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

    // ═══ Phase 2: Update — Multivariate Kalman gain ═══
    // Ref: Mandel 2009 (arXiv:0901.3725); Evensen 2003
    // Full formula:
    //   C = A·A^T / (N-1)   (n×n covariance from ensemble anomaly)
    //   S = H·C·H^T + R     (m×m innovation covariance)
    //   K = C·H^T·S^{-1}    (n×m Kalman gain)
    //   X_p = X + K·(D - H·X) (posterior ensemble)
    out.push_str("\n── Phase 2: Update (Multivariate EnKF Analysis) ──\n\n");
    out.push_str("Ref: Mandel 2009; Evensen 2003 — full matrix K = C·H^T·(H·C·H^T+R)^{-1}\n\n");

    // Build ensemble anomaly matrix A (n_state × n_ens)
    // A[k] = x_k - mean
    let anomaly: Vec<Vec<f64>> = model_states.iter()
        .map(|s| s.iter().enumerate()
            .map(|(i, x)| x - prior_mean[i])
            .collect())
        .collect();

    // Compute full covariance C = A·A^T / (N-1)  (n_state × n_state)
    let mut cov = vec![vec![0.0; n_state]; n_state];
    for i in 0..n_state {
        for j in 0..n_state {
            let mut sum = 0.0;
            for k in 0..n_ens {
                sum += anomaly[k][i] * anomaly[k][j];
            }
            cov[i][j] = sum / (n_ens - 1).max(1) as f64;
        }
    }

    // H = identity (m×n, first m state vars = observations)
    // S = H·C·H^T + R = C[0:m, 0:m] + R  (m×m)
    let m = n_obs.min(n_state);
    let mut s_matrix = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            s_matrix[i][j] = cov[i][j];
        }
        s_matrix[i][i] += noise_std * noise_std; // R diagonal
    }

    // Display covariance matrix (if small)
    if n_state <= 5 {
        out.push_str("  Prior Covariance Matrix C (n×n):\n  ");
        for i in 0..n_state {
            for j in 0..n_state {
                out.push_str(&format!("{:>8.4} ", cov[i][j]));
            }
            out.push_str("\n  ");
        }
        out.push('\n');
    }

    // S^{-1} via Gauss-Jordan elimination
    let s_inv = match matrix_inverse(&s_matrix, m) {
        Some(inv) => inv,
        None => {
            out.push_str("  ⚠️ S matrix singular — using diagonal approximation\n");
            // Fallback: diagonal only
            let mut inv = vec![vec![0.0; m]; m];
            for i in 0..m {
                inv[i][i] = 1.0 / s_matrix[i][i].max(1e-10);
            }
            inv
        }
    };

    // K = C·H^T·S^{-1}  (n×m)
    // C·H^T = C[0:n, 0:m] (since H=identity[0:m, 0:n])
    // Then multiply by S^{-1} (m×m)
    let mut k_gain = vec![vec![0.0; m]; n_state];
    for i in 0..n_state {
        for j in 0..m {
            // (C·H^T)[i][j] = C[i][j] (for H=identity)
            let mut sum = 0.0;
            for l in 0..m {
                sum += cov[i][l] * s_inv[l][j];
            }
            k_gain[i][j] = sum;
        }
    }

    // Display K matrix (first 3 rows)
    out.push_str("  Kalman Gain Matrix K (n×m):\n");
    let k_rows = n_state.min(5);
    for i in 0..k_rows {
        out.push_str(&format!("  K[{}] = [", i));
        for j in 0..m {
            out.push_str(&format!("{:>8.4} ", k_gain[i][j]));
        }
        out.push_str("]\n");
    }
    if n_state > 5 { out.push_str(&format!("  ... ({} more rows)\n", n_state - 5)); }
    out.push('\n');

    // Observation perturbations (D = d + N(0,R) per ensemble)
    let obs_pert: Vec<Vec<f64>> = (0..n_ens).map(|_| {
        (0..n_obs).map(|_| {
            let u1 = 1e-10 + rand_f64();
            let u2 = rand_f64();
            noise_std * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        }).collect()
    }).collect();

    // Posterior: X_p = X + K·(D - H·X)
    // For each ensemble member: x_a = x_f + K·(d_pert - H·x_f)
    let mut updated_states: Vec<Vec<f64>> = Vec::new();
    let mut innovation_sum = 0.0f64;

    for e in 0..n_ens {
        let mut state = model_states[e % n_ens].clone();

        // Innovation: d = y_pert - H·x = (y_obs + pert) - x[0:m]
        let mut innovation = vec![0.0; m];
        for j in 0..m {
            innovation[j] = (observations[j] + obs_pert[e][j]) - state[j];
            innovation_sum += innovation[j].abs();
        }

        // x_a = x_f + K·d  (matrix-vector multiply)
        for i in 0..n_state {
            let mut correction = 0.0;
            for j in 0..m {
                correction += k_gain[i][j] * innovation[j];
            }
            state[i] += correction;
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

    out.push_str(&format!("\n  Kalman gain K[0,0]: {:.4} (diagonal)", k_gain[0][0]));
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
    out.push_str(&format!("  Kalman gain K[0,0]: {:.4}\n", k_gain[0][0]));
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

/// Matrix inverse via Gauss-Jordan elimination
/// Pure Rust, no external dependency
/// Ref: Golub & Van Loan 1989 (Matrix Computations)
fn matrix_inverse(matrix: &[Vec<f64>], n: usize) -> Option<Vec<Vec<f64>>> {
    // Build augmented [A | I]
    let mut aug = vec![vec![0.0; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = matrix[i][j];
        }
        aug[i][n + i] = 1.0;
    }

    // Forward elimination with partial pivoting
    for col in 0..n {
        // Find pivot
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }

        if max_val < 1e-12 {
            return None; // singular
        }

        // Swap rows
        if max_row != col {
            aug.swap(col, max_row);
        }

        // Scale pivot row
        let pivot = aug[col][col];
        for j in 0..(2 * n) {
            aug[col][j] /= pivot;
        }

        // Eliminate column in other rows
        for row in 0..n {
            if row == col { continue; }
            let factor = aug[row][col];
            if factor.abs() < 1e-15 { continue; }
            for j in 0..(2 * n) {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Extract inverse from augmented [I | A^{-1}]
    let mut inv = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            inv[i][j] = aug[i][n + j];
        }
    }

    Some(inv)
}
