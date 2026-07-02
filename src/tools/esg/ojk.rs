pub fn check_compliance(entity_type: &str, disclosures: &str) -> String {
    let required = match entity_type.to_lowercase().as_str() {
        "bank" => vec![
            "environmental_policy", "carbon_emission", "energy_efficiency",
            "waste_management", "green_financing", "climate_risk",
            "social_responsibility", "financial_inclusion", "labor_practices",
            "governance_structure", "risk_management", "compliance_reporting",
        ],
        "insurance" | "asuransi" => vec![
            "environmental_policy", "climate_risk_assessment", "sustainable_investment",
            "social_impact", "governance_transparency", "risk_disclosure",
        ],
        "securities" | "efek" => vec![
            "esg_risk_assessment", "sustainable_portfolio", "disclosure_transparency",
            "social_responsibility", "governance_board", "climate_scenario",
        ],
        "financing" | "pembiayaan" => vec![
            "environmental_policy", "green_lending", "social_impact_assessment",
            "governance_structure", "risk_management", "sustainability_report",
        ],
        _ => vec!["environmental_policy", "social_responsibility", "governance_structure"],
    };

    let disclosed: Vec<&str> = disclosures.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    let mut met = Vec::new();
    let mut missing = Vec::new();

    for req in &required {
        if disclosed.iter().any(|d| d == req) {
            met.push(*req);
        } else {
            missing.push(*req);
        }
    }

    let score = if required.is_empty() { 0.0 } else { met.len() as f64 / required.len() as f64 * 100.0 };
    let status = if score >= 80.0 { "COMPLIANT" } else if score >= 50.0 { "PARTIALLY COMPLIANT" } else { "NON-COMPLIANT" };

    let mut out = format!("=== OJK POJK 51/2017 Compliance Check ===\nEntity Type: {}\nRegulation: POJK No.51/POJK.03/2017\n  Penerapan Keuangan Berkelanjutan bagi LJK, Emiten, dan Perusahaan Publik\n\n", entity_type);
    out.push_str(&format!("Compliance Score: {:.0}% — {}\n\n", score, status));
    out.push_str(&format!("Requirements Met ({}/{}):\n", met.len(), required.len()));
    for m in &met { out.push_str(&format!("  ✅ {}\n", m)); }
    if !missing.is_empty() {
        out.push_str(&format!("\nMissing Requirements ({}):\n", missing.len()));
        for m in &missing { out.push_str(&format!("  ❌ {}\n", m)); }
    }
    out.push_str("\nKey POJK 51/2017 Requirements:\n");
    out.push_str("  1. Rencana Aksi Keuangan Berkelanjutan (RAKB)\n");
    out.push_str("  2. Laporan Keberlanjutan (Sustainability Report)\n");
    out.push_str("  3. Tata Kelola Keuangan Berkelanjutan\n");
    out.push_str("  4. Manajemen Risiko Lingkungan & Sosial\n");
    out.push_str("\nReference: https://www.ojk.go.id/id/regulasi/otoritas-jasa-keuangan/peraturan-ojk/\n");
    out
}
