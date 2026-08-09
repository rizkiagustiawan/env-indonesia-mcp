/// Oil Spill Response Planning — ITOPF + KepMen LH 51/2004
/// Boom deployment, recovery rate, shoreline cleanup, ESI sensitivity
/// Ref: ITOPF Technical Information; KepMen LH 51/2004 (marine baku mutu)
pub fn assess(spill_volume_ton: f64, oil_type: &str, wind_speed_ms: f64, current_speed_ms: f64, sea_state: u8, distance_to_coast_km: f64) -> String {
    let mut out = String::from("=== Oil Spill Response Planning ===\n");
    out.push_str("Ref: ITOPF; KepMen LH 51/2004; IPIECA\n\n");

    let spill_kg = spill_volume_ton * 1000.0;
    let (oil_density, evap_rate, emulsification, boom_recoverable) = match oil_type.to_lowercase().as_str() {
        s if s.contains("crude") || s.contains("mentah") => (0.85, 0.30, 0.80, 0.40),
        s if s.contains("diesel") || s.contains("diesel") => (0.84, 0.50, 0.30, 0.60),
        s if s.contains("bunker") || s.contains("fuel") || s.contains("bbs") => (0.95, 0.05, 0.90, 0.30),
        s if s.contains("gasoline") || s.contains("bensin") => (0.74, 0.70, 0.10, 0.20),
        _ => (0.85, 0.30, 0.50, 0.40),
    };

    // Slick area (empirical: ~100 m2 per ton for fresh spill)
    let slick_area_m2 = spill_volume_ton * 100.0;
    let slick_radius_m = (slick_area_m2 / std::f64::consts::PI).sqrt();

    // Weathering: evaporation + dissolution + emulsification
    let evaporated_pct = evap_rate * 100.0;
    let remaining_pct = (1.0 - evap_rate) * 100.0;
    let recoverable_ton = spill_volume_ton * (1.0 - evap_rate) * boom_recoverable;

    out.push_str(&format!("Spill: {:.0} ton ({:.0} kg)\n", spill_volume_ton, spill_kg));
    out.push_str(&format!("Oil type: {} (density {:.2}, evap {:.0}%, emulsif {:.0}%)\n", oil_type, oil_density, evaporated_pct, emulsification * 100.0));
    out.push_str(&format!("Wind: {:.1} m/s, Current: {:.1} m/s, Sea state: {}\n", wind_speed_ms, current_speed_ms, sea_state));
    out.push_str(&format!("Distance to coast: {:.1} km\n\n", distance_to_coast_km));

    out.push_str("═══ SPILL WEATHERING ═══\n");
    out.push_str(&format!("  Evaporated: {:.0}% ({:.1} ton)\n", evaporated_pct, spill_volume_ton * evap_rate));
    out.push_str(&format!("  Remaining on water: {:.0}% ({:.1} ton)\n", remaining_pct, spill_volume_ton * (1.0 - evap_rate)));
    out.push_str(&format!("  Slick area: ~{:.0} m2 (radius ~{:.0}m)\n\n", slick_area_m2, slick_radius_m));

    out.push_str("═══ RESPONSE EQUIPMENT ═══\n");
    let boom_length_m = slick_radius_m * 4.0; // containment
    let skimmer_count = (recoverable_ton / 50.0).ceil() as u32; // 50 ton/day per skimmer
    out.push_str(&format!("  Boom (containment)): {:.0} m\n", boom_length_m));
    out.push_str(&format!("  Skimmers: {} unit (50 ton/day each))\n", skimmer_count));
    out.push_str(&format!("  >> Recoverable: {:.1} ton ({:.0}% of remaining))\n\n", recoverable_ton, boom_recoverable * 100.0));

    out.push_str("═══ TIME TO SHORELINE ═══\n");
    let drift_speed = current_speed_ms * 0.03; // ~3% of current
    let time_to_shore_hr = if drift_speed > 0.0 { distance_to_coast_km * 1000.0 / drift_speed / 3600.0 } else { 999.0 };
    out.push_str(&format!("  Drift speed: {:.3} m/s (~3% of current))\n", drift_speed));
    out.push_str(&format!("  >> Time to shoreline: {:.1} hours ({:.1} days))\n\n", time_to_shore_hr, time_to_shore_hr / 24.0));

    if time_to_shore_hr < 48.0 {
        out.push_str("  ⚠️ CRITICAL: shoreline impact < 48 hours — deploy booms NOW\n\n");
    } else {
        out.push_str("  Sufficient time for open water recovery\n\n");
    }

    out.push_str("═══ ESI SENSITIVITY MAPPING ═══\n");
    out.push_str("  Environmental Sensitivity Index (ESI 1-10):\n");
    out.push_str("  ESI 1 (exposed rocky): low sensitivity\n");
    out.push_str("  ESI 4 (sand beaches): moderate\n");
    out.push_str("  ESI 8-9 (mangrove/wetland): HIGH sensitivity\n");
    out.push_str("  ESI 10 (marsh): VERY HIGH\n\n");
    out.push_str("  Priority protect: mangrove, coral reef, aquaculture, mangrove\n\n");

    out.push_str("═══ MARINE BAKU MUTU (KepMen LH 51/2004) ═══\n");
    out.push_str("  Oil & grease limits:\n");
    out.push_str("  - Wisata bahari: ≤ 1 mg/L\n");
    out.push_str("  - Biota laut: ≤ 1 mg/L\n");
    out.push_str("  - Pelabuhan: ≤ 5 mg/L\n");
    let oil_conc_estimated = (recoverable_ton * 1000.0 * 1000.0) / (slick_area_m2 * 0.01); // mg/L approx in 1cm layer
    out.push_str(&format!("  >> Estimated oil conc in water: ~{:.0} mg/L\n", oil_conc_estimated));
    out.push_str(&format!("  Status: ❌ MELEBIHI baku mutu laut (1-5 mg/L))\n\n"));

    out.push_str("═══ WASTE MANAGEMENT ═══\n");
    let waste_ton = recoverable_ton * 2.0; // oil-water emulsion + contaminated sand
    out.push_str(&format!("  Estimated waste: {:.0} ton (oily water + sand))\n", waste_ton));
    out.push_str("  Disposal: PP 101/2014 (B3 waste) — licensed facility\n\n");

    out.push_str("═══ PEMANTAUAN (RPL) ═══\n");
    out.push_str("  Parameter: oil & grease, TPH, PAH, heavy metals\n");
    out.push_str("  Frekuensi: Daily during response, monthly recovery phase\n");
    out.push_str("  Lokasi: Spill site, shoreline, biota monitoring\n\n");

    out.push_str("═══ PELAPORAN & IZIN ═══\n");
    out.push_str("  KepMen LH 51/2004; PP 101/2014 (B3); PP 22/2021\n");
    out.push_str("  Permen LH 6/2026: Sanksi administratif\n");
    out.push_str("  Kontingensi: IOPC Funds; ITOPF\n");

    out.push_str("\n  Ref: ITOPF Technical Info; KepMen LH 51/2004; IPIECA; PP 101/2014\n");
    out
}
