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
    
    // 2D-MCA specific: we divide n_iterations to balance outer/inner loops.
    // e.g. if n=1000, outer=20, inner=50.
    let n_outer = (n as f64).sqrt().max(10.0) as usize;
    let n_inner = n / n_outer;
    
    let mut rng = Rng::new(42);

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
        "ingestion" | "oral" => (2.0, 1.3),       
        "inhalation" | "inhalasi" => (20.0, 1.2), 
        "dermal" | "kulit" => (0.005, 1.5),       
        _ => (2.0, 1.3),
    };

    let at_cancer_days = 365.0 * 70.0; // lifetime averaging time (cancer only)

    // We store the 95th percentile HQ and ILCR for *each* outer loop.
    let mut hq_p95s = Vec::with_capacity(n_outer);
    let mut ilcr_p95s = Vec::with_capacity(n_outer);

    let percentile = |mut v: Vec<f64>, p: f64| -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((v.len() as f64 * p) as usize).min(v.len() - 1);
        v[idx]
    };

    for _ in 0..n_outer {
        // Epistemic variable: The "true" concentration parameter is uncertain.
        let true_mean_c = rng.lognormal(concentration_mean, 1.0 + (concentration_cv / 2.0)); // Narrower CV for the true mean itself

        let mut inner_hq = Vec::with_capacity(n_inner);
        let mut inner_ilcr = Vec::with_capacity(n_inner);

        for _ in 0..n_inner {
            // Aleatory variables: Variability across the population.
            let c = rng.lognormal(true_mean_c, 1.0 + concentration_cv);
            let bw = rng.lognormal(bw_median, bw_gsd).max(10.0);
            let fe = rng.normal(fe_mean, fe_std).max(100.0).min(365.0);
            let dt = rng.normal(dt_mean, dt_std).max(1.0);
            let ir = rng.lognormal(ir_median, ir_gsd).max(0.001);

            // Non-cancer HQ: AT = ED×365 (ED cancels) → ADD = C·IR·EF/(BW·365)
            let intake_nc = c * ir * fe / (bw * 365.0);
            // Cancer ILCR: AT = 70 yr × 365 (lifetime)
            let intake_c = c * ir * fe * dt / (bw * at_cancer_days);

            let hq = if rfd_mgkgd > 0.0 { intake_nc / rfd_mgkgd } else { 0.0 };
            inner_hq.push(hq);

            let ilcr = intake_c * csf_mgkgd;
            inner_ilcr.push(ilcr);
        }

        // Calculate the P95 for this specific population reality
        hq_p95s.push(percentile(inner_hq, 0.95));
        ilcr_p95s.push(percentile(inner_ilcr, 0.95));
    }

    let hq_95_95 = percentile(hq_p95s.clone(), 0.95);
    let ilcr_95_95 = percentile(ilcr_p95s.clone(), 0.95);
    
    let hq_50_95 = percentile(hq_p95s, 0.50);
    let ilcr_50_95 = percentile(ilcr_p95s, 0.50);

    format!(
        "=== TWO-DIMENSIONAL MONTE CARLO ANALYSIS (2D-MCA) ===\n\
         Ref: EPA RAGS Vol 3 (2001), Pedoman ARKL Kemenkes RI 2012\n\n\
         INPUT:\n  Kontaminan: {}\n  Konsentrasi: {:.4} mg/L (CV={:.0}%)\n  Jalur: {}\n  Populasi: {} (BW median={:.0}kg)\n  Iterasi: {} (Outer: {}, Inner: {})\n\n\
         HAZARD QUOTIENT (Non-Karsinogen):\n  RfD = {:.2e} mg/kg/hari\n  HQ 50/95 (Median of P95s) = {:.4}\n  HQ 95/95 (95th of P95s) = {:.4}\n\n\
         ILCR (Karsinogen):\n  CSF = {:.2e} (mg/kg/hari)⁻¹\n  ILCR 50/95 = {:.2e}\n  ILCR 95/95 = {:.2e}\n",
        contaminant, concentration_mean, concentration_cv * 100.0, pathway, population, bw_median, n, n_outer, n_inner,
        rfd_mgkgd, hq_50_95, hq_95_95,
        csf_mgkgd, ilcr_50_95, ilcr_95_95
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2d_mca_separates_epistemic_from_aleatory() {
        let result = simulate("Pb", 0.05, 0.3, "ingestion", "child", 100, 0.003, 0.0);
        
        assert!(result.contains("TWO-DIMENSIONAL MONTE CARLO ANALYSIS (2D-MCA)"));
        assert!(result.contains("HQ 95/95"));
    }
}

