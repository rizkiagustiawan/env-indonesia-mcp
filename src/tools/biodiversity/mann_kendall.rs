/// Uji Tren Mann-Kendall dengan Sen's Slope
/// Ref: Mann (1945), Kendall (1975), Sen (1968)

pub fn trend_test(data_json: &str) -> String {
    let data: Vec<f64> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parsing JSON array: {}", e),
    };

    let n = data.len();
    if n < 4 {
        return format!(
            "ERROR: Minimal 4 data diperlukan untuk uji Mann-Kendall, diberikan {}.",
            n
        );
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
    // Simple: count ties
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
    let (trend, trend_id) = if p_value <= 0.01 {
        if z > 0.0 {
            (
                "MENINGKAT (sangat signifikan)",
                "Tren naik signifikan pada α = 0.01",
            )
        } else {
            (
                "MENURUN (sangat signifikan)",
                "Tren turun signifikan pada α = 0.01",
            )
        }
    } else if p_value <= 0.05 {
        if z > 0.0 {
            (
                "MENINGKAT (signifikan)",
                "Tren naik signifikan pada α = 0.05",
            )
        } else {
            (
                "MENURUN (signifikan)",
                "Tren turun signifikan pada α = 0.05",
            )
        }
    } else if p_value <= 0.10 {
        if z > 0.0 {
            ("MENINGKAT (marginal)", "Tren naik marginal pada α = 0.10")
        } else {
            ("MENURUN (marginal)", "Tren turun marginal pada α = 0.10")
        }
    } else {
        ("TIDAK ADA TREN", "Tidak ada tren signifikan terdeteksi")
    };

    // Data summary
    let data_min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let data_max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let data_mean = data.iter().sum::<f64>() / nn;

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("UJI TREN MANN-KENDALL + SEN'S SLOPE\n");
    result.push_str("Ref: Mann (1945), Kendall (1975), Sen (1968)\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("DATA:\n");
    result.push_str(&format!("• Jumlah data (n)      : {}\n", n));
    result.push_str(&format!("• Minimum              : {:.4}\n", data_min));
    result.push_str(&format!("• Maksimum             : {:.4}\n", data_max));
    result.push_str(&format!("• Rata-rata            : {:.4}\n\n", data_mean));

    result.push_str("HASIL UJI MANN-KENDALL:\n");
    result.push_str(&format!("• Statistik S          : {}\n", s));
    result.push_str(&format!("• Varians (σ²)         : {:.2}\n", variance));
    result.push_str(&format!("• Z-score              : {:.4}\n", z));
    result.push_str(&format!("• p-value (two-tailed) : {:.6}\n\n", p_value));

    result.push_str("SEN'S SLOPE:\n");
    result.push_str(&format!(
        "• Slope estimasi       : {:.6} per satuan waktu\n",
        sens_slope
    ));
    result.push_str(&format!(
        "• 95% CI               : [{:.6}, {:.6}]\n\n",
        lower_ci, upper_ci
    ));

    result.push_str(&format!("TREN: {}\n", trend));
    result.push_str(&format!("  {}\n\n", trend_id));

    result.push_str("SIGNIFIKANSI:\n");
    result.push_str(&format!(
        "• α = 0.01 : {} (p {} 0.01)\n",
        if p_value <= 0.01 {
            "Signifikan"
        } else {
            "Tidak signifikan"
        },
        if p_value <= 0.01 { "≤" } else { ">" }
    ));
    result.push_str(&format!(
        "• α = 0.05 : {} (p {} 0.05)\n",
        if p_value <= 0.05 {
            "Signifikan"
        } else {
            "Tidak signifikan"
        },
        if p_value <= 0.05 { "≤" } else { ">" }
    ));
    result.push_str(&format!(
        "• α = 0.10 : {} (p {} 0.10)\n",
        if p_value <= 0.10 {
            "Signifikan"
        } else {
            "Tidak signifikan"
        },
        if p_value <= 0.10 { "≤" } else { ">" }
    ));
    result.push_str("══════════════════════════════════════════════\n");

    result
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
