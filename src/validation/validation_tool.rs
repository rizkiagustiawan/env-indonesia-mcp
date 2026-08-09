/// MCP Tool: validate_model — Closed-loop validation of model predictions vs observations.
/// This is the tool that moves the system from "calculator" to "calibrated modeling system."
///
/// Usage: provide predicted[] and observed[] arrays (same length, n>=2).
/// Output: RMSE, MAE, MBE, R², NSE, KGE, PBIAS, validation badge.
///
/// Ref: Moriasi et al. 2007 (NSE thresholds); Gupta et al. 2009 (KGE);
///      Nash-Sutcliffe 1970; Legates & McCabe 1999.

use super::metrics::{rmse, mae, mbe, r_squared, nse, kge, pbias, validation_badge};

pub fn validate_model(
    model_name: &str,
    predicted: &[f64],
    observed: &[f64],
    units: &str,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════════\n");
    out.push_str("  MODEL VALIDATION REPORT (Closed-Loop)\n");
    out.push_str("═══════════════════════════════════════════════════\n\n");
    out.push_str(&format!("Model: {}\n", model_name));
    out.push_str(&format!("Units: {}\n", units));
    out.push_str(&format!("Data points (n): {}\n\n", predicted.len()));

    if predicted.len() != observed.len() {
        return format!(
            "ERROR: predicted (n={}) and observed (n={}) arrays must have equal length.",
            predicted.len(), observed.len()
        );
    }
    if predicted.len() < 2 {
        return "ERROR: need at least 2 data points for validation.".into();
    }

    // Compute all metrics
    let rmse_val = rmse(predicted, observed);
    let mae_val = mae(predicted, observed);
    let mbe_val = mbe(predicted, observed);
    let r2_val = r_squared(predicted, observed);
    let nse_val = nse(predicted, observed);
    let kge_val = kge(predicted, observed);
    let pbias_val = pbias(predicted, observed);
    let badge = validation_badge(nse_val, pbias_val);

    // Error metrics table
    out.push_str("─── ERROR METRICS ───\n\n");
    out.push_str(&format!("  RMSE  (Root Mean Square Error):  {:>10.4} {}\n", rmse_val, units));
    out.push_str(&format!("  MAE   (Mean Absolute Error):     {:>10.4} {}\n", mae_val, units));
    out.push_str(&format!("  MBE   (Mean Bias Error):         {:>10.4} {} ({})\n",
        mbe_val, units, if mbe_val > 0.0 {"OVERPREDICT"} else if mbe_val < 0.0 {"UNDERPREDICT"} else {"UNBIASED"}));
    out.push_str("\n");

    // Goodness-of-fit metrics
    out.push_str("─── GOODNESS-OF-FIT ───\n\n");
    out.push_str(&format!("  R²    (Coefficient of Determination): {:>8.4}\n", r2_val));
    out.push_str(&format!("  NSE   (Nash-Sutcliffe Efficiency):    {:>8.4}\n", nse_val));
    out.push_str(&format!("  KGE   (Kling-Gupta Efficiency):       {:>8.4}\n", kge_val));
    out.push_str(&format!("  PBIAS (Percent Bias):                {:>8.2}%\n", pbias_val));
    out.push_str("\n");

    // Performance interpretation
    out.push_str("─── PERFORMANCE INTERPRETATION ───\n\n");
    out.push_str(&format!("  >> Validation Badge: {}\n\n", badge));

    // Moriasi 2007 thresholds
    out.push_str("  Reference thresholds (Moriasi et al. 2007):\n");
    out.push_str("    NSE > 0.9: Excellent  | > 0.8: Very Good | > 0.6: Satisfactory\n");
    out.push_str("    |PBIAS| < 5%: Excellent | < 10%: Very Good | < 15%: Satisfactory\n");
    out.push_str("    KGE > 0: better than mean benchmark | > 0.6: good\n\n");

    // Detailed comparison table
    out.push_str("─── DATA COMPARISON ───\n\n");
    out.push_str(&format!("{:>6} {:>12} {:>12} {:>12} {:>10}\n",
        "#", "Predicted", "Observed", "Residual", "% Error"));
    out.push_str(&"-".repeat(56));
    out.push('\n');

    for (i, (p, o)) in predicted.iter().zip(observed.iter()).enumerate() {
        let residual = p - o;
        let pct_err = if o.abs() > 1e-10 { (residual / o) * 100.0 } else { f64::NAN };
        out.push_str(&format!("{:>6} {:>12.4} {:>12.4} {:>12.4} {:>9.1}%\n",
            i + 1, p, o, residual, pct_err));
    }
    out.push('\n');

    // Recommendations
    out.push_str("─── RECOMMENDATIONS ───\n\n");
    if nse_val < 0.0 {
        out.push_str("  [CRITICAL] Model performs WORSE than the observed mean.\n");
        out.push_str("  Action: recalibrate parameters, check formula correctness, or revise model structure.\n\n");
    } else if nse_val < 0.6 {
        out.push_str("  [WARNING] Model performance is POOR (NSE < 0.6).\n");
        out.push_str("  Action: calibrate against more data, adjust coefficients, consider model upgrades.\n\n");
    } else if pbias_val.abs() > 15.0 {
        out.push_str(&format!("  [WARNING] Significant bias detected (PBIAS={:.1}%).\n", pbias_val));
        if pbias_val > 0.0 {
            out.push_str("  Model OVERPREDICTS — consider bias correction factor.\n\n");
        } else {
            out.push_str("  Model UNDERPREDICTS — consider bias correction factor.\n\n");
        }
    } else {
        out.push_str("  [OK] Model performance is acceptable for screening-level analysis.\n");
        out.push_str("  For regulatory/policy use: validate with independent dataset.\n\n");
    }

    // Metadata
    out.push_str("─── METADATA ───\n\n");
    out.push_str("  Ref: Moriasi et al. 2007 (NSE thresholds);\n");
    out.push_str("       Gupta et al. 2009 (KGE);\n");
    out.push_str("       Nash-Sutcliffe 1970; Legates & McCabe 1999\n");
    out.push_str("  Limitations:\n");
    out.push_str("    - Validation quality depends on observation data quality\n");
    out.push_str("    - Metrics assume paired data (same time/space)\n");
    out.push_str("    - No temporal/spatial autocorrelation check\n");
    out.push_str("    - For probabilistic validation: use PIT histograms, reliability diagrams\n");
    out.push_str("═══════════════════════════════════════════════════\n");

    out
}
