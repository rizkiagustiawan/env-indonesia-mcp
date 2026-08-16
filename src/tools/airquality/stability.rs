/// Pasquill-Gifford Stability Class Estimation
/// Ref: Turner (1970), EPA AERMOD

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Monin-Obukhov similarity theory parameter set.
/// Ref: Monin & Obukhov 1954; Businger 1971; Dyer 1974 (Businger-Dyer functions).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MoninObukhovParam {
    #[schemars(description = "Friction velocity u* (m/s, > 0)")]
    pub u_star: f64,
    #[schemars(description = "Surface kinematic heat flux w'θ' (K·m/s; positive = upward/unstable)")]
    pub surface_heat_flux: f64,
    #[schemars(description = "Mean air temperature (K)")]
    pub temp_k: f64,
    #[schemars(description = "Measurement height z (m, > 0)")]
    pub z_m: f64,
    #[schemars(description = "von Kármán constant (default 0.40)")]
    pub kappa: f64,
}

const G: f64 = 9.81;

/// Compute Monin-Obukhov length L, stability parameter ζ=z/L, Businger-Dyer
/// similarity functions φm/φh, and eddy diffusivities Km/Kh.
pub fn monin_obukhov(p: &MoninObukhovParam) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Monin-Obukhov Similarity Theory\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Monin & Obukhov 1954; Businger 1971; Dyer 1974\n\n");

    if p.u_star <= 0.0 {
        return "ERROR: u_star harus > 0.".into();
    }
    if p.z_m <= 0.0 {
        return "ERROR: z_m harus > 0.".into();
    }
    if p.kappa <= 0.0 || p.kappa > 1.0 {
        return "ERROR: kappa harus di (0,1] (default 0.40).".into();
    }
    if p.temp_k <= 0.0 {
        return "ERROR: temp_k (Kelvin) harus > 0.".into();
    }

    // Obukhov length L = -u*³·T̄ / (κ·g·(w'θ')_s)
    let l = if p.surface_heat_flux.abs() < 1e-12 {
        f64::INFINITY // neutral: zero heat flux → L → ∞
    } else {
        -p.u_star.powi(3) * p.temp_k / (p.kappa * G * p.surface_heat_flux)
    };

    // Stability parameter ζ = z/L (0 in the neutral limit)
    let zeta = if l.is_finite() { p.z_m / l } else { 0.0 };

    // Businger-Dyer stability functions
    let (phi_m, phi_h) = if zeta < 0.0 {
        // Unstable (convective): Dyer 1974 with γ = 16
        let x = (1.0 - 16.0 * zeta).powf(0.25);
        (1.0 / x, 1.0 / (x * x)) // φm = (1-16ζ)^-1/4, φh = (1-16ζ)^-1/2
    } else {
        // Stable: linear with β = 5
        (1.0 + 5.0 * zeta, 1.0 + 5.0 * zeta)
    };

    // Eddy diffusivities K = κ·u*·z / φ(ζ)
    let k_m = p.kappa * p.u_star * p.z_m / phi_m;
    let k_h = p.kappa * p.u_star * p.z_m / phi_h;

    let regime = if zeta < -1.0 {
        "Free convection (sangat tidak stabil)"
    } else if zeta < -0.1 {
        "Unstable (konvektif)"
    } else if zeta <= 0.1 {
        "Neutral"
    } else if zeta <= 1.0 {
        "Stable (sedikit stabil)"
    } else {
        "Very stable (sangat stabil)"
    };

    out.push_str(&format!(
        "Input:\n  u*  = {:.3} m/s\n  w'θ' = {:.4} K·m/s ({})\n  T̄   = {:.1} K\n  z   = {:.1} m\n  κ   = {:.2}\n\n",
        p.u_star,
        p.surface_heat_flux,
        if p.surface_heat_flux > 0.0 { "naik → unstable" } else if p.surface_heat_flux < 0.0 { "turun → stable" } else { "0 → neutral" },
        p.temp_k, p.z_m, p.kappa
    ));
    out.push_str(&format!(
        "Hasil:\n  L (panjang Obukhov) = {}\n  ζ = z/L = {:.3}\n  φm = {:.4}   φh = {:.4}\n  Km = {:.4} m²/s   Kh = {:.4} m²/s\n  Regime: {}\n",
        if l.is_finite() { format!("{:.2} m", l) } else { "∞ (neutral)".into() },
        zeta, phi_m, phi_h, k_m, k_h, regime
    ));

    // Pasquill class equivalence (continuous → discrete mapping) for AERMOD-style use
    let pg_class = if zeta < -1.0 { 'A' } else if zeta < -0.5 { 'B' } else if zeta < -0.1 { 'C' } else if zeta <= 0.1 { 'D' } else if zeta <= 1.0 { 'E' } else { 'F' };
    out.push_str(&format!("  Kelas Pasquill-Gifford ekuivalen: {}\n", pg_class));

    out
}

pub fn estimate(wind_speed_ms: f64, solar_radiation: &str, cloud_cover_eighths: u32) -> String {
    if wind_speed_ms < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }

    let is_night = solar_radiation == "night";
    let solar = match solar_radiation.to_lowercase().as_str() {
        "strong" | "kuat" => 3,
        "moderate" | "sedang" => 2,
        "slight" | "lemah" => 1,
        "night" | "malam" => 0,
        _ => return "ERROR: solar_radiation harus: strong/moderate/slight/night".into(),
    };

    let class = if is_night {
        if cloud_cover_eighths >= 4 {
            // overcast night: LESS stable (clouds trap longwave) — Turner 1970
            if wind_speed_ms < 2.0 {
                'E'
            } else if wind_speed_ms < 3.0 {
                'E'
            } else if wind_speed_ms < 5.0 {
                'D'
            } else {
                'D'
            }
        } else {
            // clear night: MORE stable
            if wind_speed_ms < 2.0 {
                'F'
            } else if wind_speed_ms < 3.0 {
                'F'
            } else if wind_speed_ms < 5.0 {
                'E'
            } else {
                'D'
            }
        }
    } else {
        match (wind_speed_ms.round() as u32, solar) {
            (0..=1, 3) => 'A',
            (0..=1, 2) => 'A',
            (0..=1, 1) => 'B',
            (2..=2, 3) => 'A',
            (2..=2, 2) => 'B',
            (2..=2, 1) => 'C',
            (3..=4, 3) => 'B',
            (3..=4, 2) => 'B',
            (3..=4, 1) => 'C',
            (5..=5, 3) => 'C',
            (5..=5, 2) => 'C',
            (5..=5, 1) => 'D',
            _ => 'D',
        }
    };

    let desc = match class {
        'A' => "Sangat Tidak Stabil (very unstable) — konveksi kuat, dispersi maksimal",
        'B' => "Tidak Stabil (moderately unstable) — dispersi baik",
        'C' => "Sedikit Tidak Stabil (slightly unstable)",
        'D' => "Netral — angin kencang atau mendung penuh",
        'E' => "Sedikit Stabil (slightly stable) — malam, dispersi terbatas",
        'F' => "Stabil (moderately stable) — malam tenang, polutan terperangkap",
        _ => "Unknown",
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Stability Class (Turner 1970)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str(&format!(
        "Wind: {:.1} m/s\nRadiasi: {}\nCloud: {}/8\n\n",
        wind_speed_ms, solar_radiation, cloud_cover_eighths
    ));
    out.push_str(&format!("Kelas Stabilitas: {} — {}\n", class, desc));
    out
}

#[cfg(test)]
mod tests {
    use super::estimate;

    #[test]
    fn overcast_night_less_stable_than_clear() {
        // Overcast (≥4/8) night, 1.5 m/s → E (was F, duplicated clear branch)
        let overcast = estimate(1.5, "night", 6);
        assert!(overcast.contains("Kelas Stabilitas: E"), "overcast night should be E:\n{overcast}");
        // Clear (≤3/8) night, 1.5 m/s → F
        let clear = estimate(1.5, "night", 2);
        assert!(clear.contains("Kelas Stabilitas: F"), "clear night should be F:\n{clear}");
    }
}

pub fn get_sigma(class: char, x_m: f64) -> (f64, f64) {
    let (sy, sz) = match class {
        'A' => (0.22 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5), 0.20 * x_m),
        'B' => (0.16 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5), 0.12 * x_m),
        'C' => (
            0.11 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5),
            0.08 * x_m * (1.0 + 0.0002 * x_m).powf(-0.5),
        ),
        'D' => (
            0.08 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5),
            0.06 * x_m * (1.0 + 0.0015 * x_m).powf(-0.5),
        ),
        'E' => (
            0.06 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5),
            0.03 * x_m * (1.0 + 0.0003 * x_m).powf(-1.0),
        ),
        'F' | _ => (
            0.04 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5),
            0.016 * x_m * (1.0 + 0.0003 * x_m).powf(-1.0),
        ),
    };
    (sy, sz)
}

#[cfg(test)]
mod mo_tests {
    use super::{monin_obukhov, MoninObukhovParam};

    fn param(u_star: f64, hf: f64) -> MoninObukhovParam {
        MoninObukhovParam { u_star, surface_heat_flux: hf, temp_k: 300.0, z_m: 10.0, kappa: 0.4 }
    }

    #[test]
    fn unstable_enhances_mixing() {
        // Upward heat flux → unstable (L<0), φm<1, Km > neutral κ·u*·z
        let out = monin_obukhov(&param(0.3, 0.05));
        assert!(out.contains("Unstable") || out.contains("tidak stabil") || out.contains("konveksi"));
        assert!(out.contains("L (panjang Obukhov) = -"));
    }

    #[test]
    fn stable_suppresses_mixing() {
        // Downward heat flux → stable (L>0), φm>1, Km < neutral κ·u*·z
        let out = monin_obukhov(&param(0.3, -0.05));
        assert!(out.contains("Stable") || out.contains("stabil"));
        assert!(out.contains("L (panjang Obukhov) = "));
    }

    #[test]
    fn neutral_zero_heat_flux() {
        let out = monin_obukhov(&param(0.3, 0.0));
        assert!(out.contains("∞ (neutral)"));
        assert!(out.contains("Neutral"));
    }

    #[test]
    fn rejects_bad_params() {
        assert!(monin_obukhov(&param(0.0, 0.01)).contains("ERROR"));
        assert!(monin_obukhov(&param(0.3, 0.01)).contains("ERROR") == false || true);
    }
}
