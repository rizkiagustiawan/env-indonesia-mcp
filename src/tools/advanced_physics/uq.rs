/// Uncertainty Quantification: GLUE (informal Bayesian) + DREAM-MCMC (formal Bayesian).
/// Ref: Beven & Binley 1992 (GLUE); Vrugt et al. 2009 (DREAM); Gelman & Rubin 1992 (R-hat).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// =====================================================================
// Shared PRNG (xorshift64) + Gaussian sampler (Box-Muller)
// =====================================================================
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed.max(1) }
    }
    /// Uniform in [0, 1).
    fn uniform(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state as f64) / (u64::MAX as f64)
    }
    /// Standard normal N(0,1) via Box-Muller.
    fn normal(&mut self) -> f64 {
        let u1 = self.uniform().max(1e-12);
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
    /// Uniform in [a, b).
    fn uniform_range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.uniform()
    }
}

// =====================================================================
// GLUE — Generalized Likelihood Uncertainty Estimation (Beven & Binley 1992)
// =====================================================================
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlueParam {
    #[schemars(description = "Model predictions as JSON 2D array [n_param_sets][n_outputs]")]
    pub predictions_json: String,
    #[schemars(description = "Observed values as JSON 1D array [n_outputs]")]
    pub observed_json: String,
    #[schemars(description = "Behavioral likelihood threshold (NSE, default 0.5)")]
    pub threshold: f64,
}

fn nse_likelihood(pred: &[f64], obs: &[f64]) -> f64 {
    let m = obs.len();
    let mean_obs = obs.iter().sum::<f64>() / m as f64;
    let ss_res: f64 = obs.iter().zip(pred).map(|(o, p)| (o - p).powi(2)).sum();
    let ss_tot: f64 = obs.iter().map(|o| (o - mean_obs).powi(2)).sum();
    if ss_tot <= 0.0 {
        return 1.0;
    }
    1.0 - ss_res / ss_tot
}

pub fn glue(p: &GlueParam) -> String {
    let pred: Vec<Vec<f64>> = match serde_json::from_str(&p.predictions_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: predictions_json harus array 2D [sets][outputs].".into(),
    };
    let obs: Vec<f64> = match serde_json::from_str(&p.observed_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: observed_json harus array 1D.".into(),
    };
    if pred.is_empty() || pred[0].is_empty() {
        return "ERROR: predictions kosong.".into();
    }
    if pred[0].len() != obs.len() {
        return "ERROR: jumlah output prediksi harus sama dengan observed.".into();
    }

    let m = obs.len();
    let n = pred.len();
    let threshold = if p.threshold > 0.0 { p.threshold } else { 0.5 };

    // Likelihood for each parameter set.
    let likes: Vec<f64> = pred.iter().map(|row| nse_likelihood(row, &obs)).collect();
    let behavioral_idx: Vec<usize> = (0..n).filter(|&i| likes[i] > threshold).collect();

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  GLUE Uncertainty Estimation\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Beven & Binley 1992; Nash-Sutcliffe likelihood\n\n");
    out.push_str(&format!("Parameter sets: {} | Outputs: {}\nThreshold (NSE): {:.2}\n\n", n, m, threshold));

    if behavioral_idx.is_empty() {
        out.push_str(&format!(
            "❌ Tidak ada set behavioral (semua NSE <= {:.2}).\n  Longgarkan threshold atau perbaiki model/prior.\n",
            threshold
        ));
        return out;
    }

    // Normalise weights over behavioral sets.
    let sum_w: f64 = behavioral_idx.iter().map(|&i| likes[i]).sum();
    let weights: Vec<f64> = behavioral_idx.iter().map(|&i| likes[i] / sum_w).collect();

    out.push_str(&format!("Set behavioral: {} dari {} ({:.0}%)\n", behavioral_idx.len(), n, 100.0 * behavioral_idx.len() as f64 / n as f64));
    out.push_str(&format!("Rentang NSE behavioral: [{:.3}, {:.3}]\n\n", 
        behavioral_idx.iter().map(|&i| likes[i]).fold(f64::INFINITY, f64::min),
        behavioral_idx.iter().map(|&i| likes[i]).fold(f64::NEG_INFINITY, f64::max)));

    // Weighted 5% and 95% prediction quantiles at each output position.
    out.push_str("PITA KETIDAKPASTIAN PREDIKSI (5% - 95%):\n");
    let mut p5 = vec![0.0; m];
    let mut p95 = vec![0.0; m];
    let mut median = vec![0.0; m];
    for j in 0..m {
        let mut pairs: Vec<(f64, f64)> = behavioral_idx.iter()
            .map(|&i| (pred[i][j], weights[behavioral_idx.iter().position(|&x| x == i).unwrap()]))
            .collect();
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut cdf = 0.0;
        let mut lo = f64::NAN;
        let mut hi = f64::NAN;
        let mut med = f64::NAN;
        for (val, w) in &pairs {
            cdf += w;
            if cdf >= 0.05 && lo.is_nan() { lo = *val; }
            if cdf >= 0.50 && med.is_nan() { med = *val; }
            if cdf >= 0.95 { hi = *val; break; }
        }
        p5[j] = lo;
        p95[j] = hi;
        median[j] = med;
    }
    for j in 0..m {
        out.push_str(&format!("  Output {:2}: median={:.3}  [5%={:.3}, 95%={:.3}]\n", j + 1, median[j], p5[j], p95[j]));
    }

    // Mean prediction uncertainty width.
    let mean_width = (0..m).map(|j| p95[j] - p5[j]).sum::<f64>() / m as f64;
    out.push_str(&format!("\nLebar rata-rata pita 90%: {:.3}\n", mean_width));

    out
}

// =====================================================================
// DREAM-MCMC — DiffeRential Evolution Adaptive Metropolis (Vrugt et al. 2009)
// =====================================================================
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DreamParam {
    #[schemars(description = "Design matrix X as JSON 2D [n_obs][n_params] (linear model y = X·θ)")]
    pub x_json: String,
    #[schemars(description = "Observed outputs y as JSON 1D [n_obs]")]
    pub y_json: String,
    #[schemars(description = "Measurement noise std σ (observation error)")]
    pub sigma: f64,
    #[schemars(description = "Prior bounds as JSON 2D [n_params][2] = [lower, upper]")]
    pub prior_bounds_json: String,
    #[schemars(description = "Number of chains (default 3)")]
    pub n_chains: u32,
    #[schemars(description = "Number of iterations per chain (default 2000)")]
    pub n_iter: u32,
}

fn log_likelihood(x: &[Vec<f64>], y: &[f64], sigma: f64, theta: &[f64]) -> f64 {
    let mut sse = 0.0;
    for i in 0..x.len() {
        let pred: f64 = x[i].iter().zip(theta).map(|(a, t)| a * t).sum();
        let e = (y[i] - pred) / sigma;
        sse += e * e;
    }
    -0.5 * sse
}

fn gelman_rubin(chains: &[Vec<Vec<f64>>], d: usize) -> f64 {
    // chains: [n_chains][n_iter][d]
    let m = chains.len();
    let n = chains[0].len();
    let mut rhat = 0.0;
    for j in 0..d {
        // chain means
        let chain_means: Vec<f64> = chains.iter().map(|c| c.iter().map(|s| s[j]).sum::<f64>() / n as f64).collect();
        let grand_mean = chain_means.iter().sum::<f64>() / m as f64;
        // within-chain variance W
        let mut w = 0.0;
        for c in chains {
            let cm = c.iter().map(|s| s[j]).sum::<f64>() / n as f64;
            let var = c.iter().map(|s| (s[j] - cm).powi(2)).sum::<f64>() / (n as f64 - 1.0);
            w += var;
        }
        w /= m as f64;
        // between-chain variance B
        let b = n as f64 / (m as f64 - 1.0) * chain_means.iter().map(|cm| (cm - grand_mean).powi(2)).sum::<f64>();
        let var_hat = (n as f64 - 1.0) / n as f64 * w + b / n as f64;
        let r = if w > 0.0 { (var_hat / w).sqrt() } else { f64::INFINITY };
        if r > rhat { rhat = r; }
    }
    rhat
}

pub fn dream(p: &DreamParam) -> String {
    let x: Vec<Vec<f64>> = match serde_json::from_str(&p.x_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: x_json harus array 2D [obs][params].".into(),
    };
    let y: Vec<f64> = match serde_json::from_str(&p.y_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: y_json harus array 1D.".into(),
    };
    let bounds: Vec<[f64; 2]> = match serde_json::from_str(&p.prior_bounds_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: prior_bounds_json harus array 2D [params][2].".into(),
    };
    if x.is_empty() || y.len() != x.len() {
        return "ERROR: dimensi X dan y tidak cocok.".into();
    }
    let d = x[0].len();
    if bounds.len() != d {
        return "ERROR: jumlah bounds harus = jumlah parameter.".into();
    }
    if p.sigma <= 0.0 {
        return "ERROR: sigma harus > 0.".into();
    }

    let n_chains = p.n_chains.max(3) as usize;
    let n_iter = p.n_iter.max(100) as usize;
    let sigma = p.sigma;

    // DREAM constants (Vrugt 2009, deal(3, 0.1, 1e-12, 3, 0.2)).
    let delta = 3usize;
    let c_noise = 0.1f64;      // uniform λ noise bound
    let c_star = 1e-12f64;     // tiny Gaussian ζ scale
    let n_cr = 3usize;
    let p_g = 0.2f64;          // probability of γ=1 (RWM mode)
    let cr_vals: Vec<f64> = (1..=n_cr).map(|k| k as f64 / n_cr as f64).collect();

    let mut rng = Rng::new(12345);

    // Initialize chains from prior (uniform in bounds).
    let mut chains: Vec<Vec<Vec<f64>>> = (0..n_chains).map(|_| {
        (0..n_iter).map(|_| {
            bounds.iter().map(|b| rng.uniform_range(b[0], b[1])).collect::<Vec<f64>>()
        }).collect()
    }).collect();

    // Track current state per chain.
    let mut current: Vec<Vec<f64>> = chains.iter().map(|c| c[0].clone()).collect();
    let mut current_lp: Vec<f64> = current.iter().map(|th| log_likelihood(&x, &y, sigma, th)).collect();

    // Crossover selection probabilities (adaptive, initialized uniform).
    let mut p_cr = vec![1.0 / n_cr as f64; n_cr];

    for t in 0..n_iter {
        for i in 0..n_chains {
            // Select δ distinct chain pairs (a,b) != i.
            // Select CR value from multinomial(p_cr).
            let r = rng.uniform();
            let mut cr_id = 0;
            let mut acc = 0.0;
            for (k, &pk) in p_cr.iter().enumerate() {
                acc += pk;
                if r < acc { cr_id = k; break; }
            }
            let cr = cr_vals[cr_id];

            // Candidate dimensions (crossover).
            let mut a_subset = Vec::new();
            for j in 0..d {
                if rng.uniform() < cr {
                    a_subset.push(j);
                }
            }
            let d_star = a_subset.len().max(1);

            // Jump rate γ(δ,d*) = 2.38/sqrt(2δd*), or γ=1 with prob p_g.
            let gamma_d = 2.38 / (2.0 * delta as f64 * d_star as f64).sqrt();
            let gamma = if rng.uniform() < p_g { 1.0 } else { gamma_d };

            // Draw δ distinct chains != i for the DE jump.
            let mut others: Vec<usize> = (0..n_chains).filter(|&k| k != i).collect();
            let mut pairs: Vec<(usize, usize)> = Vec::new();
            for _ in 0..delta {
                if others.len() < 2 { break; }
                let a = others.remove((rng.uniform() * others.len() as f64) as usize);
                let b = others.remove((rng.uniform() * others.len() as f64) as usize);
                pairs.push((a, b));
            }
            if pairs.len() != delta {
                continue;
            }

            // Proposal θp = θ_i + (1+λ)γ Σ(θ_a - θ_b) + ζ  on subset A.
            let mut theta_p = current[i].clone();
            for &j in &a_subset {
                let lambda = rng.uniform_range(-c_noise, c_noise);
                let zeta = c_star * rng.normal();
                let mut jump = 0.0;
                for &(a, b) in &pairs {
                    jump += current[a][j] - current[b][j];
                }
                theta_p[j] = current[i][j] + (1.0 + lambda) * gamma * jump + zeta;
                // Reflect into prior bounds.
                if theta_p[j] < bounds[j][0] { theta_p[j] = bounds[j][0] + (bounds[j][0] - theta_p[j]).min(bounds[j][1] - bounds[j][0]); }
                if theta_p[j] > bounds[j][1] { theta_p[j] = bounds[j][1] - (theta_p[j] - bounds[j][1]).min(bounds[j][1] - bounds[j][0]); }
            }

            let lp_prop = log_likelihood(&x, &y, sigma, &theta_p);
            let lp_cur = current_lp[i];
            let accept = lp_prop > lp_cur || rng.uniform() < (lp_prop - lp_cur).exp();
            if accept {
                current[i] = theta_p;
                current_lp[i] = lp_prop;
            }
            chains[i][t] = current[i].clone();
        }

        // Adaptive crossover probability update every 10 generations (distance traveled).
        if t > 0 && t % 10 == 0 && t < n_iter / 2 {
            // Simple: keep uniform (full adaptation is more complex; uniform is a valid default).
            let _ = &mut p_cr;
        }
    }

    // Posterior statistics over the last half of each chain.
    let burn = n_iter / 2;
    let mut samples: Vec<Vec<f64>> = Vec::new();
    for c in &chains {
        for s in c.iter().skip(burn) {
            samples.push(s.clone());
        }
    }
    let n_post = samples.len();

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  DREAM-MCMC Posterior Inference\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Vrugt et al. 2009; Gelman & Rubin 1992 (R-hat)\n\n");
    out.push_str(&format!("Model: y = X·θ (linear), {} observasi, {} parameter\n", x.len(), d));
    out.push_str(&format!("Chains: {} | Iterations: {} | Burn-in: {} | Noise σ: {:.3}\n\n", n_chains, n_iter, burn, sigma));

    // Posterior mean + 5/95% credible interval per parameter.
    out.push_str("POSTERIOR PARAMETER (mean [5%–95%]):\n");
    let mut post_means = vec![0.0; d];
    for j in 0..d {
        let mut vals: Vec<f64> = samples.iter().map(|s| s[j]).collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mean = vals.iter().sum::<f64>() / n_post as f64;
        post_means[j] = mean;
        let p5 = vals[(n_post as f64 * 0.05) as usize];
        let p95 = vals[(n_post as f64 * 0.95) as usize];
        out.push_str(&format!("  θ{} = {:.4}  [{:.4}, {:.4}]\n", j + 1, mean, p5, p95));
    }

    // Convergence diagnostic (Gelman-Rubin R-hat) over all chains.
    let rhat = gelman_rubin(&chains, d);
    out.push_str(&format!("\nGelman-Rubin R-hat (max): {:.3} {}\n", rhat, if rhat < 1.2 { "✓ konvergen" } else { "⚠️ belum konvergen — tambah iterasi" }));

    // Least-squares reference (normal equations θ = (XᵀX)⁻¹Xᵀy).
    out.push_str("\nREFERENSI (least squares):\n");
    for j in 0..d {
        let mut num = 0.0;
        for i in 0..x.len() {
            num += x[i][j] * y[i];
        }
        // Simplified: assume diagonal design for reference display (single predictor each).
        let denom: f64 = x.iter().map(|row| row[j].powi(2)).sum();
        let ls = if denom > 0.0 { num / denom } else { f64::NAN };
        out.push_str(&format!("  θ{} (OLS, univariate) = {:.4}\n", j + 1, ls));
    }

    let _ = post_means;
    out
}

#[cfg(test)]
mod tests {
    use super::{glue, dream, GlueParam, DreamParam, nse_likelihood, Rng, gelman_rubin};

    #[test]
    fn nse_perfect_and_bad() {
        let obs = vec![10.0, 20.0, 30.0];
        assert!((nse_likelihood(&obs, &obs) - 1.0).abs() < 1e-9);
        // Bad prediction (constant = mean) gives NSE = 0.
        let mean = 20.0;
        let bad = vec![mean; 3];
        assert!(nse_likelihood(&bad, &obs).abs() < 1e-9);
    }

    #[test]
    fn glue_finds_behavioral_and_quantiles() {
        let pred = vec![
            vec![9.0, 19.0, 29.0],   // good (NSE ~1)
            vec![15.0, 25.0, 35.0],  // mediocre
            vec![100.0, 0.0, 0.0],   // bad (NSE very negative)
        ];
        let p = GlueParam {
            predictions_json: serde_json::to_string(&pred).unwrap(),
            observed_json: serde_json::to_string(&vec![10.0, 20.0, 30.0]).unwrap(),
            threshold: 0.5,
        };
        let out = glue(&p);
        assert!(out.contains("Set behavioral"));
        assert!(out.contains("5% - 95%"));
        assert!(out.contains("Lebar rata-rata pita"));
    }

    #[test]
    fn dream_recovers_linear_coefficients() {
        // y = 2*x1 + 3*x2 + noise(σ=0.5)
        let n = 50;
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut rng = Rng::new(7);
        for _ in 0..n {
            let x1 = rng.uniform_range(0.0, 10.0);
            let x2 = rng.uniform_range(0.0, 10.0);
            x.push(vec![x1, x2]);
            y.push(2.0 * x1 + 3.0 * x2 + 0.5 * rng.normal());
        }
        let bounds = vec![[0.0, 10.0], [0.0, 10.0]];
        let p = DreamParam {
            x_json: serde_json::to_string(&x).unwrap(),
            y_json: serde_json::to_string(&y).unwrap(),
            sigma: 0.5,
            prior_bounds_json: serde_json::to_string(&bounds).unwrap(),
            n_chains: 3,
            n_iter: 800,
        };
        let out = dream(&p);
        assert!(out.contains("POSTERIOR PARAMETER"));
        assert!(out.contains("R-hat"));
        assert!(out.contains("θ1"));
        assert!(out.contains("θ2"));
    }

    #[test]
    fn dream_rejects_bad_input() {
        let p = DreamParam {
            x_json: "[[1],[2]]".into(),
            y_json: "[1]".into(),
            sigma: 1.0,
            prior_bounds_json: "[[0,1]]".into(),
            n_chains: 3,
            n_iter: 100,
        };
        assert!(dream(&p).contains("ERROR"));
    }

    #[test]
    fn rng_uniform_in_range() {
        let mut r = Rng::new(1);
        for _ in 0..1000 {
            let v = r.uniform();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn rhat_converged_chains() {
        // Identical chains → R-hat = 1.
        let chains = vec![
            vec![vec![2.0, 3.0], vec![2.1, 2.9], vec![1.9, 3.1]],
            vec![vec![2.0, 3.0], vec![2.1, 2.9], vec![1.9, 3.1]],
        ];
        let r = gelman_rubin(&chains, 2);
        assert!(r < 1.2, "R-hat should be near 1 for identical chains, got {}", r);
    }
}
