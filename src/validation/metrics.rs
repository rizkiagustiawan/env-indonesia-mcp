/// Statistical validation metrics for model-vs-observation comparison.
/// All formulas verified against standard references (Moriasi et al. 2007; Nash-Sutcliffe 1970).

/// Compute RMSE (Root Mean Square Error).
/// RMSE = sqrt( (1/n) * Σ(pred_i - obs_i)² )
pub fn rmse(predicted: &[f64], observed: &[f64]) -> f64 {
    if predicted.len() != observed.len() || predicted.is_empty() {
        return f64::NAN;
    }
    let n = predicted.len() as f64;
    let sum_sq = predicted.iter()
        .zip(observed.iter())
        .map(|(p, o)| (p - o).powi(2))
        .sum::<f64>();
    (sum_sq / n).sqrt()
}

/// Compute MAE (Mean Absolute Error).
/// MAE = (1/n) * Σ|pred_i - obs_i|
pub fn mae(predicted: &[f64], observed: &[f64]) -> f64 {
    if predicted.len() != observed.len() || predicted.is_empty() {
        return f64::NAN;
    }
    let n = predicted.len() as f64;
    predicted.iter()
        .zip(observed.iter())
        .map(|(p, o)| (p - o).abs())
        .sum::<f64>()
        / n
}

/// Compute MBE (Mean Bias Error) — positive = overprediction, negative = underprediction.
/// MBE = (1/n) * Σ(pred_i - obs_i)
pub fn mbe(predicted: &[f64], observed: &[f64]) -> f64 {
    if predicted.len() != observed.len() || predicted.is_empty() {
        return f64::NAN;
    }
    let n = predicted.len() as f64;
    predicted.iter()
        .zip(observed.iter())
        .map(|(p, o)| p - o)
        .sum::<f64>()
        / n
}

/// Compute R² (Coefficient of Determination) — explained variance.
/// R² = 1 - SS_res / SS_tot
/// where SS_res = Σ(obs - pred)², SS_tot = Σ(obs - mean_obs)²
/// Range: 0-1 (1 = perfect). Can be negative if model is worse than mean.
pub fn r_squared(predicted: &[f64], observed: &[f64]) -> f64 {
    if predicted.len() != observed.len() || predicted.len() < 2 {
        return f64::NAN;
    }
    let mean_obs = observed.iter().sum::<f64>() / observed.len() as f64;
    let ss_res: f64 = predicted.iter()
        .zip(observed.iter())
        .map(|(p, o)| (o - p).powi(2))
        .sum();
    let ss_tot: f64 = observed.iter()
        .map(|o| (o - mean_obs).powi(2))
        .sum();
    if ss_tot < 1e-15 {
        return f64::NAN; // no variance in observations
    }
    1.0 - ss_res / ss_tot
}

/// Compute NSE (Nash-Sutcliffe Efficiency) — hydrological model performance.
/// NSE = 1 - Σ(obs - pred)² / Σ(obs - mean_obs)²
/// Same formula as R² but conventionally used for time series.
/// NSE > 0.6 = satisfactory, > 0.8 = good, > 0.9 = excellent (Moriasi 2007)
pub fn nse(predicted: &[f64], observed: &[f64]) -> f64 {
    r_squared(predicted, observed) // identical formula
}

/// Compute KGE (Kling-Gupta Efficiency) — improved NSE with correlation + bias + variability.
/// KGE = 1 - sqrt( (r-1)² + (β-1)² + (γ-1)² )
/// where r = Pearson correlation, β = mean_pred/mean_obs, γ = (std_pred/mean_pred)/(std_obs/mean_obs)
/// KGE > 0 = better than mean benchmark (Gupta et al. 2009)
pub fn kge(predicted: &[f64], observed: &[f64]) -> f64 {
    if predicted.len() != observed.len() || predicted.len() < 3 {
        return f64::NAN;
    }
    let n = predicted.len() as f64;
    let mean_p = predicted.iter().sum::<f64>() / n;
    let mean_o = observed.iter().sum::<f64>() / n;
    let std_p = (predicted.iter().map(|p| (p - mean_p).powi(2)).sum::<f64>() / n).sqrt();
    let std_o = (observed.iter().map(|o| (o - mean_o).powi(2)).sum::<f64>() / n).sqrt();

    // Pearson correlation
    let cov: f64 = predicted.iter()
        .zip(observed.iter())
        .map(|(p, o)| (p - mean_p) * (o - mean_o))
        .sum::<f64>()
        / n;
    let r = if std_p > 1e-15 && std_o > 1e-15 {
        cov / (std_p * std_o)
    } else {
        0.0
    };

    let beta = if mean_o.abs() > 1e-15 { mean_p / mean_o } else { 1.0 };
    let gamma = if mean_o.abs() > 1e-15 && mean_p.abs() > 1e-15 {
        (std_p / mean_p) / (std_o / mean_o)
    } else {
        1.0
    };

    1.0 - ((r - 1.0).powi(2) + (beta - 1.0).powi(2) + (gamma - 1.0).powi(2)).sqrt()
}

/// Percent bias (PBIAS) — measures average tendency to over/underpredict.
/// PBIAS = 100 * Σ(pred - obs) / Σ(obs)
/// Positive = overprediction, negative = underprediction.
/// |PBIAS| < 10% = good, < 15% = satisfactory (Moriasi 2007)
pub fn pbias(predicted: &[f64], observed: &[f64]) -> f64 {
    if predicted.is_empty() || observed.is_empty() {
        return f64::NAN;
    }
    let sum_diff: f64 = predicted.iter()
        .zip(observed.iter())
        .map(|(p, o)| p - o)
        .sum();
    let sum_obs: f64 = observed.iter().sum();
    if sum_obs.abs() < 1e-15 {
        return f64::NAN;
    }
    100.0 * sum_diff / sum_obs
}

/// Validation badge — classifies model performance based on NSE + PBIAS.
pub fn validation_badge(nse_val: f64, pbias_val: f64) -> &'static str {
    if nse_val.is_nan() || pbias_val.is_nan() {
        return "UNVALIDATED (insufficient data)";
    }
    if nse_val > 0.9 && pbias_val.abs() < 5.0 {
        "EXCELLENT (NSE>0.9, |PBIAS|<5%)"
    } else if nse_val > 0.8 && pbias_val.abs() < 10.0 {
        "VERY GOOD (NSE>0.8, |PBIAS|<10%)"
    } else if nse_val > 0.6 && pbias_val.abs() < 15.0 {
        "SATISFACTORY (NSE>0.6, |PBIAS|<15%)"
    } else if nse_val > 0.0 {
        "POOR (NSE>0 but <0.6)"
    } else {
        "UNSATISFACTORY (NSE<0 — worse than mean)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_prediction() {
        let pred = [1.0, 2.0, 3.0, 4.0, 5.0];
        let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((rmse(&pred, &obs) - 0.0).abs() < 1e-10);
        assert!((mae(&pred, &obs) - 0.0).abs() < 1e-10);
        assert!((r_squared(&pred, &obs) - 1.0).abs() < 1e-10);
        assert!((nse(&pred, &obs) - 1.0).abs() < 1e-10);
        assert!((kge(&pred, &obs) - 1.0).abs() < 1e-6);
        assert!((pbias(&pred, &obs) - 0.0).abs() < 1e-10);
        assert_eq!(validation_badge(1.0, 0.0), "EXCELLENT (NSE>0.9, |PBIAS|<5%)");
    }

    #[test]
    fn known_rmse() {
        let pred = [3.0, 3.0, 3.0];
        let obs = [1.0, 2.0, 3.0];
        // RMSE = sqrt((4+1+0)/3) = sqrt(5/3) = 1.291
        assert!((rmse(&pred, &obs) - (5.0_f64 / 3.0).sqrt()).abs() < 1e-4);
    }

    #[test]
    fn constant_prediction_r2() {
        // If model predicts constant = mean, R² = 0
        let pred = [3.0, 3.0, 3.0];
        let obs = [1.0, 3.0, 5.0];
        let mean = 3.0;
        let r2 = r_squared(&pred, &obs);
        // SS_res = (1-3)²+(3-3)²+(5-3)² = 8; SS_tot = (1-3)²+(3-3)²+(5-3)² = 8; R² = 0
        assert!((r2 - 0.0).abs() < 1e-10, "R²={r2} expected 0");
    }

    #[test]
    fn bias_detection() {
        let pred = [12.0, 13.0, 14.0];
        let obs = [10.0, 11.0, 12.0];
        // MBE = (2+2+2)/3 = 2 (consistent overprediction)
        assert!((mbe(&pred, &obs) - 2.0).abs() < 1e-10);
        // PBIAS = 100 * 6 / 33 = 18.18%
        assert!((pbias(&pred, &obs) - 18.18).abs() < 0.1);
    }

    #[test]
    fn kge_better_than_r2_for_bias() {
        // Model with high correlation but 2x bias (pred = 2*obs)
        let pred = [2.0, 4.0, 6.0, 8.0];
        let obs = [1.0, 3.0, 5.0, 7.0];
        let r2 = r_squared(&pred, &obs);
        let k = kge(&pred, &obs);
        // R² = 0.8 (high but penalized by mean difference)
        assert!(r2 > 0.7, "R²={r2} should be high (>0.7) for correlated data");
        // KGE must be lower than R² when there's 2x bias (KGE penalizes bias more)
        assert!(k < r2, "KGE={k} should be < R²={r2} when bias exists");
        // KGE should be positive but reduced (not negative — correlation is still high)
        assert!(k > 0.0 && k < 1.0, "KGE={k} should be between 0 and 1 for high-corr + bias");
    }

    #[test]
    fn badge_classification() {
        assert_eq!(validation_badge(0.95, 3.0), "EXCELLENT (NSE>0.9, |PBIAS|<5%)");
        assert_eq!(validation_badge(0.85, 8.0), "VERY GOOD (NSE>0.8, |PBIAS|<10%)");
        assert_eq!(validation_badge(0.65, 12.0), "SATISFACTORY (NSE>0.6, |PBIAS|<15%)");
        assert_eq!(validation_badge(0.3, 5.0), "POOR (NSE>0 but <0.6)");
        assert_eq!(validation_badge(-0.5, 20.0), "UNSATISFACTORY (NSE<0 — worse than mean)");
    }
}
