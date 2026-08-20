//! Split-sample calibration and validation.
//!
//! `calibrated` and `validated` are *earned* here from numbers, not declared by
//! a caller. The series is split contiguously (Klemeš 1986 split-sample test)
//! rather than randomly: a random split leaks information across autocorrelated
//! hydrological series and inflates apparent skill. Metrics and thresholds
//! follow Moriasi et al. (2007) for streamflow.

use crate::honesty::MaturityLevel;
use crate::validation::metrics::{kge, nse, pbias, r_squared, rmse};
use serde::{Deserialize, Serialize};

/// Below this many test points the goodness-of-fit metrics are not meaningful.
pub const MIN_TEST_POINTS: usize = 5;

/// Moriasi et al. (2007) minimum "satisfactory" bar for streamflow.
pub const SATISFACTORY_NSE: f64 = 0.50;
pub const SATISFACTORY_ABS_PBIAS: f64 = 25.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartitionMetrics {
    pub n: usize,
    pub nse: f64,
    pub pbias: f64,
    pub rmse: f64,
    pub kge: f64,
    pub r_squared: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationEvidence {
    pub train: PartitionMetrics,
    pub test: PartitionMetrics,
    pub train_fraction: f64,
    pub confidence_level: f64,
    /// Half-width of the prediction interval, from test-partition residuals.
    pub interval_half_width: f64,
    pub split_method: String,
    pub thresholds: String,
}

/// An owned (predicted, observed) partition.
pub type Paired = (Vec<f64>, Vec<f64>);

/// Split a paired series contiguously into (train, test).
pub fn split_paired(
    predicted: &[f64],
    observed: &[f64],
    train_fraction: f64,
) -> Result<(Paired, Paired), String> {
    if predicted.len() != observed.len() {
        return Err(format!(
            "predicted (n={}) and observed (n={}) must have equal length",
            predicted.len(),
            observed.len()
        ));
    }
    if !train_fraction.is_finite() || train_fraction <= 0.0 || train_fraction >= 1.0 {
        return Err(format!(
            "train_fraction must be strictly between 0 and 1 (got {})",
            train_fraction
        ));
    }
    if predicted.iter().chain(observed.iter()).any(|v| !v.is_finite()) {
        return Err("predicted and observed must contain only finite values".to_string());
    }
    let n = predicted.len();
    let train_n = (n as f64 * train_fraction).floor() as usize;
    let test_n = n - train_n;
    if train_n < 2 || test_n < 2 {
        return Err(format!(
            "split leaves too few points (train={}, test={}); supply a longer series",
            train_n, test_n
        ));
    }
    Ok((
        (predicted[..train_n].to_vec(), observed[..train_n].to_vec()),
        (predicted[train_n..].to_vec(), observed[train_n..].to_vec()),
    ))
}

/// Compute goodness-of-fit metrics for one partition.
///
/// Errors when the slices differ in length: the underlying `pbias` helper does
/// not length-check, so an unequal pair would silently yield a wrong bias.
pub fn evaluate_partition(predicted: &[f64], observed: &[f64]) -> Result<PartitionMetrics, String> {
    if predicted.len() != observed.len() {
        return Err(format!(
            "partition predicted (n={}) and observed (n={}) must have equal length",
            predicted.len(),
            observed.len()
        ));
    }
    if predicted.is_empty() {
        return Err("partition must not be empty".to_string());
    }
    Ok(PartitionMetrics {
        n: predicted.len(),
        nse: nse(predicted, observed),
        pbias: pbias(predicted, observed),
        rmse: rmse(predicted, observed),
        kge: kge(predicted, observed),
        r_squared: r_squared(predicted, observed),
    })
}

/// Run the full split-sample validation and return the evidence.
pub fn validate_split_sample(
    predicted: &[f64],
    observed: &[f64],
    train_fraction: f64,
    confidence_level: f64,
) -> Result<ValidationEvidence, String> {
    if !confidence_level.is_finite() || !(0.5..1.0).contains(&confidence_level) {
        return Err(format!(
            "confidence_level must be at least 0.5 and strictly below 1.0 (got {})",
            confidence_level
        ));
    }
    let (train, test) = split_paired(predicted, observed, train_fraction)?;
    let train_metrics = evaluate_partition(&train.0, &train.1)?;
    let test_metrics = evaluate_partition(&test.0, &test.1)?;
    // Normal-approximation prediction interval from test-partition residuals:
    // half-width = z(1 - alpha/2) * RMSE_test.
    let z = z_for(confidence_level);
    let interval_half_width = z * test_metrics.rmse;
    Ok(ValidationEvidence {
        train: train_metrics,
        test: test_metrics,
        train_fraction,
        confidence_level,
        interval_half_width,
        split_method: "contiguous split-sample (Klemes 1986)".to_string(),
        thresholds: format!(
            "Moriasi et al. 2007: NSE > {} and |PBIAS| < {}%",
            SATISFACTORY_NSE, SATISFACTORY_ABS_PBIAS
        ),
    })
}

/// Two-sided standard-normal quantile z(1 - alpha/2), interpolated between
/// tabulated anchors so intermediate confidence levels are not silently
/// rounded down to a much narrower interval.
fn z_for(confidence_level: f64) -> f64 {
    const TABLE: [(f64, f64); 6] = [
        (0.50, 0.674),
        (0.80, 1.282),
        (0.90, 1.645),
        (0.95, 1.960),
        (0.98, 2.326),
        (0.99, 2.576),
    ];
    if confidence_level <= TABLE[0].0 {
        return TABLE[0].1;
    }
    if confidence_level >= TABLE[TABLE.len() - 1].0 {
        return TABLE[TABLE.len() - 1].1;
    }
    for window in TABLE.windows(2) {
        let (lo_p, lo_z) = window[0];
        let (hi_p, hi_z) = window[1];
        if confidence_level <= hi_p {
            let t = (confidence_level - lo_p) / (hi_p - lo_p);
            return lo_z + t * (hi_z - lo_z);
        }
    }
    TABLE[TABLE.len() - 1].1
}

/// Map validation evidence onto the honesty ladder.
///
/// `Validated` requires the **test** partition to clear the Moriasi bar with
/// enough points. A model that only fits its training partition is `Calibrated`
/// (fitted but not independently confirmed), which is the overfitting case.
pub fn earned_level(evidence: &ValidationEvidence) -> MaturityLevel {
    let test_ok = evidence.test.n >= MIN_TEST_POINTS
        && evidence.test.nse > SATISFACTORY_NSE
        && evidence.test.pbias.abs() < SATISFACTORY_ABS_PBIAS;
    if test_ok {
        return MaturityLevel::Validated;
    }
    let train_ok = evidence.train.nse > SATISFACTORY_NSE
        && evidence.train.pbias.abs() < SATISFACTORY_ABS_PBIAS;
    if train_ok {
        return MaturityLevel::Calibrated;
    }
    MaturityLevel::Screening
}

/// Prediction interval around a point estimate, from test-partition residuals.
pub fn prediction_interval(value: f64, evidence: &ValidationEvidence) -> (f64, f64) {
    let half = evidence.interval_half_width.abs();
    (value - half, value + half)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear_series(n: usize, noise: f64) -> (Vec<f64>, Vec<f64>) {
        let observed: Vec<f64> = (0..n).map(|i| 10.0 + i as f64).collect();
        let predicted: Vec<f64> = observed
            .iter()
            .enumerate()
            .map(|(i, o)| o + if i % 2 == 0 { noise } else { -noise })
            .collect();
        (predicted, observed)
    }

    #[test]
    fn split_is_contiguous_and_respects_fraction() {
        let (p, o) = linear_series(20, 0.0);
        let (train, test) = split_paired(&p, &o, 0.7).unwrap();
        assert_eq!(train.0.len(), 14);
        assert_eq!(test.0.len(), 6);
        // contiguous: test must start where train ended
        assert_eq!(train.1[13], o[13]);
        assert_eq!(test.1[0], o[14]);
    }

    #[test]
    fn mismatched_or_tiny_series_are_rejected() {
        assert!(split_paired(&[1.0, 2.0], &[1.0], 0.7).is_err());
        assert!(split_paired(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0], 0.7).is_err());
        assert!(split_paired(&[f64::NAN; 20], &[1.0; 20], 0.7).is_err());
    }

    #[test]
    fn invalid_train_fraction_is_rejected() {
        let (p, o) = linear_series(20, 0.0);
        assert!(split_paired(&p, &o, 0.0).is_err());
        assert!(split_paired(&p, &o, 1.0).is_err());
        assert!(split_paired(&p, &o, f64::NAN).is_err());
    }

    #[test]
    fn near_perfect_model_earns_validated() {
        let (p, o) = linear_series(40, 0.05);
        let evidence = validate_split_sample(&p, &o, 0.7, 0.95).unwrap();
        assert!(evidence.test.nse > 0.5, "test nse = {}", evidence.test.nse);
        assert_eq!(earned_level(&evidence), MaturityLevel::Validated);
    }

    #[test]
    fn model_worse_than_the_mean_cannot_be_validated() {
        let observed: Vec<f64> = (0..40).map(|i| 10.0 + i as f64).collect();
        // Anti-correlated prediction: NSE must go strongly negative.
        let predicted: Vec<f64> = observed.iter().rev().copied().collect();
        let evidence = validate_split_sample(&predicted, &observed, 0.7, 0.95).unwrap();
        assert!(evidence.test.nse < 0.5);
        assert_ne!(earned_level(&evidence), MaturityLevel::Validated);
    }

    #[test]
    fn too_few_test_points_cannot_be_validated() {
        let (p, o) = linear_series(12, 0.01);
        // train_fraction 0.9 leaves 2 test points, below MIN_TEST_POINTS
        let evidence = validate_split_sample(&p, &o, 0.9, 0.95).unwrap();
        assert!(evidence.test.n < MIN_TEST_POINTS);
        assert_ne!(earned_level(&evidence), MaturityLevel::Validated);
    }

    #[test]
    fn prediction_interval_brackets_zero_residual_and_is_ordered() {
        let (p, o) = linear_series(40, 0.5);
        let evidence = validate_split_sample(&p, &o, 0.7, 0.95).unwrap();
        let (lower, upper) = prediction_interval(12.0, &evidence);
        assert!(lower < 12.0 && upper > 12.0, "interval {lower}..{upper}");
        assert!(upper - lower > 0.0);
    }

    #[test]
    fn evidence_is_serializable() {
        let (p, o) = linear_series(40, 0.1);
        let evidence = validate_split_sample(&p, &o, 0.7, 0.95).unwrap();
        let json = serde_json::to_string(&evidence).unwrap();
        assert!(json.contains("\"test\""));
        assert!(json.contains("\"train\""));
    }

    #[test]
    fn confidence_level_outside_the_usable_range_is_rejected() {
        let (p, o) = linear_series(40, 0.1);
        assert!(validate_split_sample(&p, &o, 0.7, 1.0).is_err());
        assert!(validate_split_sample(&p, &o, 0.7, 0.0).is_err());
        assert!(validate_split_sample(&p, &o, 0.7, f64::NAN).is_err());
    }

    #[test]
    fn interval_widens_monotonically_with_confidence() {
        let (p, o) = linear_series(40, 0.5);
        let at90 = validate_split_sample(&p, &o, 0.7, 0.90).unwrap();
        let at95 = validate_split_sample(&p, &o, 0.7, 0.95).unwrap();
        let at99 = validate_split_sample(&p, &o, 0.7, 0.99).unwrap();
        assert!(at90.interval_half_width < at95.interval_half_width);
        assert!(at95.interval_half_width < at99.interval_half_width);
        // An intermediate level must interpolate, not collapse to a lower anchor.
        let at94 = validate_split_sample(&p, &o, 0.7, 0.94).unwrap();
        assert!(at94.interval_half_width > at90.interval_half_width);
        assert!(at94.interval_half_width < at95.interval_half_width);
    }

    #[test]
    fn mismatched_partition_lengths_are_rejected() {
        assert!(evaluate_partition(&[1.0, 2.0, 3.0], &[1.0, 2.0]).is_err());
        assert!(evaluate_partition(&[], &[]).is_err());
    }
}
