/// Indonesia emission factors (IPCC 2006 + Perpres 98/2021)
pub fn calculate(activity: &str, amount: f64) -> String {
    let (co2_kg, scope, source) = match activity.to_lowercase().as_str() {
        "electricity_kwh" => (amount * 0.794, "Scope 2", "PLN Grid EF 2023: 0.794 kgCO2/kWh"),
        "fuel_liter_diesel" | "diesel" => (amount * 2.68, "Scope 1", "IPCC 2006: 2.68 kgCO2/L diesel"),
        "fuel_liter_gasoline" | "gasoline" | "bensin" => (amount * 2.31, "Scope 1", "IPCC 2006: 2.31 kgCO2/L gasoline"),
        "lpg_kg" => (amount * 2.98, "Scope 1", "IPCC 2006: 2.98 kgCO2/kg LPG"),
        "natural_gas_m3" | "gas_m3" => (amount * 2.02, "Scope 1", "IPCC 2006: 2.02 kgCO2/m³ natural gas"),
        "waste_ton" | "waste" => (amount * 1200.0, "Scope 1/3", "Est. 1.2 tCO2e/ton mixed waste (IPCC default)"),
        "flight_km" => (amount * 0.255, "Scope 3", "DEFRA 2023: 0.255 kgCO2/passenger-km (economy)"),
        "vehicle_km" | "car_km" => (amount * 0.21, "Scope 1/3", "Avg passenger car: 0.21 kgCO2/km"),
        "rice_paddy_ha" => (amount * 5000.0, "Scope 1", "Rice paddy CH4: ~5 tCO2e/ha/season (IPCC)"),
        "deforestation_ha" => (amount * 450_000.0, "Scope 1", "Tropical forest: ~450 tCO2/ha (above+below ground)"),
        "cement_ton" => (amount * 622.0, "Scope 1", "Cement: 622 kgCO2/ton (process + combustion)"),
        _ => return format!("Unknown activity: '{}'\n\nSupported activities:\n  electricity_kwh, fuel_liter_diesel, fuel_liter_gasoline, lpg_kg,\n  natural_gas_m3, waste_ton, flight_km, vehicle_km, car_km,\n  rice_paddy_ha, deforestation_ha, cement_ton", activity),
    };

    let co2_ton = co2_kg / 1000.0;
    let carbon_price_idr = co2_ton * 30.0 * 15500.0; // NEK ~$30/tCO2 * IDR rate

    format!(
        "=== Carbon Footprint Calculator ===\nActivity: {} = {:.2} units\n{}: {:.2}\n\nEmissions:\n  {:.2} kgCO2e\n  {:.4} tCO2e\n\nCarbon Valuation (NEK Indonesia):\n  ~Rp {:.0} (@ Rp 465,000/tCO2e, Perpres 98/2021)\n\nGHG Protocol Scope: {}\n\nContext NTB:\n  - Avg household electricity: ~150 kWh/month = {:.0} kgCO2/month\n  - Avg motorcycle: 12,000 km/year = {:.0} kgCO2/year\n  - 1 ha deforestasi Rinjani = {:.0} tCO2 released",
        activity, amount, source, co2_kg,
        co2_kg, co2_ton,
        carbon_price_idr,
        scope,
        150.0 * 0.794, 12000.0 * 0.21 / 1000.0 * 1000.0, 450.0
    )
}
