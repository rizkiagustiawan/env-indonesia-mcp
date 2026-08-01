/// GHG Inventory Calculator
/// Ref: PermenLHK 102/2018, IPCC 2006 Tier 1

pub fn calculate(sector: &str, activity: &str, amount: f64) -> String {
    let s = sector.to_lowercase();
    let a = activity.to_lowercase();

    if amount < 0.0 {
        return format!("ERROR [E102]: Parameter tidak boleh negatif. {}", amount);
    }

    // GWP AR5: CO2=1, CH4=28, N2O=265
    let gwp_ch4: f64 = 28.0;
    let gwp_n2o: f64 = 265.0;

    // Emission factors: (CO2 kg/unit, CH4 kg/unit, N2O kg/unit, unit_desc)
    let result: Option<(f64, f64, f64, &str)> = match (s.as_str(), a.as_str()) {
        // === ENERGY ===
        ("energy" | "energi", "electricity_kwh" | "listrik_kwh") => {
            // Grid emission factor Indonesia ~0.794 kg CO2/kWh (2019)
            Some((0.794, 0.0, 0.0, "kWh"))
        }
        ("energy" | "energi", "diesel_liter" | "solar_liter") => {
            // Diesel: ~2.676 kg CO2/L, 0.00039 kg CH4/L, 0.00039 kg N2O/L
            Some((2.676, 0.00039, 0.00039, "liter"))
        }
        ("energy" | "energi", "gasoline_liter" | "bensin_liter") => {
            // Gasoline: ~2.331 kg CO2/L, 0.00046 kg CH4/L, 0.00022 kg N2O/L
            Some((2.331, 0.00046, 0.00022, "liter"))
        }
        ("energy" | "energi", "lpg_kg") => {
            // LPG: ~2.983 kg CO2/kg, 0.00005 kg CH4/kg, 0.0001 kg N2O/kg
            Some((2.983, 0.00005, 0.0001, "kg"))
        }
        ("energy" | "energi", "natural_gas_m3" | "gas_alam_m3") => {
            // Natural gas: ~1.885 kg CO2/m³, 0.00001 kg CH4/m³, 0.0001 kg N2O/m³
            Some((1.885, 0.00001, 0.0001, "m³"))
        }
        ("energy" | "energi", "coal_kg" | "batubara_kg") => {
            // Coal: ~2.42 kg CO2/kg, 0.00001 kg CH4/kg, 0.00005 kg N2O/kg
            Some((2.42, 0.00001, 0.00005, "kg"))
        }
        // === IPPU ===
        ("ippu", "cement_ton" | "semen_ton") => {
            // Cement clinker: ~520 kg CO2/ton
            Some((520.0, 0.0, 0.0, "ton"))
        }
        ("ippu", "lime_ton" | "kapur_ton") => {
            // Lime: ~750 kg CO2/ton
            Some((750.0, 0.0, 0.0, "ton"))
        }
        // === AFOLU ===
        ("afolu", "deforestation_ha" | "deforestasi_ha") => {
            // IPCC 2006 Ch4 Table 4.7: tropical rainforest default ~120-200 tC/ha
            // above-ground + below-ground (~20%) ≈ 450 tCO2/ha total
            Some((450_000.0, 0.0, 0.0, "ha"))
        }
        ("afolu", "rice_paddy_ha" | "sawah_ha") => {
            // Rice paddy CH4: ~190 kg CH4/ha/season
            Some((0.0, 190.0, 0.0, "ha"))
        }
        ("afolu", "peatland_ha" | "gambut_ha") => {
            // Drained peatland: ~55 t CO2/ha/yr + 2.7 kg CH4/ha/yr
            Some((55_000.0, 2.7, 0.0, "ha"))
        }
        // === WASTE ===
        ("waste" | "limbah", "landfill_ton" | "tpa_ton") => {
            // Landfill: ~50 kg CH4/ton waste (tropical, unmanaged)
            Some((0.0, 50.0, 0.0, "ton"))
        }
        ("waste" | "limbah", "open_burning_ton" | "pembakaran_terbuka_ton") => {
            // Open burning: ~1200 kg CO2/ton, 6.5 kg CH4/ton, 0.15 kg N2O/ton
            Some((1200.0, 6.5, 0.15, "ton"))
        }
        ("waste" | "limbah", "wastewater_bod_kg" | "air_limbah_bod_kg") => {
            // Wastewater CH4: ~0.21 kg CH4/kg BOD removed (tropical, anaerobic)
            Some((0.0, 0.21, 0.0, "kg BOD"))
        }
        _ => None,
    };

    let (ef_co2, ef_ch4, ef_n2o, unit) = match result {
        Some(v) => v,
        None => return format!(
            "ERROR: Kombinasi sektor '{}' dan aktivitas '{}' tidak ditemukan.\n\
             Sektor & aktivitas valid:\n\
             energy: electricity_kwh, diesel_liter, gasoline_liter, lpg_kg, natural_gas_m3, coal_kg\n\
             ippu: cement_ton, lime_ton\n\
             afolu: deforestation_ha, rice_paddy_ha, peatland_ha\n\
             waste: landfill_ton, open_burning_ton, wastewater_bod_kg",
            sector, activity
        ),
    };

    let co2 = ef_co2 * amount;
    let ch4 = ef_ch4 * amount;
    let n2o = ef_n2o * amount;
    let co2e = co2 + (ch4 * gwp_ch4) + (n2o * gwp_n2o);

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  GHG Inventory Calculator\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PermenLHK 102/2018, IPCC 2006 Tier 1\n\n");
    out.push_str(&format!("Sektor    : {}\n", sector));
    out.push_str(&format!("Aktivitas : {}\n", activity));
    out.push_str(&format!("Jumlah    : {:.2} {}\n\n", amount, unit));
    out.push_str("Faktor Emisi:\n");
    out.push_str(&format!("  CO2 : {:.4} kg/{}\n", ef_co2, unit));
    out.push_str(&format!("  CH4 : {:.6} kg/{}\n", ef_ch4, unit));
    out.push_str(&format!("  N2O : {:.6} kg/{}\n\n", ef_n2o, unit));
    out.push_str("Emisi:\n");
    out.push_str(&format!("  CO2 : {:.2} kg\n", co2));
    out.push_str(&format!("  CH4 : {:.4} kg\n", ch4));
    out.push_str(&format!("  N2O : {:.6} kg\n\n", n2o));
    out.push_str(&format!(
        "GWP (AR5): CO2=1, CH4={}, N2O={}\n",
        gwp_ch4 as i32, gwp_n2o as i32
    ));
    out.push_str(&format!(
        "CO2-equivalent: {:.2} kg = {:.4} ton CO2e\n\n",
        co2e,
        co2e / 1000.0
    ));

    if co2e >= 1_000_000.0 {
        out.push_str(&format!("  ≈ {:.2} kiloton CO2e\n", co2e / 1_000_000.0));
    }

    out
}
