/// Theis Equation: s = (Q/(4πT)) × W(u), u = r²S/(4Tt)
/// Cooper-Jacob approximation: s = (2.3Q)/(4πT) × log(2.25Tt/(r²S))
/// Ref: Theis (1935), Cooper & Jacob (1946)

pub fn calculate(q_m3s: f64, transmissivity_m2s: f64, storativity: f64, r_m: f64, t_s: f64) -> String {
    let mut out = String::from("=== Persamaan Theis — Penurunan Muka Air Tanah ===\n");
    out.push_str("Ref: Theis (1935), Cooper & Jacob (1946)\n\n");

    if q_m3s <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if transmissivity_m2s <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if storativity <= 0.0 || storativity >= 1.0 { return "ERROR: Storativity (S) harus antara 0 dan 1.".into(); }
    if r_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if t_s <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    let pi = std::f64::consts::PI;

    // u parameter
    let u = (r_m * r_m * storativity) / (4.0 * transmissivity_m2s * t_s);

    // Well function W(u) — series expansion for small u
    let wu = well_function(u);

    // Theis drawdown
    let s_theis = (q_m3s / (4.0 * pi * transmissivity_m2s)) * wu;

    // Cooper-Jacob approximation (valid when u < 0.01)
    let cj_valid = u < 0.01;
    let s_cj = if cj_valid {
        (2.3 * q_m3s) / (4.0 * pi * transmissivity_m2s) * (2.25 * transmissivity_m2s * t_s / (r_m * r_m * storativity)).log10()
    } else {
        0.0
    };

    let t_hours = t_s / 3600.0;
    let t_days = t_s / 86400.0;

    out.push_str(&format!("Input:\n  Q (debit pemompaan) = {:.4} m³/s ({:.2} L/s)\n  T (transmisivitas) = {:.2e} m²/s\n  S (storativity) = {:.2e}\n  r (jarak dari sumur) = {:.1} m\n  t (waktu pemompaan) = {:.0} s",
        q_m3s, q_m3s * 1000.0, transmissivity_m2s, storativity, r_m, t_s));
    if t_hours >= 1.0 { out.push_str(&format!(" ({:.1} jam)", t_hours)); }
    if t_days >= 1.0 { out.push_str(&format!(" ({:.1} hari)", t_days)); }
    out.push_str("\n\n");

    out.push_str(&format!("Perhitungan:\n  u = r²S/(4Tt) = {:.6e}\n  W(u) = {:.4}\n\n", u, wu));

    out.push_str(&format!("Penurunan (Theis): s = {:.3} m\n", s_theis));

    if cj_valid {
        out.push_str(&format!("Penurunan (Cooper-Jacob): s = {:.3} m (u < 0.01, aproksimasi valid)\n", s_cj));
    } else {
        out.push_str(&format!("Cooper-Jacob: u = {:.4e} ≥ 0.01 — aproksimasi TIDAK valid, gunakan Theis.\n", u));
    }

    // Radius of influence (where s ≈ 0.01 m)
    // s = Q/(4πT) × W(u) → solve iteratively for r where s = 0.01
    let s_target = 0.01;
    let mut r_inf = r_m;
    for _ in 0..200 {
        let u_try = (r_inf * r_inf * storativity) / (4.0 * transmissivity_m2s * t_s);
        let w_try = well_function(u_try);
        let s_try = (q_m3s / (4.0 * pi * transmissivity_m2s)) * w_try;
        if (s_try - s_target).abs() < 0.001 { break; }
        if s_try > s_target {
            r_inf *= 1.1;
        } else {
            r_inf *= 0.95;
        }
    }
    out.push_str(&format!("\nRadius pengaruh (s ≈ 0.01 m): R ≈ {:.0} m\n", r_inf));

    // Drawdown at multiple distances
    out.push_str(&format!("\nProfil penurunan pada t = {:.0} s:\n", t_s));
    out.push_str("  r (m)   | s (m)\n");
    for &dist in &[1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 200.0, 500.0] {
        let u_d = (dist * dist * storativity) / (4.0 * transmissivity_m2s * t_s);
        let w_d = well_function(u_d);
        let s_d = (q_m3s / (4.0 * pi * transmissivity_m2s)) * w_d;
        if s_d > 0.0 {
            out.push_str(&format!("  {:>6.0}  | {:.3}\n", dist, s_d));
        }
    }

    out
}

/// Well function W(u) using series expansion
fn well_function(u: f64) -> f64 {
    if u <= 0.0 { return 0.0; }
    if u > 60.0 { return 0.0; } // negligible

    // For small u, use series: W(u) = -γ - ln(u) + u - u²/(2·2!) + u³/(3·3!) - ...
    // Euler-Mascheroni constant γ = 0.5772156649
    let gamma = 0.5772156649;
    let mut w = -gamma - u.ln();
    let mut term = u;
    let mut sign = 1.0;
    for n in 1..=50 {
        w += sign * term / (n as f64);
        term *= u / ((n + 1) as f64);
        sign *= -1.0;
        if term.abs() < 1e-15 { break; }
    }
    w.max(0.0)
}
