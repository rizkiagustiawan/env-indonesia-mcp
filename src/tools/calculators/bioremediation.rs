/// Bioremediation Design — Monod Kinetics + First-Order Decay
/// Ref: Chen et al. 1992 "Transport and biodegradation of benzene/toluene"
///   Suarez & Rifai 1999 "Biodegradation rates for fuel hydrocarbons"
///   Widdowson 1999 "SEAM3D"; Rifai 1989 "Numerical techniques"

pub fn design(
    contaminant: &str,
    initial_conc_mg_l: f64,
    target_conc_mg_l: f64,
    k_first_order_day: f64,   // first-order decay rate (day⁻¹)
    soil_volume_m3: f64,
    porosity: f64,
    _bulk_density_kg_m3: f64,
) -> String {
    let mut out = String::from("=== Bioremediation Design ===\n");
    out.push_str("Ref: Chen et al. 1992; Suarez & Rifai 1999; Widdowson 1999 (SEAM3D)\n\n");

    if initial_conc_mg_l <= 0.0 || target_conc_mg_l <= 0.0 {
        return "ERROR [E102]: concentrations must be > 0.".into();
    }
    if target_conc_mg_l >= initial_conc_mg_l {
        return "ERROR: target must be < initial concentration.".into();
    }

    // Default k values from Suarez & Rifai 1999
    let (k_eff, k_max, ks, _x_biomass, reaction_type) = if k_first_order_day > 0.0 {
        (k_first_order_day, k_first_order_day * 10.0, initial_conc_mg_l * 0.5, 1.0, "user-specified")
    } else {
        match contaminant.to_lowercase().as_str() {
            "benzene" | "btx" | "bTEX" => (0.10, 1.0, 0.5, 1.0, "aerobic BTEX"),
            "toluene" => (0.15, 1.5, 0.5, 1.0, "aerobic toluene"),
            "xylene" => (0.08, 0.8, 0.5, 1.0, "aerobic xylene"),
            "naphthalene" | "pah" => (0.01, 0.1, 0.2, 0.5, "aerobic PAH"),
            "phenanthrene" => (0.005, 0.05, 0.1, 0.5, "aerobic PAH"),
            "pyrene" => (0.002, 0.02, 0.1, 0.5, "aerobic PAH"),
            "tce" | "trichloroethylene" => (0.01, 0.1, 0.5, 0.5, "reductive dechlorination"),
            "pce" | "perchloroethylene" => (0.008, 0.08, 0.5, 0.5, "reductive dechlorination"),
            "diesel" | "tph" | "total petroleum" => (0.05, 0.5, 0.5, 1.0, "aerobic petroleum"),
            "bod" | "organic" => (0.20, 2.0, 0.5, 2.0, "aerobic organic"),
            _ => (0.03, 0.3, 0.5, 0.5, "assumed generic"),
        }
    };

    out.push_str(&format!("Contaminant: {} ({})\n", contaminant, reaction_type));
    out.push_str(&format!("  C_initial: {:.1} mg/L → C_target: {:.2} mg/L\n", initial_conc_mg_l, target_conc_mg_l));
    out.push_str(&format!("  First-order k: {:.4} day⁻¹\n", k_eff));
    out.push_str(&format!("  Monod k_max: {:.3} day⁻¹, Ks: {:.2} mg/L\n\n", k_max, ks));

    // ═══ Cleanup Time (First-Order) ═══
    out.push_str("── Cleanup Time (First-Order Decay) ──\n\n");

    // C = C₀ × exp(-k × t) → t = ln(C₀/C_target) / k
    let ln_ratio = (initial_conc_mg_l / target_conc_mg_l).ln();
    let cleanup_time_days = ln_ratio / k_eff;
    let cleanup_time_years = cleanup_time_days / 365.0;

    out.push_str(&format!("  ln(C₀/C_target) = {:.4}\n", ln_ratio));
    out.push_str(&format!("  ► Cleanup time: {:.0} days ({:.1} years)\n\n", cleanup_time_days, cleanup_time_years));

    // ═══ Half-Life ═══
    let half_life = 0.693 / k_eff;
    out.push_str(&format!("  Half-life (t½): {:.1} days ({:.2} years)\n\n", half_life, half_life / 365.0));

    // ═══ Monod Kinetics Check ═══
    out.push_str("── Monod Kinetics Verification ──\n\n");

    // When C << Ks, Monod → first-order: k' = k_max × X / Ks
    let k_monod_first = k_max * _x_biomass / ks;
    let ratio = initial_conc_mg_l / ks;

    out.push_str(&format!("  C₀/Ks = {:.2}\n", ratio));
    if ratio < 0.1 {
        out.push_str("  C₀ << Ks → first-order approximation is VALID\n");
        out.push_str(&format!("  k'(from Monod) = k_max×X/Ks = {:.4} day⁻¹\n\n", k_monod_first));
    } else {
        out.push_str("  C₀ > Ks → first-order approximation MAY UNDERESTIMATE rate\n");
        out.push_str("  Monod: rate ≈ k_max at high C, first-order at low C\n");
        out.push_str("  Use first-order as conservative (worst case) estimate\n\n");
    }

    // ═══ Oxygen Demand ═══
    out.push_str("── Oxygen Demand ──\n\n");

    // Stoichiometric O₂ demand (simplified)
    // For hydrocarbons CₙH₂ₙ: CₙH₂ₙ + 1.5n O₂ → n CO₂ + n H₂O
    // O₂/C ratio ≈ 3.4 g O₂ per g hydrocarbon
    let o2_ratio: f64 = match contaminant.to_lowercase().as_str() {
        "benzene" | "toluene" | "xylene" | "btx" | "gasoline" | "diesel" | "tph" => 3.4,
        "naphthalene" | "pah" | "phenanthrene" | "pyrene" => 3.0,
        "tce" | "pce" | "chlorinated" => 0.5, // reductive — O₂ not primary electron acceptor
        "bod" | "organic" => 1.1, // BOD5/UBOD ≈ 1.1
        _ => 2.5,
    };

    let contaminant_mass_kg = soil_volume_m3 * porosity * initial_conc_mg_l * 1e-3; // mg/L → kg/m³ → kg
    let delta_c = initial_conc_mg_l - target_conc_mg_l;
    let mass_removed_kg = soil_volume_m3 * porosity * delta_c * 1e-3;
    let o2_demand_kg = mass_removed_kg * o2_ratio;
    let o2_demand_ton = o2_demand_kg / 1000.0;

    out.push_str(&format!("  Contaminant mass (dissolved): {:.2} kg\n", contaminant_mass_kg));
    out.push_str(&format!("  Mass to be removed: {:.2} kg\n", mass_removed_kg));
    out.push_str(&format!("  O₂/C ratio: {:.1} g O₂ per g contaminant\n", o2_ratio));
    out.push_str(&format!("  ► O₂ demand: {:.1} kg ({:.2} tons)\n\n", o2_demand_kg, o2_demand_ton));

    // Oxygen delivery methods
    out.push_str("  Oxygen delivery options:\n");
    out.push_str(&format!("    Air sparging: {:.0} m³ air needed (O₂ in air = 280g/m³)\n", o2_demand_kg * 1000.0 / 280.0));
    out.push_str(&format!("    H₂O₂ injection: {:.0} kg 30% H₂O₂ (contains ~{:.0} kg O₂)\n",
        o2_demand_kg / 0.14, o2_demand_kg));
    out.push_str(&format!("    ORC (oxygen release compound): {:.0} kg ORC\n\n", o2_demand_kg / 0.10));

    // ═══ Nutrient Demand (C:N:P = 100:10:1) ═══
    out.push_str("── Nutrient Demand (C:N:P = 100:10:1) ──\n\n");

    let carbon_mass = mass_removed_kg * 0.85; // ~85% carbon in hydrocarbons
    let n_demand = carbon_mass * 0.10; // 10% of C
    let p_demand = carbon_mass * 0.01; // 1% of C

    out.push_str(&format!("  Carbon mass: {:.2} kg\n", carbon_mass));
    out.push_str(&format!("  ► Nitrogen demand: {:.2} kg (as N)\n", n_demand));
    out.push_str(&format!("  ► Phosphorus demand: {:.3} kg (as P)\n\n", p_demand));

    // Fertilizer amounts
    out.push_str("  Fertilizer equivalents:\n");
    out.push_str(&format!("    Urea (46% N): {:.2} kg\n", n_demand / 0.46));
    out.push_str(&format!("    TSP (20% P): {:.3} kg\n\n", p_demand / 0.20));

    // ═══ Assessment ═══
    out.push_str("═══ BIOREMEDIATION ASSESSMENT ═══\n\n");

    if cleanup_time_years > 5.0 {
        out.push_str("  ⚠️ Long cleanup (>5 years). Consider enhanced bioremediation.\n");
        out.push_str("     Options: electron donor/acceptor injection, bioaugmentation\n");
    } else if cleanup_time_years > 1.0 {
        out.push_str("  🟡 Moderate cleanup (1-5 years). Monitor quarterly.\n");
    } else {
        out.push_str("  🟢 Good cleanup time (<1 year).\n");
    }

    if k_eff < 0.005 {
        out.push_str("  ⚠️ Low biodegradation rate (<0.005 day⁻¹). Bioaugmentation may help.\n");
    }

    // ═══ Summary ═══
    out.push_str("\n═══ SUMMARY ═══\n\n");
    out.push_str(&format!("  Contaminant: {} ({})\n", contaminant, reaction_type));
    out.push_str(&format!("  k = {:.4} day⁻¹, half-life = {:.1} days\n", k_eff, half_life));
    out.push_str(&format!("  Cleanup time: {:.1} years\n", cleanup_time_years));
    out.push_str(&format!("  O₂ demand: {:.1} kg\n", o2_demand_kg));
    out.push_str(&format!("  N demand: {:.2} kg, P demand: {:.3} kg\n", n_demand, p_demand));

    out.push_str("\n  Ref: Chen et al. 1992; Suarez & Rifai 1999; Widdowson 1999 (SEAM3D)\n");
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • First-order assumes C << Ks (low concentration regime)\n");
    out.push_str("  • No inhibitory/toxicity effects modeled\n");
    out.push_str("  • No spatial heterogeneity in biomass/moisture\n");
    out.push_str("  • For design: batch/-column tests + BIOPLUME/RT3D modeling\n");

    out
}
