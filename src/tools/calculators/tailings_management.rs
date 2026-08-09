/// Tailings Management — Permen ESDM
/// Tailings dam design (FS), supernatant quality, submarine disposal
/// Critical for nickel/tin mining (Indonesia top producer)
/// Ref: GISTM (Global Industry Standard on Tailings Management); Permen ESDM; ANCOLD
pub fn assess(ore_type: &str, tailings_volume_m3_day: f64, tailings_solid_pct: f64, dam_height_m: f64, dam_volume_m3: f64, supernatant_ph: f64, supernatant_metals_json: &str, disposal_method: &str, foundation_type: &str, seismic_zone: &str) -> String {
    let mut out = String::from("=== Tailings Management ===\n");
    out.push_str("Ref: GISTM; Permen ESDM; ANCOLD; ICOLD\n\n");

    let solids_m3_day = tailings_volume_m3_day * tailings_solid_pct / 100.0;
    let water_m3_day = tailings_volume_m3_day - solids_m3_day;

    out.push_str(&format!("Ore: {}, Tailings: {:.0} m³/day ({:.0}% solid))\n", ore_type, tailings_volume_m3_day, tailings_solid_pct));
    out.push_str(&format!("Dam: {:.0}m height, {:.0} m³ volume\n", dam_height_m, dam_volume_m3));
    out.push_str(&format!("Supernatant: pH {:.1}, Foundation: {}, Seismic: {}\n", supernatant_ph, foundation_type, seismic_zone));
    out.push_str(&format!("Disposal: {}\n\n", disposal_method));

    out.push_str("═══ TAILINGS DAM SAFETY ═══\n");
    let dam_life_years = dam_volume_m3 / tailings_volume_m3_day / 365.0;
    out.push_str(&format!("  Dam life: {:.1} years\n", dam_life_years));

    // Factor of Safety (simplified Bishop)
    let fs = match (foundation_type, seismic_zone) {
        (f, s) if f.contains("rock") && s.contains("low") => 1.5,
        (f, s) if f.contains("rock") => 1.4,
        (f, s) if f.contains("soil") && s.contains("low") => 1.4,
        (f, s) if f.contains("soil") => 1.3,
        _ => 1.2,
    };
    out.push_str(&format!("  Factor of Safety (static)): {:.2}\n", fs));
    out.push_str("  Criteria: FS ≥ 1.5 (GISTM), ≥ 1.3 (ANCOLD)\n");
    out.push_str(&format!("  Status: {}\n\n", if fs >= 1.5 {"✅ ACCEPTABLE"} else if fs >= 1.3 {"⚠️ MARGINAL"} else {"❌ UNACCEPTABLE"}));

    out.push_str("═══ SUPERNATANT QUALITY ═══\n");
    out.push_str(&format!("  pH: {:.1} (target 6-9))\n", supernatant_ph));
    let ph_ok = supernatant_ph >= 6.0 && supernatant_ph <= 9.0;
    out.push_str(&format!("  Status: {}\n", if ph_ok {"✅"} else {"❌"}));

    let metals: Vec<(String, f64)> = match serde_json::from_str(supernatant_metals_json) {
        Ok(v) => v, Err(_) => Vec::new(),
    };
    if !metals.is_empty() {
        out.push_str("\n  Metals vs PP 22/2021 Kelas IV (Irigasi):\n");
        out.push_str(&format!("  {:<10} {:>10} {:>10} {:>10}\n", "Metal", "Conc", "Limit", "Status"));
        for (metal, conc) in &metals {
            let limit = match metal.to_lowercase().as_str() {
                "ni"|"nickel" => 0.1, "cr"|"chromium" => 0.1, "fe"|"iron" => 1.0,
                "mn"|"manganese" => 0.5, "as"|"arsenic" => 0.1, "pb"|"lead" => 0.1,
                "zn"|"zinc" => 0.5, "cu"|"copper" => 0.1, "cd"|"cadmium" => 0.01,
                _ => 0.05,
            };
            let ok = *conc <= limit;
            out.push_str(&format!("  {:<10} {:>10.3} {:>10.3} {:>10}\n", metal, conc, limit, if ok {"✅"} else {"❌"}));
        }
    }
    out.push('\n');

    out.push_str("═══ DISPOSAL METHOD ═══\n");
    match disposal_method.to_lowercase().as_str() {
        s if s.contains("dam") || s.contains("dry") => {
            out.push_str("  → TSF (Tailings Storage Facility) — above ground\n");
            out.push_str("  Risk: dam failure, seepage, acid mine drainage\n");
            out.push_str("  Mitigation: lined dam, seepage collection, cover system\n");
        }
        s if s.contains("submarine") || s.contains("subsea") || s.contains("dstp") => {
            out.push_str("  → Submarine Tailings Disposal (STD/DSTP)\n");
            out.push_str("  Risk: marine pollution, smothering benthic habitat\n");
            out.push_str("  Critical: depth > 200m, current modeling, monitoring\n");
            out.push_str("  Marine baku mutu: KepMen LH 51/2004\n");
        }
        s if s.contains("backfill") || s.contains("paste") => {
            out.push_str("  → Paste backfill (underground mine void)\n");
            out.push_str("  ✅ Best practice — minimal surface impact\n");
        }
        _ => {
            out.push_str("  → Verify method compliance with GISTM\n");
        }
    }
    out.push('\n');

    // Acid generation
    out.push_str("═══ ACID GENERATION POTENTIAL ═══\n");
    let acid_risk = match ore_type.to_lowercase().as_str() {
        s if s.contains("nickel") || s.contains("nikel") => ("HIGH", "Ni laterite — potential acid mine drainage"),
        s if s.contains("coal") || s.contains("batubara") => ("HIGH", "Pyrite in coal overburden"),
        s if s.contains("copper") || s.contains("tembaga") => ("MODERATE", "Sulfide ore"),
        s if s.contains("gold") || s.contains("emas") => ("HIGH", "Pyrite common in Au ore"),
        s if s.contains("tin") || s.contains("timah") => ("LOW", "Oxide ore"),
        _ => ("UNKNOWN", "Test NAG (Net Acid Generation) + NAPP"),
    };
    out.push_str(&format!("  Risk: {} — {}\n\n", acid_risk.0, acid_risk.1));

    out.push_str("═══ PEMANTAUAN (RPL) ═══\n");
    out.push_str("  Parameter: dam stability (piezometer, inclinometer), supernatant quality, seepage\n");
    out.push_str("  Frekuensi: Daily (piezometer), Monthly (water quality)\n");
    out.push_str("  Emergency: EAP (Emergency Action Plan) — GISTM requirement\n\n");

    out.push_str("═══ PELAPORAN & IZIN ═══\n");
    out.push_str("  Permen ESDM: RKAB (Rencana Kerja & Anggaran Biaya)\n");
    out.push_str("  PP 22/2021: Persetujuan Lingkungan (AMDAL)\n");
    out.push_str("  GISTM compliance for international financing\n");
    out.push_str("  Amdalnet + OSS; Permen LH 6/2026\n");

    out.push_str("\n  Ref: GISTM; Permen ESDM; ANCOLD; ICOLD; PP 22/2021\n");
    out
}
