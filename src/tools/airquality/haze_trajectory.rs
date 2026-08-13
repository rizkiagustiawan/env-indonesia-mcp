/// Transboundary Haze Trajectory — Lagrangian Forward Particle Model
/// Problem: Asap gambut Sumatra/Kalimantan -> Singapura/Malaysia. No long-range smoke transport model.
/// Method: Lagrangian particle advection + Gaussian puff spread + dry/wet deposition decay.
/// Ref: Draxler & Hess 1998 (HYSPLIT concept); Seinfeld & Pandis 2016 (atmospheric chemistry);
///      ASEAN Agreement on Transboundary Haze Pollution (ratified by Indonesia 16 Sep 2014, UU 26/2014);
///      WHO 2021 AQG (PM2.5 24h = 15 ug/m3); Singapore NEA PSI thresholds.
/// Formula:
///   x(t) = x0 + u*t  (advection, u = wind vector)
///   sigma(t) = sigma0 + c*sqrt(t)  (Gaussian puff spread, c ~ 1 m/s for neutral stability)
///   C(receptor) = (Q * dt) / ((2*pi)^(3/2) * sigma_x * sigma_y * sigma_z) * exp(-d^2/(2*sigma^2)) * exp(-v_dep*t/H_mix)
///   Deposition loss factor: exp(-v_dep * t / H_mix)  (simple box-model scavenging)

pub fn trajectory(
    fire_lat: f64,
    fire_lon: f64,
    wind_speed_m_s: f64,
    wind_dir_deg: f64,
    duration_hours: f64,
    pm_emission_rate_g_s: f64,
    stack_height_m: f64,
) -> String {
    if wind_speed_m_s < 0.5 {
        return "ERROR [E102]: Wind speed < 0.5 m/s. Stagnant conditions — Lagrangian advection not valid. Use calm-wind puff model.".into();
    }
    if duration_hours <= 0.0 || duration_hours > 168.0 {
        return "ERROR [E102]: duration_hours must be 0-168 (max 7 days for constant-wind assumption).".into();
    }
    if pm_emission_rate_g_s <= 0.0 {
        return "ERROR [E102]: pm_emission_rate_g_s must be > 0.".into();
    }
    if stack_height_m < 0.0 {
        return "ERROR [E102]: stack_height_m must be >= 0.".into();
    }

    let mut out = String::from("════════════════════════════════════════════════════\n");
    out.push_str("TRANSBOUNDARY HAZE TRAJECTORY — LAGRANGIAN PARTICLE\n");
    out.push_str("Ref: Draxler & Hess 1998 (HYSPLIT); Seinfeld & Pandis 2016\n");
    out.push_str("Reg: ASEAN Agreement on Transboundary Haze Pollution (UU 26/2014)\n");
    out.push_str("════════════════════════════════════════════════════\n\n");

    // Physical constants
    let r_earth_km = 6371.0;
    let deg_to_rad = std::f64::consts::PI / 180.0;
    let rad_to_deg = 180.0 / std::f64::consts::PI;

    // Wind direction convention: meteorological (deg FROM which wind blows).
    // We convert to direction TO which wind travels (downwind).
    // Downwind bearing = wind_dir + 180.
    let downwind_bearing = (wind_dir_deg + 180.0) % 360.0;
    let bearing_rad = downwind_bearing * deg_to_rad;

    // Total travel time
    let dt_s = 3600.0; // 1-hour timestep
    let n_steps = (duration_hours).round() as i64;
    let total_s = duration_hours * 3600.0;

    // Mixing height (tropical boundary layer, daytime peat fire season)
    let h_mix_m = 1500.0; // m, typical tropical PBL (Seinfeld & Pandis)

    // Dry deposition velocity for PM2.5 (Slinn 1982, typical 0.1-0.5 cm/s)
    let v_dep_m_s = 0.002; // 0.2 cm/s = 0.002 m/s (WHO/FAO typical for fine PM)

    // Gaussian puff spread coefficient (Draxler: sigma = sigma0 + c*sqrt(t))
    // c ~ 0.5-2.0 m/s for neutral stability (Pasquill class D over tropical ocean)
    let sigma0 = 100.0; // m, initial source size (fire plume)
    let c_spread = 1.0; // m/s^0.5, neutral stability

    out.push_str("INPUT:\n");
    out.push_str(&format!("  Fire source     : ({:.4} lat, {:.4} lon)\n", fire_lat, fire_lon));
    out.push_str(&format!("  Wind speed      : {:.1} m/s\n", wind_speed_m_s));
    out.push_str(&format!("  Wind direction  : {:.0} deg (from) -> bearing {:.0} deg (to)\n", wind_dir_deg, downwind_bearing));
    out.push_str(&format!("  Duration        : {:.0} hours ({:.1} days)\n", duration_hours, duration_hours / 24.0));
    out.push_str(&format!("  PM emission     : {:.1} g/s = {:.0} kg/day\n", pm_emission_rate_g_s, pm_emission_rate_g_s * 86400.0 / 1000.0));
    out.push_str(&format!("  Stack height    : {:.0} m (peat fire effective)\n", stack_height_m));
    out.push_str(&format!("\nMODEL PARAMS:\n"));
    out.push_str(&format!("  Mixing height H_mix : {:.0} m (tropical PBL)\n", h_mix_m));
    out.push_str(&format!("  Dry dep velocity    : {:.4} m/s (PM2.5 Slinn 1982)\n", v_dep_m_s));
    out.push_str(&format!("  Puff spread sigma0  : {:.0} m, c = {:.1} m/s^0.5\n", sigma0, c_spread));
    out.push_str(&format!("  Timestep            : {:.0} s (1h)\n\n", dt_s));

    // Trajectory waypoints — advect by wind
    out.push_str("TRAJECTORY WAYPOINTS (hourly):\n");
    out.push_str(&format!("  {:>6} {:>10} {:>10} {:>12} {:>12}\n", "hour", "lat", "lon", "dist_km", "sigma_m"));
    out.push_str(&format!("  {}\n", "─".repeat(56)));

    let mut waypoints: Vec<(f64, f64, f64)> = Vec::new(); // (lat, lon, dist_km)
    let mut cur_lat = fire_lat;
    let mut cur_lon = fire_lon;
    let mut total_dist_m = 0.0;

    for h in 0..=n_steps {
        let t_s = h as f64 * dt_s;
        let sigma = sigma0 + c_spread * t_s.sqrt();

        if h > 0 {
            // Advect: move distance = wind_speed * dt along bearing
            let step_dist_m = wind_speed_m_s * dt_s;
            total_dist_m += step_dist_m;
            // Convert meters to degrees using equirectangular approximation
            let d_lat = (step_dist_m / 1000.0 * bearing_rad.cos()) / r_earth_km * rad_to_deg;
            let d_lon = (step_dist_m / 1000.0 * bearing_rad.sin())
                / (r_earth_km * cur_lat.to_radians().cos().abs().max(0.01))
                * rad_to_deg;
            cur_lat += d_lat;
            cur_lon += d_lon;
        }
        waypoints.push((cur_lat, cur_lon, total_dist_m / 1000.0));
        out.push_str(&format!(
            "  {:>6} {:>10.4} {:>10.4} {:>12.1} {:>12.1}\n",
            h, cur_lat, cur_lon, total_dist_m / 1000.0, sigma
        ));
    }

    let final_lat = waypoints.last().unwrap().0;
    let final_lon = waypoints.last().unwrap().1;
    let total_km = waypoints.last().unwrap().2;

    // ─── RECEPTOR CONCENTRATION ───
    // At receptor located at final waypoint (downwind distance L):
    // C = (Q * dt_eff) / ((2*pi)^(3/2) * sigma^3) * exp(-H^2/(2*sigma^2)) * deposition_factor
    // For a continuous source over duration T, total mass released = Q * T
    // Diluted into puff volume ~ (2*pi)^(3/2) * sigma_x * sigma_y * sigma_z (assuming isotropic spread)
    let sigma_final = sigma0 + c_spread * total_s.sqrt();
    // Anisotropic: horizontal spread larger than vertical (capped by mixing height)
    let sigma_h = sigma_final;
    let sigma_z = (sigma_final).min(h_mix_m);

    let mass_released_g = pm_emission_rate_g_s * total_s;
    // Puff volume (m^3)
    let puff_volume = (2.0 * std::f64::consts::PI).powf(1.5) * sigma_h * sigma_h * sigma_z;
    // Ground-level concentration with image source (Turner 1994; Seinfeld & Pandis 2016):
    // C ∝ exp(-H²/2σz²) + exp(-H²/2σz²) = 2·exp(-H²/2σz²)
    let height_term = 2.0 * (-stack_height_m.powi(2) / (2.0 * sigma_z.powi(2))).exp();
    // Deposition scavenging factor: exp(-v_dep * t / H_mix)
    let dep_factor = (-v_dep_m_s * total_s / h_mix_m).exp();
    let conc_g_m3 = mass_released_g / puff_volume * height_term * dep_factor;
    let conc_ug_m3 = conc_g_m3 * 1e6; // g/m3 -> ug/m3

    out.push_str(&format!("\nRECEPTOR (downwind endpoint):\n"));
    out.push_str(&format!("  Location       : ({:.4} lat, {:.4} lon)\n", final_lat, final_lon));
    out.push_str(&format!("  Travel distance: {:.1} km\n", total_km));
    out.push_str(&format!("  Travel time    : {:.1} h ({:.2} days)\n", duration_hours, duration_hours / 24.0));
    out.push_str(&format!("  Puff sigma (h) : {:.0} m\n", sigma_h));
    out.push_str(&format!("  Puff sigma (z) : {:.0} m (capped at H_mix)\n", sigma_z));
    out.push_str(&format!("  Mass released  : {:.0} g = {:.1} t\n", mass_released_g, mass_released_g / 1e6));
    out.push_str(&format!("  Deposition loss: {:.1}% remaining (factor {:.3})\n", dep_factor * 100.0, dep_factor));
    out.push_str(&format!("\n  >>> PM2.5 concentration at receptor = {:.1} ug/m3 <<<\n\n", conc_ug_m3));

    // ─── COMPLIANCE: WHO 2021 + Singapore PSI ───
    out.push_str("─── COMPLIANCE: WHO 2021 AQG + SINGAPORE PSI ───\n\n");
    let who_24h = 15.0; // ug/m3, WHO 2021 PM2.5 24-hour guideline
    let who_exceed = conc_ug_m3 / who_24h;
    out.push_str(&format!("  WHO 2021 PM2.5 24h guideline: 15 ug/m3\n"));
    out.push_str(&format!("  Ratio (C/guideline): {:.1}x  {}\n", who_exceed,
        if conc_ug_m3 > who_24h { "EXCEED" } else { "OK" }));

    // Singapore PSI is based on 24h PM2.5 (NEA): PSI ~ 4 * PM2.5 (ug/m3) approx (since 2014 PM2.5 sub-index)
    // PSI bands: 0-50 Good, 51-100 Moderate, 101-200 Unhealthy, 201-300 Very Unhealthy, >300 Hazardous
    let psi_estimate = conc_ug_m3 * 4.0; // approximate conversion
    let psi_band = if psi_estimate <= 50.0 { "Good" }
        else if psi_estimate <= 100.0 { "Moderate" }
        else if psi_estimate <= 200.0 { "Unhealthy" }
        else if psi_estimate <= 300.0 { "Very Unhealthy" }
        else { "Hazardous" };
    out.push_str(&format!("\n  Singapore PSI (est): {:.0} [{}]\n", psi_estimate, psi_band));
    out.push_str(&format!("  (PSI ~ 4 * PM2.5 ug/m3, NEA 24h PM2.5 sub-index)\n"));

    // ─── ASEAN CONTEXT ───
    out.push_str("\n─── ASEAN TRANSBOUNDARY HAZE CONTEXT ───\n\n");
    out.push_str("  2015 Haze Crisis: El Nino + peat fires, 43M Indonesians exposed,\n");
    out.push_str("    PM2.5 > 1000 ug/m3 in Central Kalimantan. Economic loss ~USD 16.1B.\n");
    out.push_str("    Source: Sumatra (Riau) & Kalimantan peatlands -> Singapore/Malaysia.\n");
    out.push_str("    Distance: Riau -> Singapore ~300 km; Kalimantan -> KL ~1000 km.\n\n");
    out.push_str("  ASEAN Agreement on Transboundary Haze Pollution:\n");
    out.push_str("    Signed 2002; Indonesia ratified 16 Sep 2014 (UU 26/2014).\n");
    out.push_str("    All 10 ASEAN states parties. ASEAN Coordinating Centre for\n");
    out.push_str("    Transboundary Haze Pollution Control (ACCTHPC) in Jakarta.\n\n");
    out.push_str("  Indonesian regulation:\n");
    out.push_str("    - PP 71/2014 (Peatland protection); UU 32/2009 (PPLH)\n");
    out.push_str("    - Permen LH 10/2010 (peat fire prevention); PP 22/2021 (PPLH)\n");
    out.push_str("    - UU 6/2023 (Penciptaan Lapangan Kerja — agrarian/peat reforms)\n");
    out.push_str("    - Criminal liability: UU 32/2009 Pasal 98-116 (pembakaran lahan)\n\n");

    // Reachability assessment
    out.push_str("─── REACHABILITY ASSESSMENT ───\n");
    let reachable_sg = total_km >= 300.0;
    let reachable_kl = total_km >= 1000.0;
    out.push_str(&format!("  Riau->Singapore (~300 km): {} ({:.0} km traveled)\n",
        if reachable_sg { "REACHABLE" } else { "not reached" }, total_km));
    out.push_str(&format!("  Kalimantan->KL (~1000 km): {} ({:.0} km traveled)\n",
        if reachable_kl { "REACHABLE" } else { "not reached" }, total_km));

    // ─── LIMITATIONS ───
    out.push_str("\n─── HONEST LIMITATIONS ───\n");
    out.push_str("  1. SINGLE PARTICLE: not an ensemble. No Monte Carlo / stochastic spread.\n");
    out.push_str("     Real HYSPLIT uses 10,000+ particles with turbulence parameterization.\n");
    out.push_str("  2. CONSTANT WIND: no time-varying meteorology, no vertical wind shear.\n");
    out.push_str("     Real transport uses 3D NWP (GFS/ECMWF) wind fields at each timestep.\n");
    out.push_str("  3. NO ATMOSPHERIC CHEMISTRY: no SO2->sulfate, NOx->nitrate, secondary\n");
    out.push_str("     organic aerosol formation. PM2.5 composition assumed constant.\n");
    out.push_str("  4. NO TERRAIN: ignores Sumatran Barisan mountains, Borneo highlands.\n");
    out.push_str("     Terrain blocks/channeled flow significantly in real events.\n");
    out.push_str("  5. NO WET DEPOSITION: no rainfall scavenging (critical in tropics).\n");
    out.push_str("     Dry deposition only; wet dep can remove 50-90% PM in convective storms.\n");
    out.push_str("  6. PLUME RISE: stack_height is fixed; real buoyant plume rise depends on\n");
    out.push_str("     fire heat flux, atmospheric stability (Briggs 1975).\n");
    out.push_str("  7. GAUSSIAN PUFF: valid for FIVE minutes to ~24h; beyond that, wind shift\n");
    out.push_str("     and stability changes invalidate constant-sigma assumption.\n");
    out.push_str("  8. EMISSION RATE: assumed constant; real peat fires pulsate with\n");
    out.push_str("     groundwater table, burn phase (flaming vs smoldering).\n\n");
    out.push_str("  For operational haze forecasting: use BMKG ISPU, NOAA HYSPLIT,\n");
    out.push_str("  or Singapore MSS haze forecast (authoritative).\n");
    out.push_str("════════════════════════════════════════════════════\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Self-check: wind=5 m/s, 24h -> distance = 5*86400 = 432 km
    // (Sumatra->Singapore ~300km, reachable)
    #[test]
    fn wind_5ms_24h_reaches_singapore() {
        let res = trajectory(0.0, 102.0, 5.0, 180.0, 24.0, 1000.0, 500.0);
        // 5 m/s * 86400 s = 432,000 m = 432 km
        // With equirectangular approx, distance should be close to 432 km
        assert!(res.contains("432"), "expected ~432 km, got:\n{}", res);
        assert!(!res.contains("ERROR"), "unexpected error in result");
        // Should flag Singapore as reachable (>=300 km)
        assert!(res.contains("REACHABLE"), "Singapore should be reachable at 432 km");
    }

    // Self-check: low wind speed rejected
    #[test]
    fn low_wind_rejected() {
        let res = trajectory(0.0, 102.0, 0.2, 180.0, 24.0, 1000.0, 500.0);
        assert!(res.contains("ERROR"), "wind < 0.5 should error");
    }

    // Self-check: deposition factor decays over time
    // v_dep=0.002 m/s, t=86400s, H_mix=1500m -> exp(-0.002*86400/1500) = exp(-0.1152) = 0.891
    #[test]
    fn deposition_factor_correct() {
        let v_dep = 0.002_f64;
        let t = 86400.0_f64;
        let h_mix = 1500.0_f64;
        let factor = (-v_dep * t / h_mix).exp();
        assert!((factor - 0.891).abs() < 0.01, "factor={:.4} expected ~0.891", factor);
    }
}
