/// Pump-and-Treat Groundwater Remediation Design
/// Ref: Suthersan et al. 2016 "Remediation Engineering: Design Concepts"
///   Sharma & Reddy 2004 "Geoenvironmental Engineering"
///   Wagner 1995 (simulation-optimization); US EPA 1989

pub fn design(
    hk_m_s: f64,           // horizontal hydraulic conductivity
    aquifer_thickness_m: f64,
    hydraulic_gradient: f64, // dh/dx (dimensionless)
    pumping_rate_m3_day: f64,
    porosity: f64,
    contaminant: &str,
    initial_conc_ug_l: f64,
    target_conc_ug_l: f64,
    cleanup_time_years: f64,
) -> String {
    let mut out = String::from("=== Pump-and-Treat Remediation Design ===\n");
    out.push_str("Ref: Suthersan et al. 2016; Sharma & Reddy 2004; US EPA 1989\n\n");

    if hk_m_s <= 0.0 || aquifer_thickness_m <= 0.0 || pumping_rate_m3_day <= 0.0 {
        return "ERROR [E102]: K, thickness, pumping rate must be > 0.".into();
    }

    let T = hk_m_s * aquifer_thickness_m; // transmissivity (m²/s)
    let Q = pumping_rate_m3_day / 86400.0; // m³/s
    let b = aquifer_thickness_m;
    let i = hydraulic_gradient;

    out.push_str(&format!("Aquifer: K={:.2e} m/s, b={:.1} m, T={:.2e} m²/s\n", hk_m_s, b, T));
    out.push_str(&format!("Gradient: {:.4}\n", i));
    out.push_str(&format!("Pumping: Q={:.1} m³/day ({:.4} m³/s)\n", pumping_rate_m3_day, Q));
    out.push_str(&format!("Porosity: {:.2}\n", porosity));
    out.push_str(&format!("Contaminant: {}, C₀={:.0} µg/L → target={:.0} µg/L\n\n", contaminant, initial_conc_ug_l, target_conc_ug_l));

    // ═══ Capture Zone Analysis ═══
    out.push_str("── Capture Zone Analysis ──\n\n");

    // Natural groundwater flux (Darcy velocity)
    let q_darcy = hk_m_s * i; // m/s
    let q_natural_m3_day = q_darcy * b * 1.0 * 86400.0; // per meter width
    out.push_str(&format!("  Natural Darcy flux: {:.2e} m/s ({:.2} m³/day per m width)\n", q_darcy, q_natural_m3_day));

    // Capture zone width (steady-state, single well)
    // W = 2 * Q / (K * b * i) for uniform flow
    let capture_width = if i > 0.0 {
        2.0 * Q / (q_darcy * b).max(1e-15)
    } else {
        out.push_str("  ⚠️ Gradient = 0, capture zone is circular\n");
        2.0 * std::f64::consts::PI * (Q / (std::f64::consts::PI * b * porosity)).sqrt()
    };
    out.push_str(&format!("  ► Capture zone width: {:.1} m\n", capture_width));

    // Stagnation point distance
    let x_stag = Q / (std::f64::consts::PI * b * q_darcy).max(1e-15);
    out.push_str(&format!("  Stagnation point: {:.1} m downgradient\n\n", x_stag));

    // ═══ Drawdown Analysis (Theis) ═══
    out.push_str("── Drawdown Analysis (Theis) ──\n\n");

    let S = porosity * 0.1; // storativity ≈ Sy × 0.1 (confined approximation)
    let t_sec = cleanup_time_years * 365.0 * 86400.0;

    // Drawdown at pumping well (r = well radius, assume 0.15 m)
    let r_well = 0.15;
    let u_well = (r_well * r_well * S) / (4.0 * T * t_sec).max(1e-15);
    // Cooper-Jacob approximation: W(u) ≈ -0.5772 - ln(u) for u < 0.01
    let w_u = if u_well < 0.01 {
        -0.5772 - u_well.ln()
    } else {
        -0.5772 - u_well.ln() // approximation
    };
    let s_well = (Q / (4.0 * std::f64::consts::PI * T).max(1e-15)) * w_u;

    out.push_str(&format!("  Storativity S: {:.2e}\n", S));
    out.push_str(&format!("  Duration: {:.1} years ({:.0} days)\n", cleanup_time_years, cleanup_time_years * 365.0));
    out.push_str(&format!("  Well radius: {:.2} m\n", r_well));
    out.push_str(&format!("  u = {:.6}\n", u_well));
    out.push_str(&format!("  W(u) = {:.4}\n", w_u));
    out.push_str(&format!("  ► Drawdown at well: {:.2} m\n\n", s_well));

    // ═══ Pore Volume Analysis ═══
    out.push_str("── Pore Volume & Cleanup Time ──\n\n");

    // Plume volume (assume rectangular: capture_width × plume_length × b)
    let plume_length = capture_width; // assume square plume
    let plume_volume_m3 = capture_width * plume_length * b;
    let pore_volume = plume_volume_m3 * porosity;
    let pv_per_day = pumping_rate_m3_day / pore_volume.max(1e-6);

    out.push_str(&format!("  Plume volume: {:.0} m³ ({}×{:.1}×{:.1})\n", plume_volume_m3, capture_width as i32, plume_length, b));
    out.push_str(&format!("  Pore volume: {:.0} m³\n", pore_volume));
    out.push_str(&format!("  Pore volumes pumped/day: {:.3}\n", pv_per_day));

    // Number of pore volumes needed (depends on desorption)
    // For typical sandy aquifer: 10-100 PV for cleanup
    let n_pv_needed = 20.0; // conservative
    let cleanup_time_days = n_pv_needed / pv_per_day.max(1e-6);
    let cleanup_time_calc_years = cleanup_time_days / 365.0;

    out.push_str(&format!("\n  Pore volumes needed (conservative): {:.0}\n", n_pv_needed));
    out.push_str(&format!("  ► Estimated cleanup time: {:.1} years ({:.0} days)\n\n", cleanup_time_calc_years, cleanup_time_days));

    // ═══ Mass Removal Rate ═══
    out.push_str("── Mass Removal Rate ──\n\n");

    let c_kg_m3 = initial_conc_ug_l * 1e-6; // µg/L → kg/m³
    let mass_rate_kg_day = pumping_rate_m3_day * c_kg_m3;
    let total_mass_kg = plume_volume_m3 * porosity * c_kg_m3; // dissolved mass
    let removal_time_days = total_mass_kg / mass_rate_kg_day.max(1e-15);

    out.push_str(&format!("  Initial dissolved mass: {:.2} kg\n", total_mass_kg));
    out.push_str(&format!("  Removal rate: {:.4} kg/day ({:.1} g/day)\n", mass_rate_kg_day, mass_rate_kg_day * 1000.0));
    out.push_str(&format!("  ► Time to remove dissolved mass: {:.0} days ({:.1} years)\n\n", removal_time_days, removal_time_days / 365.0));

    // ═══ Assessment ═══
    out.push_str("═══ DESIGN ASSESSMENT ═══\n\n");

    if capture_width < 10.0 {
        out.push_str("  ⚠️ Capture zone narrow (<10m). Consider multiple wells.\n");
    } else if capture_width > 200.0 {
        out.push_str("  🟢 Wide capture zone. Single well may be sufficient.\n");
    } else {
        out.push_str("  🟡 Moderate capture zone. Verify with monitoring.\n");
    }

    if s_well > 10.0 {
        out.push_str("  ⚠️ Drawdown >10m. Pump may lose efficiency. Reduce Q or use multiple wells.\n");
    } else if s_well > 3.0 {
        out.push_str("  🟡 Moderate drawdown. Monitor aquifer response.\n");
    } else {
        out.push_str("  🟢 Drawdown acceptable.\n");
    }

    if cleanup_time_calc_years > 30.0 {
        out.push_str("  ⚠️ Cleanup time >30 years. Consider alternative remediation (PRB, bioremediation).\n");
    } else if cleanup_time_calc_years > 10.0 {
        out.push_str("  🟡 Long cleanup (10-30 yr). Monitored natural attenuation may supplement.\n");
    } else {
        out.push_str("  🟢 Reasonable cleanup time.\n");
    }

    // Limitations
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • Assumes homogeneous aquifer, steady-state flow\n");
    out.push_str("  • Desorption kinetics not modeled (simplified to PV count)\n");
    out.push_str("  • NAPL presence significantly extends cleanup time\n");
    out.push_str("  • For production: use MODFLOW + MT3D for full simulation\n");

    out
}
