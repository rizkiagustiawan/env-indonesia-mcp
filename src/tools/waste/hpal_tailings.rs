use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// HPAL Nickel Tailings ESG Compliance Tool
/// Evaluates slurry parameters against Indonesian regulations (PP 101/2014 & PermenLHK).
/// Critical for EV Battery supply chain in Morowali, Obi, Weda Bay.

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HpalTailingsParam {
    pub ph: f64,
    pub cr6_mg_l: f64, // Hexavalent Chromium in extract
    pub ni_mg_l: f64,  // Nickel
    pub co_mg_l: f64,  // Cobalt
    pub mn_mg_l: f64,  // Manganese
    pub is_dry_stack: bool, // True for DST (Dry Stack Tailings), False for DSTP (Deep Sea)
}

pub fn evaluate_hpal_tailings(param: &HpalTailingsParam) -> String {
    let mut out = String::from("=== HPAL Nickel Tailings ESG Compliance ===\n");
    out.push_str("Ref: PP 101/2014, IFC Performance Standards (EV Supply Chain)\n\n");

    let mut violations = Vec::new();

    // 1. Corrosivity (pH) - PP 101/2014 Lampiran III
    let is_corrosive = param.ph <= 2.0 || param.ph >= 12.5;
    if is_corrosive {
        violations.push(format!("pH ({:.1}) - KOROSIF (Kategori 1 B3 Akut)", param.ph));
    }

    // 2. TCLP Heavy Metals - PP 101/2014 Lampiran IV (TCLP-A & TCLP-B)
    // Cr(VI) limit: TCLP-A (5.0 mg/L), TCLP-B (2.5 mg/L)
    if param.cr6_mg_l > 5.0 {
        violations.push(format!("Cr(VI) ({:.2} mg/L) > 5.0 - Gagal TCLP-A (Kategori 1 B3)", param.cr6_mg_l));
    } else if param.cr6_mg_l > 2.5 {
        violations.push(format!("Cr(VI) ({:.2} mg/L) > 2.5 - Gagal TCLP-B (Kategori 2 B3)", param.cr6_mg_l));
    }

    // General toxicity flags (Ni, Co, Mn lack explicit Indonesian TCLP limits, but are tracked for ESG/IFC)
    // Using IFC/WHO proxy limits for groundwater protection
    let mut esg_warnings = Vec::new();
    if param.ni_mg_l > 0.02 { esg_warnings.push(format!("Nikel tinggi ({:.2} mg/L) - Risiko pencemaran laut/air tanah", param.ni_mg_l)); }
    if param.co_mg_l > 0.05 { esg_warnings.push(format!("Kobalt terdeteksi ({:.2} mg/L)", param.co_mg_l)); }
    if param.mn_mg_l > 0.5 { esg_warnings.push(format!("Mangan tinggi ({:.2} mg/L) - Potensi toksisitas akuatik", param.mn_mg_l)); }

    out.push_str("Hasil Uji Karakteristik (Simulasi):\n");
    out.push_str(&format!("  pH               : {:.1}\n", param.ph));
    out.push_str(&format!("  Cr(VI) [Toksik]  : {:.2} mg/L\n", param.cr6_mg_l));
    out.push_str(&format!("  Nickel           : {:.2} mg/L\n", param.ni_mg_l));
    out.push_str(&format!("  Cobalt           : {:.2} mg/L\n", param.co_mg_l));
    out.push_str(&format!("  Manganese        : {:.2} mg/L\n", param.mn_mg_l));
    out.push_str(&format!("  Manajemen Tailing: {}\n\n", if param.is_dry_stack { "Dry Stack Tailings (DST)" } else { "Deep Sea Tailings Placement (DSTP)" }));

    out.push_str("Status Hukum (PP 101/2014):\n");
    if violations.is_empty() {
        out.push_str("  [PASS] Tidak melebihi baku mutu B3 Akut/TCLP.\n");
    } else {
        out.push_str("  [FAIL] PELANGGARAN TERDETEKSI:\n");
        for v in &violations {
            out.push_str(&format!("    - {}\n", v));
        }
    }

    out.push_str("\nAudit ESG (IFC Performance Standard 6):\n");
    if param.is_dry_stack {
        out.push_str("  - DST mengurangi risiko pencemaran laut, namun membutuhkan pemantauan leachate (air lindi) ketat.\n");
    } else {
        out.push_str("  - [!] DSTP SANGAT BERISIKO: Penolakan pasar global (Tesla/Eropa) terhadap nikel dari fasilitas DSTP karena kerusakan terumbu karang dan zona benthik.\n");
    }
    
    for w in &esg_warnings {
        out.push_str(&format!("  - {}\n", w));
    }

    // Chaining payload
    let risk_score = violations.len() as f64 * 10.0 + if !param.is_dry_stack { 50.0 } else { 0.0 };
    out.push_str("\n");
    let payload = crate::result_contract::ScientificResult::new(
        "HPAL_ESG_Risk_Score",
        risk_score,
        "index",
    );
    out.push_str(&serde_json::to_string(&payload).unwrap_or_default());

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpal_compliant_dst() {
        let p = HpalTailingsParam {
            ph: 8.0,
            cr6_mg_l: 1.0,
            ni_mg_l: 0.01,
            co_mg_l: 0.01,
            mn_mg_l: 0.1,
            is_dry_stack: true,
        };
        let out = evaluate_hpal_tailings(&p);
        assert!(out.contains("[PASS]"));
        assert!(out.contains("Dry Stack Tailings"));
    }

    #[test]
    fn test_hpal_violation_dstp() {
        let p = HpalTailingsParam {
            ph: 1.5, // Corrosive
            cr6_mg_l: 6.0, // Fails TCLP-A
            ni_mg_l: 5.0,
            co_mg_l: 2.0,
            mn_mg_l: 10.0,
            is_dry_stack: false,
        };
        let out = evaluate_hpal_tailings(&p);
        assert!(out.contains("KOROSIF"));
        assert!(out.contains("Gagal TCLP-A"));
        assert!(out.contains("DSTP SANGAT BERISIKO"));
    }
}
