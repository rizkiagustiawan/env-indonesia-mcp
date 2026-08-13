//! Environmental Statistics Module
//! Descriptive stats, correlation, exceedance, bootstrap CI, trend significance
//! Ref: Helsel & Hirsch (2002) Statistical Methods in Water Resources

/// S1: Descriptive Statistics — mean, median, std, skewness, kurtosis, percentiles
pub fn descriptive(data_json: &str) -> String {
    let data: Vec<f64> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR [E103]: JSON array tidak valid — {}", e),
    };

    let n = data.len();
    if n < 2 {
        return "ERROR [E102]: Minimal 2 data point.".to_string();
    }
    if let Some((index, value)) = data
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        return format!(
            "ERROR [E104]: Data point pada index {} harus finite, got {}.",
            index, value
        );
    }

    let mut sorted = data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mean = data.iter().sum::<f64>() / n as f64;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let std = variance.sqrt();
    let median = if n.is_multiple_of(2) {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };

    // Skewness (Fisher's)
    let (skewness, kurtosis) = if std > f64::EPSILON {
        // Degenerate constant series have no defined standardized moments.
        let m3 = data.iter().map(|x| ((x - mean) / std).powi(3)).sum::<f64>() / n as f64;
        // Adjusted Fisher-Pearson: G1 = n²·m3 / ((n-1)(n-2))  (scipy bias=False convention)
        let skewness = m3 * (n as f64).powi(2) / ((n - 1) as f64 * (n - 2).max(1) as f64);

        // Kurtosis (excess, Fisher's)
        let m4 = data.iter().map(|x| ((x - mean) / std).powi(4)).sum::<f64>() / n as f64;
        (skewness, m4 - 3.0)
    } else {
        (0.0, 0.0)
    };

    // Percentiles
    let pctl = |p: f64| -> f64 {
        let idx = (p * (n - 1) as f64) as usize;
        let frac = p * (n - 1) as f64 - idx as f64;
        if idx + 1 < n {
            sorted[idx] * (1.0 - frac) + sorted[idx + 1] * frac
        } else {
            sorted[idx]
        }
    };

    // Coefficient of variation
    let cv = if mean.abs() > 1e-10 {
        std / mean.abs() * 100.0
    } else {
        0.0
    };

    // Standard error
    let se = std / (n as f64).sqrt();
    let ci95_lower = mean - 1.96 * se;
    let ci95_upper = mean + 1.96 * se;

    // IQR and outlier detection
    let q1 = pctl(0.25);
    let q3 = pctl(0.75);
    let iqr = q3 - q1;
    let outlier_lower = q1 - 1.5 * iqr;
    let outlier_upper = q3 + 1.5 * iqr;
    let n_outliers = data
        .iter()
        .filter(|&&x| x < outlier_lower || x > outlier_upper)
        .count();

    // Normality hint (Jarque-Bera approximation)
    let jb = n as f64 / 6.0 * (skewness.powi(2) + kurtosis.powi(2) / 4.0);
    let likely_normal = jb < 5.99; // chi2(2) at 0.05

    format!(
        "=== STATISTIK DESKRIPTIF ===\nn = {}\nMean = {:.6}\nMedian = {:.6}\nStd Dev = {:.6}\nVariance = {:.6}\nCV = {:.1}%\nSE = {:.6}\n95% CI = [{:.6}, {:.6}]\n\nPercentiles:\n  P5  = {:.6}\n  P10 = {:.6}\n  P25 (Q1) = {:.6}\n  P50 (Med) = {:.6}\n  P75 (Q3) = {:.6}\n  P90 = {:.6}\n  P95 = {:.6}\n  P99 = {:.6}\n\nMin = {:.6}\nMax = {:.6}\nRange = {:.6}\nIQR = {:.6}\n\nSkewness = {:.4} ({})\nKurtosis = {:.4} ({})\nJarque-Bera = {:.2} ({})\n\nOutliers: {} dari {} (threshold: <{:.4} atau >{:.4})",
        n, mean, median, std, variance, cv, se, ci95_lower, ci95_upper,
        pctl(0.05), pctl(0.10), q1, median, q3, pctl(0.90), pctl(0.95), pctl(0.99),
        sorted[0], sorted[n-1], sorted[n-1] - sorted[0], iqr,
        skewness, if skewness.abs() < 0.5 { "simetris" } else if skewness > 0.0 { "right-skewed" } else { "left-skewed" },
        kurtosis, if kurtosis.abs() < 1.0 { "mesokurtic" } else if kurtosis > 0.0 { "leptokurtic" } else { "platykurtic" },
        jb, if likely_normal { "kemungkinan normal" } else { "kemungkinan tidak normal" },
        n_outliers, n, outlier_lower, outlier_upper
    )
}

/// S2: Correlation Matrix — Pearson + Spearman for multi-parameter data
pub fn correlation(data_json: &str, names_json: &str) -> String {
    // data_json: [[p1_vals], [p2_vals], ...] — each inner array is one parameter
    // names_json: ["BOD", "COD", "TSS", ...]
    let data: Vec<Vec<f64>> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: JSON tidak valid — {}", e),
    };
    let names: Vec<String> = serde_json::from_str(names_json).unwrap_or_default();

    let p = data.len(); // number of parameters
    if p < 2 {
        return "ERROR: Minimal 2 parameter.".to_string();
    }
    let n = data[0].len();
    if n < 3 {
        return "ERROR: Minimal 3 data point per parameter.".to_string();
    }
    if data.iter().any(|series| series.len() != n) {
        return "ERROR [E102]: Semua parameter harus memiliki panjang series yang sama."
            .to_string();
    }
    if data.iter().flatten().any(|value| !value.is_finite()) {
        return "ERROR [E104]: Semua data korelasi harus finite.".to_string();
    }

    // Pearson correlation
    let pearson = |x: &[f64], y: &[f64]| -> f64 {
        let n = x.len() as f64;
        let mx = x.iter().sum::<f64>() / n;
        let my = y.iter().sum::<f64>() / n;
        let cov = x
            .iter()
            .zip(y)
            .map(|(a, b)| (a - mx) * (b - my))
            .sum::<f64>();
        let sx = x.iter().map(|a| (a - mx).powi(2)).sum::<f64>().sqrt();
        let sy = y.iter().map(|b| (b - my).powi(2)).sum::<f64>().sqrt();
        if sx * sy > 0.0 {
            cov / (sx * sy)
        } else {
            0.0
        }
    };

    // Spearman rank correlation
    let spearman = |x: &[f64], y: &[f64]| -> f64 {
        let rank = |v: &[f64]| -> Vec<f64> {
            let mut indexed: Vec<(usize, f64)> =
                v.iter().enumerate().map(|(i, &val)| (i, val)).collect();
            indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let mut ranks = vec![0.0; v.len()];
            for (rank, &(idx, _)) in indexed.iter().enumerate() {
                ranks[idx] = (rank + 1) as f64;
            }
            ranks
        };
        let rx = rank(x);
        let ry = rank(y);
        pearson(&rx, &ry)
    };

    let mut result = format!("=== MATRIKS KORELASI ===\nRef: Helsel & Hirsch (2002)\nn = {}\n\n--- Pearson (linear) ---\n", n);

    // Header
    result.push_str(&format!("{:>12}", ""));
    for j in 0..p {
        let name = names
            .get(j)
            .cloned()
            .unwrap_or_else(|| format!("P{}", j + 1));
        let display = &name[..name.len().min(9)];
        result.push_str(&format!("{:>10}", display));
    }
    result.push('\n');

    for i in 0..p {
        let name_i = names
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("P{}", i + 1));
        let display_i = &name_i[..name_i.len().min(11)];
        result.push_str(&format!("{:>12}", display_i));
        for j in 0..p {
            let r = pearson(&data[i], &data[j]);
            result.push_str(&format!("{:>10.4}", r));
        }
        result.push('\n');
    }

    result.push_str("\n--- Spearman (rank/non-parametric) ---\n");
    result.push_str(&format!("{:>12}", ""));
    for j in 0..p {
        let name = names
            .get(j)
            .cloned()
            .unwrap_or_else(|| format!("P{}", j + 1));
        let display = &name[..name.len().min(9)];
        result.push_str(&format!("{:>10}", display));
    }
    result.push('\n');

    for i in 0..p {
        let name_i = names
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("P{}", i + 1));
        let display_i = &name_i[..name_i.len().min(11)];
        result.push_str(&format!("{:>12}", display_i));
        for j in 0..p {
            let r = spearman(&data[i], &data[j]);
            result.push_str(&format!("{:>10.4}", r));
        }
        result.push('\n');
    }

    // Significant correlations summary
    result.push_str("\nKorelasi signifikan (|r| > 0.7):\n");
    for i in 0..p {
        for j in (i + 1)..p {
            let r = pearson(&data[i], &data[j]);
            if r.abs() > 0.7 {
                let ni = names.get(i).cloned().unwrap_or_else(|| "?".to_string());
                let nj = names.get(j).cloned().unwrap_or_else(|| "?".to_string());
                let strength = if r.abs() > 0.9 { "SANGAT KUAT" } else { "KUAT" };
                let direction = if r > 0.0 { "positif" } else { "negatif" };
                result.push_str(&format!(
                    "  {} vs {}: r={:.4} ({} {})\n",
                    ni, nj, r, strength, direction
                ));
            }
        }
    }

    result
}

/// S3: Exceedance Probability — % time parameter exceeds baku mutu
pub fn exceedance(
    data_json: &str,
    threshold: f64,
    parameter_name: &str,
    baku_mutu_ref: &str,
) -> String {
    let data: Vec<f64> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: JSON tidak valid — {}", e),
    };

    let n = data.len();
    if n < 5 {
        return "ERROR: Minimal 5 data point.".to_string();
    }

    let n_exceed = data.iter().filter(|&&x| x > threshold).count();
    let pct_exceed = 100.0 * n_exceed as f64 / n as f64;

    let mean = data.iter().sum::<f64>() / n as f64;
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);

    // Duration of exceedance (consecutive)
    let mut max_consecutive = 0usize;
    let mut current = 0usize;
    for &v in &data {
        if v > threshold {
            current += 1;
            if current > max_consecutive {
                max_consecutive = current;
            }
        } else {
            current = 0;
        }
    }

    // Return period estimate (1 / probability of non-exceedance)
    let return_period = if pct_exceed > 0.0 {
        100.0 / pct_exceed
    } else {
        f64::INFINITY
    };

    format!(
        "=== ANALISIS EXCEEDANCE ===\nParameter: {}\nBaku Mutu: {:.4} ({}) \nn = {}\n\nHasil:\n  Mean = {:.4}\n  Min = {:.4}\n  Max = {:.4}\n  Jumlah melebihi = {} dari {} ({:.1}%)\n  Konsekutif maks = {} kali berturut-turut\n  Return period ≈ {:.1} pengukuran\n\nKesimpulan: {}\nRef: PP 22/2021, KepmenLH 51/2004",
        parameter_name, threshold, baku_mutu_ref, n,
        mean, min, max, n_exceed, n, pct_exceed,
        max_consecutive, return_period,
        if pct_exceed == 0.0 { "SELALU MEMENUHI baku mutu" }
        else if pct_exceed < 10.0 { "UMUMNYA MEMENUHI (exceedance <10%)" }
        else if pct_exceed < 50.0 { "SERING MELEBIHI (10-50%)" }
        else { "DOMINAN MELEBIHI (>50%) — tindakan diperlukan" }
    )
}

/// S4: Bootstrap Confidence Interval
pub fn bootstrap_ci(data_json: &str, statistic: &str, confidence: f64, n_bootstrap: u32) -> String {
    let data: Vec<f64> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: JSON tidak valid — {}", e),
    };

    let n = data.len();
    if n < 5 {
        return "ERROR: Minimal 5 data point.".to_string();
    }
    let nb = n_bootstrap.clamp(100, 50000) as usize;

    // Statistic function
    let calc_stat = |sample: &[f64]| -> f64 {
        match statistic {
            "mean" => sample.iter().sum::<f64>() / sample.len() as f64,
            "median" => {
                let mut s = sample.to_vec();
                s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                if s.len().is_multiple_of(2) {
                    (s[s.len() / 2 - 1] + s[s.len() / 2]) / 2.0
                } else {
                    s[s.len() / 2]
                }
            }
            "std" => {
                let m = sample.iter().sum::<f64>() / sample.len() as f64;
                (sample.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (sample.len() - 1) as f64)
                    .sqrt()
            }
            _ => sample.iter().sum::<f64>() / sample.len() as f64,
        }
    };

    let observed = calc_stat(&data);

    // Bootstrap resampling (LCG PRNG)
    let mut state: u64 = 12345;
    let mut boot_stats: Vec<f64> = Vec::with_capacity(nb);

    for _ in 0..nb {
        let mut sample = Vec::with_capacity(n);
        for _ in 0..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let idx = (state >> 33) as usize % n;
            sample.push(data[idx]);
        }
        boot_stats.push(calc_stat(&sample));
    }

    boot_stats.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let alpha = 1.0 - confidence;
    let lo_idx = ((alpha / 2.0) * nb as f64) as usize;
    let hi_idx = ((1.0 - alpha / 2.0) * nb as f64) as usize;
    let ci_lower = boot_stats[lo_idx.min(nb - 1)];
    let ci_upper = boot_stats[hi_idx.min(nb - 1)];

    let boot_mean = boot_stats.iter().sum::<f64>() / nb as f64;
    let boot_se = (boot_stats
        .iter()
        .map(|x| (x - boot_mean).powi(2))
        .sum::<f64>()
        / (nb - 1) as f64)
        .sqrt();
    let bias = boot_mean - observed;

    format!(
        "=== BOOTSTRAP CONFIDENCE INTERVAL ===\nStatistik: {}\nn data = {}\nIterasi bootstrap = {}\nConfidence = {:.0}%\n\nObserved {} = {:.6}\nBootstrap mean = {:.6}\nBootstrap SE = {:.6}\nBias = {:.6}\n\n{:.0}% CI = [{:.6}, {:.6}]\n\nInterpretasi: {} berada antara {:.6} dan {:.6} dengan kepercayaan {:.0}%.",
        statistic, n, nb, confidence * 100.0,
        statistic, observed, boot_mean, boot_se, bias,
        confidence * 100.0, ci_lower, ci_upper,
        statistic, ci_lower, ci_upper, confidence * 100.0
    )
}

/// S5: Trend Significance Test — Mann-Kendall + Sen's slope (for time series data)
pub fn trend_test(data_json: &str, time_labels_json: &str) -> String {
    let data: Vec<f64> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: JSON tidak valid — {}", e),
    };
    let _labels: Vec<String> = serde_json::from_str(time_labels_json).unwrap_or_default();

    let n = data.len();
    if n < 4 {
        return "ERROR: Minimal 4 data point untuk trend test.".to_string();
    }

    // Mann-Kendall S statistic
    let mut s: i64 = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            if data[j] > data[i] {
                s += 1;
            } else if data[j] < data[i] {
                s -= 1;
            }
        }
    }

    // Variance of S (no ties correction for simplicity)
    let var_s = (n * (n - 1) * (2 * n + 5)) as f64 / 18.0;

    // Z-score
    let z = if s > 0 {
        (s as f64 - 1.0) / var_s.sqrt()
    } else if s < 0 {
        (s as f64 + 1.0) / var_s.sqrt()
    } else {
        0.0
    };

    // p-value (two-tailed, normal approximation)
    // Using error function approximation
    let p_value = 2.0 * (1.0 - erf_approx(z.abs() / std::f64::consts::SQRT_2));

    // Kendall's tau
    let tau = 2.0 * s as f64 / (n * (n - 1)) as f64;

    // Sen's slope
    let mut slopes: Vec<f64> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            slopes.push((data[j] - data[i]) / (j - i) as f64);
        }
    }
    slopes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let sen_slope = if slopes.len().is_multiple_of(2) && !slopes.is_empty() {
        (slopes[slopes.len() / 2 - 1] + slopes[slopes.len() / 2]) / 2.0
    } else if !slopes.is_empty() {
        slopes[slopes.len() / 2]
    } else {
        0.0
    };

    // Sen's intercept
    let sen_intercept = {
        let mut medians: Vec<f64> = (0..n).map(|i| data[i] - sen_slope * i as f64).collect();
        medians.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if medians.len().is_multiple_of(2) {
            (medians[medians.len() / 2 - 1] + medians[medians.len() / 2]) / 2.0
        } else {
            medians[medians.len() / 2]
        }
    };

    let sig_level = if p_value < 0.001 {
        "*** (p<0.001)"
    } else if p_value < 0.01 {
        "** (p<0.01)"
    } else if p_value < 0.05 {
        "* (p<0.05)"
    } else {
        "ns (p≥0.05)"
    };

    let trend_dir = if z > 0.0 {
        "NAIK ↑"
    } else if z < 0.0 {
        "TURUN ↓"
    } else {
        "TIDAK ADA TREND"
    };

    format!(
        "=== MANN-KENDALL TREND TEST ===\nRef: Mann (1945), Sen (1968), Kendall (1975)\nn = {}\n\nMann-Kendall:\n  S = {}\n  Var(S) = {:.1}\n  Z = {:.4}\n  p-value = {:.6}\n  Signifikansi: {}\n  Kendall's tau = {:.4}\n\nSen's Slope:\n  Slope = {:.6} per time step\n  Intercept = {:.6}\n  Trend: {}\n\nKesimpulan: {}",
        n, s, var_s, z, p_value, sig_level, tau,
        sen_slope, sen_intercept, trend_dir,
        if p_value < 0.05 {
            format!("Trend {} SIGNIFIKAN secara statistik (p={:.4})", trend_dir, p_value)
        } else {
            "Tidak ada trend signifikan (p≥0.05). Data fluktuatif/stasioner.".to_string()
        }
    )
}

/// Error function approximation (Abramowitz & Stegun 7.1.26)
fn erf_approx(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

#[cfg(test)]
mod tests {
    use super::descriptive;

    #[test]
    fn descriptive_rejects_malformed_values() {
        let result = descriptive("[1.0, null]");

        assert!(result.starts_with("ERROR [E103]:"));
    }

    #[test]
    fn descriptive_handles_constant_data_without_nan_or_infinity() {
        let result = descriptive("[5.0, 5.0, 5.0]");

        assert!(!result.contains("NaN"));
        assert!(!result.contains("inf"));
        assert!(result.contains("Std Dev = 0.000000"));
    }

    #[test]
    fn correlation_rejects_series_with_different_lengths() {
        let result = super::correlation("[[1.0, 2.0, 3.0], [3.0, 4.0]]", "[\"a\", \"b\"]");

        assert!(result.starts_with("ERROR [E102]:"));
        assert!(result.contains("panjang series"));
    }

    #[test]
    fn skewness_fisher_pearson_adjusted() {
        // [1,2,100]: adjusted Fisher-Pearson skewness ≈ 1.73 (buggy n(n-1)/(n-2) factor gave ≈ 2.31)
        let result = descriptive("[1.0, 2.0, 100.0]");
        assert!(result.contains("Skewness = 1.73"), "skewness wrong:\n{result}");
        assert!(!result.contains("Skewness = 2.30"), "buggy n(n-1)/(n-2) factor present:\n{result}");
    }
}
