//! Settling Velocity Calculator — Stokes, Newton, Hazen
//! For particles in water (sedimentation) and air (particulate deposition)
//! Ref: Metcalf & Eddy (2014), Hinds (1999) Aerosol Technology

pub fn calculate(
    particle_diameter_um: f64,  // particle diameter (micrometers)
    particle_density_kgm3: f64, // particle density (kg/m³)
    fluid: &str,                // "water" or "air"
    temperature_c: f64,         // fluid temperature (°C)
) -> String {
    let g = 9.81;

    // Fluid properties
    let (rho_f, mu) = match fluid.to_lowercase().as_str() {
        "water" | "air_limbah" => {
            let rho = 999.0 - 0.05 * (temperature_c - 4.0).powi(2) / 100.0; // simplified
            let _mu_w = 0.001
                * 10.0_f64.powf(
                    -(1.3272 * (temperature_c - 20.0) - 0.001053 * (temperature_c - 20.0).powi(2))
                        / (temperature_c + 105.0),
                );
            let mu_actual = 0.001002
                * 10.0_f64.powf(-((temperature_c - 20.0) * 1.3272) / (temperature_c + 105.0)); // Poiseuille
            (rho, mu_actual.max(0.0003))
        }
        "air" | "udara" => {
            let rho = 1.225 * 293.15 / (temperature_c + 273.15); // ideal gas
            let mu_a = 1.81e-5 * ((temperature_c + 273.15) / 293.15).powf(0.7); // Sutherland approx
            (rho, mu_a)
        }
        _ => {
            return format!(
                "ERROR: Fluid '{}' tidak dikenal. Gunakan: water, air",
                fluid
            )
        }
    };

    let d_m = particle_diameter_um * 1e-6; // convert to meters
    let rho_p = particle_density_kgm3;
    let delta_rho = rho_p - rho_f;

    if delta_rho <= 0.0 {
        return "ERROR: Partikel lebih ringan dari fluida — tidak akan mengendap.".to_string();
    }

    // Stokes settling velocity (Re < 1, laminar)
    let v_stokes = delta_rho * g * d_m.powi(2) / (18.0 * mu);

    // Reynolds number check
    let re_stokes = rho_f * v_stokes * d_m / mu;

    // Newton/intermediate regime — iterative
    let v_newton;
    let regime;
    if re_stokes < 1.0 {
        v_newton = v_stokes;
        regime = "Stokes (Re < 1, laminar)";
    } else if re_stokes < 1000.0 {
        // Intermediate — Schiller-Naumann: Cd = 24/Re × (1 + 0.15 Re^0.687)
        // Iterate
        let mut v = v_stokes;
        for _ in 0..50 {
            let re = (rho_f * v * d_m / mu).max(0.001);
            let cd = 24.0 / re * (1.0 + 0.15 * re.powf(0.687));
            v = (4.0 * delta_rho * g * d_m / (3.0 * cd * rho_f)).sqrt();
        }
        v_newton = v;
        regime = "Intermediate (1 < Re < 1000)";
    } else {
        // Newton regime: Cd ≈ 0.44
        v_newton = (4.0 * delta_rho * g * d_m / (3.0 * 0.44 * rho_f)).sqrt();
        regime = "Newton (Re > 1000, turbulent)";
    }

    let re_final = rho_f * v_newton * d_m / mu;

    // Hazen settling: overflow rate = v_s
    let ofr = v_newton * 3600.0; // m/hr (for tank design)

    // Time to settle 1m
    let t_settle_1m = if v_newton > 0.0 {
        1.0 / v_newton
    } else {
        f64::INFINITY
    };

    format!(
        "=== SETTLING VELOCITY ===\n\
         Ref: Stokes (1851), Metcalf & Eddy (2014), Hinds (1999)\n\n\
         INPUT:\n  Diameter = {:.1} μm ({:.2e} m)\n  Densitas partikel = {:.0} kg/m³\n  Fluida = {} (ρ = {:.1} kg/m³, μ = {:.2e} Pa·s)\n  Suhu = {:.1}°C\n\n\
         KECEPATAN PENGENDAPAN:\n  v_Stokes = {:.6} m/s ({:.4} mm/s)\n  v_final = {:.6} m/s ({:.4} mm/s)\n  Regime = {}\n  Re = {:.3}\n\n\
         DESAIN SEDIMENTASI:\n  Overflow rate = {:.4} m/jam\n  Waktu endap 1m = {:.1} detik ({:.2} menit)\n\
         Hazen: Luas tank = Q / v_s\n\n\
         Klasifikasi partikel:\n  {}",
        particle_diameter_um, d_m, rho_p, fluid, rho_f, mu, temperature_c,
        v_stokes, v_stokes * 1000.0, v_newton, v_newton * 1000.0,
        regime, re_final,
        ofr, t_settle_1m, t_settle_1m / 60.0,
        if particle_diameter_um > 1000.0 { "Pasir (sand) — gravitasi cepat" }
        else if particle_diameter_um > 50.0 { "Silt — gravitasi lambat" }
        else if particle_diameter_um > 1.0 { "Clay — butuh koagulan/flokulan" }
        else { "Koloid/nano — butuh koagulasi-flokulasi-sedimentasi" }
    )
}
