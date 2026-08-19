/// Aquaculture Waste Load — Permen LH/BPLH 2/2026 + Permen KP 30/2021
/// FCR-based N/P/COD load, effluent BOD/TSS, carrying capacity
/// Ref: Permen LH 2/2026 (pakan akuakultur); Permen KP 30/2021; FAO
pub fn assess(fish_type: &str, production_ton_year: f64, fcr: f64, _feed_protein_pct: f64, feed_n_pct: f64, feed_p_pct: f64, water_body_volume_m3: f64, outflow_m3_s: f64) -> String {
    let mut out = String::from("=== Aquaculture Waste Load ===\n");
    out.push_str("Ref: Permen LH 2/2026; Permen KP 30/2021; FAO\n\n");

    // N load: (FCR-1) * feed_N + uneaten feed
    let feed_kg_year = production_ton_year * 1000.0 * fcr;
    let n_in_feed_kg = feed_kg_year * (feed_n_pct / 100.0);
    let p_in_feed_kg = feed_kg_year * (feed_p_pct / 100.0);
    let fish_n_kg = production_ton_year * 1000.0 * 0.025; // ~2.5% N in fish body
    let fish_p_kg = production_ton_year * 1000.0 * 0.004; // ~0.4% P in fish body
    let n_waste_kg = (n_in_feed_kg - fish_n_kg).max(0.0);
    let p_waste_kg = (p_in_feed_kg - fish_p_kg).max(0.0);
    let cod_load_kg = n_waste_kg * 4.57; // N to COD conversion (4.57 g O2/g N)
    let bod_load_kg = cod_load_kg * 0.5; // BOD/COD ~0.5

    // Effluent concentration
    let outflow_m3_year = outflow_m3_s * 86400.0 * 365.0;
    let n_effluent_mg_l = if outflow_m3_year > 0.0 { n_waste_kg * 1000.0 / outflow_m3_year } else { 0.0 };
    let p_effluent_mg_l = if outflow_m3_year > 0.0 { p_waste_kg * 1000.0 / outflow_m3_year } else { 0.0 };
    let bod_effluent_mg_l = if outflow_m3_year > 0.0 { bod_load_kg * 1000.0 / outflow_m3_year } else { 0.0 };

    out.push_str(&format!("Fish: {} (production: {:.0} ton/year))\n", fish_type, production_ton_year));
    out.push_str(&format!("FCR: {:.2}, Feed: {:.0} kg/year (N={:.0}%, P={:.0}%))\n\n", fcr, feed_kg_year, feed_n_pct, feed_p_pct));

    out.push_str("═══ WASTE LOAD ═══\n");
    out.push_str(&format!("  N waste: {:.0} kg/year ({:.1} kg/ton fish)\n", n_waste_kg, n_waste_kg / production_ton_year));
    out.push_str(&format!("  P waste: {:.0} kg/year ({:.1} kg/ton fish)\n", p_waste_kg, p_waste_kg / production_ton_year));
    out.push_str(&format!("  COD load: {:.0} kg/year\n", cod_load_kg));
    out.push_str(&format!("  BOD load: {:.0} kg/year\n\n", bod_load_kg));

    out.push_str("═══ EFFLUENT CONCENTRATION ═══\n");
    out.push_str(&format!("  BOD: {:.1} mg/L\n", bod_effluent_mg_l));
    out.push_str(&format!("  NH3-N: {:.1} mg/L\n", n_effluent_mg_l));
    out.push_str(&format!("  Total P: {:.1} mg/L\n\n", p_effluent_mg_l));

    out.push_str("═══ BAKU MUTU (Permen LH 2/2026 — Pakan Akuakultur) ═══\n");
    out.push_str("  Parameter  | Baku Mutu | Actual  | Status\n");
    let bod_ok = bod_effluent_mg_l <= 100.0;
    let nh3_ok = n_effluent_mg_l <= 10.0;
    out.push_str(&format!("  BOD        | 100 mg/L  | {:.1}    | {}\n", bod_effluent_mg_l, if bod_ok {"✅"} else {"❌"}));
    out.push_str(&format!("  NH3-N      | 10 mg/L   | {:.1}    | {}\n", n_effluent_mg_l, if nh3_ok {"✅"} else {"❌"}));
    out.push_str(&format!("  pH         | 6-9       | —       | (monitor))\n\n"));

    // Carrying capacity
    out.push_str("═══ CARRYING CAPACITY ═══\n");
    let dilution_ratio = if outflow_m3_year > 0.0 { water_body_volume_m3 / outflow_m3_year } else { 0.0 };
    out.push_str(&format!("  Water body: {:.0} m3, Outflow: {:.1} m3/s\n", water_body_volume_m3, outflow_m3_s));
    out.push_str(&format!("  Dilution ratio: {:.1}\n", dilution_ratio));
    let max_production = if bod_effluent_mg_l > 0.0 { production_ton_year * 100.0 / bod_effluent_mg_l } else { 0.0 };
    out.push_str(&format!("  >> Max production (to meet BOD ≤100)): {:.0} ton/year\n\n", max_production));

    out.push_str("═══ PEMANTAUAN (RPL) ═══\n");
    out.push_str("  Parameter: BOD, NH3-N, Total P, TSS, pH, DO\n");
    out.push_str("  Frekuensi: Bulanan (effluent), musiman (water body)\n");
    out.push_str("  Lokasi: Outfall + 3 titik di water body (hulu, tengah, hilir)\n\n");

    out.push_str("═══ PELAPORAN & IZIN ═══\n");
    out.push_str("  Permen LH 2/2026: Baku mutu pakan akuakultur\n");
    out.push_str("  Permen KP 30/2021: Kinerja perusahaan perikanan\n");
    out.push_str("  PP 22/2021: Kelas air peruntukan\n");
    out.push_str("  Amdalnet + OSS; Permen LH 6/2026\n");

    out.push_str("\n  Ref: Permen LH 2/2026; Permen KP 30/2021; FAO Aquaculture\n");
    out
}
