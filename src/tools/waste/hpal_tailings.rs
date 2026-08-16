use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// HPAL Nickel Tailings ESG Compliance Tool
/// Evaluates slurry parameters against Indonesian regulations (PP 101/2014 & PermenLHK).
/// Critical for EV Battery supply chain in Morowali, Obi, Weda Bay.
///
/// Chromium speciation (Delina et al., ACS EST 2024/2025):
///   - Cr(III) is structurally incorporated into secondary iron (oxyhydr)oxides
///     (goethite/ferrihydrite) → largely IMMOBILE.
///   - Cr(VI) exists as oxyanions (CrO4^2-), pH-dependent adsorption: strongly bound
///     on positively-charged oxide surfaces at low pH, but desorbs and becomes MOBILE
///     at alkaline pH (e.g. after liming/neutralisation of acidic HPAL tailings).

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HpalTailingsParam {
    #[schemars(description = "Tailings pH")]
    pub ph: f64,
    #[schemars(description = "Total chromium in extract (mg/L)")]
    pub total_cr_mg_l: f64,
    #[schemars(description = "Hexavalent chromium Cr(VI) in extract (mg/L)")]
    pub cr6_mg_l: f64,
    #[schemars(description = "Nickel (mg/L)")]
    pub ni_mg_l: f64,
    #[schemars(description = "Cobalt (mg/L)")]
    pub co_mg_l: f64,
    #[schemars(description = "Manganese (mg/L)")]
    pub mn_mg_l: f64,
    #[schemars(description = "true = Dry Stack Tailings (DST), false = Deep Sea Tailings Placement (DSTP)")]
    pub is_dry_stack: bool,
}

pub fn evaluate_hpal_tailings(param: &HpalTailingsParam) -> String {
    let mut out = String::from("=== HPAL Nickel Tailings ESG Compliance ===\n");
    out.push_str("Ref: PP 101/2014, IFC Performance Standards; Delina et al. 2024 (ACS EST) Cr speciation\n\n");

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

    // 3. Chromium speciation (Delina et al. 2024)
    // Cr(III) = total - Cr(VI); structurally incorporated → immobile
    let cr3_mg_l = (param.total_cr_mg_l - param.cr6_mg_l).max(0.0);
    let cr3_fraction = if param.total_cr_mg_l > 0.0 {
        cr3_mg_l / param.total_cr_mg_l
    } else {
        0.0
    };

    // 4. Cr(VI) mobility: alkaline pH desorbs Cr(VI) oxyanions (mobile risk)
    // HPAL tailings are acidic (bound), but liming/neutralisation mobilises Cr(VI).
    let cr6_mobile_risk = param.ph > 7.0 && param.cr6_mg_l > 0.05;

    // General toxicity flags (Ni, Co, Mn lack explicit Indonesian TCLP limits, but are tracked for ESG/IFC)
    // Using IFC/WHO proxy limits for groundwater protection
    let mut esg_warnings = Vec::new();
    if param.ni_mg_l > 0.02 { esg_warnings.push(format!("Nikel tinggi ({:.2} mg/L) - Risiko pencemaran laut/air tanah", param.ni_mg_l)); }
    if param.co_mg_l > 0.05 { esg_warnings.push(format!("Kobalt terdeteksi ({:.2} mg/L)", param.co_mg_l)); }
    if param.mn_mg_l > 0.5 { esg_warnings.push(format!("Mangan tinggi ({:.2} mg/L) - Potensi toksisitas akuatik", param.mn_mg_l)); }

    out.push_str("Hasil Uji Karakteristik (Simulasi):\n");
    out.push_str(&format!("  pH               : {:.1}\n", param.ph));
    out.push_str(&format!("  Kromium Total    : {:.2} mg/L\n", param.total_cr_mg_l));
    out.push_str(&format!("  Cr(VI) [Toksik]  : {:.2} mg/L\n", param.cr6_mg_l));
    out.push_str(&format!("  Cr(III) [Imobil] : {:.2} mg/L ({:.0}% dari total)\n", cr3_mg_l, cr3_fraction * 100.0));
    out.push_str(&format!("  Nickel           : {:.2} mg/L\n", param.ni_mg_l));
    out.push_str(&format!("  Cobalt           : {:.2} mg/L\n", param.co_mg_l));
    out.push_str(&format!("  Manganese        : {:.2} mg/L\n", param.mn_mg_l));
    out.push_str(&format!("  Manajemen Tailing: {}\n\n", if param.is_dry_stack { "Dry Stack Tailings (DST)" } else { "Deep Sea Tailings Placement (DSTP)" }));

    out.push_str("Spesiasi & Mobilitas Kromium (Delina et al. 2024):\n");
    out.push_str(&format!("  Cr(III) terinkorporasi struktural dalam besi oksihidroksida → imobil ({:.0}%).\n", cr3_fraction * 100.0));
    if cr6_mobile_risk {
        out.push_str("  [RISIKO] pH ALKALIN (>7): Cr(VI) oksianion terdesorpsi dari permukaan oksida → SANGAT MOBIL.\n");
        out.push_str("           Netralisasi/kapur pada tailing HPAL asam dapat melepaskan Cr(VI) ke air tanah/laut.\n");
    } else if param.ph <= 7.0 {
        out.push_str("  pH asam: Cr(VI) teradsorpsi kuat pada permukaan oksida (muatan positif) → mobilitas rendah.\n");
    }

    out.push_str("\nStatus Hukum (PP 101/2014):\n");
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
    let risk_score = violations.len() as f64 * 10.0
        + if !param.is_dry_stack { 50.0 } else { 0.0 }
        + if cr6_mobile_risk { 20.0 } else { 0.0 };
    out.push_str("\n");
    let payload = crate::result_contract::ScientificResult::new(
        "HPAL_ESG_Risk_Score",
        risk_score,
        "index",
    );
    out.push_str(&payload.emit_validated());

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpal_compliant_dst() {
        let p = HpalTailingsParam {
            ph: 8.0,
            total_cr_mg_l: 2.0,
            cr6_mg_l: 1.0,
            ni_mg_l: 0.01,
            co_mg_l: 0.01,
            mn_mg_l: 0.1,
            is_dry_stack: true,
        };
        let out = evaluate_hpal_tailings(&p);
        assert!(out.contains("[PASS]"));
        assert!(out.contains("Dry Stack Tailings"));
        assert!(out.contains("SANGAT MOBIL")); // alkaline pH + Cr(VI) > 0.05
    }

    #[test]
    fn test_hpal_violation_dstp() {
        let p = HpalTailingsParam {
            ph: 1.5, // Corrosive
            total_cr_mg_l: 8.0,
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
        assert!(out.contains("Cr(III)")); // speciation section present
    }
}
