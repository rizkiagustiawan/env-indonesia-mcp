/// Baku Mutu Emisi Sumber Tidak Bergerak
/// Ref: PermenLHK 15/2019

pub fn check(industry: &str, parameter: &str, concentration: f64) -> String {
    let ind = industry.to_lowercase();
    let par = parameter.to_uppercase();

    if concentration < 0.0 {
        return format!("ERROR: Konsentrasi ({}) tidak boleh negatif.", concentration);
    }

    // (industry_key, parameter) -> limit in mg/Nm³ (opacity in %)
    let limit: Option<(f64, &str)> = match (ind.as_str(), par.as_str()) {
        // PLTU Batubara
        ("pltu_batubara" | "pltu", "TSP")     => Some((50.0, "mg/Nm³")),
        ("pltu_batubara" | "pltu", "SO2")     => Some((200.0, "mg/Nm³")),
        ("pltu_batubara" | "pltu", "NO2")     => Some((200.0, "mg/Nm³")),
        ("pltu_batubara" | "pltu", "OPACITY") => Some((10.0, "%")),
        // Semen
        ("semen" | "cement", "TSP")     => Some((50.0, "mg/Nm³")),
        ("semen" | "cement", "SO2")     => Some((150.0, "mg/Nm³")),
        ("semen" | "cement", "NO2")     => Some((400.0, "mg/Nm³")),
        // Smelter
        ("smelter", "TSP")     => Some((50.0, "mg/Nm³")),
        ("smelter", "SO2")     => Some((400.0, "mg/Nm³")),
        // Kimia
        ("kimia" | "chemical", "TSP")     => Some((50.0, "mg/Nm³")),
        ("kimia" | "chemical", "SO2")     => Some((200.0, "mg/Nm³")),
        ("kimia" | "chemical", "NO2")     => Some((300.0, "mg/Nm³")),
        // Pembangkit Gas
        ("pembangkit_gas" | "gas", "TSP")     => Some((30.0, "mg/Nm³")),
        ("pembangkit_gas" | "gas", "SO2")     => Some((35.0, "mg/Nm³")),
        ("pembangkit_gas" | "gas", "NO2")     => Some((320.0, "mg/Nm³")),
        // Incinerator
        ("incinerator", "TSP")     => Some((20.0, "mg/Nm³")),
        ("incinerator", "SO2")     => Some((80.0, "mg/Nm³")),
        ("incinerator", "NO2")     => Some((250.0, "mg/Nm³")),
        ("incinerator", "CO")      => Some((50.0, "mg/Nm³")),
        ("incinerator", "HCL")     => Some((35.0, "mg/Nm³")),
        _ => None,
    };

    let (lim, unit) = match limit {
        Some(v) => v,
        None => return format!(
            "ERROR: Kombinasi industri '{}' dan parameter '{}' tidak ditemukan.\n\
             Industri: pltu_batubara, semen, smelter, kimia, pembangkit_gas, incinerator\n\
             Parameter: TSP, SO2, NO2, CO, opacity, HCl",
            industry, parameter
        ),
    };

    let pct = (concentration / lim) * 100.0;
    let status = if concentration <= lim { "Memenuhi Baku Mutu ✅" } else { "Melebihi Baku Mutu ❌" };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Emisi Sumber Tidak Bergerak\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PermenLHK No. 15 Tahun 2019\n\n");
    out.push_str(&format!("Industri    : {}\n", industry));
    out.push_str(&format!("Parameter   : {}\n", parameter));
    out.push_str(&format!("Konsentrasi : {:.2} {}\n", concentration, unit));
    out.push_str(&format!("Baku Mutu   : {:.2} {}\n", lim, unit));
    out.push_str(&format!("Persentase  : {:.1}% dari baku mutu\n\n", pct));
    out.push_str(&format!("Status: {}\n", status));
    out
}
