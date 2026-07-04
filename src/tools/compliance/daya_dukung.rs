/// Daya Dukung Lingkungan Hidup
/// Ref: PermenLH 17/2009

pub fn calculate(
    approach: &str,
    area_ha: f64,
    population: f64,
    water_supply_m3_yr: Option<f64>,
    water_demand_m3_yr: Option<f64>,
    food_production_ton_yr: Option<f64>,
    food_demand_ton_yr: Option<f64>,
) -> String {
    let a = approach.to_lowercase();

    if area_ha <= 0.0 {
        return format!("ERROR: Luas wilayah ({:.1} ha) harus > 0.", area_ha);
    }
    if population <= 0.0 {
        return format!("ERROR: Jumlah penduduk ({:.0}) harus > 0.", population);
    }

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Daya Dukung Lingkungan Hidup\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PermenLH No. 17 Tahun 2009\n\n");
    out.push_str(&format!("Luas Wilayah : {:.1} ha\n", area_ha));
    out.push_str(&format!("Penduduk     : {:.0} jiwa\n\n", population));

    match a.as_str() {
        "population" | "penduduk" => {
            // Asumsi: kebutuhan lahan per kapita = 0.07 ha (700 m²) untuk kebutuhan dasar
            let land_per_capita_ha = 0.07;
            let optimal_pop = area_ha / land_per_capita_ha;
            let ratio = population / optimal_pop;
            let status = if ratio <= 1.0 { "Masih Mampu (DDL Belum Terlampaui) ✅" } else { "Terlampaui (DDL Terlampaui) ❌" };

            out.push_str("Pendekatan: Berbasis Populasi\n\n");
            out.push_str(&format!("Kebutuhan lahan/kapita : {:.2} ha\n", land_per_capita_ha));
            out.push_str(&format!("Daya Dukung Populasi   : {:.0} jiwa\n", optimal_pop));
            out.push_str(&format!("Populasi Aktual        : {:.0} jiwa\n", population));
            out.push_str(&format!("Rasio (aktual/DDL)     : {:.2}\n\n", ratio));
            out.push_str(&format!("Status: {}\n", status));
        }
        "water" | "air" => {
            let supply = match water_supply_m3_yr {
                Some(v) if v > 0.0 => v,
                _ => return "ERROR: water_supply_m3_yr harus diisi dan > 0 untuk pendekatan air.".into(),
            };
            let demand_per_cap = match water_demand_m3_yr {
                Some(v) if v > 0.0 => v,
                _ => {
                    // Default: 60 L/hari * 365 = 21.9 m³/tahun/kapita (kebutuhan dasar WHO)
                    21.9
                }
            };
            let total_demand = demand_per_cap * population;
            let ddl_pop = supply / demand_per_cap;
            let ratio = total_demand / supply;
            let status = if ratio <= 1.0 { "Masih Mampu (DDL Belum Terlampaui) ✅" } else { "Terlampaui (DDL Terlampaui) ❌" };

            out.push_str("Pendekatan: Berbasis Sumber Daya Air\n\n");
            out.push_str(&format!("Pasokan Air            : {:.0} m³/tahun\n", supply));
            out.push_str(&format!("Kebutuhan Air/kapita   : {:.1} m³/tahun\n", demand_per_cap));
            out.push_str(&format!("Total Kebutuhan Air    : {:.0} m³/tahun\n", total_demand));
            out.push_str(&format!("DDL Populasi (air)     : {:.0} jiwa\n", ddl_pop));
            out.push_str(&format!("Rasio (demand/supply)  : {:.2}\n\n", ratio));
            out.push_str(&format!("Status: {}\n", status));
        }
        "food" | "pangan" => {
            let production = match food_production_ton_yr {
                Some(v) if v > 0.0 => v,
                _ => return "ERROR: food_production_ton_yr harus diisi dan > 0 untuk pendekatan pangan.".into(),
            };
            let demand_per_cap = match food_demand_ton_yr {
                Some(v) if v > 0.0 => v,
                _ => {
                    // Default: ~0.3 ton beras/kapita/tahun (Indonesia average)
                    0.3
                }
            };
            let total_demand = demand_per_cap * population;
            let ddl_pop = production / demand_per_cap;
            let ratio = total_demand / production;
            let status = if ratio <= 1.0 { "Masih Mampu (DDL Belum Terlampaui) ✅" } else { "Terlampaui (DDL Terlampaui) ❌" };

            out.push_str("Pendekatan: Berbasis Produksi Pangan\n\n");
            out.push_str(&format!("Produksi Pangan        : {:.0} ton/tahun\n", production));
            out.push_str(&format!("Kebutuhan/kapita       : {:.2} ton/tahun\n", demand_per_cap));
            out.push_str(&format!("Total Kebutuhan Pangan : {:.0} ton/tahun\n", total_demand));
            out.push_str(&format!("DDL Populasi (pangan)  : {:.0} jiwa\n", ddl_pop));
            out.push_str(&format!("Rasio (demand/prod)    : {:.2}\n\n", ratio));
            out.push_str(&format!("Status: {}\n", status));
        }
        _ => {
            return format!(
                "ERROR: Pendekatan '{}' tidak valid.\nPilihan: population, water, food",
                approach
            );
        }
    }

    out
}
