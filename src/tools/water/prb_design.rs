/// Permeable Reactive Barrier (PRB) Design
/// Ref: Tratnyek et al. 2003 "PRBs of iron and other zero-valent metals"
///   Seyyedalipour et al. 2026 "PRB comprehensive review"
///   Kalmakhanova et al. 2026; Birke et al. 2003

pub fn design(
    contaminant: &str,
    c_inflow_ug_l: f64,
    c_target_ug_l: f64,
    k_first_order_hr: f64,   // first-order degradation rate (hr⁻¹)
    gw_velocity_m_day: f64,  // groundwater velocity (seepage)
    porosity: f64,
    barrier_width_m: f64,    // perpendicular to flow
    barrier_depth_m: f64,
    bulk_density_kg_m3: f64, // ZVI bulk density in barrier
) -> String {
    let mut out = String::from("=== Permeable Reactive Barrier (PRB) Design ===\n");
    out.push_str("Ref: Tratnyek et al. 2003; Seyyedalipour et al. 2026; Kalmakhanova et al. 2026\n\n");

    if c_target_ug_l >= c_inflow_ug_l {
        return "ERROR: target concentration must be < inflow concentration.".into();
    }
    if k_first_order_hr <= 0.0 {
        out.push_str("⚠️ k=0 — no degradation. Using default for ZVI/TCE: 0.05 hr⁻¹\n");
    }

    // Default k values for common contaminants (if k=0 or unknown)
    let k_eff = if k_first_order_hr > 0.0 {
        k_first_order_hr
    } else {
        match contaminant.to_lowercase().as_str() {
            "tce" | "trichloroethylene" => 0.05,
            "pce" | "perchloroethylene" => 0.03,
            "cis-12-dce" | "dce" => 0.01,
            "vc" | "vinyl chloride" => 0.02,
            "cr6" | "chromium" | "cr(vi)" => 0.1,
            "as" | "arsenic" => 0.08,
            "u" | "uranium" => 0.15,
            "nitrate" | "no3" => 0.02,
            _ => 0.05, // default ZVI
        }
    };

    let v = gw_velocity_m_day / 24.0; // m/hr (seepage velocity = Darcy velocity / porosity)
    let n = porosity;

    out.push_str(&format!("Contaminant: {}\n", contaminant));
    out.push_str(&format!("  C_inflow: {:.0} µg/L → C_target: {:.0} µg/L\n", c_inflow_ug_l, c_target_ug_l));
    out.push_str(&format!("  First-order k: {:.4} hr⁻¹ ({:.3} day⁻¹)\n", k_eff, k_eff * 24.0));
    out.push_str(&format!("  GW velocity: {:.2} m/day ({:.4} m/hr)\n", gw_velocity_m_day, v));
    out.push_str(&format!("  Porosity: {:.2}\n\n", n));

    // ═══ Required Barrier Thickness ═══
    out.push_str("── Required Barrier Thickness ──\n\n");

    // C/C₀ = exp(-k × τ), where τ = L×n/v (residence time)
    // L = -v × ln(C_target/C₀) / (k × n)
    let ln_ratio = (c_target_ug_l / c_inflow_ug_l).ln(); // negative
    let L_required = -v * ln_ratio / (k_eff * n).max(1e-15);
    let L_design = L_required * 1.5; // safety factor 1.5

    out.push_str(&format!("  ln(C_target/C₀) = {:.4}\n", ln_ratio));
    out.push_str(&format!("  Minimum thickness (L_min): {:.2} m\n", L_required));
    out.push_str(&format!("  Design thickness (×1.5 safety): {:.2} m\n\n", L_design));

    // ═══ Residence Time ═══
    let tau_min = L_required * n / v.max(1e-15);
    let tau_design = L_design * n / v.max(1e-15);

    out.push_str("── Residence Time ──\n\n");
    out.push_str(&format!("  τ_min = L×n/v = {:.2} hr ({:.1} days)\n", tau_min, tau_min / 24.0));
    out.push_str(&format!("  τ_design = {:.2} hr ({:.1} days)\n\n", tau_design, tau_design / 24.0));

    // ═══ Outlet Concentration (at design thickness) ═══
    let c_outlet = c_inflow_ug_l * (-k_eff * tau_design).exp();
    let removal_eff = (1.0 - c_outlet / c_inflow_ug_l) * 100.0;

    out.push_str("── Performance at Design Thickness ──\n\n");
    out.push_str(&format!("  C_outlet = C₀ × exp(-k×τ) = {:.2} µg/L\n", c_outlet));
    out.push_str(&format!("  Removal efficiency: {:.1}%\n\n", removal_eff));

    if c_outlet <= c_target_ug_l {
        out.push_str("  ✅ Design meets target concentration\n\n");
    } else {
        out.push_str("  ⚠️ Design does NOT meet target. Increase thickness.\n\n");
    }

    // ═══ Mass of Reactive Media (ZVI) ═══
    out.push_str("── Reactive Media Requirements ──\n\n");

    let barrier_volume = L_design * barrier_width_m * barrier_depth_m;
    let zvi_volume = barrier_volume * (1.0 - n); // solid fraction
    let zvi_mass_kg = zvi_volume * bulk_density_kg_m3;
    let zvi_mass_ton = zvi_mass_kg / 1000.0;

    out.push_str(&format!("  Barrier dimensions: {:.2}m × {:.1}m × {:.1}m\n", L_design, barrier_width_m, barrier_depth_m));
    out.push_str(&format!("  Barrier volume: {:.1} m³\n", barrier_volume));
    out.push_str(&format!("  ZVI volume (solid): {:.1} m³\n", zvi_volume));
    out.push_str(&format!("  ► ZVI mass required: {:.0} kg ({:.1} tons)\n\n", zvi_mass_kg, zvi_mass_ton));

    // ═══ Longevity Estimate ═══
    out.push_str("── Longevity Estimate ──\n\n");

    // ZVI consumption rate (simplified): assume 5% ZVI consumed per year
    let consumption_rate = 0.05; // 5%/year
    let longevity_years = 1.0 / consumption_rate * (zvi_mass_kg / (barrier_volume * 500.0).max(1.0)).min(1.0);
    let longevity = longevity_years.max(10.0).min(50.0);

    out.push_str(&format!("  ZVI consumption rate: ~{:.0}%/year (typical)\n", consumption_rate * 100.0));
    out.push_str(&format!("  ► Estimated longevity: {:.0} years (before replacement)\n\n", longevity));

    if longevity < 15.0 {
        out.push_str("  ⚠️ Short longevity. Consider larger barrier or replenishment plan.\n");
    } else if longevity > 30.0 {
        out.push_str("  🟢 Good longevity (>30 years expected).\n");
    } else {
        out.push_str("  🟡 Moderate longevity. Monitor performance annually.\n");
    }

    // ═══ Summary ═══
    out.push_str("\n═══ PRB DESIGN SUMMARY ═══\n\n");
    out.push_str(&format!("  Contaminant: {} (k={:.4} hr⁻¹)\n", contaminant, k_eff));
    out.push_str(&format!("  Design thickness: {:.2} m (safety factor 1.5)\n", L_design));
    out.push_str(&format!("  Residence time: {:.1} days\n", tau_design / 24.0));
    out.push_str(&format!("  Outlet concentration: {:.2} µg/L (target: {:.0})\n", c_outlet, c_target_ug_l));
    out.push_str(&format!("  Removal efficiency: {:.1}%\n", removal_eff));
    out.push_str(&format!("  ZVI mass: {:.1} tons\n", zvi_mass_ton));
    out.push_str(&format!("  Estimated longevity: {:.0} years\n", longevity));

    out.push_str("\n  Ref: Tratnyek et al. 2003; Seyyedalipour et al. 2026; Kalmakhanova et al. 2026\n");
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • First-order kinetics (real ZVI: surface-limited, pH-dependent)\n");
    out.push_str("  • No geochemical precipitation/fouling modeled\n");
    out.push_str("  • Heterogeneous aquifer not considered\n");
    out.push_str("  • For design: validate with column tests + MODFLOW\n");

    out
}
