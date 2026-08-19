use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Environmental Management Plan (RKL-RPL) Generator + KPI
/// Ref: PermenLHK No. 5/2021 (AMDAL), PermenLHK No. 6/2021
/// 2026 SOTA: Anggreini 2026 (peatland EMP+KPI), Harmuzan 2026 (sustainable finance KPIs)
/// ISO 14001:2026 link: Clause 8.1 (Operational Control), Clause 9.1 (Monitoring)
/// Rani et al. 2026 (automated KPI dashboard for EMP)

pub fn generate(
    impacts_json: &str,
    project_type: &str,
    location: &str,
) -> String {
    let impacts: Vec<(String, String, f64, f64)> = match serde_json::from_str(impacts_json) {
        Ok(v) => v,
        Err(e) => return json!({"error": "E100", "message": format!("impacts_json parse: {}. Format: [[\"dampak\",\"komponen\",magnitude,importance],...]", e)}).to_string(),
    };

    if impacts.is_empty() {
        return json!({"error": "E102", "message": "impacts_json kosong. Minimal 1 dampak signifikan"}).to_string();
    }

    // Filter significant impacts (|magnitude × importance| >= 30)
    let significant: Vec<&(String, String, f64, f64)> = impacts.iter()
        .filter(|(_, _, mag, imp)| (mag * imp).abs() >= 30.0)
        .collect();

    if significant.is_empty() {
        let res = ScientificResult::new("kpi_overall", 100.0, "%")
            .with_status(ResultStatus::ValidWithAssumptions)
            .with_provenance(Provenance::new("generator", "PermenLHK_5_2021", "2026-08-19T00:00:00Z"))
            .with_claim(Claim::new("info", "Tidak ada dampak signifikan. EMP tidak diperlukan."));
        return json!([serde_json::from_str::<serde_json::Value>(&res.emit_validated()).unwrap()]).to_string();
    }

    let mut rkl_count = 0;
    let mut claims = vec![];
    claims.push(Claim::new("project_type", project_type));
    claims.push(Claim::new("location", location));

    for (i, (dampak, komponen, mag, imp)) in significant.iter().enumerate() {
        let sig = mag * imp;
        let mitigasi = suggest_mitigation(dampak, komponen, sig, project_type);
        let target = suggest_target(dampak, komponen, sig);
        let indikator = suggest_indicator(dampak, komponen);
        
        let (_param, frekuensi, metode, _baku_mutu) = suggest_monitoring(dampak, komponen);

        claims.push(Claim::new(&format!("rkl_mitigation_{}", i), &mitigasi));
        claims.push(Claim::new(&format!("rkl_target_{}", i), &target));
        claims.push(Claim::new(&format!("rpl_indicator_{}", i), &indikator));
        claims.push(Claim::new(&format!("rpl_freq_{}", i), frekuensi));
        claims.push(Claim::new(&format!("rpl_method_{}", i), &metode));
        rkl_count += 1;
    }

    let kpi_mitigation = (rkl_count as f64 / significant.len() as f64) * 100.0;
    let kpi_monitoring = 85.0; // PLACEHOLDER
    let kpi_compliance = 90.0; // PLACEHOLDER
    let kpi_overall = kpi_mitigation * 0.4 + kpi_monitoring * 0.35 + kpi_compliance * 0.25;

    let res_kpi = ScientificResult::new("kpi_overall", kpi_overall, "%")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("calculation", "Anggreini_Rani_2026", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("limitation", "KPI monitoring and compliance are placeholders. Requires actual field data for operational validity."))
        .with_claim(Claim::new("iso14001", "RKL maps to Clause 8.1; RPL maps to Clause 9.1; KPI maps to Clause 9.3"));

    let mut res_kpi_mut = res_kpi;
    for claim in claims {
         res_kpi_mut = res_kpi_mut.with_claim(claim);
    }

    json!([
        serde_json::from_str::<serde_json::Value>(&res_kpi_mut.emit_validated()).unwrap()
    ]).to_string()
}

fn suggest_mitigation(dampak: &str, komponen: &str, _sig: f64, project_type: &str) -> String {
    let d = dampak.to_lowercase();
    let k = komponen.to_lowercase();
    if k.contains("air") || k.contains("water") {
        if d.contains("pencemar") || d.contains("pollut") {
            "IPAL + wetland treatment".into()
        } else {
            "Sediment trap + retention pond".into()
        }
    } else if k.contains("udara") || k.contains("air_quality") {
        "Wet spray + dust suppressant".into()
    } else if k.contains("tanah") || k.contains("soil") || k.contains("land") {
        "Erosion control + revegetation".into()
    } else if k.contains("flora") || k.contains("fauna") || k.contains("bio") {
        "Revegetasi + wildlife corridor".into()
    } else if k.contains("sosial") || k.contains("social") {
        "CSR + community engagement".into()
    } else if k.contains("kebisingan") || k.contains("noise") {
        "Barrier akustik + working hours".into()
    } else if project_type.to_lowercase().contains("tambang") || project_type.to_lowercase().contains("mine") {
        "Reklamasi + mine closure plan".into()
    } else {
        "Good housekeeping + SOP".into()
    }
}

fn suggest_target(_dampak: &str, komponen: &str, _sig: f64) -> String {
    let k = komponen.to_lowercase();
    if k.contains("air") || k.contains("water") {
        "Patuhi PP 22/2021 baku mutu".into()
    } else if k.contains("udara") || k.contains("air_quality") {
        "SPU ≤ baku mutu PP 22/2021".into()
    } else if k.contains("sosial") || k.contains("social") {
        "Konflik = 0; complain < 5/bulan".into()
    } else if k.contains("bio") {
        "Revegetasi 80% area terbuka".into()
    } else {
        "Dampak residual ≤ 30% baseline".into()
    }
}

fn suggest_indicator(_dampak: &str, komponen: &str) -> String {
    let k = komponen.to_lowercase();
    if k.contains("air") || k.contains("water") {
        "TSS, pH, COD".into()
    } else if k.contains("udara") || k.contains("air_quality") {
        "PM10, SO2, NO2".into()
    } else if k.contains("tanah") || k.contains("soil") {
        "Erosion rate, cover %".into()
    } else if k.contains("bio") {
        "Survival rate, species count".into()
    } else if k.contains("sosial") || k.contains("social") {
        "Complain count, CSR spending".into()
    } else if k.contains("kebisingan") || k.contains("noise") {
        "dB(A) at receiver".into()
    } else {
        "Visual inspection + log".into()
    }
}

fn suggest_monitoring(_dampak: &str, komponen: &str) -> (String, &'static str, String, String) {
    let k = komponen.to_lowercase();
    if k.contains("air") || k.contains("water") {
        ("Kualitas air limbah".into(), "Bulanan", "SNI 6989".into(), "PP 22/2021".into())
    } else if k.contains("udara") || k.contains("air_quality") {
        ("Kualitas udara ambien".into(), "Bulanan", "Gravimetric".into(), "PP 22/2021".into())
    } else if k.contains("tanah") || k.contains("soil") {
        ("Erosi & vegetasi".into(), "Triwulanan", "USLE + transect".into(), "Permen LH".into())
    } else if k.contains("bio") {
        ("Biodiversity survey".into(), "Semester", "Point count + plot".into(), "Permen LH".into())
    } else if k.contains("sosial") || k.contains("social") {
        ("Community survey".into(), "Semester", "Kuesioner + FGD".into(), "Perda setempat".into())
    } else if k.contains("kebisingan") || k.contains("noise") {
        ("Tingkat kebisingan".into(), "Bulanan", "Sound level meter".into(), "PP 22/2021".into())
    } else {
        ("Inspeksi visual".into(), "Bulanan", "Checklist".into(), "SOP internal".into())
    }
}
