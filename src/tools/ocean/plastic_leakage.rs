/// Plastic Leakage Estimate — Jambeck et al. 2015 method
/// Ref: Jambeck et al. 2015 (Science); Nursyahputra 2026; Anuar 2025; Adnan 2025

pub fn estimate(
    population: u64,
    waste_generation_kg_cap_day: f64,
    plastic_fraction_pct: f64,
    mismanaged_waste_pct: f64,
    coastal_population_pct: f64,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Plastic Leakage Estimate (Jambeck Method)\n");
    out.push_str("Ref: Jambeck et al. 2015 (Science); Anuar 2025; Nursyahputra 2026\n\n");

    let waste_kg_day = population as f64 * waste_generation_kg_cap_day;
    let waste_tons_yr = waste_kg_day * 365.0 / 1000.0;

    let plastic_tons_yr = waste_tons_yr * (plastic_fraction_pct / 100.0);
    let mismanaged_plastic_tons = plastic_tons_yr * (mismanaged_waste_pct / 100.0);
    let coastal_leakage_pct = 15.0;
    let plastic_to_sea_tons = mismanaged_plastic_tons * (coastal_leakage_pct / 100.0) * (coastal_population_pct / 100.0);

    out.push_str("INPUTS:\n");
    out.push_str(&format!("  Population: {}\n", population));
    out.push_str(&format!("  Waste generation: {:.2} kg/cap/day\n", waste_generation_kg_cap_day));
    out.push_str(&format!("  Plastic fraction: {:.1}%\n", plastic_fraction_pct));
    out.push_str(&format!("  Mismanaged waste: {:.1}%\n", mismanaged_waste_pct));
    out.push_str(&format!("  Coastal population: {:.1}%\n\n", coastal_population_pct));

    out.push_str("CALCULATION (Jambeck et al. 2015):\n");
    out.push_str(&format!("  Total waste: {:.0} tons/year\n", waste_tons_yr));
    out.push_str(&format!("  Plastic waste: {:.0} tons/year\n", plastic_tons_yr));
    out.push_str(&format!("  Mismanaged plastic: {:.0} tons/year\n", mismanaged_plastic_tons));
    out.push_str(&format!("  Coastal leakage rate: {}%\n", coastal_leakage_pct));
    out.push_str(&format!("  → Plastic to sea: {:.0} tons/year\n\n", plastic_to_sea_tons));

    out.push_str("CONTEXT:\n");
    out.push_str(&format!("  Indonesia total (Jambeck 2015): 0.48-1.29 million tons/year\n"));
    out.push_str(&format!("  Global rank: #2 (after China)\n"));
    out.push_str(&format!("  National target: 70% reduction by 2025 (KLHK)\n"));
    out.push_str(&format!("  Actual achieved: ~41.68% by 2024 (Aisyah 2024)\n\n"));

    if plastic_to_sea_tons > 0.0 {
        let cars_equivalent = plastic_to_sea_tons / 4.6;
        out.push_str(&format!("  Equivalent to {:.0} cars/year CO2\n", cars_equivalent));
    }

    out.push_str("\nMITIGATION:\n");
    out.push_str("  1. Waste management improvement (reduce mismanaged %)\n");
    out.push_str("  2. River barriers (trash booms, interceptors)\n");
    out.push_str("  3. Ban single-use plastic (already in many Indonesian cities)\n");
    out.push_str("  4. Circular economy: recycling, EPR (Extended Producer Responsibility)\n");
    out.push_str("  5. Mangrove restoration (natural plastic trap)\n\n");

    out.push_str("LIMITATION:\n");
    out.push_str("  - Jambeck 2015 is simplified (does not model river transport)\n");
    out.push_str("  - Coastal leakage rate (15%) is global default\n");
    out.push_str("  - Does not account for river-specific transport efficiency\n");
    out.push_str("  - Population data should be coastal (within 50km of coast)\n");
    out.push_str("  - Waste generation rate varies by income level\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}
