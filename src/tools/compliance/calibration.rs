/// Calibration & Verification (ISO 17025 / EPA)
/// Ref: ISO 17025 Sec 6.4-6.6; EPA Methods; NIST SP 1029
pub fn assess(instrument: &str, std_concs_json: &str, measured_concs_json: &str, calibration_range_low: f64, calibration_range_high: f64) -> String {
    let mut out = String::from("=== Calibration & Verification (ISO 17025) ===\n");
    out.push_str("Ref: ISO 17025 Sec 6.4-6.6; EPA Methods\n\n");

    let std: Vec<f64> = match serde_json::from_str(std_concs_json) {
        Ok(v) => v, Err(_) => return "ERROR: std_concs must be JSON array".into(),
    };
    let meas: Vec<f64> = match serde_json::from_str(measured_concs_json) {
        Ok(v) => v, Err(_) => return "ERROR: measured_concs must be JSON array".into(),
    };
    if std.len() != meas.len() || std.len() < 3 {
        return "ERROR: Need >= 3 paired points, same length arrays.".into();
    }

    let n = std.len() as f64;
    let sum_x: f64 = std.iter().sum();
    let sum_y: f64 = meas.iter().sum();
    let sum_xy: f64 = std.iter().zip(meas.iter()).map(|(x,y)| x*y).sum();
    let sum_x2: f64 = std.iter().map(|x| x*x).sum();

    // Linear regression: y = m*x + b
    let m = (n*sum_xy - sum_x*sum_y) / (n*sum_x2 - sum_x*sum_x).max(1e-15);
    let b = (sum_y - m*sum_x) / n;

    // R^2
    let mean_y = sum_y / n;
    let ss_tot: f64 = meas.iter().map(|y| (y - mean_y).powi(2)).sum();
    let ss_res: f64 = std.iter().zip(meas.iter()).map(|(x,y)| (y - (m*x + b)).powi(2)).sum();
    let r2 = 1.0 - ss_res / ss_tot.max(1e-15);

    // RSD of response factors (EPA requires <15% for most methods)
    let rfs: Vec<f64> = std.iter().zip(meas.iter()).map(|(x,y)| y / x.max(1e-9)).collect();
    let rf_mean: f64 = rfs.iter().sum::<f64>() / rfs.len() as f64;
    let rf_sd: f64 = ((rfs.iter().map(|r| (r-rf_mean).powi(2)).sum::<f64>()) / (rfs.len()-1) as f64).sqrt();
    let rsd = (rf_sd / rf_mean.abs().max(1e-9)) * 100.0;

    // ICV (Initial Calibration Verification) - 80-120%
    // CCV (Continuing Calibration Verification) - mid-level, 85-115%
    let midpoint = (calibration_range_low + calibration_range_high) / 2.0;
    let icv_pred = m * midpoint + b;
    let icv_recovery = icv_pred / midpoint * 100.0;

    out.push_str(&format!("Instrument: {}\n", instrument));
    out.push_str(&format!("Calibration points: {} (range: {:.2}-{:.2})\n\n", std.len(), calibration_range_low, calibration_range_high));

    out.push_str("-- Linear Regression --\n\n");
    out.push_str(&format!("  y = {:.4}x + {:.4}\n", m, b));
    out.push_str(&format!("  R2 = {:.6}\n", r2));
    out.push_str(&format!("  Response Factor RSD: {:.2}% (EPA limit: <15%)\n\n", rsd));

    out.push_str("-- Verification --\n\n");
    out.push_str(&format!("  ICV at mid-point ({:.2}): predicted={:.2}, recovery={:.1}%\n", midpoint, icv_pred, icv_recovery));
    out.push_str("  ICV criteria: 80-120%\n\n");

    // Pass/fail
    let mut pass = true;
    if r2 < 0.995 { out.push_str("  [WARN] R2 < 0.995 — recalibrate\n"); pass = false; }
    else { out.push_str("  [OK] R2 >= 0.995\n"); }

    if rsd > 15.0 { out.push_str("  [WARN] RSD > 15% — recalibrate or use alternative calibration\n"); pass = false; }
    else { out.push_str("  [OK] RSD < 15%\n"); }

    if icv_recovery < 80.0 || icv_recovery > 120.0 { out.push_str("  [WARN] ICV outside 80-120%\n"); pass = false; }
    else { out.push_str("  [OK] ICV within 80-120%\n"); }

    if pass {
        out.push_str("\n  >> CALIBRATION ACCEPTABLE\n");
    } else {
        out.push_str("\n  >> CALIBRATION FAILED — recalibrate before analysis\n");
    }

    out.push_str("\n  Ref: ISO 17025 Sec 6.4-6.6; EPA Method-specific criteria\n");
    out
}
