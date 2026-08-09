/// Remediation Target Levels — PP 22/2021 (soil/groundwater) + PP 101/2014 (B3)
/// Site-specific cleanup based on receptor pathway (J&E vapor, ingestion, dermal)
/// Ref: PP 22/2021 Annex VI; PP 101/2014; US EPA RAGS; ASTM E2081 RBCA
pub fn assess(contaminant: &str, contaminant_conc_mg_kg: f64, groundwater_conc_mg_l: f64, land_use: &str, has_residential_receptor: bool, depth_to_groundwater_m: f64, soil_organic_carbon_pct: f64) -> String {
    let mut out = String::from("=== Remediation Target Levels (PP 22/2021 + PP 101/2014) ===\n");
    out.push_str("Ref: PP 22/2021 Annex VI; PP 101/2014 (B3); EPA RAGS; ASTM E2081\n\n");

    out.push_str(&format!("Contaminant: {}\n", contaminant));
    out.push_str(&format!("Soil: {} mg/kg, Groundwater: {} mg/L\n", contaminant_conc_mg_kg, groundwater_conc_mg_l));
    out.push_str(&format!("Land use: {}, Residential: {}\n", land_use, if has_residential_receptor { "Yes" } else { "No (industrial)" }));
    out.push_str(&format!("Depth to groundwater: {:.1} m, SOC: {:.1}%\n\n", depth_to_groundwater_m, soil_organic_carbon_pct));

    // Screening levels (PP 22/2021 Annex VI groundwater + EPA RSL)
    let (gw_target, soil_target, source) = match contaminant.to_lowercase().as_str() {
        "benzene" | "benzena" => (0.005, 1.2, "PP 22/2021 Kelas I + EPA RSL"),
        "toluene" => (0.7, 16.0, "EPA RSL"),
        "ethylbenzene" => (0.3, 15.0, "EPA RSL"),
        "xylene" => (0.53, 15.0, "EPA RSL"),
        "tce" | "trichloroethylene" => (0.003, 0.52, "PP 22/2021 Kelas I"),
        "pce" | "tetrachloroethylene" => (0.003, 0.88, "PP 22/2021 Kelas I"),
        "pb" | "timbal" | "lead" => (0.05, 400.0, "PP 22/2021 Kelas I soil=EPA RSL"),
        "cd" | "cadmium" | "kadmium" => (0.01, 70.0, "PP 22/2021 + EPA RSL"),
        "cr6" | "cr(vi)" => (0.05, 240.0, "PP 22/2021 + EPA RSL"),
        "as" | "arsenic" | "arsenik" => (0.05, 68.0, "PP 22/2021 + EPA RSL"),
        "hg" | "mercury" | "merkuri" => (0.001, 31.0, "PP 22/2021 + EPA RSL"),
        "phenol" | "fenol" => (0.001, 4000.0, "PP 22/2021 Kelas I"),
        "tpH" | "tph" | "minyak" => (0.05, 100.0, "PP 22/2021 oil grease"),
        _ => (0.001, 100.0, "Generic screening"),
    };

    let residential_factor = if has_residential_receptor { 1.0 } else { 10.0 }; // industrial less strict
    let soil_target_adjusted = soil_target * residential_factor;

    out.push_str("═══ REMEDIATION TARGET LEVELS ═══\n\n");
    out.push_str(&format!("  Groundwater target: {:.4} mg/L ({}))\n", gw_target, source));
    out.push_str(&format!("  Soil target: {:.2} mg/kg ({}x for {})) ({})\n\n", soil_target_adjusted, if has_residential_receptor {1.0} else {10.0}, land_use, source));

    // Compliance
    let gw_ok = groundwater_conc_mg_l <= gw_target;
    let soil_ok = contaminant_conc_mg_kg <= soil_target_adjusted;

    out.push_str("═══ STATUS KEPATUHAN ═══\n");
    out.push_str(&format!("  Groundwater: {} mg/L vs target {:.4} → {}\n", groundwater_conc_mg_l, gw_target, if gw_ok { "✅ MEMENUHI" } else { "❌ MELEBIHI" }));
    out.push_str(&format!("  Soil: {} mg/kg vs target {:.2} → {}\n\n", contaminant_conc_mg_kg, soil_target_adjusted, if soil_ok { "✅ MEMENUHI" } else { "❌ MELEBIHI" }));

    // Exceedance
    if !soil_ok || !gw_ok {
        out.push_str("  ❌ REMEDIASI DIPERLUKAN\n\n");
        out.push_str("═══ PILIHAN TEKNOLOGI REMEDIASI ═══\n");
        out.push_str("  Soil:\n");
        match contaminant.to_lowercase().as_str() {
            c if c.contains("benzene") || c.contains("toluene") || c.contains("tce") || c.contains("pce") || c.contains("xylene") || c.contains("ethyl") => {
                out.push_str("  - Soil Vapor Extraction (SVE) — volatile organics\n");
                out.push_str("  - Bioventing — biodegradation in situ\n");
                out.push_str("  - Thermal desorption — ex situ, 95-99% removal\n");
            }
            c if c.contains("pb") || c.contains("cd") || c.contains("cr") || c.contains("as") || c.contains("hg") => {
                out.push_str("  - Solidification/Stabilization — bind metals\n");
                out.push_str("  - Soil washing — physical separation\n");
                out.push_str("  - Phytoremediation — hyperaccumulator plants\n");
            }
            c if c.contains("tph") || c.contains("minyak") => {
                out.push_str("  - Land farming — biodegradation ex situ\n");
                out.push_str("  - Bioventing + biopile\n");
            }
            _ => {
                out.push_str("  - Pump & treat (groundwater)\n");
                out.push_str("  - Permeable Reactive Barrier (PRB)\n");
                out.push_str("  - Bioremediation\n");
            }
        }
        out.push_str("  Groundwater:\n");
        out.push_str("  - Pump & Treat (P&T)\n");
        out.push_str("  - Permeable Reactive Barrier (PRB) — ZVI for chlorinated\n");
        out.push_str("  - Monitored Natural Attenuation (MNA)\n");
        out.push_str("  - In-situ Chemical Oxidation (ISCO) — persulfate/ozone\n\n");
    } else {
        out.push_str("  ✅ Tidak perlu remediasi — di bawah target\n\n");
    }

    out.push_str("═══ PEMANTAUAN (RPL) ═══\n");
    out.push_str("  Parameter: contaminant in soil + groundwater\n");
    out.push_str("  Frekuensi: Quarterly (active remediation), Semi-annual (MNA)\n");
    out.push_str("  Lokasi: Source area + plume boundaries + receptor wells\n\n");

    out.push_str("═══ PELAPORAN & IZIN ═══\n");
    out.push_str("  PP 22/2021 Pasal 124-131; PP 101/2014 (B3-contaminated land)\n");
    out.push_str("  Persetujuan Lingkungan (PP 28/2025); Amdalnet\n");
    out.push_str("  Permen LH 6/2026: Sanksi jika tidak remediasi\n");

    out.push_str("\n  Ref: PP 22/2021 Annex VI; PP 101/2014; EPA RAGS; ASTM E2081 RBCA\n");
    out
}
