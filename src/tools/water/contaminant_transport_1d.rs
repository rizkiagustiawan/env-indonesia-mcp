/// Contaminant Transport 1D — Ogata-Banks Analytical Solution
/// Ref: Ogata & Banks 1961; Sethi & Di Molfetta 2019; Freeze & Cherry 1979
/// C/C0 = 0.5 * erfc((x - v*t) / (2*sqrt(D*t))) for continuous source
pub fn assess(
    distance_m: f64,
    velocity_m_day: f64,
    dispersion_m2_day: f64,
    time_days: f64,
    retardation_factor: f64,
    decay_rate_day: f64,
    initial_conc_mg_l: f64,
) -> String {
    let mut out = String::from("=== Contaminant Transport 1D (Ogata-Banks) ===\n");
    out.push_str("Ref: Ogata & Banks 1961; Sethi & Di Molfetta 2019\n\n");

    if distance_m <= 0.0 || velocity_m_day <= 0.0 || time_days <= 0.0 {
        return "ERROR [E102]: distance, velocity, time must be > 0.".into();
    }

    let v = velocity_m_day / retardation_factor; // retarded velocity
    let D = dispersion_m2_day / retardation_factor; // retarded dispersion
    let t = time_days;

    out.push_str(&format!("Distance: {:.1} m\n", distance_m));
    out.push_str(&format!("Velocity: {:.2} m/day (retarded: {:.2})\n", velocity_m_day, v));
    out.push_str(&format!("Dispersion: {:.2e} m2/day (retarded: {:.2e})\n", dispersion_m2_day, D));
    out.push_str(&format!("Retardation R: {:.2}\n", retardation_factor));
    out.push_str(&format!("Decay rate: {:.4} day-1 (half-life: {:.1} days)\n", decay_rate_day, if decay_rate_day > 0.0 { 0.693/decay_rate_day } else { f64::INFINITY }));
    out.push_str(&format!("Time: {:.0} days ({:.1} years)\n\n", t, t/365.0));

    // Peclet number
    let pe = v * distance_m / D.max(1e-15);
    out.push_str(&format!("Peclet number: {:.1} (Pe>>1 = advection dominated)\n\n", pe));

    // Ogata-Banks: C/C0 = 0.5 * erfc((x - v*t) / (2*sqrt(D*t)))
    let arg = (distance_m - v * t) / (2.0 * (D * t).sqrt().max(1e-15));
    let erfc_val = erfc_approx(arg);
    let c_ratio_no_decay = 0.5 * erfc_val;

    // With first-order decay: multiply by exp(-lambda * t)
    let decay_factor = if decay_rate_day > 0.0 { (-decay_rate_day * t).exp() } else { 1.0 };
    let c_ratio = c_ratio_no_decay * decay_factor;
    let conc = initial_conc_mg_l * c_ratio;

    out.push_str("-- Ogata-Banks Solution --\n\n");
    out.push_str(&format!("  arg = (x - v*t) / (2*sqrt(D*t)) = {:.4}\n", arg));
    out.push_str(&format!("  erfc(arg) = {:.6}\n", erfc_val));
    out.push_str(&format!("  C/C0 (no decay) = 0.5 * erfc = {:.6}\n", c_ratio_no_decay));
    out.push_str(&format!("  Decay factor = exp(-lambda*t) = {:.6}\n", decay_factor));
    out.push_str(&format!("  >> C/C0 = {:.6}\n", c_ratio));
    out.push_str(&format!("  >> Concentration at {:.1}m after {:.0} days: {:.4} mg/L\n\n", distance_m, t, conc));

    // Travel time
    let travel_time = distance_m / v;
    out.push_str(&format!("  Travel time (advection): {:.1} days ({:.1} years)\n", travel_time, travel_time/365.0));
    out.push_str(&format!("  Arrived at receptor? {}\n\n", if t >= travel_time { "YES" } else { "NOT YET" }));

    // Plume front
    let plume_front = v * t;
    out.push_str(&format!("  Plume front position: {:.1} m\n", plume_front));
    out.push_str(&format!("  Dispersion spread: +/-{:.1} m (2*sqrt(D*t))\n\n", 2.0*(D*t).sqrt()));

    // ─── PP 22/2021 COMPLIANCE FOOTER ───
    out.push_str("\n─── STATUS KEPATUHAN (PP 22/2021 Annex VI — Air Tanah) ───\n\n");
    out.push_str(&format!("  Concentration at receptor: {:.4} mg/L\n", conc));
    out.push_str("  Baku Mutu Air Tanah (PP 22/2021 Annex VI):\n");
    out.push_str("  - Benzene: ≤0.01 mg/L\n");
    out.push_str("  - TCE/PCE: ≤0.003 mg/L\n");
    out.push_str("  - Pb: ≤0.05 mg/L | Cd: ≤0.01 | Hg: ≤0.001 | As: ≤0.05\n");
    out.push_str("  - Total Coliform: NIHIL (0)\n\n");

    let exceeds = conc > 0.001;
    if exceeds {
        out.push_str(&format!("  ❌ MELEBIHI baku mutu air tanah ({:.4} > 0.001)\n", conc));
        out.push_str("  Tindakan: remediasi (P&T, PRB, SVE, bioremediation)\n");
    } else {
        out.push_str("  ✅ DI BAWAH baku mutu air tanah\n");
    }

    out.push_str("\n─── REKOMENDASI MITIGASI ───\n");
    if exceeds {
        out.push_str("  1. Pump & Treat (groundwater extraction + treatment)\n");
        out.push_str("  2. Permeable Reactive Barrier (PRB) di downstream\n");
        out.push_str("  3. Monitored Natural Attenuation (MNA) jika low risk\n");
        out.push_str("  4. Source removal/control di area kontaminasi\n");
    } else {
        out.push_str("  Monitoring berkala — pertahankan kondisi\n");
    }

    out.push_str("\n─── PEMANTAUAN (RPL) ───\n");
    out.push_str("  Parameter: kontaminan spesifik + parameter pendukung (pH, DO, EC)\n");
    out.push_str("  Frekuensi: Quarterly (active plume), Semi-annual (stable)\n");
    out.push_str("  Lokasi: Source area + plume centerline + receptor wells\n");

    out.push_str("\n─── PELAPORAN & IZIN ───\n");
    out.push_str("  PP 22/2021 Annex VI (air tanah) + Pasal 124-131\n");
    out.push_str("  PP 101/2014 (B3-contaminated land)\n");
    out.push_str("  Amdalnet + OSS; Permen LH 6/2026 (sanksi)\n");

    out.push_str("\n  Ref: Ogata & Banks 1961; Freeze & Cherry 1979; PP 22/2021 Annex VI\n");
    out
}

fn erfc_approx(x: f64) -> f64 {
    // Numerical approximation of complementary error function
    // Ref: Abramowitz & Stegun 1964, Formula 7.1.26
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    // A&S 7.1.26: erfc(x) = poly * e^(-x^2). BUG FIX: e^(-x^2) factor was missing.
    let erfc_pos = poly * (-x * x).exp();
    if x >= 0.0 { erfc_pos } else { 2.0 - erfc_pos }
}

#[cfg(test)]
mod tests {
    use super::erfc_approx;
    #[test]
    fn erfc_reference_values() {
        assert!((erfc_approx(0.0) - 1.0).abs() < 1e-4);
        assert!((erfc_approx(1.0) - 0.157299).abs() < 1e-4, "erfc(1)={}", erfc_approx(1.0));
        assert!((erfc_approx(2.0) - 0.004678).abs() < 1e-4, "erfc(2)={}", erfc_approx(2.0));
    }
}


use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 1D Advection-Dispersion-Reaction (ADR) Parameter Set with Non-Linear Sorption.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AdrSorptionParam {
    #[schemars(description = "Groundwater velocity (m/day)")]
    pub velocity_m_day: f64,
    #[schemars(description = "Hydrodynamic dispersion (m2/day)")]
    pub dispersion_m2_day: f64,
    #[schemars(description = "First-order decay rate (1/day)")]
    pub decay_rate_day: f64,
    #[schemars(description = "Soil bulk density ρb (kg/L or g/cm3, typical 1.2-1.6)")]
    pub bulk_density_kg_l: f64,
    #[schemars(description = "Effective porosity θ (0.1-0.5)")]
    pub porosity: f64,
    #[schemars(description = "Sorption model: 'linear', 'freundlich', or 'langmuir'")]
    pub sorption_model: String,
    #[schemars(description = "Sorption param 1: Kd (linear), Kf (freundlich), or b (langmuir binding const)")]
    pub param_k: f64,
    #[schemars(description = "Sorption param 2: n (freundlich exponent), or Smax (langmuir max capacity). Ignored for linear.")]
    pub param_n_or_smax: f64,
    #[schemars(description = "Continuous source concentration at x=0 (mg/L)")]
    pub source_conc_mg_l: f64,
    #[schemars(description = "Domain length (m)")]
    pub length_m: f64,
    #[schemars(description = "Simulation time (days)")]
    pub time_days: f64,
    #[schemars(description = "Spatial step dx (m, default 1.0)")]
    pub dx_m: f64,
}

fn retardation_factor(c: f64, p: &AdrSorptionParam) -> f64 {
    let rho_theta = p.bulk_density_kg_l / p.porosity.max(1e-6);
    if c <= 1e-9 {
        // Limit for c->0 to avoid singularity in Freundlich if n<1
        return match p.sorption_model.to_lowercase().as_str() {
            "linear" => 1.0 + rho_theta * p.param_k,
            "freundlich" => {
                if p.param_n_or_smax < 1.0 { 1.0 + rho_theta * p.param_k * p.param_n_or_smax * 1e-9_f64.powf(p.param_n_or_smax - 1.0) }
                else { 1.0 }
            },
            "langmuir" => 1.0 + rho_theta * p.param_n_or_smax * p.param_k, // (1+bC)^2 -> 1
            _ => 1.0,
        };
    }
    match p.sorption_model.to_lowercase().as_str() {
        "linear" => 1.0 + rho_theta * p.param_k,
        "freundlich" => 1.0 + rho_theta * p.param_k * p.param_n_or_smax * c.powf(p.param_n_or_smax - 1.0),
        "langmuir" => {
            let denom = 1.0 + p.param_k * c;
            1.0 + rho_theta * p.param_n_or_smax * p.param_k / (denom * denom)
        },
        _ => 1.0, // default no sorption
    }
}

pub fn solve_adr_sorption(p: &AdrSorptionParam) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  ADR Non-Linear Sorption Solver (1D)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Zheng & Bennett 2002; Langmuir 1918; Freundlich 1909\n\n");

    let dx = p.dx_m.max(0.1);
    let nx = (p.length_m / dx).ceil() as usize + 1;
    let mut c = vec![0.0; nx];
    c[0] = p.source_conc_mg_l; // Dirichlet BC at x=0

    // Courant-Friedrichs-Lewy (CFL) limits for explicit scheme
    // max dt for advection: dx / v
    // max dt for dispersion: 0.5 * dx^2 / D
    let dt_adv = if p.velocity_m_day > 0.0 { dx / p.velocity_m_day } else { f64::INFINITY };
    let dt_disp = if p.dispersion_m2_day > 0.0 { 0.5 * dx * dx / p.dispersion_m2_day } else { f64::INFINITY };
    let dt = (0.5 * dt_adv).min(0.5 * dt_disp).min(p.time_days / 10.0).max(1e-6);
    let n_steps = (p.time_days / dt).ceil() as usize;
    let actual_dt = p.time_days / n_steps as f64;

    for _ in 0..n_steps {
        let mut c_new = c.clone();
        for i in 1..nx-1 {
            let r = retardation_factor(c[i], p);
            // Upwind for advection
            let adv = p.velocity_m_day * (c[i] - c[i-1]) / dx;
            // Central for dispersion
            let disp = p.dispersion_m2_day * (c[i+1] - 2.0 * c[i] + c[i-1]) / (dx * dx);
            // Decay
            let dec = p.decay_rate_day * c[i];
            
            let dc_dt = (disp - adv) / r - dec;
            c_new[i] = c[i] + actual_dt * dc_dt;
            c_new[i] = c_new[i].max(0.0); // positivity preservation
        }
        // Outlet Neumann BC (dC/dx = 0)
        c_new[nx-1] = c_new[nx-2];
        c = c_new;
    }

    out.push_str(&format!("Model Sorpsi     : {}\n", p.sorption_model.to_uppercase()));
    out.push_str(&format!("Velocity (v)     : {:.3} m/hari\n", p.velocity_m_day));
    out.push_str(&format!("Dispersion (D)   : {:.3} m²/hari\n", p.dispersion_m2_day));
    out.push_str(&format!("Grid Resolusi    : dx = {:.2} m, dt = {:.4} hari ({} steps)\n\n", dx, actual_dt, n_steps));

    out.push_str("Profil Konsentrasi Akhir (mg/L):\n");
    let step_print = (nx / 10).max(1);
    for i in (0..nx).step_by(step_print) {
        let x = i as f64 * dx;
        let r_val = retardation_factor(c[i], p);
        out.push_str(&format!("  x = {:5.1} m | C = {:8.4} mg/L | R(C) = {:6.2}\n", x, c[i], r_val));
    }
    
    // Check receptor hit
    if c[nx-1] > 1e-4 {
        out.push_str(&format!("\n⚠️ KONTAMINAN MENCAPAI RESEPTOR (x={:.1}m) dengan konsentrasi {:.4} mg/L\n", p.length_m, c[nx-1]));
    } else {
        out.push_str(&format!("\n✅ Kontaminan belum mencapai ujung domain (x={:.1}m).\n", p.length_m));
    }

    out
}

#[cfg(test)]
mod adr_tests {
    use super::{solve_adr_sorption, AdrSorptionParam};

    #[test]
    fn test_langmuir_retardation() {
        let p = AdrSorptionParam {
            velocity_m_day: 1.0, dispersion_m2_day: 0.1, decay_rate_day: 0.0,
            bulk_density_kg_l: 1.5, porosity: 0.3,
            sorption_model: "langmuir".into(), param_k: 0.5, param_n_or_smax: 10.0,
            source_conc_mg_l: 100.0, length_m: 50.0, time_days: 10.0, dx_m: 1.0,
        };
        let out = solve_adr_sorption(&p);
        assert!(out.contains("LANGMUIR"));
        assert!(out.contains("R(C) ="));
    }
}
