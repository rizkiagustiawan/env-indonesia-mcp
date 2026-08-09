/// Sistem Registri Unit Karbon — Permen LH 10/2026
/// Carbon trading registry: issuance, transfer, retirement of carbon units
/// Ref: Permen LH 10/2026 (Sistem Registri Unit Karbon); Second NDC 2025
pub fn assess(project_type: &str, emission_reduction_ton_co2e: f64, vintage_year: u32, buyer: &str, seller: &str, price_rp_per_ton: f64) -> String {
    let mut out = String::from("=== Sistem Registri Unit Karbon (Permen LH 10/2026) ===\n");
    out.push_str("Ref: Permen LH 10/2026; Second NDC 2025; Permen LH 7/2026\n\n");

    let total_value = emission_reduction_ton_co2e * price_rp_per_ton;

    out.push_str(&format!("Project type: {}\n", project_type));
    out.push_str(&format!("Emission reduction: {:.0} ton CO2e (vintage {}))\n", emission_reduction_ton_co2e, vintage_year));
    out.push_str(&format!("Seller: {}, Buyer: {}\n", seller, buyer));
    out.push_str(&format!("Price: Rp {:.0}/ton CO2e\n\n", price_rp_per_ton));

    out.push_str("═══ REGISTRATION ═══\n");
    out.push_str("  1. Project registration in SRN-PPI (Sistem Registri Nasional PPI)\n");
    out.push_str("  2. Validation by accredited third party (VVB)\n");
    out.push_str("  3. Verification of emission reductions\n");
    out.push_str("  4. Issuance of carbon units in Sistem Registri Unit Karbon\n\n");

    out.push_str("═══ CARBON UNIT TYPES ═══\n");
    out.push_str("  - PASTI (Pengurangan Emisi Sumber Tidak Bergerak): energy/industry\n");
    out.push_str("  - PASTI-Bergerak: transport\n");
    out.push_str("  - FOLU Net Sink: forestry\n");
    out.push_str("  - Teknologi (CCS, renewable)\n\n");

    out.push_str("═══ TRANSACTION ═══\n");
    out.push_str(&format!("  Volume: {:.0} ton CO2e\n", emission_reduction_ton_co2e));
    out.push_str(&format!("  Price: Rp {:.0}/ton\n", price_rp_per_ton));
    out.push_str(&format!("  >> Total value: Rp {:.0} ({:.2} M USD))\n\n", total_value, total_value / 16_000.0 / 1_000_000.0));

    let project_cat = match project_type.to_lowercase().as_str() {
        s if s.contains("energy") || s.contains("listrik") || s.contains("pltu") => "Energy",
        s if s.contains("forest") || s.contains("folu") || s.contains("hutan") => "FOLU",
        s if s.contains("mangrove") || s.contains("blue_carbon") => "Karbon Biru",
        s if s.contains("transport") => "Transport",
        s if s.contains("waste") || s.contains("limbah") => "Waste",
        _ => "Other",
    };
    out.push_str(&format!("  Category: {}\n", project_cat));

    out.push_str("\n═══ COMPLIANCE vs VOLUNTARY ═══\n");
    out.push_str("  Compliance: BPU (Beban Pelepasan Unit) — cap-and-trade\n");
    out.push_str("  Voluntary: carbon credits for corporate/net zero\n\n");

    out.push_str("═══ NDC ALIGNMENT ═══\n");
    out.push_str("  Permen LH 7/2026: NDC sektor baru (kelautan, karbon biru, migas)\n");
    out.push_str("  Second NDC 2025: absolute target 2035\n");
    out.push_str(&format!("  Contribution: {:.4}% of 118 MTon FOLU target\n\n", emission_reduction_ton_co2e / 118_000_000.0 * 100.0));

    out.push_str("═══ PEMANTAUAN (MRV) ═══\n");
    out.push_str("  VVB (Validation/Verification Body): accredited third party\n");
    out.push_str("  MRV: annual monitoring, reporting, verification\n");
    out.push_str("  Registry: Sistem Registri Unit Karbon (Permen 10/2026)\n");

    out.push_str("\n  Ref: Permen LH 10/2026; Permen LH 7/2026; Second NDC 2025\n");
    out
}
