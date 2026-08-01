/// Transport Emission Calculator (IPCC Basis Volume BBM)
/// Ref: 2006 IPCC Guidelines for National Greenhouse Gas Inventories

pub fn calculate(fuel_type: &str, liters: f64) -> String {
    if liters < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }

    let (ef_kg_l, ncv_tj_gq, fuel_name) = match fuel_type.to_lowercase().as_str() {
        "bensin" | "gasoline" | "pertalite" | "pertamax" => (2.31, 44.3, "Motor Gasoline (Bensin)"),
        "solar" | "diesel" | "dexlite" => (2.68, 43.0, "Gas/Diesel Oil (Solar)"),
        "avtur" | "jet_fuel" => (2.52, 44.1, "Jet Kerosene (Avtur)"),
        _ => {
            return format!(
                "Tipe bahan bakar '{}' tidak valid. Gunakan: bensin, solar, atau avtur.",
                fuel_type
            )
        }
    };

    let co2_kg = liters * ef_kg_l;
    let co2_ton = co2_kg / 1000.0;

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Transport Emission (IPCC)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: 2006 IPCC Guidelines (Volume-based)\n");
    out.push_str("⚠️ Estimasi per kilometer tidak akurat tanpa spesifikasi mesin. Gunakan basis liter (pembakaran mutlak).\n\n");

    out.push_str(&format!(
        "Bahan Bakar: {}\nVolume: {:.2} Liter\n\n",
        fuel_name, liters
    ));
    out.push_str(&format!(
        "Faktor Emisi (Default): {:.2} kg CO₂/Liter\nNet Calorific Value: {:.1} TJ/Gg\n\n",
        ef_kg_l, ncv_tj_gq
    ));

    out.push_str(&format!(
        "Total Emisi CO₂:\n  = {:.2} kg CO₂\n  = {:.4} ton CO₂\n",
        co2_kg, co2_ton
    ));

    out
}
