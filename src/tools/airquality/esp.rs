/// Electrostatic Precipitator (ESP) Design
/// Ref: Vallero 2019 "Air Pollution Calculations"; Deutsch-Anderson equation

pub fn design(
    gas_flow_m3_s: f64,
    particle_density_kg_m3: f64,
    target_efficiency_pct: f64,
    field_strength_kv_cm: f64,
    particle_diameter_um: f64,
) -> String {
    let mut out = String::from("=== Electrostatic Precipitator (ESP) Design ===\n");
    out.push_str("Ref: Vallero 2019; Deutsch-Anderson equation; White 1963\n\n");

    if gas_flow_m3_s <= 0.0 || field_strength_kv_cm <= 0.0 {
        return "ERROR [E102]: gas flow and field strength must be > 0.".into();
    }

    let mu_g = 1.81e-5; // Pa·s
    let eps0 = 8.854e-12; // permittivity of free space (F/m)
    let E_v_m = field_strength_kv_cm * 1e5; // V/m (1 kV/cm = 1e5 V/m)
    let d_p = particle_diameter_um * 1e-6; // particle diameter in m

    out.push_str(&format!(
        "Gas flow: {:.2} m3/s ({:.0} m3/hr)\n",
        gas_flow_m3_s,
        gas_flow_m3_s * 3600.0
    ));
    out.push_str(&format!(
        "Particle diameter: {:.2} µm\n",
        particle_diameter_um
    ));
    out.push_str(&format!(
        "Field strength: {:.1} kV/cm ({:.2e} V/m)\n\n",
        field_strength_kv_cm, E_v_m
    ));

    // === Migration Velocity (Deutsch-Anderson) ===
    out.push_str("-- Migration Velocity --\n\n");

    // w = (2 * eps0 * E2 * d_p) / (12 * mu) — for dielectric particles
    // For conductive particles: w = eps0 * E2 * d_p / (6 * mu)
    let w_migration = 2.0 * eps0 * E_v_m * E_v_m * d_p / (12.0 * mu_g);
    let w_migration_cm_s = w_migration * 100.0;

    out.push_str(&format!("  Deutsch-Anderson: w = 2eps0E2d / 12mu\n"));
    out.push_str(&format!(
        "  >Migration velocity: {:.4} m/s ({:.3} cm/s)\n\n",
        w_migration, w_migration_cm_s
    ));

    // Typical migration velocities for reference
    out.push_str("  Reference migration velocities (typical):\n");
    out.push_str("    Fly ash: 0.08-0.15 m/s\n");
    out.push_str("    Cement dust: 0.06-0.12 m/s\n");
    out.push_str("    Sulfuric acid mist: 0.20-0.30 m/s\n\n");

    // === Required Plate Area (Deutsch Equation) ===
    out.push_str("-- Required Plate Area (Deutsch Equation) --\n\n");

    // eta = 1 - exp(-w * A / Q)
    // A = -Q * ln(1-eta) / w
    let eta = target_efficiency_pct / 100.0;
    let a_required = -gas_flow_m3_s * (1.0 - eta).ln() / w_migration.max(1e-15);

    out.push_str(&format!(
        "  Target efficiency: {:.1}% (eta={:.3})\n",
        target_efficiency_pct, eta
    ));
    out.push_str(&format!("  ln(1-eta) = {:.4}\n", (1.0 - eta).ln()));
    out.push_str(&format!("  >Required plate area: {:.0} m2\n\n", a_required));

    // === Specific Collection Area (SCA) ===
    let sca = a_required / gas_flow_m3_s;
    out.push_str("-- Specific Collection Area (SCA) --\n\n");
    out.push_str(&format!("  >SCA = A/Q = {:.1} m2 per m3/s\n", sca));

    if sca < 100.0 {
        out.push_str("  ! Low SCA (<100). May not meet target. Increase plate area.\n\n");
    } else if sca > 500.0 {
        out.push_str(
            "  ! Very high SCA (>500). Consider multi-field ESP or check migration velocity.\n\n",
        );
    } else {
        out.push_str("  OK SCA in typical range (100-500 m2 per m3/s)\n\n");
    }

    // === Physical Dimensions ===
    out.push_str("-- Physical Dimensions --\n\n");

    // Assume: plate height = 10m, plate spacing = 0.3m, gas velocity = 1.5 m/s
    let plate_height = 10.0;
    let plate_spacing = 0.3;
    let gas_velocity = 1.5;

    let total_plate_length = a_required / (2.0 * plate_height); // both sides of plates
    let n_passages = (gas_flow_m3_s / (gas_velocity * plate_height * plate_spacing)).ceil() as u32;
    let n_plates = n_passages + 1;
    let field_length = total_plate_length / n_passages as f64;

    out.push_str(&format!("  Plate height: {:.0} m\n", plate_height));
    out.push_str(&format!("  Plate spacing: {:.2} m\n", plate_spacing));
    out.push_str(&format!("  Gas velocity: {:.1} m/s\n", gas_velocity));
    out.push_str(&format!("  >Number of passages: {}\n", n_passages));
    out.push_str(&format!("  >Number of plates: {}\n", n_plates));
    out.push_str(&format!(
        "  >Field length: {:.1} m\n\n",
        field_length.max(1.0)
    ));

    // === Number of Fields (Series) ===
    let n_fields = if target_efficiency_pct > 99.5 {
        4
    } else if target_efficiency_pct > 99.0 {
        3
    } else if target_efficiency_pct > 95.0 {
        2
    } else {
        1
    };

    out.push_str(&format!(
        "  >Recommended fields (in series): {}\n",
        n_fields
    ));
    out.push_str(&format!(
        "  Total ESP length: {:.1} m\n\n",
        field_length.max(1.0) * n_fields as f64
    ));

    // === Corona Power ===
    out.push_str("-- Corona Power --\n\n");

    // P_corona = V * I, typical corona current density = 0.1-0.5 mA/m2
    let corona_current_density = 0.3e-3; // A/m2
    let corona_current = corona_current_density * a_required;
    let corona_voltage = field_strength_kv_cm * 1000.0 * plate_spacing * 100.0; // kV
    let corona_power_kw = corona_voltage * corona_current / 1000.0;

    out.push_str(&format!("  Corona current density: 0.3 mA/m2\n"));
    out.push_str(&format!(
        "  Total corona current: {:.2} A\n",
        corona_current
    ));
    out.push_str(&format!("  Voltage: {:.0} kV\n", corona_voltage));
    out.push_str(&format!("  >Corona power: {:.1} kW\n\n", corona_power_kw));

    // Specific corona power (kW per 1000 m3/s)
    let specific_power = corona_power_kw / (gas_flow_m3_s / 1000.0).max(0.001);
    out.push_str(&format!(
        "  Specific power: {:.1} kW/(1000 m3/s)\n\n",
        specific_power
    ));

    // === Summary ===
    out.push_str("=== ESP DESIGN SUMMARY ===\n\n");
    out.push_str(&format!(
        "  Migration velocity: {:.3} cm/s\n",
        w_migration_cm_s
    ));
    out.push_str(&format!("  Plate area: {:.0} m2\n", a_required));
    out.push_str(&format!("  SCA: {:.1} m2 per m3/s\n", sca));
    out.push_str(&format!(
        "  Plates: {} ({} fields, {:.1}m each)\n",
        n_plates,
        n_fields,
        field_length.max(1.0)
    ));
    out.push_str(&format!("  Corona power: {:.1} kW\n", corona_power_kw));

    // Verify efficiency
    let achieved_eff = (1.0 - (-w_migration * a_required / gas_flow_m3_s).exp()) * 100.0;
    out.push_str(&format!("  Achieved efficiency: {:.2}%\n", achieved_eff));

    out.push_str("\n  Ref: Vallero 2019; Deutsch 1922; White 1963\n");
    out.push_str("\n-- Limitations (honest) --\n");
    out.push_str("  • Migration velocity depends on particle resistivity (back corona effect)\n");
    out.push_str("  • Deutsch equation assumes uniform field, no re-entrainment\n");
    out.push_str("  • No temperature/pressure correction\n");
    out.push_str("  • For high-resistivity dust: pulse energization or conditioning needed\n");

    out
}
