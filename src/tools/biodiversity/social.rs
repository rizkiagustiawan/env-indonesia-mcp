use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Social Impact Assessment Matrix for AMDAL
/// Ref: PermenLH 17/2012 (Partisipasi Masyarakat)

pub fn impact_matrix(impacts_json: &str) -> String {
    let impacts: Vec<serde_json::Value> = match serde_json::from_str(impacts_json) {
        Ok(v) => v,
        Err(e) => return json!({"error": "E100", "message": format!("Error parsing JSON: {}", e)}).to_string(),
    };

    if impacts.is_empty() {
        return json!({"error": "E102", "message": "Daftar dampak kosong"}).to_string();
    }

    let mut total_positive = 0.0_f64;
    let mut total_negative = 0.0_f64;
    let mut component_scores: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
    let mut claims = vec![];

    for (i, impact) in impacts.iter().enumerate() {
        let component = impact.get("component").and_then(|v| v.as_str()).unwrap_or("lainnya");
        let impact_desc = impact.get("impact").and_then(|v| v.as_str()).unwrap_or("Tidak diketahui");
        let magnitude = impact.get("magnitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let importance = impact.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let score = magnitude * importance;

        if magnitude >= 0.0 {
            total_positive += score;
        } else {
            total_negative += score;
        }

        let entry = component_scores.entry(component.to_string()).or_insert((0.0, 0.0));
        if magnitude >= 0.0 {
            entry.0 += score;
        } else {
            entry.1 += score;
        }

        claims.push(Claim::new(&format!("impact_{}", i), &format!("Component: {}, Desc: {}, Score: {:.2}", component, impact_desc, score)));
    }

    let total_score = total_positive + total_negative;
    let assessment = if total_score > 20.0 {
        "LAYAK"
    } else if total_score > 0.0 {
        "LAYAK BERSYARAT"
    } else if total_score > -20.0 {
        "PERLU PERHATIAN"
    } else {
        "PERLU KAJIAN ULANG"
    };

    let status = if total_score > -20.0 { ResultStatus::ValidWithAssumptions } else { ResultStatus::ValidationFailed };

    let mut res = ScientificResult::new("leopold_net_score", total_score, "index")
        .with_status(status)
        .with_provenance(Provenance::new("calculation", "PermenLH_17_2012", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("total_positive", &total_positive.to_string()))
        .with_claim(Claim::new("total_negative", &total_negative.to_string()))
        .with_claim(Claim::new("assessment", assessment));

    for claim in claims {
        res = res.with_claim(claim);
    }

    for (comp, (pos, neg)) in &component_scores {
        res = res.with_claim(Claim::new(&format!("component_{}_net", comp), &(pos + neg).to_string()));
    }

    json!([serde_json::from_str::<serde_json::Value>(&res.emit_validated()).unwrap()]).to_string()
}

pub fn health_impact(
    _population: u64,
    pollutant: &str,
    concentration: f64,
    exposure_hours: f64,
) -> String {
    struct PollutantRef {
        name: &'static str,
        unit: &'static str,
        rfd: f64,
        _rfd_unit: &'static str,
        cancer_slope: Option<f64>,
        ir: f64,
        _description: &'static str,
    }

    let pollutants = vec![
        PollutantRef { name: "pm25", unit: "µg/m³", rfd: 0.0035, _rfd_unit: "mg/kg/hari", cancer_slope: None, ir: 20.0, _description: "PM2.5" },
        PollutantRef { name: "pm10", unit: "µg/m³", rfd: 0.005, _rfd_unit: "mg/kg/hari", cancer_slope: None, ir: 20.0, _description: "PM10" },
        PollutantRef { name: "so2", unit: "µg/m³", rfd: 0.02, _rfd_unit: "mg/kg/hari", cancer_slope: None, ir: 20.0, _description: "SO2" },
        PollutantRef { name: "no2", unit: "µg/m³", rfd: 0.02, _rfd_unit: "mg/kg/hari", cancer_slope: None, ir: 20.0, _description: "NO2" },
        PollutantRef { name: "co", unit: "mg/m³", rfd: 0.057, _rfd_unit: "mg/kg/hari", cancer_slope: None, ir: 20.0, _description: "CO" },
        PollutantRef { name: "benzene", unit: "µg/m³", rfd: 0.004, _rfd_unit: "mg/kg/hari", cancer_slope: Some(0.029), ir: 20.0, _description: "Benzene" },
        PollutantRef { name: "toluene", unit: "µg/m³", rfd: 0.08, _rfd_unit: "mg/kg/hari", cancer_slope: None, ir: 20.0, _description: "Toluene" },
        PollutantRef { name: "pb", unit: "µg/m³", rfd: 0.00036, _rfd_unit: "mg/kg/hari", cancer_slope: Some(0.042), ir: 20.0, _description: "Pb" },
        PollutantRef { name: "h2s", unit: "µg/m³", rfd: 0.003, _rfd_unit: "mg/kg/hari", cancer_slope: None, ir: 20.0, _description: "H2S" },
        PollutantRef { name: "nh3", unit: "µg/m³", rfd: 0.1, _rfd_unit: "mg/kg/hari", cancer_slope: None, ir: 20.0, _description: "NH3" },
    ];

    let query = pollutant.to_lowercase();
    let pol = match pollutants.iter().find(|p| p.name == query) {
        Some(p) => p,
        None => return json!({"error": "E100", "message": format!("Polutan '{}' tidak ditemukan", pollutant)}).to_string(),
    };

    let bw = 70.0_f64; 
    let at_noncarc = 365.0 * 30.0;
    let ef = 350.0; 
    let ed = 30.0; 
    let et = exposure_hours; 
    let ir = pol.ir; 

    let conc_mg = if pol.unit == "µg/m³" { concentration / 1000.0 } else { concentration };

    let add_noncarc = conc_mg * ir * (et / 24.0) * ef * ed / (bw * at_noncarc);
    let hq = add_noncarc / pol.rfd;

    let mut results = vec![];

    let res_hq = ScientificResult::new("hazard_quotient", hq, "dimensionless")
        .with_status(if hq <= 1.0 { ResultStatus::Valid } else { ResultStatus::ValidationFailed })
        .with_provenance(Provenance::new("calculation", "EPA_IRIS", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("add_mg_kg_d", &add_noncarc.to_string()))
        .with_claim(Claim::new("rfd", &pol.rfd.to_string()));
    
    results.push(res_hq);

    if let Some(csf) = pol.cancer_slope {
        let at_carc = 365.0 * 70.0;
        let add_carc = conc_mg * ir * (et / 24.0) * ef * ed / (bw * at_carc);
        let cancer_risk = add_carc * csf;
        
        let res_cancer = ScientificResult::new("excess_cancer_risk", cancer_risk, "probability")
            .with_status(if cancer_risk <= 1e-4 { ResultStatus::Valid } else { ResultStatus::ValidationFailed })
            .with_provenance(Provenance::new("calculation", "EPA_IRIS", "2026-08-19T00:00:00Z"))
            .with_claim(Claim::new("csf", &csf.to_string()));
            
        results.push(res_cancer);
    }

    let json_array: Vec<serde_json::Value> = results.iter()
        .map(|r| serde_json::from_str(&r.clone().emit_validated()).unwrap())
        .collect();

    json!(json_array).to_string()
}
