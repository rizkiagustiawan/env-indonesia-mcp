/// Flood Frequency Analysis (Gumbel & Log-Pearson III)
/// Ref: Chow (1951), USGS Bulletin 17C

pub fn gumbel(data: &[f64], return_period: f64) -> String {
    if data.len() < 10 { return "ERROR: Minimum 10 tahun data untuk analisis frekuensi.".into(); }
    if return_period <= 1.0 { return "ERROR: Return period harus > 1.".into(); }

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
