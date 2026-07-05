/// Flood Frequency Analysis (Gumbel Distribution)
/// Ref: Chow (1951), USGS Bulletin 17C

pub fn gumbel(data: &[f64], return_period: f64) -> String {
    if data.len() < 10 { return "ERROR [E105]: Minimum 10 tahun data untuk analisis frekuensi.".into(); }
    if return_period <= 1.0 { return "ERROR [E102]: Parameter harus > 1.".into(); }

    let n = data.len() as f64;
    let mean: f64 = data.iter().sum::<f64>() / n;
    let variance: f64 = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    let k = -(6.0_f64.sqrt() / std::f64::consts::PI) * (0.5772 + (return_period / (return_period - 1.0)).ln().ln());
    let xt = mean + k * std_dev;

    let mut out = format!("=== Flood Frequency (Gumbel) ===\nData: {} tahun\nMean = {:.1}, Std = {:.1}\nT = {:.0} tahun\nK = {:.4}\nX(T) = {:.1}\n", data.len(), mean, std_dev, return_period, k, xt);
    out.push_str("\nTabel Return Period:\n");
    for t in [2.0_f64, 5.0, 10.0, 25.0, 50.0, 100.0] {
        let ki = -(6.0_f64.sqrt() / std::f64::consts::PI) * (0.5772 + (t / (t - 1.0)).ln().ln());
        out.push_str(&format!("  T={:.0} tahun → X = {:.1}\n", t, mean + ki * std_dev));
    }
    out
}

/// Log-Pearson Type III Flood Frequency Analysis
/// Ref: USGS Bulletin 17C, SNI 2415:2016
pub fn log_pearson_iii(data: &[f64], return_period: f64) -> String {
    if data.len() < 10 { return "ERROR [E105]: Minimum 10 tahun data.".into(); }
    if return_period <= 1.0 { return "ERROR [E102]: Parameter harus > 1.".into(); }
    if data.iter().any(|&x| x <= 0.0) { return "ERROR [E102]: Parameter harus > 0 untuk Log-Pearson III.".into(); }

    let n = data.len() as f64;
    let logs: Vec<f64> = data.iter().map(|q| q.log10()).collect();

    let y_bar = logs.iter().sum::<f64>() / n;
    let sy = (logs.iter().map(|y| (y - y_bar).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let cs = n * logs.iter().map(|y| (y - y_bar).powi(3)).sum::<f64>()
             / ((n - 1.0) * (n - 2.0) * sy.powi(3));

    // Standard normal quantile (Abramowitz & Stegun rational approximation)
    let p = 1.0 - 1.0 / return_period;
    let z = normal_quantile(p);

    // Wilson-Hilferty approximation for KT (Bulletin 17B)
    let kt = if cs.abs() < 1e-10 {
        z
    } else {
        let k = cs / 6.0;
        z + (z * z - 1.0) * k
          + (1.0 / 3.0) * (z.powi(3) - 6.0 * z) * k * k
          - (z * z - 1.0) * k.powi(3)
          + z * k.powi(4)
          + (1.0 / 3.0) * k.powi(5)
    };

    let log_qt = y_bar + kt * sy;
    let qt = 10.0_f64.powf(log_qt);

    let mut out = format!("=== Log-Pearson Type III ===\nRef: USGS Bulletin 17C, SNI 2415:2016\n\n");
    out.push_str(&format!("Data: {} tahun\n", data.len()));
    out.push_str(&format!("Log Mean (ȳ) = {:.4}\nLog Std (Sy) = {:.4}\nSkewness (Cs) = {:.4}\n", y_bar, sy, cs));
    out.push_str(&format!("T = {:.0} tahun\nz = {:.4}\nKT = {:.4}\n", return_period, z, kt));
    out.push_str(&format!("log Q(T) = {:.4}\nQ(T) = {:.1}\n", log_qt, qt));

    out.push_str("\nTabel Return Period:\n");
    for t in [2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 200.0, 500.0] {
        let p_t = 1.0 - 1.0 / t;
        let z_t = normal_quantile(p_t);
        let kt_t = if cs.abs() < 1e-10 { z_t } else {
            let k = cs / 6.0;
            z_t + (z_t * z_t - 1.0) * k + (1.0/3.0) * (z_t.powi(3) - 6.0*z_t) * k*k
            - (z_t*z_t - 1.0) * k.powi(3) + z_t * k.powi(4) + (1.0/3.0) * k.powi(5)
        };
        let log_q = y_bar + kt_t * sy;
        out.push_str(&format!("  T={:.0} tahun → Q = {:.1}\n", t, 10.0_f64.powf(log_q)));
    }
    out
}

/// Abramowitz & Stegun rational approximation for inverse normal CDF
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 { return 0.0; }
    if p == 0.5 { return 0.0; }

    let sign = if p < 0.5 { -1.0 } else { 1.0 };
    let p_adj = if p < 0.5 { p } else { 1.0 - p };

    let t = (-2.0 * p_adj.ln()).sqrt();

    // Coefficients from Abramowitz & Stegun 26.2.23
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;

    sign * (t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t))
}
