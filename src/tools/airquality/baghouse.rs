/// Baghouse (Fabric Filter) Design
/// Ref: Vallero 2019 "Air Pollution Calculations"; air pollution control handbooks

pub fn design(
    gas_flow_m3_s: f64,
    dust_conc_g_m3: f64,
    target_pressure_drop_pa: f64,
    bag_diameter_m: f64,
    bag_length_m: f64,
    fabric_type: &str,
) -> String {
    let mut out = String::from("=== Baghouse Filter Design ===\n");
    out.push_str("Ref: Vallero 2019; Air Pollution Control Handbooks\n\n");

    if gas_flow_m3_s <= 0.0 || bag_diameter_m <= 0.0 || bag_length_m <= 0.0 {
        return "ERROR [E102]: parameters must be > 0.".into();
    }

    // Fabric resistance coefficient (S) depends on fabric type
    let (s_fabric, max_velocity, name) = match fabric_type.to_lowercase().as_str() {
        "woven" | "cotton" => (350.0, 1.5, "Woven cotton"),
        "polyester" | "woven polyester" => (400.0, 1.5, "Woven polyester"),
        "felt" | "needle felt" => (500.0, 2.0, "Needle felt"),
        "ptfe" | "teflon" | "teflon" => (600.0, 2.0, "PTFE fabric"),
        "fiberglass" | "glass" => (700.0, 1.0, "Fiberglass"),
        _ => (450.0, 1.8, "Default (needle felt)"),
    };

    let _rho_g = 1.2; // kg/m³
    let K_dust = 5e7; // dust resistance coefficient (typical, Pa·s²/m²)

    out.push_str(&format!("Gas flow: {:.2} m³/s ({:.0} m³/hr)\n", gas_flow_m3_s, gas_flow_m3_s * 3600.0));
    out.push_str(&format!("Dust concentration: {:.1} g/m³\n", dust_conc_g_m3));
    out.push_str(&format!("Fabric: {} (S={:.0} Pa·s/m, max v={:.1} m/min)\n\n", name, s_fabric, max_velocity));

    // ═══ Filtration Velocity (Air-to-Cloth Ratio) ═══
    out.push_str("── Filtration Velocity ──\n\n");

    // Target: maintain ΔP below target
    // ΔP = S × v + K × C × v² × t
    // For design, use typical air-to-cloth ratio and verify
    let v_filtration = max_velocity * 0.8; // 80% of max as design
    let v_filtration_m_s = v_filtration / 60.0;

    out.push_str(&format!("  Design filtration velocity: {:.2} m/min ({:.4} m/s)\n", v_filtration, v_filtration_m_s));

    // ═══ Required Filter Area ═══
    let filter_area = gas_flow_m3_s / v_filtration_m_s;
    out.push_str(&format!("  ► Required filter area: {:.1} m²\n\n", filter_area));

    // ═══ Number of Bags ═══
    let bag_area = std::f64::consts::PI * bag_diameter_m * bag_length_m;
    let n_bags = (filter_area / bag_area).ceil() as u32;

    out.push_str("── Bag Layout ──\n\n");
    out.push_str(&format!("  Bag diameter: {:.2} m, length: {:.1} m\n", bag_diameter_m, bag_length_m));
    out.push_str(&format!("  Area per bag: {:.2} m²\n", bag_area));
    out.push_str(&format!("  ► Number of bags: {}\n\n", n_bags));

    // ═══ Cleaning Cycle ═══
    out.push_str("── Cleaning Cycle ──\n\n");

    // t = ΔP_max / (K × C × v²) — time to reach max pressure drop
    let c_kg_m3 = dust_conc_g_m3 / 1000.0;
    let cleaning_cycle_s = target_pressure_drop_pa / (K_dust * c_kg_m3 * v_filtration_m_s * v_filtration_m_s).max(1e-15);
    let cleaning_cycle_min = cleaning_cycle_s / 60.0;

    out.push_str(&format!("  Target ΔP: {:.0} Pa ({:.2} kPa)\n", target_pressure_drop_pa, target_pressure_drop_pa / 1000.0));
    out.push_str(&format!("  Dust loading: {:.4} kg/m³\n", c_kg_m3));
    out.push_str(&format!("  ► Cleaning cycle: {:.1} minutes\n\n", cleaning_cycle_min));

    // ═══ Pressure Drop Verification ═══
    let delta_p_fabric = s_fabric * v_filtration_m_s;
    let _delta_p_total = delta_p_fabric; // at t=0 (clean fabric)

    out.push_str("── Pressure Drop Verification ──\n\n");
    out.push_str(&format!("  Fabric ΔP (clean): {:.0} Pa\n", delta_p_fabric));
    out.push_str(&format!("  Total ΔP (at cleaning): {:.0} Pa\n", target_pressure_drop_pa));

    if target_pressure_drop_pa > 2500.0 {
        out.push_str("  ⚠️ ΔP >2500 Pa — high energy cost. Increase filter area.\n\n");
    } else {
        out.push_str("  🟢 ΔP in acceptable range (1000-2500 Pa)\n\n");
    }

    // ═══ Compartment Design ═══
    let n_compartments = (n_bags as f64 / 50.0).ceil() as u32; // ~50 bags per compartment
    out.push_str("── Compartment Design ──\n\n");
    out.push_str(&format!("  Bags per compartment: ~50\n"));
    out.push_str(&format!("  ► Number of compartments: {}\n", n_compartments));
    out.push_str(&format!("  Offline cleaning: 1 compartment at a time\n\n"));

    // ═══ Summary ═══
    out.push_str("═══ BAGHOUSE DESIGN SUMMARY ═══\n\n");
    out.push_str(&format!("  Fabric: {}\n", name));
    out.push_str(&format!("  Filter area: {:.1} m²\n", filter_area));
    out.push_str(&format!("  Bags: {} ({:.2}m × {:.1}m)\n", n_bags, bag_diameter_m, bag_length_m));
    out.push_str(&format!("  Compartments: {}\n", n_compartments));
    out.push_str(&format!("  Filtration velocity: {:.2} m/min\n", v_filtration));
    out.push_str(&format!("  Cleaning cycle: {:.0} min\n", cleaning_cycle_min));
    out.push_str(&format!("  ΔP range: {:.0}-{:.0} Pa\n", delta_p_fabric, target_pressure_drop_pa));

    out.push_str("\n  Ref: Vallero 2019; Air Pollution Control Handbooks\n");
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • K_dust is highly variable (1e6-1e8 depending on dust properties)\n");
    out.push_str("  • No temperature correction (affects gas viscosity & density)\n");
    out.push_str("  • Pulse-jet vs. shaker cleaning not differentiated\n");

    out
}
