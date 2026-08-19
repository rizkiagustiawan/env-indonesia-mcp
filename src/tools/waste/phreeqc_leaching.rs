use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// PHREEQC Geochemical Leaching Model Generator
/// Menghasilkan script termodinamika untuk memprediksi pelepasan (leaching) logam berat
/// dari limbah tambang (misal: tailing nikel, limbah B3) berdasarkan perubahan pH.
/// Ref: USGS PHREEQC Version 3; Parkhurst & Appelo

pub fn generate_phreeqc_script(
    waste_type: &str,
    solid_mass_g: f64,
    water_volume_l: f64,
    target_ph: f64,
    initial_metals_mg_kg: &str, // e.g. "Ni: 15000.0, Cr: 500.0, Fe: 200000.0"
) -> String {
    let mut script = String::new();

    script.push_str(&format!("TITLE Speciation and Leaching Model for {}\n", waste_type));
    script.push_str("DATABASE minteq.v4.dat\n\n"); // Minteq database for heavy metals

    script.push_str("SOLUTION 1 Initial Groundwater / Leachate Extractant\n");
    script.push_str("    temp      25.0\n");
    script.push_str("    pH        7.0\n");
    script.push_str("    pe        4.0\n");
    script.push_str("    redox     pe\n");
    script.push_str("    units     mg/kgw\n");
    script.push_str("    density   1.0\n");
    script.push_str(&format!("    water     {:.2} # kg (approx L)\n", water_volume_l));
    script.push_str("END\n\n");

    script.push_str("USE solution 1\n");
    script.push_str("EQUILIBRIUM_PHASES 1\n");
    
    // Simulate target pH using acid/base addition (HCl or NaOH proxy)
    // Actually, PHREEQC can fix pH by adding/removing a phase
    script.push_str(&format!("    Fix_H+  -{:.2}  NaOH  10.0\n", target_ph));
    
    // Add common minerals controlling metal solubility in laterite/tailings
    script.push_str("    Ni(OH)2           0.0 0.0\n");
    script.push_str("    Cr(OH)3           0.0 0.0\n");
    script.push_str("    Fe(OH)3(a)        0.0 0.0\n");
    script.push_str("    Al(OH)3(a)        0.0 0.0\n\n");

    // Add Solid Solution or Total Elements
    script.push_str("REACTION 1 Addition of heavy metals from waste solid\n");
    
    // Parse pseudo-JSON/String like "Ni: 15000, Cr: 500"
    // For generator, we convert mg/kg of solid into total moles added.
    let metals: Vec<&str> = initial_metals_mg_kg.split(',').collect();
    for m in metals {
        let parts: Vec<&str> = m.split(':').collect();
        if parts.len() == 2 {
            let elem = parts[0].trim();
            if let Ok(conc_mg_kg) = parts[1].trim().parse::<f64>() {
                // Approximate molar mass (Cr: 52, Ni: 58.7, Fe: 55.8)
                let mw = match elem {
                    "Ni" => 58.69,
                    "Cr" => 51.99,
                    "Fe" => 55.845,
                    "Pb" => 207.2,
                    "As" => 74.92,
                    "Cd" => 112.41,
                    "Cu" => 63.54,
                    _ => 50.0, // fallback
                };
                let moles = (conc_mg_kg * solid_mass_g / 1000.0) / (mw * 1000.0);
                script.push_str(&format!("    {}    {:.6e} # Moles added\n", elem, moles));
            }
        }
    }
    script.push_str("    1.0 moles in 1 step\n\n");

    script.push_str("SELECTED_OUTPUT\n");
    script.push_str("    -file         leaching_results.csv\n");
    script.push_str("    -reset        false\n");
    script.push_str("    -ph           true\n");
    script.push_str("    -pe           true\n");
    script.push_str("    -totals       Ni Cr Fe Pb As Cd Cu\n");
    script.push_str("    -saturation_indices Ni(OH)2 Cr(OH)3 Fe(OH)3(a)\n");
    script.push_str("END\n");

    // We output the script string inside a claim, so the orchestrator can still parse the output as JSON
    let res = ScientificResult::new("phreeqc_script_generated", 1.0, "boolean")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("generator", "USGS_PHREEQC_v3", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("script_content", &script))
        .with_claim(Claim::new("limitation", "This is a script generator only. Does not execute reactive transport directly."));

    json!([
        serde_json::from_str::<serde_json::Value>(&res.emit_validated()).unwrap()
    ]).to_string()
}
