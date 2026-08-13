/// Electrostatic Precipitator (ESP) Design
/// Ref: Vallero 2019 "Air Pollution Calculations"; Deutsch-Anderson equation; White 1963
/// Enhanced: Conductive vs dielectric particles + resistivity (back corona) + size distribution

pub fn design(
    gas_flow_m3_s: f64,
    particle_density_kg_m3: f64,
    target_efficiency_pct: f64,
    field_strength_kv_cm: f64,
    particle_diameter_um: f64,
    particle_type: &str,         // "dielectric" or "conductive"
    resistivity_ohm_cm: f64,     // particle resistivity (ohm·cm)
) -> String {
    let mut out = String::from("=== Electrostatic Precipitator (ESP) Design ===\n");
    out.push_str("Ref: Vallero 2019; Deutsch-Anderson; White 1963\n\n");

    if gas_flow_m3_s <= 0.0 || field_strength_kv_cm <= 0.0 {
        return "ERROR [E102]: gas flow and field strength must be > 0.".into();
    }

    let mu_g = 1.81e-5; // Pa·s
    let eps0 = 8.854e-12; // F/m
    let E_v_m = field_strength_kv_cm * 1e5; // V/m
    let d_p = particle_diameter_um * 1e-6; // m

    out.push_str(&format!("Gas flow: {:.2} m3/s ({:.0} m3/hr)\n", gas_flow_m3_s, gas_flow_m3_s * 3600.0));
    out.push_str(&format!("Particle: {:.2} um, density {:.0} kg/m3\n", particle_diameter_um, particle_density_kg_m3));
    out.push_str(&format!("Particle type: {}\n", particle_type));
    out.push_str(&format!("Resistivity: {:.2e} ohm·cm\n", resistivity_ohm_cm));
    out.push_str(&format!("Field strength: {:.1} kV/cm ({:.2e} V/m)\n\n", field_strength_kv_cm, E_v_m));

    // ═══ Migration Velocity (particle-type dependent) ═══
    out.push_str("-- Migration Velocity --\n\n");

    // Dielectric: w = eps0*E^2*d / (6*mu)
    // Conductive: w = 2*eps0*E^2*d / (6*mu)  (2x faster — field charging)
    let (w_migration, type_factor) = if particle_type.to_lowercase().contains("conduc") {
        let w = 2.0 * eps0 * E_v_m * E_v_m * d_p / (6.0 * mu_g);
        (w, 2.0) // 2x factor vs dielectric
    } else {
        let w = eps0 * E_v_m * E_v_m * d_p / (6.0 * mu_g);
        (w, 1.0) // standard dielectric
    };
    let w_cm_s = w_migration * 100.0;

    out.push_str(&format!("  Deutsch-Anderson ({}): factor = {}x\n", particle_type, type_factor));
    out.push_str(if particle_type.to_lowercase().contains("conduc") {
        "  Conductive: w = eps0*E^2*d / (6*mu) [field distortion, 2x faster]\n"
    } else {
        "  Dielectric: w = 2*eps0*E^2*d / (12*mu) [standard]\n"
    });
    out.push_str(&format!("  >> Migration velocity: {:.4} m/s ({:.3} cm/s)\n\n", w_migration, w_cm_s));

    // ═══ Back Corona Check (resistivity) ═══
    out.push_str("-- Resistivity / Back Corona Check --\n\n");

    // Resistivity ranges (White 1963):
    // < 1e7 ohm·cm: low — normal operation
    // 1e7 - 1e10: moderate — some effect
    // > 1e10: high — BACK CORONA risk (reduces efficiency 30-50%)
    // > 1e11: very high — severe back corona, pulse energization needed

    let (corona_status, efficiency_derate) = if resistivity_ohm_cm < 1e7 {
        ("[OK] Low resistivity — normal operation", 1.0)
    } else if resistivity_ohm_cm < 1e10 {
        ("[OK] Moderate resistivity — minor effect", 0.9)
    } else if resistivity_ohm_cm < 1e11 {
        ("[WARN] High resistivity — back corona risk, derate 50%", 0.5)
    } else {
        ("[CRITICAL] Very high resistivity — severe back corona. Use pulse energization or conditioning.", 0.3)
    };

    out.push_str(&format!("  Resistivity: {:.2e} ohm·cm\n", resistivity_ohm_cm));
    out.push_str(&format!("  {}\n", corona_status));
    out.push_str(&format!("  Efficiency derate factor: {:.2}\n\n", efficiency_derate));

    // ═══ Required Plate Area (Deutsch with derate) ═══
    out.push_str("-- Required Plate Area (Deutsch Equation) --\n\n");

    let eta = target_efficiency_pct / 100.0;
    let w_effective = w_migration * efficiency_derate;
    let a_required = -gas_flow_m3_s * (1.0 - eta).ln() / w_effective.max(1e-15);

    out.push_str(&format!("  Target efficiency: {:.1}%\n", target_efficiency_pct));
    out.push_str(&format!("  w_effective = w x derate = {:.4} x {:.2} = {:.4} m/s\n", w_migration, efficiency_derate, w_effective));
    out.push_str(&format!("  >> Required plate area: {:.0} m2\n\n", a_required));

    // ═══ SCA ═══
    let sca = a_required / gas_flow_m3_s;
    out.push_str("-- Specific Collection Area (SCA) --\n\n");
    out.push_str(&format!("  >> SCA = A/Q = {:.1} m2/(m3/s)\n", sca));

    if sca < 100.0 {
        out.push_str("  [WARN] Low SCA. May not meet target.\n\n");
    } else if sca > 500.0 {
        out.push_str("  [WARN] Very high SCA. Check migration velocity or resistivity.\n\n");
    } else {
        out.push_str("  [OK] SCA in typical range (100-500)\n\n");
    }

    // ═══ Physical Dimensions ═══
    out.push_str("-- Physical Dimensions --\n\n");

    let plate_height = 10.0;
    let plate_spacing = 0.3;
    let gas_velocity = 1.5;

    let total_plate_length = a_required / (2.0 * plate_height);
    let n_passages = (gas_flow_m3_s / (gas_velocity * plate_height * plate_spacing)).ceil() as u32;
    let n_plates = n_passages + 1;
    let field_length = (total_plate_length / n_passages as f64).max(1.0);

    out.push_str(&format!("  Plates: {} ({} passages, {:.1}m field length)\n", n_plates, n_passages, field_length));

    // Number of fields
    let n_fields = if target_efficiency_pct > 99.5 { 4 }
        else if target_efficiency_pct > 99.0 { 3 }
        else if target_efficiency_pct > 95.0 { 2 }
        else { 1 };

    out.push_str(&format!("  Fields (series): {} (total {:.1}m)\n\n", n_fields, field_length * n_fields as f64));

    // ═══ Corona Power ═══
    out.push_str("-- Corona Power --\n\n");

    let corona_current_density = 0.3e-3; // A/m2
    let corona_current = corona_current_density * a_required;
    let corona_voltage = field_strength_kv_cm * 1000.0 * plate_spacing * 100.0; // kV
    let corona_power_kw = corona_voltage * corona_current / 1000.0;

    out.push_str(&format!("  >> Corona power: {:.1} kW ({:.1} kW per 1000 m3/s)\n\n", corona_power_kw, corona_power_kw / (gas_flow_m3_s / 1000.0).max(0.001)));

    // ═══ Integrated Efficiency with Size Distribution ═══
    out.push_str("-- Size-Integrated Efficiency --\n\n");

    // Assume log-normal distribution: MMD = particle_diameter_um, GSD = 2.0
    let mmd = particle_diameter_um;
    let gsd: f64 = 2.0;

    // Compute efficiency at multiple sizes and integrate
    let sizes = [0.5, 1.0, 2.0, 5.0, 10.0, 20.0];
    let mut weighted_eff = 0.0;
    let mut total_weight = 0.0;

    out.push_str(&format!("{:>8} {:>10} {:>10} {:>10}\n", "d (um)", "w (cm/s)", "eta (%)", "weight"));
    out.push_str(&"-".repeat(42));
    out.push('\n');

    for &d in &sizes {
        let d_m = d * 1e-6;
        let w_d = if particle_type.to_lowercase().contains("conduc") {
            2.0 * eps0 * E_v_m * E_v_m * d_m / (6.0 * mu_g) * efficiency_derate
        } else {
            eps0 * E_v_m * E_v_m * d_m / (6.0 * mu_g) * efficiency_derate
        };
        let eta_d = 1.0 - (-w_d * a_required / gas_flow_m3_s).exp();
        // Log-normal weight (simplified — mass fraction at each size)
        let ln_d = d.ln();
        let ln_mmd = mmd.ln();
        let sigma = gsd.ln();
        let weight = (-(ln_d - ln_mmd).powi(2) / (2.0 * sigma * sigma)).exp() / (sigma * (2.0 * std::f64::consts::PI).sqrt());
        weighted_eff += eta_d * weight;
        total_weight += weight;
        out.push_str(&format!("{:>8.1} {:>10.4} {:>10.1} {:>10.4}\n", d, w_d * 100.0, eta_d * 100.0, weight));
    }

    let integrated_eff = (weighted_eff / total_weight.max(1e-10)) * 100.0;
    out.push_str(&format!("\n  >> Size-integrated efficiency: {:.1}%\n\n", integrated_eff));

    // ═══ Summary ═══
    out.push_str("=== ESP DESIGN SUMMARY ===\n\n");
    out.push_str(&format!("  Particle: {} ({}), resistivity: {:.2e} ohm·cm\n", particle_type, particle_diameter_um, resistivity_ohm_cm));
    out.push_str(&format!("  Migration velocity: {:.4} cm/s (derated: {:.4})\n", w_cm_s, w_effective * 100.0));
    out.push_str(&format!("  Plate area: {:.0} m2, SCA: {:.1}\n", a_required, sca));
    out.push_str(&format!("  Plates: {} ({} fields), Corona: {:.1} kW\n", n_plates, n_fields, corona_power_kw));
    out.push_str(&format!("  Integrated efficiency: {:.1}% (target: {:.1}%)\n", integrated_eff, target_efficiency_pct));

    out.push_str("\n  Ref: Vallero 2019; Deutsch 1922; White 1963\n");
    out.push_str("\n-- Limitations (honest) --\n");
    out.push_str("  • Migration velocity simplified (real: depends on particle shape, charge)\n");
    out.push_str("  • Back corona derate is empirical (real: complex field interaction)\n");
    out.push_str("  • Size distribution assumes log-normal (real: may be bimodal)\n");
    out.push_str("  • For design: pilot ESP test + EPA Method 5 sizing\n");

    out
}

#[cfg(test)]
mod tests {
    use super::design;

    #[test]
    fn conductive_faster_than_dielectric() {
        // dielectric w = eps0*E^2*d/(6mu) ≈ 0.0147 m/s; conductive = 2x ≈ 0.0293 m/s
        let diel = design(10.0, 2000.0, 99.0, 3.0, 2.0, "dielectric", 1e6);
        let cond = design(10.0, 2000.0, 99.0, 3.0, 2.0, "conductive", 1e6);
        assert!(diel.contains("Migration velocity: 0.0147"), "dielectric w wrong:\n{diel}");
        assert!(cond.contains("Migration velocity: 0.0294"), "conductive w should be 2x dielectric:\n{cond}");
    }
}
