use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use serde_json::json;

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
    let mut risk_score = 0.0;
    
    // 1. Corrosivity (pH) - PP 101/2014 Lampiran III
    let is_corrosive = param.ph <= 2.0 || param.ph >= 12.5;
    if is_corrosive {
        risk_score += 10.0;
    }

    // 2. TCLP Heavy Metals - PP 101/2014 Lampiran IV (TCLP-A & TCLP-B)
    if param.cr6_mg_l > 5.0 {
        risk_score += 10.0; // Fails TCLP-A (Kategori 1 B3)
    } else if param.cr6_mg_l > 2.5 {
        risk_score += 10.0; // Fails TCLP-B (Kategori 2 B3)
    }

    // 3. Chromium speciation (Delina et al. 2024)
    let cr3_mg_l = (param.total_cr_mg_l - param.cr6_mg_l).max(0.0);
    let cr3_fraction = if param.total_cr_mg_l > 0.0 {
        cr3_mg_l / param.total_cr_mg_l
    } else {
        0.0
    };

    // 4. Cr(VI) mobility
    let cr6_mobile_risk = param.ph > 7.0 && param.cr6_mg_l > 0.05;
    if cr6_mobile_risk {
        risk_score += 20.0;
    }

    // ESG Risk
    if !param.is_dry_stack {
        risk_score += 50.0; // DSTP is highly penalized
    }

    let mut claims = vec![];
    
    if cr6_mobile_risk {
        claims.push(Claim::new("warning", "Alkaline pH (>7): Cr(VI) oxyanions desorb and become highly mobile"));
    } else if param.ph <= 7.0 {
        claims.push(Claim::new("observation", "Acidic pH: Cr(VI) strongly adsorbed, low mobility"));
    }

    if !param.is_dry_stack {
        claims.push(Claim::new("esg_warning", "DSTP poses severe benthic/coral risk and faces global market rejection"));
    }

    let res_risk = ScientificResult::new("hpal_esg_risk_score", risk_score, "index")
        .with_status(if is_corrosive || param.cr6_mg_l > 5.0 { ResultStatus::ValidationFailed } else { ResultStatus::Valid })
        .with_provenance(Provenance::new("calculation", "ESG_IFC_PP101", "2026-08-19T00:00:00Z"));

    let res_cr3 = ScientificResult::new("cr3_immobile_fraction", cr3_fraction, "ratio")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "Delina_et_al_2024", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("methodology", "Cr(III) structurally incorporated into secondary iron oxides"));

    let mut res_cr3_mut = res_cr3;
    for claim in claims {
        res_cr3_mut = res_cr3_mut.with_claim(claim);
    }

    json!([
        serde_json::from_str::<serde_json::Value>(&res_risk.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_cr3_mut.emit_validated()).unwrap()
    ]).to_string()
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
        assert!(out.contains("hpal_esg_risk_score"));
        assert!(out.contains("Alkaline pH (>7): Cr(VI)")); 
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
        assert!(out.contains("validation_failed")); // Fix string assert from "ValidationFailed"
        assert!(out.contains("cr3_immobile_fraction"));
    }
}
