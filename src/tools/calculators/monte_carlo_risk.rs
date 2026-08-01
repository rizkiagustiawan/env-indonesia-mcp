//! Monte Carlo Risk Assessment — Probabilistic ARKL/HHRA
//! Randomizes exposure parameters (BW, fE, Dt, concentration) to generate
//! risk distributions (HQ, ILCR) instead of single deterministic values.
//! Ref: US EPA RAGS Vol 3 Part A (2001), Pedoman ARKL Kemenkes RI 2012

/// Simple LCG PRNG (no external crate needed)
struct Rng {
    state: u64,
}
impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Normal distribution via Box-Muller
    fn normal(&mut self, mean: f64, std: f64) -> f64 {
        let u1 = self.next_f64().max(1e-10);
        let u2 = self.next_f64();
        mean + std * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
    /// Log-normal: exp(Normal(ln(median), sigma))
    fn lognormal(&mut self, median: f64, gsd: f64) -> f64 {
        let mu = median.ln();
        let sigma = gsd.ln();
        (self.normal(mu, sigma)).exp()
    }
}

pub fn simulate(
    contaminant: &str,
    concentration_mean: f64, // mg/L or mg/kg
    concentration_cv: f64,   // coefficient of variation (0.3 = 30%)
    pathway: &str,           // "ingestion", "inhalation", "dermal"
    population: &str,        // "adult", "child"
    n_iterations: u32,       // number of Monte Carlo iterations (1000-10000)
    rfd_mgkgd: f64,          // Reference Dose (mg/kg/day)
    csf_mgkgd: f64,          // Cancer Slope Factor (mg/kg/day)⁻¹, 0 if non-carcinogen
) -> String {
    let n = n_iterations.max(100).min(100000) as usize;
    let mut rng = Rng::new(42);

    // Exposure parameter distributions (Indonesia-specific, Kemenkes 2012)
    // BW: lognormal, median=55 kg adult, GSD=1.2
    // fE: normal, mean=350 d/yr, std=15
    // Dt: normal, mean=30 yr, std=5
    // IR (ingestion rate): lognormal, median=2 L/d (water) or 0.03 kg/d (soil)
    let (bw_median, bw_gsd) = match population {
        "child" | "anak" => (18.0, 1.3),
        _ => (55.0, 1.2),
    };
    let (fe_mean, fe_std) = (350.0, 15.0);
    let (dt_mean, dt_std) = match population {
        "child" | "anak" => (6.0, 2.0),
        _ => (30.0, 5.0),
    };
    let (ir_median, ir_gsd) = match pathway {
        "ingestion" | "oral" => (2.0, 1.3),       // L/day water
        "inhalation" | "inhalasi" => (20.0, 1.2), // m³/day
        "dermal" | "kulit" => (0.005, 1.5),       // kg/event
        _ => (2.0, 1.3),
    };

    let at_days = 365.0 * 70.0; // averaging time (lifetime = 70 yr)

    let mut hq_values: Vec<f64> = Vec::with_capacity(n);
    let mut ilcr_values: Vec<f64> = Vec::with_capacity(n);
    let mut intake_values: Vec<f64> = Vec::with_capacity(n);

    for _ in 0..n {
        let c = rng.lognormal(concentration_mean, 1.0 + concentration_cv);
        let bw = rng.lognormal(bw_median, bw_gsd).max(10.0);
        let fe = rng.normal(fe_mean, fe_std).max(100.0).min(365.0);
        let dt = rng.normal(dt_mean, dt_std).max(1.0);
        let ir = rng.lognormal(ir_median, ir_gsd).max(0.001);

        // Intake = (C × IR × fE × Dt) / (BW × AT)
        let intake = c * ir * fe * dt / (bw * at_days);
        intake_values.push(intake);

        // HQ = Intake / RfD
        let hq = if rfd_mgkgd > 0.0 {
            intake / rfd_mgkgd
        } else {
            0.0
        };
        hq_values.push(hq);

        // ILCR = Intake × CSF
        let ilcr = intake * csf_mgkgd;
        ilcr_values.push(ilcr);
    }

    // Sort for percentiles
    hq_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ilcr_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    intake_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let percentile = |v: &[f64], p: f64| -> f64 {
        let idx = ((v.len() as f64 * p) as usize).min(v.len() - 1);
        v[idx]
    };
    let mean = |v: &[f64]| -> f64 { v.iter().sum::<f64>() / v.len() as f64 };

    let hq_exceed = hq_values.iter().filter(|&&x| x > 1.0).count();
    let ilcr_exceed = ilcr_values.iter().filter(|&&x| x > 1e-4).count();

    format!(
        "=== MONTE CARLO RISK ASSESSMENT ===\n\
         Ref: EPA RAGS Vol 3 (2001), Pedoman ARKL Kemenkes RI 2012\n\n\
         INPUT:\n  Kontaminan: {}\n  Konsentrasi: {:.4} mg/L (CV={:.0}%)\n  Jalur: {}\n  Populasi: {} (BW median={:.0}kg)\n  Iterasi: {}\n\n\
         DISTRIBUSI INTAKE (mg/kg/hari):\n  Mean = {:.2e}\n  P5 = {:.2e}\n  P25 = {:.2e}\n  P50 (median) = {:.2e}\n  P75 = {:.2e}\n  P95 = {:.2e}\n  P99 = {:.2e}\n\n\
         HAZARD QUOTIENT (Non-Karsinogen):\n  RfD = {:.2e} mg/kg/hari\n  HQ Mean = {:.4}\n  HQ P50 = {:.4}\n  HQ P95 = {:.4}\n  HQ P99 = {:.4}\n  Prob(HQ > 1) = {:.1}%\n  Risiko: {}\n\n\
         ILCR (Karsinogen):\n  CSF = {:.2e} (mg/kg/hari)⁻¹\n  ILCR Mean = {:.2e}\n  ILCR P50 = {:.2e}\n  ILCR P95 = {:.2e}\n  ILCR P99 = {:.2e}\n  Prob(ILCR > 10⁻⁴) = {:.1}%\n  Risiko: {}\n\n\
         PERBANDINGAN DETERMINISTIK vs PROBABILISTIK:\n  Deterministik HQ = {:.4} (single value)\n  Probabilistik P95 HQ = {:.4} (95th percentile)\n  Rasio P95/det = {:.2}x\n",
        contaminant, concentration_mean, concentration_cv * 100.0, pathway, population, bw_median, n,
        mean(&intake_values),
        percentile(&intake_values, 0.05), percentile(&intake_values, 0.25),
        percentile(&intake_values, 0.50), percentile(&intake_values, 0.75),
        percentile(&intake_values, 0.95), percentile(&intake_values, 0.99),
        rfd_mgkgd,
        mean(&hq_values), percentile(&hq_values, 0.50),
        percentile(&hq_values, 0.95), percentile(&hq_values, 0.99),
        100.0 * hq_exceed as f64 / n as f64,
        if percentile(&hq_values, 0.95) > 1.0 { "TIDAK AMAN (P95 > 1)" } else { "AMAN (P95 ≤ 1)" },
        csf_mgkgd,
        mean(&ilcr_values), percentile(&ilcr_values, 0.50),
        percentile(&ilcr_values, 0.95), percentile(&ilcr_values, 0.99),
        100.0 * ilcr_exceed as f64 / n as f64,
        if percentile(&ilcr_values, 0.95) > 1e-4 { "RISIKO TINGGI (P95 > 10⁻⁴)" }
        else if percentile(&ilcr_values, 0.95) > 1e-6 { "RISIKO SEDANG (10⁻⁶ < P95 < 10⁻⁴)" }
        else { "RISIKO RENDAH (P95 < 10⁻⁶)" },
        // Deterministic comparison using median values
        concentration_mean * ir_median * fe_mean * dt_mean / (bw_median * at_days) / rfd_mgkgd,
        percentile(&hq_values, 0.95),
        percentile(&hq_values, 0.95) / (concentration_mean * ir_median * fe_mean * dt_mean / (bw_median * at_days) / rfd_mgkgd).max(1e-10),
    )
}
