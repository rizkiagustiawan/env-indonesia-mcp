/// Land Subsidence Calculator (Terzaghi 1D Consolidation)
/// Ref: Terzaghi (1943), relevant untuk Jakarta/Semarang/Pekalongan

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Time-dependent 1D consolidation parameter set (Terzaghi + Biot α).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ConsolidationParam {
    #[schemars(description = "Clay layer thickness H (m)")]
    pub clay_thickness_m: f64,
    #[schemars(description = "Effective stress increment Δσ' (kPa)")]
    pub delta_stress_kpa: f64,
    #[schemars(description = "Compression index Cc (dimensionless)")]
    pub cc: f64,
    #[schemars(description = "Initial void ratio e0 (dimensionless)")]
    pub e0: f64,
    #[schemars(description = "Initial effective overburden σ'0 (kPa)")]
    pub sigma0_kpa: f64,
    #[schemars(description = "Coefficient of consolidation cv (m²/yr)")]
    pub cv_m2_yr: f64,
    #[schemars(description = "Elapsed time t (yr)")]
    pub t_yr: f64,
    #[schemars(description = "true = double drainage (H_dr = H/2), false = single drainage (H_dr = H)")]
    pub double_drainage: bool,
    #[schemars(description = "Biot coefficient α (0..1], fraction of stress carried by solid skeleton)")]
    pub biot_alpha: f64,
}

/// Degree of consolidation U(Tv) via the Terzaghi series:
/// U = 1 - Σ_{m=0}^∞ (2/M²) exp(-M² Tv), M = (π/2)(2m+1).
fn degree_of_consolidation(tv: f64) -> f64 {
    if tv <= 0.0 {
        return 0.0;
    }
    if tv > 3.0 {
        // Asymptotic: first term dominates.
        return 1.0 - (8.0 / std::f64::consts::PI.powi(2)) * (-std::f64::consts::PI.powi(2) * tv / 4.0).exp();
    }
    let mut sum = 0.0;
    for m in 0..100 {
        let mm = std::f64::consts::FRAC_PI_2 * (2.0 * m as f64 + 1.0);
        sum += (2.0 / (mm * mm)) * (-mm * mm * tv).exp();
    }
    (1.0 - sum).clamp(0.0, 1.0)
}

/// Time-dependent consolidation settlement with Biot poroelastic coefficient.
pub fn calculate_consolidation(p: &ConsolidationParam) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Land Subsidence (Terzaghi 1D + Biot α)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Terzaghi 1943; Biot 1941 (poroelasticity)\n\n");

    if p.clay_thickness_m <= 0.0 || p.delta_stress_kpa <= 0.0 || p.e0 <= 0.0 || p.sigma0_kpa <= 0.0 {
        return "ERROR: H, Δσ, e0, σ'0 harus > 0.".into();
    }
    if p.cv_m2_yr <= 0.0 || p.t_yr < 0.0 {
        return "ERROR: cv harus > 0, t >= 0.".into();
    }
    if p.biot_alpha <= 0.0 || p.biot_alpha > 1.0 {
        return "ERROR: biot_alpha harus di (0, 1] (soft clay ~1.0).".into();
    }

    // Drainage path
    let h_dr = if p.double_drainage { p.clay_thickness_m / 2.0 } else { p.clay_thickness_m };

    // Ultimate primary settlement (Terzaghi), Biot-modulated effective stress.
    let effective_stress_ratio = (p.sigma0_kpa + p.biot_alpha * p.delta_stress_kpa) / p.sigma0_kpa;
    let sc_m = (p.cc * p.clay_thickness_m / (1.0 + p.e0)) * effective_stress_ratio.log10();

    // Time factor and degree of consolidation
    let tv = p.cv_m2_yr * p.t_yr / (h_dr * h_dr);
    let u_avg = degree_of_consolidation(tv);
    let s_t_m = u_avg * sc_m;

    out.push_str(&format!(
        "Input:\n  H = {:.1} m   Δσ' = {:.1} kPa   Cc = {:.3}   e0 = {:.2}\n  σ'0 = {:.1} kPa   cv = {:.2} m²/yr   t = {:.1} yr\n  Drainase = {} (H_dr = {:.2} m)   Biot α = {:.2}\n\n",
        p.clay_thickness_m, p.delta_stress_kpa, p.cc, p.e0,
        p.sigma0_kpa, p.cv_m2_yr, p.t_yr,
        if p.double_drainage { "double" } else { "single" }, h_dr, p.biot_alpha
    ));
    out.push_str(&format!(
        "Hasil:\n  Penurunan ultimit Sc = {:.4} m = {:.1} cm\n  Time factor Tv = {:.4}\n  Derajat konsolidasi U(t) = {:.1}%\n  Penurunan saat t = {:.4} m = {:.1} cm\n",
        sc_m, sc_m * 100.0, tv, u_avg * 100.0, s_t_m, s_t_m * 100.0
    ));

    // Time to reach key consolidation thresholds
    let t50 = 0.197 * h_dr * h_dr / p.cv_m2_yr;
    let t90 = 0.848 * h_dr * h_dr / p.cv_m2_yr;
    out.push_str(&format!(
        "  Waktu U=50%: {:.1} thn | U=90%: {:.1} thn\n",
        t50, t90
    ));

    let residual_cm = (sc_m - s_t_m) * 100.0;
    out.push_str(&format!("  Sisa penurunan: {:.1} cm\n\n", residual_cm));

    if s_t_m * 100.0 > 10.0 {
        out.push_str("⚠️ Penurunan > 10 cm: risiko kerusakan infrastruktur (retak, miring).\n");
    }
    if s_t_m * 100.0 > 50.0 {
        out.push_str("⛔ Penurunan > 50 cm: zona KRITIS (Jakarta/Semarang-level). Rob & banjir permanen.\n");
    }

    out
}

pub fn calculate(
    clay_thickness_m: f64,
    delta_stress_kpa: f64,
    cc: f64,
    e0: f64,
    sigma0_kpa: f64,
) -> String {
    if clay_thickness_m <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if delta_stress_kpa <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if e0 <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if sigma0_kpa <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    // Terzaghi 1D primary consolidation
    let settlement_m = (cc * clay_thickness_m / (1.0 + e0))
        * ((sigma0_kpa + delta_stress_kpa) / sigma0_kpa).log10();
    let settlement_cm = settlement_m * 100.0;

    let mut out = format!("=== Land Subsidence (Terzaghi 1D) ===\nRef: Terzaghi (1943)\n\nH (tebal lempung) = {:.1} m\nΔσ (tambahan tegangan) = {:.1} kPa\nCc (compression index) = {:.3}\ne0 (angka pori) = {:.2}\nσ'0 (overburden) = {:.1} kPa\n\n", clay_thickness_m, delta_stress_kpa, cc, e0, sigma0_kpa);
    out.push_str(&format!(
        "Sc = Cc×H/(1+e0) × log[(σ'0+Δσ)/σ'0]\nSc = {:.4} m = {:.1} cm\n\n",
        settlement_m, settlement_cm
    ));
    if settlement_cm > 10.0 {
        out.push_str("⚠️ Penurunan > 10 cm: Risiko kerusakan infrastruktur.\n");
    }
    if settlement_cm > 50.0 {
        out.push_str("⛔ Penurunan > 50 cm: Zona KRITIS (Jakarta-level subsidence).\n");
    }
    out
}

#[cfg(test)]
mod cons_tests {
    use super::{calculate_consolidation, ConsolidationParam, degree_of_consolidation};

    fn param(t_yr: f64) -> ConsolidationParam {
        ConsolidationParam {
            clay_thickness_m: 10.0,
            delta_stress_kpa: 50.0,
            cc: 0.3,
            e0: 1.0,
            sigma0_kpa: 100.0,
            cv_m2_yr: 2.0,
            t_yr,
            double_drainage: true,
            biot_alpha: 1.0,
        }
    }

    #[test]
    fn u_zero_at_t_zero() {
        assert_eq!(degree_of_consolidation(0.0), 0.0);
    }

    #[test]
    fn u_approaches_one_for_large_tv() {
        assert!(degree_of_consolidation(10.0) > 0.999);
    }

    #[test]
    fn u_50_percent_at_tv_0197() {
        // Classic Terzaghi: Tv=0.197 → U≈50%
        let u = degree_of_consolidation(0.197);
        assert!((u - 0.50).abs() < 0.02, "U(Tv=0.197)={} should be ~0.50", u);
    }

    #[test]
    fn u_90_percent_at_tv_0848() {
        // Classic Terzaghi: Tv=0.848 → U≈90%
        let u = degree_of_consolidation(0.848);
        assert!((u - 0.90).abs() < 0.01, "U(Tv=0.848)={} should be ~0.90", u);
    }

    #[test]
    fn consolidation_outputs_time_factor() {
        let out = calculate_consolidation(&param(10.0));
        assert!(out.contains("Time factor Tv"));
        assert!(out.contains("Derajat konsolidasi"));
    }

    #[test]
    fn rejects_invalid() {
        assert!(calculate_consolidation(&param(-1.0)).contains("ERROR"));
        let mut p = param(1.0);
        p.biot_alpha = 0.0;
        assert!(calculate_consolidation(&p).contains("ERROR"));
    }
}
