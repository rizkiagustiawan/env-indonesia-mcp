use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Desain Sistem Liner TPA
/// Ref: PermenPU 3/2013 tentang Penyelenggaraan Prasarana dan Sarana Persampahan
/// Ref: Giroud & Bonaparte (1989) — Leakage through Liners

pub fn design(
    liner_type: &str,
    area_m2: f64,
    head_on_liner_m: f64,
    k_clay: f64,
    clay_thickness_m: f64,
) -> String {
    if area_m2 <= 0.0 {
        return json!({"error": "E102", "message": "Parameter area harus > 0"}).to_string();
    }
    if head_on_liner_m < 0.0 {
        return json!({"error": "E102", "message": "Parameter head_on_liner_m tidak boleh negatif"}).to_string();
    }
    if k_clay <= 0.0 {
        return json!({"error": "E102", "message": "Parameter k_clay harus > 0"}).to_string();
    }
    if clay_thickness_m <= 0.0 {
        return json!({"error": "E102", "message": "Parameter clay_thickness_m harus > 0"}).to_string();
    }

    let liner_lower = liner_type.to_lowercase();
    let (_liner_name, leakage_rate_m3_s, geomembrane_mm, description) = match liner_lower.as_str() {
        "single_clay" => {
            // Q = k × i × A, i = (h + d) / d
            let gradient = (head_on_liner_m + clay_thickness_m) / clay_thickness_m;
            let q = k_clay * gradient * area_m2; // m³/s
            (
                "Single Clay Liner",
                q,
                0.0,
                "Liner tunggal lempung padat. Cocok untuk TPA kelas III (non-B3, rendah risiko).",
            )
        }
        "composite" => {
            // Giroud-Bonaparte (good contact):
            // Q = 0.21 × a^0.1 × h^0.9 × ks^0.74
            // a = defect area per hole ≈ 1 cm² = 1e-4 m², assume 2.5 holes/ha
            let defect_area = 1e-4_f64; // m² per hole
            let holes_per_m2 = 2.5 / 10000.0; // 2.5 holes per hectare
            let n_holes = (area_m2 * holes_per_m2).max(1.0);
            let q_per_hole = 0.21
                * defect_area.powf(0.1)
                * head_on_liner_m.max(0.01).powf(0.9)
                * k_clay.powf(0.74);
            let q = q_per_hole * n_holes;
            (
                "Composite Liner (Geomembrane + Clay)",
                q,
                1.5,
                "Geomembrane HDPE di atas lapisan clay padat. Standar TPA kelas I.",
            )
        }
        "double_liner" => {
            // Double liner: primary + leak detection + secondary
            // Very low leakage, use composite formula with extra reduction factor
            let defect_area = 1e-4_f64;
            let holes_per_m2 = 1.0 / 10000.0; // stricter QC, 1 hole/ha
            let n_holes = (area_m2 * holes_per_m2).max(1.0);
            let q_per_hole = 0.21
                * defect_area.powf(0.1)
                * head_on_liner_m.max(0.01).powf(0.9)
                * k_clay.powf(0.74);
            let q = q_per_hole * n_holes * 0.1; // secondary liner reduces by ~10x
            (
                "Double Liner (Primary + LDS + Secondary)",
                q,
                2.0,
                "Sistem liner ganda dengan leak detection system. Wajib untuk TPA B3.",
            )
        }
        _ => {
            return json!({"error": "E100", "message": format!("Tipe liner '{}' tidak dikenal", liner_type)}).to_string();
        }
    };

    let leakage_l_day = leakage_rate_m3_s * 86400.0 * 1000.0;
    let leakage_l_ha_day = if area_m2 > 0.0 {
        leakage_l_day / (area_m2 / 10000.0)
    } else {
        0.0
    };

    let res_leakage_m3_s = ScientificResult::new("leakage_rate", leakage_rate_m3_s, "m3/s")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "Giroud_Bonaparte_1989", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("methodology", description));

    let res_leakage_l_day = ScientificResult::new("leakage_rate_daily", leakage_l_day, "L/day")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "Giroud_Bonaparte_1989", "2026-08-19T00:00:00Z"));

    let res_leakage_l_ha_day = ScientificResult::new("leakage_rate_per_ha", leakage_l_ha_day, "L/ha/day")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "Giroud_Bonaparte_1989", "2026-08-19T00:00:00Z"));

    let res_geomembrane = ScientificResult::new("geomembrane_thickness", geomembrane_mm, "mm")
        .with_status(ResultStatus::Valid)
        .with_provenance(Provenance::new("specification", "PermenPU_3_2013", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("material", "HDPE"));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_leakage_m3_s.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_leakage_l_day.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_leakage_l_ha_day.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_geomembrane.emit_validated()).unwrap()
    ]).to_string()
}
