use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult, Uncertainty};
use serde_json::json;

/// Uji Tren Mann-Kendall dengan Sen's Slope
/// Ref: Mann (1945), Kendall (1975), Sen (1968)

pub fn trend_test(data_json: &str) -> String {
    let data: Vec<f64> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return json!({"error": "E100", "message": format!("Gagal parsing JSON array: {}", e)}).to_string(),
    };

    let n = data.len();
    if n < 4 {
        return json!({"error": "E102", "message": format!("Minimal 4 data diperlukan untuk uji Mann-Kendall, diberikan {}.", n)}).to_string();
    }

    // S = Σ Σ sgn(xj - xi) for all i<j
    let mut s: i64 = 0;
    let mut slopes: Vec<f64> = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let diff = data[j] - data[i];
            if diff > 0.0 {
                s += 1;
            } else if diff < 0.0 {
                s -= 1;
            }
            // Sen's slope
            let time_diff = (j - i) as f64;
            if time_diff > 0.0 {
                slopes.push(diff / time_diff);
            }
        }
    }

    // Variance: σ² = n(n-1)(2n+5)/18 (without tie correction)
    // Tie correction: subtract Σ tp(tp-1)(2tp+5)/18 for each tie group of size tp
    let mut tie_groups: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    for val in &data {
        // Round to avoid floating point issues
        let key = (*val * 1e6) as i64;
        *tie_groups.entry(key).or_insert(0) += 1;
    }
    let mut tie_correction: f64 = 0.0;
    for (_, tp) in &tie_groups {
        if *tp > 1 {
            let t = *tp as f64;
            tie_correction += t * (t - 1.0) * (2.0 * t + 5.0) / 18.0;
        }
    }

    let nn = n as f64;
    let variance = nn * (nn - 1.0) * (2.0 * nn + 5.0) / 18.0 - tie_correction;
    let sigma = variance.sqrt();

    // Z-score
    let z = if s > 0 {
        (s as f64 - 1.0) / sigma
    } else if s < 0 {
        (s as f64 + 1.0) / sigma
    } else {
        0.0
    };

    // Two-tailed p-value (approximation using error function)
    let p_value = 2.0 * (1.0 - normal_cdf(z.abs()));

    // Sen's slope = median of all slopes
    slopes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let sens_slope = if slopes.is_empty() {
        0.0
    } else if slopes.len() % 2 == 0 {
        (slopes[slopes.len() / 2 - 1] + slopes[slopes.len() / 2]) / 2.0
    } else {
        slopes[slopes.len() / 2]
    };

    // Confidence interval for Sen's slope (approximate 95%)
    let c_alpha = 1.96 * sigma;
    let m1 = ((nn * (nn - 1.0) / 2.0 - c_alpha) / 2.0) as usize;
    let m2 = ((nn * (nn - 1.0) / 2.0 + c_alpha) / 2.0) as usize;
    let lower_ci = if m1 < slopes.len() {
        slopes[m1]
    } else {
        slopes[0]
    };
    let upper_ci = if m2 < slopes.len() {
        slopes[m2]
    } else {
        slopes[slopes.len() - 1]
    };

    // Trend determination
    let (trend, _trend_id) = if p_value <= 0.01 {
        if z > 0.0 {
            ("MENINGKAT_SANGAT_SIGNIFIKAN", "Tren naik signifikan pada α = 0.01")
        } else {
            ("MENURUN_SANGAT_SIGNIFIKAN", "Tren turun signifikan pada α = 0.01")
        }
    } else if p_value <= 0.05 {
        if z > 0.0 {
            ("MENINGKAT_SIGNIFIKAN", "Tren naik signifikan pada α = 0.05")
        } else {
            ("MENURUN_SIGNIFIKAN", "Tren turun signifikan pada α = 0.05")
        }
    } else if p_value <= 0.10 {
        if z > 0.0 {
            ("MENINGKAT_MARGINAL", "Tren naik marginal pada α = 0.10")
        } else {
            ("MENURUN_MARGINAL", "Tren turun marginal pada α = 0.10")
        }
    } else {
        ("TIDAK_ADA_TREN", "Tidak ada tren signifikan terdeteksi")
    };

    let is_significant = p_value <= 0.05;

    let res_z = ScientificResult::new("mann_kendall_z_score", z, "dimensionless")
        .with_status(ResultStatus::Valid)
        .with_provenance(Provenance::new("calculation", "Mann_1945", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("p_value", &p_value.to_string()))
        .with_claim(Claim::new("trend_classification", trend))
        .with_claim(Claim::new("is_significant_alpha_0_05", &is_significant.to_string()));

    let res_slope = ScientificResult::new("sens_slope", sens_slope, "units/time")
        .with_status(ResultStatus::Valid)
        .with_provenance(Provenance::new("calculation", "Sen_1968", "2026-08-19T00:00:00Z"))
        .with_uncertainty(Uncertainty::confidence_interval(lower_ci, upper_ci, 0.95));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_z.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_slope.emit_validated()).unwrap()
    ]).to_string()
}

/// Approximate normal CDF using Abramowitz & Stegun
fn normal_cdf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989422804014327; // 1/sqrt(2π)
    let p = d * (-x * x / 2.0).exp();
    let poly = t
        * (0.319381530
            + t * (-0.356563782 + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429))));
    if x >= 0.0 {
        1.0 - p * poly
    } else {
        p * poly
    }
}
