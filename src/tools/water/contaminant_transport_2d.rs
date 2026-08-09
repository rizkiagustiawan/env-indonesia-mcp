/// Contaminant Transport 2D — Domenico Analytical Solution
/// Ref: Domenico 1987; Devlin 2012; Park & Zhan 2001
/// C = (C0/4) * exp(-lambda*t) * exp(x*lambda/(2*vx)) * erfc_term * [erf_y1 - erf_y2] * [erf_z1 - erf_z2]
pub fn assess(
    distance_x_m: f64,
    source_width_y_m: f64,
    source_depth_z_m: f64,
    velocity_m_day: f64,
    dispersion_x_m2_day: f64,
    dispersion_y_m2_day: f64,
    dispersion_z_m2_day: f64,
    time_days: f64,
    retardation_factor: f64,
    decay_rate_day: f64,
    initial_conc_mg_l: f64,
) -> String {
    let mut out = String::from("=== Contaminant Transport 2D (Domenico) ===\n");
    out.push_str("Ref: Domenico 1987; Devlin 2012; Park & Zhan 2001\n\n");

    if distance_x_m <= 0.0 || velocity_m_day <= 0.0 {
        return "ERROR [E102]: distance and velocity must be > 0.".into();
    }

    let vx = velocity_m_day / retardation_factor;
    let Dx = dispersion_x_m2_day / retardation_factor;
    let Dy = dispersion_y_m2_day / retardation_factor;
    let Dz = dispersion_z_m2_day / retardation_factor;
    let t = time_days;
    let lambda = decay_rate_day;

    out.push_str(&format!("Source: {:.0}m (x) x {:.0}m (y) x {:.0}m (z)\n", distance_x_m, source_width_y_m, source_depth_z_m));
    out.push_str(&format!("Velocity: {:.2} m/day (retarded: {:.2})\n", velocity_m_day, vx));
    out.push_str(&format!("Dispersion: Dx={:.2e}, Dy={:.2e}, Dz={:.2e} m2/day\n", Dx, Dy, Dz));
    out.push_str(&format!("Time: {:.0} days ({:.1} years)\n\n", t, t/365.0));

    // Domenico 2D solution (planar source, finite width Y, full depth):
    // C/C0 = 0.5 * exp(-lambda*t) * erfc((x - vx*t)/(2*sqrt(Dx*t))) 
    //        * [erf((y + Y/2)/(2*sqrt(Dy*x/vx))) - erf((y - Y/2)/(2*sqrt(Dy*x/vx)))]
    // At y=0 (centerline): erf(Y/(4*sqrt(Dy*x/vx))) - erf(-Y/(4*sqrt(Dy*x/vx))) = 2*erf(Y/(4*sqrt(Dy*x/vx)))

    let arg_x = (distance_x_m - vx * t) / (2.0 * (Dx * t).sqrt().max(1e-15));
    let erfc_x = erfc_approx(arg_x);

    // Transverse y (centerline, y=0)
    let dy_eff = (Dy * distance_x_m / vx.max(1e-15)).sqrt();
    let erf_y = erf_approx(source_width_y_m / (4.0 * dy_eff.max(1e-15)));
    let y_factor = 2.0 * erf_y; // centerline

    // Vertical z (full depth assumed, z_factor = 1 for 2D)
    let z_factor = 1.0;

    // Decay
    let decay_factor = if lambda > 0.0 { (-lambda * t).exp() } else { 1.0 };

    let c_ratio = 0.5 * decay_factor * erfc_x * y_factor * z_factor;
    let conc = initial_conc_mg_l * c_ratio;

    out.push_str("-- Domenico 2D Solution --\n\n");
    out.push_str(&format!("  arg_x = {:.4}, erfc = {:.6}\n", arg_x, erfc_x));
    out.push_str(&format!("  dy_eff = {:.4} m, erf_y = {:.6}, y_factor = {:.6}\n", dy_eff, erf_y, y_factor));
    out.push_str(&format!("  Decay factor = {:.6}\n", decay_factor));
    out.push_str(&format!("  >> C/C0 = {:.6}\n", c_ratio));
    out.push_str(&format!("  >> Concentration: {:.4} mg/L\n\n", conc));

    // Plume dimensions
    let plume_length = vx * t;
    let plume_width = 4.0 * dy_eff; // 4*sigma approx
    out.push_str(&format!("  Plume length: {:.1} m\n", plume_length));
    out.push_str(&format!("  Plume width (centerline): +/-{:.1} m\n", plume_width/2.0));
    out.push_str(&format!("  Travel time: {:.1} years\n\n", distance_x_m/vx/365.0));

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
    out.push_str("  Lokasi: Source area + plume centerline + plume fringes (transverse spread)\n");

    out.push_str("\n─── PELAPORAN & IZIN ───\n");
    out.push_str("  PP 22/2021 Annex VI (air tanah) + Pasal 124-131\n");
    out.push_str("  PP 101/2014 (B3-contaminated land)\n");
    out.push_str("  Amdalnet + OSS; Permen LH 6/2026 (sanksi)\n");

    out.push_str("\n  Ref: Domenico 1987; Devlin 2012; PP 22/2021 Annex VI\n");
    out
}

fn erfc_approx(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    if x >= 0.0 { poly } else { 2.0 - poly }
}

fn erf_approx(x: f64) -> f64 {
    1.0 - erfc_approx(x)
}
