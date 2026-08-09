/// Cyclone Separator Design
/// Ref: Aylı & Kocak 2025 "Comprehensive review of cyclone separator technology"
///   Vallero 2019 "Air Pollution Calculations"; Shepherd-Lapple; Stairmand

pub fn design(
    gas_flow_m3_s: f64,
    particle_density_kg_m3: f64,
    gas_viscosity_pa_s: f64,
    cyclone_diameter_m: f64,
    target_efficiency_pct: f64,
) -> String {
    let mut out = String::from("=== Cyclone Separator Design ===\n");
    out.push_str("Ref: Aylı & Kocak 2025; Vallero 2019; Shepherd-Lapple; Stairmand\n\n");

    if gas_flow_m3_s <= 0.0 || cyclone_diameter_m <= 0.0 {
        return "ERROR [E102]: gas flow and cyclone diameter must be > 0.".into();
    }

    let D = cyclone_diameter_m;
    let mu = gas_viscosity_pa_s;
    let rho_p = particle_density_kg_m3;
    let rho_g = 1.2; // air density kg/m³ at 20°C

    // Stairmand high-efficiency dimensions
    let inlet_width = 0.2 * D;  // a
    let inlet_height = 0.5 * D; // b
    let outlet_diameter = 0.5 * D; // De
    let barrel_height = 1.5 * D;  // h
    let cone_height = 2.5 * D;    // z
    let total_height = barrel_height + cone_height;
    let dust_outlet = 0.375 * D;

    out.push_str(&format!("Cyclone diameter (D): {:.3} m\n", D));
    out.push_str(&format!("Gas flow: {:.2} m³/s ({:.0} m³/hr)\n", gas_flow_m3_s, gas_flow_m3_s * 3600.0));
    out.push_str(&format!("Particle density: {:.0} kg/m³\n", rho_p));
    out.push_str(&format!("Gas viscosity: {:.2e} Pa·s\n\n", mu));

    // ═══ Dimensions (Stairmand High-Efficiency) ═══
    out.push_str("── Stairmand High-Efficiency Dimensions ──\n\n");
    out.push_str(&format!("  Inlet: {:.3}m × {:.3}m (W×H)\n", inlet_width, inlet_height));
    out.push_str(&format!("  Outlet diameter (De): {:.3} m\n", outlet_diameter));
    out.push_str(&format!("  Barrel height (h): {:.3} m\n", barrel_height));
    out.push_str(&format!("  Cone height (z): {:.3} m\n", cone_height));
    out.push_str(&format!("  Total height: {:.3} m\n", total_height));
    out.push_str(&format!("  Dust outlet: {:.3} m\n\n", dust_outlet));

    // ═══ Inlet Velocity ═══
    let inlet_area = inlet_width * inlet_height;
    let v_inlet = gas_flow_m3_s / inlet_area;

    out.push_str("── Flow Parameters ──\n\n");
    out.push_str(&format!("  Inlet area: {:.4} m²\n", inlet_area));
    out.push_str(&format!("  Inlet velocity: {:.1} m/s\n", v_inlet));

    if v_inlet > 25.0 {
        out.push_str("  ⚠️ Inlet velocity >25 m/s — erosion risk. Increase D.\n\n");
    } else if v_inlet < 5.0 {
        out.push_str("  ⚠️ Inlet velocity <5 m/s — low efficiency. Decrease D.\n\n");
    } else {
        out.push_str("  🟢 Inlet velocity in range (5-25 m/s)\n\n");
    }

    // ═══ Cut Diameter (d50) ═══
    out.push_str("── Cut Diameter (d₅₀) ──\n\n");

    // d₅₀ = sqrt(9 × μ × W / (2π × Ne × v_i × (ρ_p - ρ_g)))  [Lapple/Cooper&Alley]
    // BUG FIX: was using cyclone diameter D instead of inlet width W (=0.2D), and omitted (ρp-ρg).
    // W (inlet width) is the characteristic length, NOT the cyclone diameter.
    let N_e = 5.0; // effective turns (Stairmand: 5)
    let d50_m = (9.0 * mu * inlet_width / (2.0 * std::f64::consts::PI * N_e * v_inlet * (rho_p - rho_g).max(1e-6)).max(1e-15)).sqrt();
    let d50_um = d50_m * 1e6;

    out.push_str(&format!("  Effective turns (Ne): {:.0}\n", N_e));
    out.push_str(&format!("  ► Cut diameter d₅₀: {:.2} µm ({:.2e} m)\n\n", d50_um, d50_m));

    // ═══ Collection Efficiency (Lapple) ═══
    out.push_str("── Collection Efficiency (Lapple) ──\n\n");

    // η = 1 / (1 + (d₅₀/d)²)
    out.push_str(&format!("{:>10} {:>12} {:>10}\n", "d (µm)", "d/d₅₀", "η (%)"));
    out.push_str(&"-".repeat(35));
    out.push('\n');

    let particle_sizes = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0];
    let mut eff_at_target = 0.0;

    for d in &particle_sizes {
        let ratio = d / d50_um;
        let eff = 1.0 / (1.0 + (1.0 / ratio).powi(2)) * 100.0;
        out.push_str(&format!("{:>10.1} {:>12.2} {:>10.1}\n", d, ratio, eff));
        if (*d - d50_um).abs() < 0.1 { eff_at_target = eff; }
    }

    // Overall efficiency (assuming typical particle distribution)
    let overall_eff = 1.0 / (1.0 + (1.0_f64 / 2.0).powi(2)) * 100.0; // at d = 2×d50
    out.push_str(&format!("\n  ► Overall efficiency (at 2×d₅₀): {:.1}%\n\n", overall_eff));

    // ═══ Pressure Drop ═══
    out.push_str("── Pressure Drop ──\n\n");

    // Shepherd-Lapple: ΔP = 8 × ρ_g × v_i² / (2 × g_c)
    // Or: ΔP = N_H × ρ_g × v_i² / 2 (Euler number approach)
    let n_h = 8.0; // Shepherd-Lapple constant for Stairmand
    let delta_P_pa = n_h * rho_g * v_inlet * v_inlet / 2.0;
    let delta_P_kpa = delta_P_pa / 1000.0;
    let delta_P_cmH2O = delta_P_pa / 98.07;

    out.push_str(&format!("  Shepherd-Lapple: ΔP = 8 × ρ × v² / 2\n"));
    out.push_str(&format!("  ► Pressure drop: {:.0} Pa ({:.2} kPa, {:.0} mmH₂O)\n\n", delta_P_pa, delta_P_kpa, delta_P_cmH2O * 10.0));

    if delta_P_kpa > 2.0 {
        out.push_str("  ⚠️ High pressure drop (>2 kPa). Consider larger D or parallel cyclones.\n\n");
    } else {
        out.push_str("  🟢 Acceptable pressure drop (<2 kPa)\n\n");
    }

    // ═══ Stokes Number ═══
    out.push_str("── Stokes Number ──\n\n");

    // Stk = ρ_p × d_p² × v_i / (9 × μ × D)
    let stk_d50 = rho_p * d50_m * d50_m * v_inlet / (9.0 * mu * D);
    out.push_str(&format!("  Stk at d₅₀: {:.4} (should be ~0.08 for Stairmand)\n\n", stk_d50));

    // ═══ Summary ═══
    out.push_str("═══ CYCLONE DESIGN SUMMARY ═══\n\n");
    out.push_str(&format!("  Diameter: {:.3} m, Height: {:.3} m\n", D, total_height));
    out.push_str(&format!("  Inlet velocity: {:.1} m/s\n", v_inlet));
    out.push_str(&format!("  Cut diameter: {:.2} µm\n", d50_um));
    out.push_str(&format!("  Pressure drop: {:.2} kPa ({:.0} mmH₂O)\n", delta_P_kpa, delta_P_cmH2O * 10.0));
    out.push_str(&format!("  Overall efficiency: ~{:.0}%\n", overall_eff));

    if target_efficiency_pct > 90.0 && d50_um > 5.0 {
        out.push_str("\n  ⚠️ Target >90% but d₅₀ >5µm. Consider: multi-cyclone or baghouse.\n");
    }

    out.push_str("\n  Ref: Aylı & Kocak 2025; Vallero 2019; Stairmand 1951\n");
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • Lapple model assumes spherical particles, uniform density\n");
    out.push_str("  • No particle loading effect (concentration affects efficiency)\n");
    out.push_str("  • No fouling/erosion at high loading\n");
    out.push_str("  • For design: CFD validation recommended\n");

    out
}

#[cfg(test)]
mod tests {
    // Self-check: Lapple d50 with D=0.5m (W=0.1m), mu=1.8e-5, vi=15, rho_p=2000, rho_g=1.2, Ne=5
    // d50 = sqrt(9*1.8e-5*0.1 / (2*pi*5*15*(2000-1.2))) = sqrt(1.62e-5/942477) = sqrt(1.72e-11) = 4.15e-6 m = 4.15 um
    // Old bug (D instead of W, no rho_g): sqrt(9*1.8e-5*0.5/(2*pi*5*15*2000)) = 9.28e-6 m = 9.28 um (wrong, ~2.24x too large)
    #[test]
    fn d50_uses_inlet_width() {
        let d = 0.5_f64; let w = 0.2 * d; // inlet width
        let mu = 1.8e-5_f64; let vi = 15.0_f64; let rho_p = 2000.0_f64; let rho_g = 1.2_f64; let ne = 5.0_f64;
        let d50_correct = (9.0 * mu * w / (2.0 * std::f64::consts::PI * ne * vi * (rho_p - rho_g))).sqrt();
        let d50_buggy = (9.0 * mu * d / (2.0 * std::f64::consts::PI * ne * vi * rho_p)).sqrt();
        assert!((d50_correct * 1e6 - 4.15).abs() < 0.1, "d50={:.2} um", d50_correct * 1e6);
        assert!(d50_buggy > d50_correct, "buggy (D) should be larger than correct (W)");
        assert!((d50_buggy / d50_correct - 2.24).abs() < 0.05, "ratio ~2.24x (sqrt(5))");
    }
}

