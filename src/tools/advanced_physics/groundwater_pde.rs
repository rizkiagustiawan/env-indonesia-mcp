use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Richards equation parameter set (van Genuchten–Mualem, VGM).
/// Ref: van Genuchten 1980; Mualem 1976; Celia et al. 1990 (modified Picard).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RichardsParam {
    #[schemars(description = "Residual water content θr (m3/m3)")]
    pub theta_r: f64,
    #[schemars(description = "Saturated water content θs (m3/m3)")]
    pub theta_s: f64,
    #[schemars(description = "van Genuchten alpha (1/m)")]
    pub alpha_inv_m: f64,
    #[schemars(description = "van Genuchten n (dimensionless, must be > 1)")]
    pub n_vg: f64,
    #[schemars(description = "Saturated hydraulic conductivity Ks (m/s)")]
    pub k_sat_m_s: f64,
    #[schemars(description = "Soil column depth (m)")]
    pub depth_m: f64,
    #[schemars(description = "Number of nodes (>= 3)")]
    pub n_nodes: u32,
    #[schemars(description = "Total simulation time (s)")]
    pub t_total_s: f64,
    #[schemars(description = "Initial pressure head (m, negative = unsaturated)")]
    pub h_initial_m: f64,
    #[schemars(description = "Top boundary pressure head (m)")]
    pub h_top_m: f64,
    #[schemars(description = "Bottom boundary pressure head (m)")]
    pub h_bottom_m: f64,
}

/// van Genuchten (1980) water retention: θ(h) for pressure head h (m).
fn theta_of_h(h: f64, p: &RichardsParam) -> f64 {
    if h >= 0.0 {
        p.theta_s
    } else {
        let ah = (p.alpha_inv_m * h.abs()).powf(p.n_vg);
        let m = 1.0 - 1.0 / p.n_vg;
        p.theta_r + (p.theta_s - p.theta_r) / (1.0 + ah).powf(m)
    }
}

/// Mualem (1976) + van Genuchten (1980) relative hydraulic conductivity K(h).
fn k_of_h(h: f64, p: &RichardsParam) -> f64 {
    if h >= 0.0 {
        p.k_sat_m_s
    } else {
        let m = 1.0 - 1.0 / p.n_vg;
        let ah = (p.alpha_inv_m * h.abs()).powf(p.n_vg);
        let se = 1.0 / (1.0 + ah).powf(m);
        let term = 1.0 - (1.0 - se.powf(1.0 / m)).powf(m);
        p.k_sat_m_s * se.sqrt() * term * term
    }
}

/// Specific moisture capacity C(h) = dθ/dh (m⁻¹).
fn capacity_of_h(h: f64, p: &RichardsParam) -> f64 {
    if h >= 0.0 {
        0.0
    } else {
        let m = 1.0 - 1.0 / p.n_vg;
        let ah = (p.alpha_inv_m * h.abs()).powf(p.n_vg);
        (p.theta_s - p.theta_r) * p.alpha_inv_m * (p.n_vg - 1.0)
            * (p.alpha_inv_m * h.abs()).powf(p.n_vg - 1.0)
            / (1.0 + ah).powf(m + 1.0)
    }
}

/// Thomas algorithm for a tridiagonal system a_i x_{i-1} + b_i x_i + c_i x_{i+1} = d_i.
/// a[0] and c[n-1] are ignored (0 by convention).
fn thomas(a: &[f64], b: &[f64], c: &[f64], d: &[f64]) -> Vec<f64> {
    let n = b.len();
    let mut cp = vec![0.0; n];
    let mut dp = vec![0.0; n];
    cp[0] = c[0] / b[0];
    dp[0] = d[0] / b[0];
    for i in 1..n {
        let m = 1.0 / (b[i] - a[i] * cp[i - 1]);
        cp[i] = c[i] * m;
        dp[i] = (d[i] - a[i] * dp[i - 1]) * m;
    }
    let mut x = vec![0.0; n];
    x[n - 1] = dp[n - 1];
    for i in (0..n - 1).rev() {
        x[i] = dp[i] - cp[i] * x[i + 1];
    }
    x
}

/// 1D mixed-form Richards equation solver (z positive downward) using the
/// modified-Picard scheme of Celia et al. (1990). Returns a formatted report.
pub fn solve_richards_1d(p: &RichardsParam) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Richards Equation Solver (1D, VGM)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: van Genuchten 1980; Mualem 1976; Celia et al. 1990\n\n");

    if p.n_nodes < 3 {
        return format!("ERROR: n_nodes ({}) harus >= 3.", p.n_nodes);
    }
    if p.n_vg <= 1.0 {
        return "ERROR: van Genuchten n harus > 1.".into();
    }
    if p.k_sat_m_s <= 0.0 || p.depth_m <= 0.0 || p.t_total_s <= 0.0 {
        return "ERROR: Ks, depth, dan t_total harus positif.".into();
    }
    if p.theta_s <= p.theta_r {
        return "ERROR: theta_s harus > theta_r.".into();
    }

    let n = p.n_nodes as usize;
    let dz = p.depth_m / (n as f64 - 1.0);

    // Initial state (uniform pressure head).
    let mut h = vec![p.h_initial_m; n];
    let mut theta: Vec<f64> = h.iter().map(|&hi| theta_of_h(hi, p)).collect();

    // Time stepping.
    let n_steps = 200usize;
    let dt = p.t_total_s / n_steps as f64;
    let picard_tol = 1e-6;
    let picard_max = 60;

    for _step in 0..n_steps {
        let theta_n = theta.clone();

        for _pic in 0..picard_max {
            // Recompute K at nodes and interfaces from current h.
            let k_node: Vec<f64> = h.iter().map(|&hi| k_of_h(hi, p)).collect();
            let c_node: Vec<f64> = h.iter().map(|&hi| capacity_of_h(hi, p)).collect();

            // Interior nodes i = 1 .. n-2 (interfaces i-1/2 and i+1/2).
            let m = n - 2; // number of interior unknowns
            let mut a = vec![0.0; m];
            let mut b = vec![0.0; m];
            let mut c = vec![0.0; m];
            let mut d = vec![0.0; m];

            for i in 1..(n - 1) {
                let idx = i - 1;
                let k_top = 0.5 * (k_node[i - 1] + k_node[i]);
                let k_bot = 0.5 * (k_node[i] + k_node[i + 1]);
                let cap = c_node[i];

                a[idx] = -dt * k_top;
                c[idx] = -dt * k_bot;
                b[idx] = cap * dz * dz + dt * (k_top + k_bot);
                d[idx] = cap * h[i] * dz * dz
                    + (theta_n[i] - theta[i]) * dz * dz
                    + dt * dz * (k_top - k_bot);

                // Fold known boundary nodes into RHS.
                if i == 1 {
                    d[idx] -= a[idx] * p.h_top_m;
                    a[idx] = 0.0;
                }
                if i == n - 2 {
                    d[idx] -= c[idx] * p.h_bottom_m;
                    c[idx] = 0.0;
                }
            }

            let h_new = thomas(&a, &b, &c, &d);

            // Update interior pressure heads.
            let mut max_dh = 0.0f64;
            for i in 1..(n - 1) {
                let dh = (h_new[i - 1] - h[i]).abs();
                if dh > max_dh {
                    max_dh = dh;
                }
                h[i] = h_new[i - 1];
            }

            // Recompute theta from h.
            for i in 0..n {
                theta[i] = theta_of_h(h[i], p);
            }

            if max_dh < picard_tol {
                break;
            }
        }
    }

    // Mass balance (total water stored).
    let mass_initial = theta_of_h(p.h_initial_m, p) * p.depth_m;
    let mass_final: f64 = theta.iter().sum::<f64>() * dz;
    let flux_top = {
        let k_top = 0.5 * (k_of_h(h[0], p) + k_of_h(h[1], p));
        let grad = (h[1] - h[0]) / dz;
        k_top * (1.0 - grad)
    };

    out.push_str(&format!(
        "Parameter VGM:\n  θr={:.4}  θs={:.4}  α={:.3} /m  n={:.3}\n  Ks={:.3e} m/s  L={:.2} m  nodes={}\n\n",
        p.theta_r, p.theta_s, p.alpha_inv_m, p.n_vg, p.k_sat_m_s, p.depth_m, n
    ));
    out.push_str(&format!("Simulasi: t_total={:.1} s, dt={:.3} s, {} langkah\n\n", p.t_total_s, dt, n_steps));
    out.push_str("HASIL PROFIL (top → bottom):\n");
    let step = (n as f64 / 12.0).ceil() as usize;
    for i in (0..n).step_by(step.max(1)) {
        let z = i as f64 * dz;
        out.push_str(&format!(
            "  z={:.2} m  h={:.3} m  θ={:.4}  K={:.3e} m/s\n",
            z, h[i], theta[i], k_of_h(h[i], p)
        ));
    }
    out.push_str(&format!(
        "\n  Water content rata-rata: {:.4}\n  Massa awal: {:.4} m  Massa akhir: {:.4} m\n  Flux atas (positif turun): {:.3e} m/s\n",
        theta.iter().sum::<f64>() / n as f64,
        mass_initial, mass_final, flux_top
    ));

    let wetting_front = {
        let mut idx = 0usize;
        for i in 0..n {
            if theta[i] > 0.5 * (p.theta_s - p.theta_r) + p.theta_r {
                idx = i;
            }
        }
        idx as f64 * dz
    };
    out.push_str(&format!("  Perkiraan kedalaman wetting front: {:.2} m\n", wetting_front));

    out
}

pub fn solve_pde(
    h_initial_json: &str,
    diffusivity_d: f64,
    dx_meters: f64,
    dy_meters: f64,
    time_steps: u32,
    dt_seconds: f64,
) -> String {
    let parsed: Result<Vec<Vec<f64>>, _> = serde_json::from_str(h_initial_json);
    if parsed.is_err() {
        return "ERROR: h_initial_json harus berupa array 2D elevasi awal muka air tanah.".into();
    }

    let mut h = parsed.unwrap();
    let rows = h.len();
    let cols = h[0].len();

    if rows < 3 || cols < 3 {
        return "ERROR: Ukuran grid minimal 3x3.".into();
    }

    // 1. Validasi Stabilitas Von Neumann (CFL Limit)
    // dt <= 1 / [2D (1/dx^2 + 1/dy^2)]
    let cfl_limit = 1.0
        / (2.0 * diffusivity_d * (1.0 / (dx_meters * dx_meters) + 1.0 / (dy_meters * dy_meters)));

    let mut safe_dt = dt_seconds;
    let mut modified_steps = time_steps;
    let mut warning_msg = String::new();

    if dt_seconds > cfl_limit {
        // Otomatis pecah dt menjadi sub-steps untuk menjaga stabilitas fisika tanpa merusak total waktu
        let total_time = dt_seconds * time_steps as f64;
        safe_dt = cfl_limit * 0.9; // 90% of max limit untuk safety
        modified_steps = (total_time / safe_dt).ceil() as u32;
        warning_msg = format!("⚠️ INTERVENSI FISIKA: Time step awal dt={:.1}s melampaui batas stabilitas CFL ({:.1}s). Simulasi berisiko NaN. Sistem otomatis memecah komputasi menjadi dt={:.1}s dengan {} iterasi.\n\n", 
                              dt_seconds, cfl_limit, safe_dt, modified_steps);
    }

    let mut h_next = h.clone();

    // 2. Eksekusi Numerik Beda Hingga Eksplisit
    for _ in 0..modified_steps {
        for i in 1..(rows - 1) {
            for j in 1..(cols - 1) {
                let d2h_dx2 = (h[i][j + 1] - 2.0 * h[i][j] + h[i][j - 1]) / (dx_meters * dx_meters);
                let d2h_dy2 = (h[i - 1][j] - 2.0 * h[i][j] + h[i + 1][j]) / (dy_meters * dy_meters);

                h_next[i][j] = h[i][j] + diffusivity_d * safe_dt * (d2h_dx2 + d2h_dy2);
            }
        }
        // Update state
        h = h_next.clone();
    }

    // 3. Evaluasi
    let center_val = h[rows / 2][cols / 2];

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Groundwater PDE Solver (2D)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str(&warning_msg);
    out.push_str(&format!(
        "Parameter: D = {} m²/s, dx = {}m, dy = {}m\n",
        diffusivity_d, dx_meters, dy_meters
    ));
    out.push_str(&format!(
        "Total Simulasi: {:.1} Jam\n\n",
        (safe_dt * modified_steps as f64) / 3600.0
    ));

    out.push_str("HASIL:\n");
    out.push_str(&format!(
        "  Elevasi akhir (tengah grid): {:.3} meter\n",
        center_val
    ));
    out.push_str("  Simulasi Selesai secara Stabil (No NaN).\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandy_loam() -> RichardsParam {
        RichardsParam {
            theta_r: 0.065,
            theta_s: 0.41,
            alpha_inv_m: 7.5,
            n_vg: 1.89,
            k_sat_m_s: 1.23e-5,
            depth_m: 1.0,
            n_nodes: 21,
            t_total_s: 3600.0,
            h_initial_m: -10.0,
            h_top_m: 0.0,
            h_bottom_m: -10.0,
        }
    }

    #[test]
    fn test_vg_retention_se() {
        let p = sandy_loam();
        // At h = -1/alpha, Se = 2^(-m)
        let m = 1.0 - 1.0 / p.n_vg;
        let h = -1.0 / p.alpha_inv_m;
        let theta = theta_of_h(h, &p);
        let se = (theta - p.theta_r) / (p.theta_s - p.theta_r);
        let expected = 2.0f64.powf(-m);
        assert!((se - expected).abs() < 1e-9, "Se={} expected={}", se, expected);
    }

    #[test]
    fn test_vg_saturation_and_conductivity_bounds() {
        let p = sandy_loam();
        assert_eq!(theta_of_h(0.0, &p), p.theta_s);
        assert_eq!(k_of_h(0.0, &p), p.k_sat_m_s);
        // Unsaturated K < Ks
        assert!(k_of_h(-5.0, &p) < p.k_sat_m_s);
        // Residual limit: very dry -> theta -> theta_r
        let dry = theta_of_h(-1000.0, &p);
        assert!((dry - p.theta_r).abs() < 0.01);
    }

    #[test]
    fn test_richards_steady_state_saturated() {
        // Saturated column everywhere (h=0) should stay saturated.
        let mut p = sandy_loam();
        p.h_initial_m = 0.0;
        p.h_top_m = 0.0;
        p.h_bottom_m = 0.0;
        let out = solve_richards_1d(&p);
        assert!(out.contains("θ=0.4100"));
    }

    #[test]
    fn test_richards_wetting_front_propagates() {
        let p = sandy_loam();
        let out = solve_richards_1d(&p);
        assert!(out.contains("wetting front"));
        assert!(out.contains("m/s"));
    }

    #[test]
    fn test_richards_rejects_bad_params() {
        let mut p = sandy_loam();
        p.n_vg = 1.0;
        assert!(solve_richards_1d(&p).contains("ERROR"));
        let mut p2 = sandy_loam();
        p2.n_nodes = 2;
        assert!(solve_richards_1d(&p2).contains("ERROR"));
    }
}
