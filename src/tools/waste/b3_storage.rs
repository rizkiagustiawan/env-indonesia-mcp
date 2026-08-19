use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Persyaratan TPS Limbah B3 (Tempat Penyimpanan Sementara)
/// Ref: PP 101/2014 tentang Pengelolaan Limbah B3

pub fn calculate(waste_type: &str, volume_m3_per_month: f64, density_kg_m3: f64) -> String {
    if volume_m3_per_month <= 0.0 {
        return json!({"error": "E102", "message": "Parameter volume harus > 0"}).to_string();
    }
    if density_kg_m3 <= 0.0 {
        return json!({"error": "E102", "message": "Parameter densitas harus > 0"}).to_string();
    }

    let wt_lower = waste_type.to_lowercase();

    let (_type_name, _max_storage_days_cat1, max_storage_days_cat2, stack_height_m, _container_desc) =
        match wt_lower.as_str() {
            "padat" => (
                "Padat",
                90,
                180,
                3.0,
                "Drum 200L (standar) atau kontainer tertutup",
            ),
            "cair" => (
                "Cair",
                90,
                180,
                2.0, // single stack drums
                "Drum 200L HDPE/baja atau tangki IBC 1000L",
            ),
            "lumpur" => (
                "Lumpur (Sludge)",
                90,
                180,
                2.0,
                "Drum 200L tertutup atau bak penampung berlapis",
            ),
            "gas" => (
                "Gas (Tabung Bertekanan)",
                90,
                180,
                1.5, // single row
                "Tabung gas bertekanan standar DOT/SNI",
            ),
            _ => {
                return json!({"error": "E100", "message": format!("Jenis limbah '{}' tidak dikenal", waste_type)}).to_string();
            }
        };

    let mass_kg_per_month = volume_m3_per_month * density_kg_m3;
    let mass_ton_per_month = mass_kg_per_month / 1000.0;

    // Storage volume for max duration (kategori 2 = 180 days = 6 months)
    let max_stored_volume_m3 = volume_m3_per_month * (max_storage_days_cat2 as f64 / 30.0);
    let floor_area_m2 = max_stored_volume_m3 / stack_height_m;

    // Aisle factor: 40% additional for access
    let floor_area_with_aisle = floor_area_m2 * 1.4;

    // Containment: 110% of largest single container or 25% of total, whichever greater
    let drum_volume_m3 = 0.2; // 200L drum
    let containment_110: f64 = drum_volume_m3 * 1.1;
    let containment_25pct: f64 = max_stored_volume_m3 * 0.25;
    let containment_volume = containment_110.max(containment_25pct);

    // Number of drums (200L)
    let n_drums = (max_stored_volume_m3 / drum_volume_m3).ceil() as f64;

    let res_mass_month = ScientificResult::new("mass_per_month", mass_ton_per_month, "ton/month")
        .with_status(ResultStatus::Valid)
        .with_provenance(Provenance::new("calculation", "Volume_Density_Product", "2026-08-19T00:00:00Z"));

    let res_max_vol = ScientificResult::new("max_stored_volume", max_stored_volume_m3, "m3")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "PP_101_2014_Cat2", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("limitation", "Assumes category 2 maximum storage duration of 180 days"));

    let res_area = ScientificResult::new("required_floor_area", floor_area_with_aisle, "m2")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "Area_with_Aisle_Factor", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("limitation", "Includes 40% additional area for aisle access"));

    let res_containment = ScientificResult::new("bund_volume", containment_volume, "m3")
        .with_status(ResultStatus::Valid)
        .with_provenance(Provenance::new("calculation", "Containment_Criteria", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("rule", "110% of largest container or 25% of total, whichever is greater"));

    let res_drums = ScientificResult::new("estimated_200L_drums", n_drums, "count")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "Drum_Count", "2026-08-19T00:00:00Z"));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_mass_month.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_max_vol.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_area.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_containment.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_drums.emit_validated()).unwrap()
    ]).to_string()
}
