use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Water Footprint Calculator
/// Ref: ISO 14046, Water Footprint Network (Hoekstra et al., 2011)

pub fn calculate(product: &str, quantity: f64, _unit: &str) -> String {
    if quantity <= 0.0 {
        return json!({"error": "E102", "message": "Parameter harus > 0"}).to_string();
    }

    // Water footprint database (L per unit)
    // Format: (blue, green, grey, unit_desc, source)
    let (blue, green, grey, _unit_desc, source) = match product.to_lowercase().as_str() {
        "rice" | "beras" | "padi" => (
            341.0,
            1710.0,
            449.0,
            "kg",
            "Mekonnen & Hoekstra (2011), global avg",
        ),
        "palm_oil" | "sawit" | "minyak_sawit" => {
            (20.0, 4550.0, 430.0, "kg", "Mekonnen & Hoekstra (2011)")
        }
        "rubber" | "karet" => (100.0, 14800.0, 1100.0, "kg", "Mekonnen & Hoekstra (2011)"),
        "coffee" | "kopi" => (1100.0, 15300.0, 2500.0, "kg", "Mekonnen & Hoekstra (2011)"),
        "beef" | "sapi" | "daging_sapi" => {
            (550.0, 14200.0, 650.0, "kg", "Mekonnen & Hoekstra (2012)")
        }
        "chicken" | "ayam" | "daging_ayam" => {
            (313.0, 3545.0, 442.0, "kg", "Mekonnen & Hoekstra (2012)")
        }
        "egg" | "telur" => (244.0, 2592.0, 464.0, "kg", "Mekonnen & Hoekstra (2012)"),
        "milk" | "susu" => (86.0, 863.0, 72.0, "L", "Mekonnen & Hoekstra (2012)"),
        "cotton" | "kapas" => (4482.0, 4235.0, 1283.0, "kg", "Mekonnen & Hoekstra (2011)"),
        "paper" | "kertas" => (768.0, 8282.0, 950.0, "kg", "Van Oel & Hoekstra (2012)"),
        "steel" | "baja" => (3400.0, 0.0, 9600.0, "ton", "WSA (2019)"),
        "cement" | "semen" => (130.0, 0.0, 670.0, "ton", "Gerbens-Leenes et al. (2009)"),
        "electricity_coal" | "listrik_batubara" => (1.5, 0.0, 0.5, "kWh", "Mekonnen et al. (2015)"),
        "electricity_gas" | "listrik_gas" => (0.4, 0.0, 0.1, "kWh", "Mekonnen et al. (2015)"),
        "tobacco" | "tembakau" => (
            205.0,
            2375.0,
            45.0,
            "kg",
            "Mekonnen & Hoekstra (2011), NTB crop",
        ),
        "corn" | "jagung" => (81.0, 947.0, 194.0, "kg", "Mekonnen & Hoekstra (2011)"),
        "sugar" | "gula" | "tebu" => (57.0, 1168.0, 275.0, "kg", "Mekonnen & Hoekstra (2011)"),
        _ => {
            return json!({"error": "E100", "message": format!("Produk '{}' tidak ditemukan", product)}).to_string();
        }
    };

    let total_wf_per_unit = blue + green + grey;
    let total_wf = total_wf_per_unit * quantity;
    let blue_total = blue * quantity;
    let green_total = green * quantity;
    let grey_total = grey * quantity;

    let blue_pct = if total_wf > 0.0 {
        blue_total / total_wf * 100.0
    } else {
        0.0
    };
    
    let res_wf = ScientificResult::new("total_water_footprint", total_wf, "L")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("database", source, "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("blue_water_L", &blue_total.to_string()))
        .with_claim(Claim::new("green_water_L", &green_total.to_string()))
        .with_claim(Claim::new("grey_water_L", &grey_total.to_string()))
        .with_claim(Claim::new("blue_pct", &blue_pct.to_string()));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_wf.emit_validated()).unwrap()
    ]).to_string()
}
