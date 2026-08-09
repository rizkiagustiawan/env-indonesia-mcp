/// NDC & MRV Tracker — Second NDC (Oct 2025) + Permen LH 7/2026
/// Indonesia NDC absolute targets:
///   2030 peak: 1,345,707 (LCCP_L) to 1,491,474 (LCCP_H) Gg CO2e
///   2035: 1,257,717 (LCCP_L) to 1,488,866 (LCCP_H) Gg CO2e
/// FOLU Net Sink 2030: -118 MTon CO2e absorbed; 2035: -206 MTon
/// Sektor baru wajib NDC (Permen 7/2026): Kelautan, Perikanan, Karbon Biru, Migas
/// Ref: Second NDC 2025 (UNFCCC); Permen LH 7/2026; Enhanced NDC 2022; FOLU Net Sink 2030
pub fn assess(
    current_emissions_gg_co2e: f64,
    sector: &str,
    year: u32,
    has_mrv: bool,
    ndc_scenario: &str, // "LCCP_L" or "LCCP_H" or "CM1" or "CM2"
) -> String {
    let mut out = String::from("=== NDC & MRV Tracker (Second NDC 2025 + Permen LH 7/2026) ===\n");
    out.push_str("Ref: Second NDC Oct 2025 (UNFCCC); Permen LH 7/2026; Enhanced NDC 2022\n\n");

    // ─── NDC Targets (Second NDC 2025) ───
    let bau_2030 = 2_869_000.0; // Gg CO2e (BAU 2030)

    // Absolute targets
    let (target_2030, target_2035, label) = match ndc_scenario.to_uppercase().as_str() {
        "LCCP_L" => (1_345_707.0, 1_257_717.0, "LCCP_L (Low growth, Paris-compatible)"),
        "LCCP_H" => (1_491_474.0, 1_488_866.0, "LCCP_H (High growth, Paris-compatible)"),
        "CM1" => (1_954_000.0, 0.0, "CM1 (Unconditional 31.89%, Enhanced NDC)"),
        "CM2" => (1_629_000.0, 0.0, "CM2 (Conditional 43.20%, Enhanced NDC)"),
        _ => (1_345_707.0, 1_257_717.0, "LCCP_L (default)"),
    };

    out.push_str(&format!("Tahun: {}\n", year));
    out.push_str(&format!("Sektor: {}\n", sector));
    out.push_str(&format!("Emisi saat ini: {:.0} Gg CO2e\n", current_emissions_gg_co2e as u64));
    out.push_str(&format!("Skenario NDC: {}\n", ndc_scenario));
    out.push_str(&format!("MRV aktif: {}\n\n", if has_mrv { "Ya" } else { "Tidak" }));

    // ─── NDC Target Comparison ───
    out.push_str("═══ TARGET NDC INDONESIA ═══\n\n");
    out.push_str("Second NDC (Oct 2025) — Absolute Targets:\n");
    out.push_str(&format!("  2030 (peak): {:.0} Gg CO2e ({:.2} GtCO2e)\n", target_2030 as u64, target_2030 / 1e6));
    if target_2035 > 0.0 {
        out.push_str(&format!("  2035 (decline): {:.0} Gg CO2e ({:.2} GtCO2e)\n", target_2035 as u64, target_2035 / 1e6));
    }
    out.push_str(&format!("  BAU 2030: {:.0} Gg CO2e\n\n", bau_2030 as u64));

    // Enhanced NDC (2022) comparison
    out.push_str("Enhanced NDC (2022):\n");
    out.push_str("  CM1 (unconditional): 31.89% = 915 MTon reduction from BAU 2,869\n");
    out.push_str("  CM2 (conditional): 43.20% = 1,240 MTon reduction from BAU 2,869\n\n");

    // FOLU Net Sink 2030
    out.push_str("FOLU Net Sink 2030:\n");
    out.push_str("  2030: -118 MTon CO2e (penyerapan)\n");
    out.push_str("  2035: -206 MTon CO2e (penyerapan)\n");
    out.push_str("  Target: FOLU menjadi carbon sink (penyerap) by 2030\n\n");

    // ─── Sektor NDC (Permen LH 7/2026) ───
    out.push_str("═══ SEKTOR WAJIB NDC (Permen LH 7/2026) ═══\n");
    let sector_lower = sector.to_lowercase();
    let sector_wajib = match sector_lower.as_str() {
        s if s.contains("energi") || s.contains("energy") || s.contains("listrik") || s.contains("pltu") => ("Energi", true),
        s if s.contains("ippu") || s.contains("industri") || s.contains("industry") => ("IPPU", true),
        s if s.contains("pertanian") || s.contains("agriculture") || s.contains("agrikultur") => ("Pertanian", true),
        s if s.contains("limbah") || s.contains("waste") || s.contains("sampah") => ("Limbah", true),
        s if s.contains("folu") || s.contains("kehutanan") || s.contains("forest") || s.contains("lulucf") => ("FOLU/LULUCF", true),
        s if s.contains("kelautan") || s.contains("marine") || s.contains("laut") => ("Kelautan", true),
        s if s.contains("perikanan") || s.contains("fisheries") || s.contains("fishery") || s.contains("akuakultur") => ("Perikanan", true),
        s if s.contains("karbon_biru") || s.contains("blue_carbon") || s.contains("mangrove") => ("Karbon Biru", true),
        s if s.contains("migas") || s.contains("minyak") || s.contains("gas") || s.contains("oil") => ("Migas", true),
        _ => ("Tidak dikenal", false),
    };

    out.push_str(&format!("  Sektor input: {} → Wajib NDC: {}\n", sector_wajib.0, if sector_wajib.1 { "YA" } else { "Periksa" }));

    out.push_str("\n  Sektor wajib NDC (Permen LH 7/2026):\n");
    out.push_str("  1. Energi (transportasi, kelistrikan, industri energi)\n");
    out.push_str("  2. IPPU (Industrial Processes & Product Use)\n");
    out.push_str("  3. Pertanian\n");
    out.push_str("  4. Limbah\n");
    out.push_str("  5. FOLU (Forest & Other Land Use) — Net Sink 2030\n");
    out.push_str("  6. Kelautan & Perikanan (BARU per Permen 7/2026)\n");
    out.push_str("  7. Karbon Biru (BARU — mangrove, rumput laut)\n");
    out.push_str("  8. Migas (BARU per Permen 7/2026)\n\n");

    // ─── Gap Analysis ───
    out.push_str("═══ GAP ANALYSIS ═══\n");
    let target_year = if year <= 2030 { target_2030 } else { target_2035 };
    let gap = current_emissions_gg_co2e - target_year;
    let gap_pct = (gap / target_year) * 100.0;

    out.push_str(&format!("  Target {} ({}= = {:.0} Gg CO2e\n", year, label, target_year as u64));
    out.push_str(&format!("  Emisi saat ini = {:.0} Gg CO2e\n", current_emissions_gg_co2e as u64));
    out.push_str(&format!("  Gap = {:.0} Gg CO2e ({:.1}%\n\n", gap as i64, gap_pct));

    if gap > 0.0 {
        out.push_str(&format!("  ❌ MELEBIHI target NDC — perlu reduksi {:.0} Gg CO2e\n", gap as u64));
        out.push_str("  Status: OFF TRACK\n\n");
    } else {
        out.push_str(&format!("  ✅ DI BAWAH target NDC — surplus {:.0} Gg CO2e\n", ((-gap) as u64)));
        out.push_str("  Status: ON TRACK\n\n");
    }

    // ─── MRV Framework ───
    out.push_str("═══ MRV (Measurement, Reporting, Verification) ═══\n");
    out.push_str("  Sistem Registri Nasional PPI (SRN-PPI): web-based\n");
    out.push_str("  SIGN-SMART: platform data emisi KLHK\n");
    out.push_str("  BUR (Biennial Update Report): ke UNFCCC\n");
    out.push_str("  FREL: Forest Reference Emission Level (REDD+)\n");
    out.push_str("  Sistem Registri Unit Karbon (Permen LH 10/2026)\n\n");

    if has_mrv {
        out.push_str("  ✅ MRV aktif — lanjutkan pelaporan rutin\n");
    } else {
        out.push_str("  ❌ MRV belum aktif — WAJIB segera:\n");
        out.push_str("  1. Daftar di SRN-PPI\n");
        out.push_str("  2. Laporkan emisi via SIGN-SMART\n");
        out.push_str("  3. Siapkan verifikasi pihak ketika\n");
        out.push_str("  4. Integrasi dengan Sistem Registri Unit Karbon (Permen 10/2026)\n");
    }

    // ─── Sectoral Breakdown (Enhanced NDC CM1) ───
    out.push_str("\n═══ SEKTORAL BREAKDOWN (Enhanced NDC CM1) ═══\n");
    out.push_str("  Sektor          BAU 2030    Target CM1   Reduksi\n");
    out.push_str("  Energi          1,669       1,253        416 MTon\n");
    out.push_str("  IPPU             47          36           11\n");
    out.push_str("  Pertanian       121          110          11\n");
    out.push_str("  Limbah          296          285          11\n");
    out.push_str("  FOLU            714         714-118=596   118 (net sink)\n");
    out.push_str("  Kelautan+Migas  (BARU)       (per Permen 7/2026)\n\n");

    out.push_str("  Ref: Second NDC 2025 (UNFCCC); Permen LH 7/2026; Permen LH 10/2026 (registri karbon)\n");
    out
}
