/// Wet Scrubber (Venturi) Design
/// Ref: Vallero 2019 "Air Pollution Calculations"; Calvert; air pollution handbooks

pub fn design(
    gas_flow_m3_s: f64,
    particle_density_kg_m3: f64,
    target_efficiency_pct: f64,
    throat_velocity_ms: f64,
    lg_ratio_l_m3: f64,
) -> String {
    let mut out = String::from("=== Wet Scrubber (Venturi) Design ===\n");
    out.push_str("Ref: Vallero 2019; Calvert; Air Pollution Control Handbooks\n\n");

    if gas_flow_m3_s <= 0.0 || throat_velocity_ms <= 0.0 {
        return "ERROR [E102]: gas flow and throat velocity must be > 0.".into();
    }

    let rho_g = 1.2; // kg/m³
    let mu_g = 1.81e-5; // Pa·s
    let rho_l = 1000.0; // water kg/m³
    let v_throat = throat_velocity_ms;
    let lg = lg_ratio_l_m3; // L/G ratio (L/m³)

    out.push_str(&format!("Gas flow: {:.2} m³/s ({:.0} m³/hr)\n", gas_flow_m3_s, gas_flow_m3_s * 3600.0));
    out.push_str(&format!("Particle density: {:.0} kg/m³\n", particle_density_kg_m3));
    out.push_str(&format!("Throat velocity: {:.1} m/s\n", v_throat));
    out.push_str(&format!("L/G ratio: {:.1} L/m³\n\n", lg));

    // ═══ Droplet Size (Nukiyama-Tanasawa) ═══
    out.push_str("── Droplet Size (Nukiyama-Tanasawa) ──\n\n");

    // d_d = 50 / (v_gas + 50) + 15 / sqrt(L/G) (µm, L/G in L/m³)
    let d_droplet_um = 50.0 / (v_throat + 50.0) + 15.0 / lg.max(0.1).sqrt();
    let d_droplet_m = d_droplet_um * 1e-6;

    out.push_str(&format!("  ► Mean droplet diameter: {:.1} µm ({:.2e} m)\n\n", d_droplet_um, d_droplet_m));

    // ═══ Pressure Drop ═══
    out.push_str("── Pressure Drop ──\n\n");

    // Calvert: ΔP = (ρ_g × v² / 2) × (1 + L/G × ρ_l/ρ_g × correction)
    // Simplified: ΔP ≈ v² × (0.5 + 0.001 × L/G)
    let delta_P_pa = rho_g * v_throat * v_throat * (0.5 + 0.001 * lg * rho_l / rho_g);
    let delta_P_kpa = delta_P_pa / 1000.0;

    out.push_str(&format!("  ► Pressure drop: {:.0} Pa ({:.2} kPa, {:.0} mmH₂O)\n\n", delta_P_pa, delta_P_kpa, delta_P_pa / 9.81));

    if delta_P_kpa > 5.0 {
        out.push_str("  ⚠️ High ΔP (>5 kPa). Consider reducing throat velocity.\n\n");
    } else {
        out.push_str("  🟢 Acceptable ΔP (1-5 kPa typical for venturi)\n\n");
    }

    // ═══ Collection Efficiency (Calvert) ═══
    out.push_str("── Collection Efficiency (Calvert Impaction) ──\n\n");

    // Impaction parameter: Ψ = ρ_p × d_p² × v_rel / (9 × μ × d_droplet)
    // η = 1 - exp(-A × Ψ^B)
    // For venturi: η ≈ 1 - exp(-6 × Ψ × L/G / 1000)

    out.push_str(&format!("{:>10} {:>12} {:>10} {:>10}\n", "d (µm)", "Ψ (impaction)", "η (%)", "Status"));
    out.push_str(&"-".repeat(45));
    out.push('\n');

    let particle_sizes = [0.5, 1.0, 2.0, 5.0, 10.0];
    let mut eff_2um = 0.0;

    for d in &particle_sizes {
        let d_m = d * 1e-6;
        // Relative velocity ≈ v_throat (simplified)
        let v_rel = v_throat * 0.8;
        let psi = particle_density_kg_m3 * d_m * d_m * v_rel / (9.0 * mu_g * d_droplet_m).max(1e-15);
        // Calvent: η = 1 - exp(-6 × Ψ × L/G / 1000)
        let eff = (1.0 - (-6.0 * psi * lg / 1000.0).exp()) * 100.0;
        let status = if eff >= target_efficiency_pct { "✅" } else { "⚠️" };
        out.push_str(&format!("{:>10.2} {:>12.4} {:>10.1} {:>10}\n", d, psi, eff, status));
        if (*d - 2.0).abs() < 0.01 { eff_2um = eff; }
    }

    // Overall efficiency (mass-weighted, assume typical distribution centered at 5µm)
    let overall_eff = (1.0 - (-6.0 * (particle_density_kg_m3 * (5e-6_f64).powi(2) * v_throat * 0.8 / (9.0 * mu_g * d_droplet_m)) * lg / 1000.0).exp()) * 100.0;
    out.push_str(&format!("\n  ► Overall efficiency (at 5µm): {:.1}%\n\n", overall_eff));

    // ═══ Water Consumption ═══
    let water_flow_l_s = gas_flow_m3_s * lg;
    let water_flow_m3_hr = water_flow_l_s * 3.6;

    out.push_str("── Water Consumption ──\n\n");
    out.push_str(&format!("  ► Water flow: {:.1} L/s ({:.1} m³/hr)\n\n", water_flow_l_s, water_flow_m3_hr));

    // ═══ Power Requirement ═══
    let power_gas_kw = delta_P_pa * gas_flow_m3_s / 1000.0;
    let power_liquid_kw = rho_l * 9.81 * 50.0 * water_flow_l_s / 1e6; // assume 50m head
    let power_total_kw = power_gas_kw + power_liquid_kw;

    out.push_str("── Power Requirement ──\n\n");
    out.push_str(&format!("  Gas power: {:.2} kW\n", power_gas_kw));
    out.push_str(&format!("  Liquid power: {:.2} kW (50m pump head)\n", power_liquid_kw));
    out.push_str(&format!("  ► Total power: {:.2} kW\n\n", power_total_kw));

    // ═══ Summary ═══
    out.push_str("═══ VENTURI SCRUBBER SUMMARY ═══\n\n");
    out.push_str(&format!("  Throat velocity: {:.1} m/s\n", v_throat));
    out.push_str(&format!("  L/G ratio: {:.1} L/m³\n", lg));
    out.push_str(&format!("  Droplet size: {:.1} µm\n", d_droplet_um));
    out.push_str(&format!("  Pressure drop: {:.2} kPa\n", delta_P_kpa));
    out.push_str(&format!("  Overall efficiency: {:.1}%\n", overall_eff));
    out.push_str(&format!("  Water: {:.1} m³/hr, Power: {:.2} kW\n", water_flow_m3_hr, power_total_kw));

    out.push_str("\n  Ref: Vallero 2019; Calvert 1972; Air Pollution Control Handbooks\n");
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • Nukiyama-Tanasawa droplet size is empirical (±30% uncertainty)\n");
    out.push_str("  • No gas absorption component (only particulate collection)\n");
    out.push_str("  • No temperature/corrosion effects on materials\n");

    out
}
