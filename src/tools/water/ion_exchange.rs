/// Ion Exchange Design
/// Ref: Crittenden et al. 2012 (MWH "Water Treatment")

pub fn design(
    target_ion: &str,
    c_influent_mg_l: f64,
    exchange_capacity_eq_l: f64, // resin capacity (eq/L)
    flow_m3_day: f64,
    bed_volume_m3: f64,
    selectivity_coeff: f64,      // K_AB (separation factor)
    regenerant_type: &str,
) -> String {
    let mut out = String::from("=== Ion Exchange Design ===\n");
    out.push_str("Ref: Crittenden et al. 2012 (MWH)\n\n");

    if c_influent_mg_l <= 0.0 || flow_m3_day <= 0.0 || exchange_capacity_eq_l <= 0.0 {
        return "ERROR [E102]: parameters must be > 0.".into();
    }

    // Get ion properties (charge, MW)
    let (z, mw, name) = match target_ion.to_lowercase().as_str() {
        "ca2+" | "calcium" => (2.0, 40.08, "Ca²⁺"),
        "mg2+" | "magnesium" => (2.0, 24.31, "Mg²⁺"),
        "na+" | "sodium" => (1.0, 22.99, "Na⁺"),
        "k+" | "potassium" => (1.0, 39.10, "K⁺"),
        "no3-" | "nitrate" => (1.0, 62.00, "NO₃⁻"),
        "cl-" | "chloride" => (1.0, 35.45, "Cl⁻"),
        "so42-" | "sulfate" => (2.0, 96.06, "SO₄²⁻"),
        "fe3+" | "iron" => (3.0, 55.85, "Fe³⁺"),
        "mn2+" | "manganese" => (2.0, 54.94, "Mn²⁺"),
        "cr6+" | "chromate" => (2.0, 52.00, "CrO₄²⁻"),
        _ => (1.0, 50.0, "Unknown"),
    };

    // Convert influent to eq/L
    let c_influent_eq_l = c_influent_mg_l / mw * z / 1000.0; // mg/L → meq/L → eq/L

    out.push_str(&format!("Target ion: {} (z={}, MW={:.2})\n", name, z, mw));
    out.push_str(&format!("  C_influent: {:.1} mg/L ({:.4} eq/L)\n", c_influent_mg_l, c_influent_eq_l));
    out.push_str(&format!("  Resin capacity: {:.2} eq/L\n", exchange_capacity_eq_l));
    out.push_str(&format!("  Flow: {:.1} m³/day\n", flow_m3_day));
    out.push_str(&format!("  Bed volume: {:.2} m³\n", bed_volume_m3));
    out.push_str(&format!("  Selectivity K: {:.2}\n", selectivity_coeff));
    out.push_str(&format!("  Regenerant: {}\n\n", regenerant_type));

    // ═══ Total Exchange Capacity ═══
    out.push_str("── Total Exchange Capacity ──\n\n");

    let total_capacity_eq = exchange_capacity_eq_l * bed_volume_m3 * 1000.0; // eq
    out.push_str(&format!("  ► Total capacity: {:.1} eq ({:.0} kg resin)\n\n", total_capacity_eq, bed_volume_m3 * 700.0));

    // ═══ Throughput Volume (BV to breakthrough) ═══
    out.push_str("── Throughput to Breakthrough ──\n\n");

    // BV = capacity / C_influent (in eq/L)
    let bv_to_breakthrough = exchange_capacity_eq_l / c_influent_eq_l.max(1e-10) * selectivity_coeff;
    let throughput_m3 = bv_to_breakthrough * bed_volume_m3;
    let throughput_days = throughput_m3 / flow_m3_day;

    out.push_str(&format!("  BV to breakthrough: {:.0} bed volumes\n", bv_to_breakthrough));
    out.push_str(&format!("  ► Throughput: {:.1} m³ ({:.1} days of operation)\n\n", throughput_m3, throughput_days));

    // ═══ Regeneration Cycle ═══
    out.push_str("── Regeneration Cycle ──\n\n");

    let regen_time_hours = 4.0; // typical regeneration time
    let cycle_time_days = throughput_days + regen_time_hours / 24.0;
    let n_cycles_per_year = 365.0 / cycle_time_days;

    out.push_str(&format!("  Regeneration time: {:.0} hours\n", regen_time_hours));
    out.push_str(&format!("  ► Cycle time: {:.1} days (run + regen)\n", cycle_time_days));
    out.push_str(&format!("  ► Cycles per year: {:.0}\n\n", n_cycles_per_year));

    // ═══ Regenerant Consumption ═══
    out.push_str("── Regenerant Consumption ──\n\n");

    // Regenerant dose: typically 2-5 eq regenerant per eq exchanged
    let regen_dose_eq = 3.0; // 3× stoichiometric
    let regenerant_eq_per_cycle = total_capacity_eq * regen_dose_eq;

    let (regen_name, _regen_mw, regen_conc, regen_kg_per_cycle) = match regenerant_type.to_lowercase().as_str() {
        "nacl" | "sodium chloride" | "brine" => {
            let kg = regenerant_eq_per_cycle * 58.44; // NaCl MW
            ("NaCl brine", 58.44, 0.10, kg)
        },
        "hcl" | "hydrochloric acid" => {
            let kg = regenerant_eq_per_cycle * 36.46;
            ("HCl", 36.46, 0.05, kg)
        },
        "naoh" | "caustic" | "sodium hydroxide" => {
            let kg = regenerant_eq_per_cycle * 40.00;
            ("NaOH", 40.00, 0.04, kg)
        },
        _ => {
            let kg = regenerant_eq_per_cycle * 58.44;
            ("NaCl (default)", 58.44, 0.10, kg)
        },
    };

    let regen_solution_l = regen_kg_per_cycle / (regen_conc * 1000.0_f64).max(1.0); // liters of solution
    let annual_regen_kg = regen_kg_per_cycle * n_cycles_per_year;

    out.push_str(&format!("  Regenerant: {} ({:.0}% solution)\n", regen_name, regen_conc * 100.0));
    out.push_str(&format!("  Dose: {}× stoichiometric\n", regen_dose_eq));
    out.push_str(&format!("  ► Per cycle: {:.1} kg {} ({:.0} L solution)\n", regen_kg_per_cycle, regen_name, regen_solution_l));
    out.push_str(&format!("  ► Annual: {:.0} kg/year\n\n", annual_regen_kg));

    // ═══ Leakage ═══
    out.push_str("── Leakage Estimate ──\n\n");

    // C_leak ≈ C_influent / (1 + K × q_max / C_influent)
    let q_max = exchange_capacity_eq_l;
    let c_leak_eq = c_influent_eq_l / (1.0 + selectivity_coeff * q_max / c_influent_eq_l.max(1e-10));
    let c_leak_mg_l = c_leak_eq * mw / z * 1000.0;

    out.push_str(&format!("  ► Leakage: {:.4} mg/L ({:.4} eq/L)\n\n", c_leak_mg_l, c_leak_eq));

    // ═══ Bed Geometry ═══
    let bed_diameter = ((bed_volume_m3 / 3.0 * 4.0 / std::f64::consts::PI).max(0.1)).sqrt();
    let bed_height = bed_volume_m3 / (std::f64::consts::PI * bed_diameter * bed_diameter / 4.0).max(0.01);
    let ebct_min = bed_volume_m3 / (flow_m3_day / 24.0 / 60.0).max(1e-6);

    out.push_str("── Bed Geometry ──\n\n");
    out.push_str(&format!("  Bed diameter: {:.2} m, height: {:.2} m\n", bed_diameter, bed_height));
    out.push_str(&format!("  EBCT: {:.1} min\n", ebct_min));

    if ebct_min < 1.5 {
        out.push_str("  ⚠️ EBCT <1.5 min — insufficient contact. Increase bed volume.\n\n");
    } else {
        out.push_str("  🟢 EBCT adequate (1.5-10 min typical)\n\n");
    }

    // ═══ Summary ═══
    out.push_str("═══ ION EXCHANGE SUMMARY ═══\n\n");
    out.push_str(&format!("  Ion: {} (C₀={:.1} mg/L)\n", name, c_influent_mg_l));
    out.push_str(&format!("  Capacity: {:.1} eq total\n", total_capacity_eq));
    out.push_str(&format!("  Throughput: {:.0} BV ({:.1} days)\n", bv_to_breakthrough, throughput_days));
    out.push_str(&format!("  Regeneration: every {:.1} days ({:.0} cycles/yr)\n", cycle_time_days, n_cycles_per_year));
    out.push_str(&format!("  Regenerant: {:.0} kg/year\n", annual_regen_kg));
    out.push_str(&format!("  Leakage: {:.3} mg/L\n", c_leak_mg_l));

    out.push_str("\n  Ref: Crittenden et al. 2012 (MWH)\n");
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • Selectivity simplified (real: multi-ion competition)\n");
    out.push_str("  • No kinetic limitations (film/particle diffusion)\n");
    out.push_str("  • Regeneration efficiency assumed 100% (real: 80-95%)\n");
    out.push_str("  • For design: column test + pilot study recommended\n");

    out
}
