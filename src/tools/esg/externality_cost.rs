/// Environmental Externality / Damage Cost Calculator
/// Ref: EPA Social Cost of Carbon (2023), ExternE, UU 32/2009

fn fmt_num(v: f64) -> String {
    let s = format!("{:.0}", v.abs());
    let bytes: Vec<u8> = s.bytes().collect();
    let mut result = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 { result.push('.'); }
        result.push(*b as char);
    }
    if v < 0.0 { format!("-{}", result) } else { result }
}

fn fmt_usd(v: f64) -> String {
    let s = format!("{:.2}", v.abs());
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let bytes: Vec<u8> = int_part.bytes().collect();
    let mut result = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 { result.push(','); }
        result.push(*b as char);
    }
    let formatted = format!("{}.{}", result, parts[1]);
    if v < 0.0 { format!("-{}", formatted) } else { formatted }
}

pub fn calculate(pollutant: &str, amount: f64, unit: &str, location_type: &str) -> String {
    if amount <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    // Location multiplier for health damage (population density effect)
    let (loc_mult, loc_desc) = match location_type.to_lowercase().as_str() {
        "urban" | "perkotaan" | "kota" => (2.0, "Urban / Perkotaan"),
        "suburban" | "pinggiran" => (1.0, "Suburban / Pinggiran"),
        "rural" | "pedesaan" | "desa" => (0.4, "Rural / Pedesaan"),
        _ => return format!("ERROR: Location type '{}' tidak dikenal. Gunakan: urban/perkotaan, suburban/pinggiran, rural/pedesaan.", location_type),
    };

    // Convert amount to tons
    let amount_ton = match unit.to_lowercase().as_str() {
        "ton" | "tonnes" => amount,
        "kg" => amount / 1000.0,
        "gram" | "g" => amount / 1_000_000.0,
        _ => return format!("ERROR: Unit '{}' tidak dikenal. Gunakan: ton, kg, gram.", unit),
    };

    // Damage cost data: (base_usd_per_ton, health_pct, ecosystem_pct, material_pct, climate_pct, source)
    let (base_cost, health_pct, ecosystem_pct, material_pct, climate_pct, pollutant_desc, source) =
        match pollutant.to_lowercase().as_str() {
            "co2" | "carbon_dioxide" | "karbon_dioksida" => (
                51.0, 0.0, 10.0, 0.0, 90.0,
                "CO₂ (Karbon Dioksida)",
                "EPA Social Cost of Carbon 2023 (3% discount rate)"
            ),
            "so2" | "sulfur_dioxide" | "sulfur_dioksida" => (
                8500.0, 55.0, 20.0, 25.0, 0.0,
                "SO₂ (Sulfur Dioksida)",
                "ExternE (EU), WHO (2015)"
            ),
            "nox" | "nitrogen_oxide" | "nitrogen_oksida" => (
                5500.0, 50.0, 35.0, 15.0, 0.0,
                "NOx (Nitrogen Oksida)",
                "ExternE (EU), US EPA"
            ),
            "pm25" | "pm2.5" | "particulate_25" => (
                30000.0, 85.0, 5.0, 10.0, 0.0,
                "PM2.5 (Partikulat Halus)",
                "WHO (2021), GBD Study"
            ),
            "pm10" | "particulate_10" => (
                12000.0, 75.0, 10.0, 15.0, 0.0,
                "PM10 (Partikulat Kasar)",
                "WHO (2021), ExternE"
            ),
            "bod" => (
                350.0, 30.0, 50.0, 0.0, 20.0,
                "BOD (Biochemical Oxygen Demand)",
                "World Bank water treatment cost proxy"
            ),
            "cod" => (
                500.0, 25.0, 55.0, 0.0, 20.0,
                "COD (Chemical Oxygen Demand)",
                "World Bank water treatment cost proxy"
            ),
            "nh3" | "ammonia" | "amonia" => (
                4000.0, 40.0, 45.0, 15.0, 0.0,
                "NH₃ (Amonia)",
                "ExternE, eutrophication damage"
            ),
            "ch4" | "methane" | "metana" => (
                1500.0, 5.0, 5.0, 0.0, 90.0,
                "CH₄ (Metana)",
                "EPA Social Cost of Methane (GWP28)"
            ),
            "n2o" | "nitrous_oxide" => (
                18000.0, 5.0, 5.0, 0.0, 90.0,
                "N₂O (Dinitrogen Oksida)",
                "EPA Social Cost (GWP265)"
            ),
            "pb" | "lead" | "timbal" => (
                50000.0, 90.0, 5.0, 5.0, 0.0,
                "Pb (Timbal)",
                "WHO, IQ damage valuation"
            ),
            "hg" | "mercury" | "merkuri" => (
                80000.0, 85.0, 10.0, 5.0, 0.0,
                "Hg (Merkuri)",
                "Minamata Convention, neurological damage"
            ),
            _ => return format!(
                "ERROR: Polutan '{}' tidak ditemukan.\n\nPolutan tersedia:\n  Udara: co2, so2, nox, pm25, pm10, nh3, ch4, n2o\n  Air: bod, cod\n  Logam berat: pb (timbal), hg (merkuri)",
                pollutant
            ),
        };

    // Apply location multiplier to health-related portion
    let health_damage = base_cost * (health_pct / 100.0) * loc_mult;
    let ecosystem_damage = base_cost * (ecosystem_pct / 100.0);
    let material_damage = base_cost * (material_pct / 100.0);
    let climate_damage = base_cost * (climate_pct / 100.0);
    let adjusted_cost = health_damage + ecosystem_damage + material_damage + climate_damage;

    let total_damage_usd = adjusted_cost * amount_ton;
    let usd_to_idr = 15_500.0; // approximate
    let total_damage_idr = total_damage_usd * usd_to_idr;

    // Indonesian carbon tax comparison
    let indo_carbon_tax = 30_000.0; // IDR/ton CO2e (Perpres 98/2021)
    let compliance_cost_idr = amount_ton * indo_carbon_tax;

    // Annual equivalents for context
    let avg_indo_income = 56_000_000.0; // IDR/year per capita

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  BIAYA EKSTERNALITAS LINGKUNGAN\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: EPA SCC (2023), ExternE, WHO, UU 32/2009\n\n");

    out.push_str(&format!(
        "INPUT:\n  Polutan       : {}\n  Jumlah        : {:.2} {} ({:.4} ton)\n  Lokasi        : {}\n  Referensi     : {}\n\n",
        pollutant_desc, amount, unit, amount_ton, loc_desc, source
    ));

    out.push_str(&format!(
        "BIAYA KERUSAKAN PER TON (adjusted):\n  Base cost     = USD {:.0}/ton\n  Lokasi factor = x{:.1} (untuk komponen kesehatan)\n  Adjusted cost = USD {:.0}/ton\n\n",
        base_cost, loc_mult, adjusted_cost
    ));

    out.push_str("BREAKDOWN PER KATEGORI DAMPAK:\n");
    out.push_str(&format!("  {:25} {:>12} {:>8}\n", "Kategori", "USD/ton", "%"));
    out.push_str(&format!("  {:25} {:>12} {:>8}\n", "─".repeat(25), "─".repeat(12), "─".repeat(8)));
    let total_pct = health_damage + ecosystem_damage + material_damage + climate_damage;
    if health_damage > 0.0 {
        out.push_str(&format!("  {:25} {:>12.0} {:>7.1}%\n", "Kesehatan (health)", health_damage, health_damage / total_pct * 100.0));
    }
    if ecosystem_damage > 0.0 {
        out.push_str(&format!("  {:25} {:>12.0} {:>7.1}%\n", "Ekosistem (ecosystem)", ecosystem_damage, ecosystem_damage / total_pct * 100.0));
    }
    if material_damage > 0.0 {
        out.push_str(&format!("  {:25} {:>12.0} {:>7.1}%\n", "Material/infrastruktur", material_damage, material_damage / total_pct * 100.0));
    }
    if climate_damage > 0.0 {
        out.push_str(&format!("  {:25} {:>12.0} {:>7.1}%\n", "Iklim (climate)", climate_damage, climate_damage / total_pct * 100.0));
    }

    out.push_str(&format!(
        "\nTOTAL BIAYA KERUSAKAN:\n  USD {}\n  IDR {} (@ Rp {:.0}/USD)\n\n",
        fmt_usd(total_damage_usd), fmt_num(total_damage_idr), usd_to_idr
    ));

    // Comparison with compliance cost
    if pollutant.to_lowercase().contains("co2") {
        out.push_str(&format!(
            "PERBANDINGAN BIAYA KEPATUHAN:\n  Pajak karbon Indonesia (Perpres 98/2021): Rp {} (@ Rp 30,000/tCO2)\n  Rasio damage/compliance: {:.1}x\n\n",
            fmt_num(compliance_cost_idr), total_damage_idr / compliance_cost_idr.max(1.0)
        ));
        out.push_str("  ⚠️ Biaya kerusakan riil >> pajak karbon Indonesia saat ini.\n");
    }

    // Context
    out.push_str(&format!(
        "\nKONTEKS:\n  Setara {:.1} pendapatan per kapita Indonesia/tahun\n",
        total_damage_idr / avg_indo_income
    ));

    if total_damage_idr > 1_000_000_000.0 {
        out.push_str("  ⚠️ Kerusakan > Rp 1 miliar — dampak ekonomi signifikan.\n");
    }
    out.push_str("  Ref: Polluter Pays Principle (UU 32/2009 Pasal 87)\n");
    out
}
