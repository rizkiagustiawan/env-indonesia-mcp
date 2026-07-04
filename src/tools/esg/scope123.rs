/// GHG Protocol Scope 1/2/3 Breakdown
/// Ref: GHG Protocol Corporate Standard, Perpres 98/2021, IPCC 2006

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

pub fn calculate(scope1_json: &str, scope2_json: &str, scope3_json: &str) -> String {
    #[derive(serde::Deserialize)]
    struct EmissionSource {
        source: String,
        amount: f64,
        unit: String,
    }

    let scope1: Vec<EmissionSource> = match serde_json::from_str(scope1_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parse scope1_json: {}.\nFormat: [{{\"source\":\"boiler_diesel\",\"amount\":50000,\"unit\":\"liter\"}}]", e),
    };
    let scope2: Vec<EmissionSource> = match serde_json::from_str(scope2_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parse scope2_json: {}.\nFormat: [{{\"source\":\"grid_electricity\",\"amount\":500000,\"unit\":\"kwh\"}}]", e),
    };
    let scope3: Vec<EmissionSource> = match serde_json::from_str(scope3_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parse scope3_json: {}.\nFormat: [{{\"source\":\"business_travel\",\"amount\":100000,\"unit\":\"km\"}}]", e),
    };

    // Emission factor lookup: returns kgCO2e per unit
    fn get_ef(source: &str, unit: &str) -> Option<(f64, &'static str)> {
        let key = format!("{}_{}", source.to_lowercase(), unit.to_lowercase());
        match key.as_str() {
            // Scope 1 - combustion
            "boiler_diesel_liter" | "diesel_liter" | "diesel_generator_liter" | "solar_liter" => Some((2.68, "IPCC 2006: 2.68 kgCO2/L")),
            "boiler_gas_m3" | "natural_gas_m3" | "gas_m3" => Some((2.02, "IPCC 2006: 2.02 kgCO2/m³")),
            "fleet_gasoline_liter" | "gasoline_liter" | "bensin_liter" => Some((2.31, "IPCC 2006: 2.31 kgCO2/L")),
            "fleet_diesel_liter" => Some((2.68, "IPCC 2006: 2.68 kgCO2/L")),
            "lpg_kg" | "lpg_cooking_kg" => Some((2.98, "IPCC 2006: 2.98 kgCO2/kg")),
            "coal_ton" | "batubara_ton" => Some((2460.0, "IPCC 2006: 2.46 tCO2/ton")),
            "fugitive_ch4_kg" | "methane_kg" => Some((28.0, "GWP100 CH4: 28 kgCO2e/kg")),
            "refrigerant_kg" | "hfc_kg" => Some((1430.0, "HFC-134a GWP100: 1430")),
            // Scope 2 - electricity
            "grid_electricity_kwh" | "electricity_kwh" | "listrik_kwh" => Some((0.794, "PLN Grid EF 2023: 0.794 kgCO2/kWh")),
            "steam_gj" | "uap_gj" => Some((66.7, "Steam from gas boiler: 66.7 kgCO2/GJ")),
            // Scope 3
            "business_travel_km" | "perjalanan_dinas_km" | "guest_flights_km" | "flights_km" => Some((0.255, "DEFRA 2023: 0.255 kgCO2/pax-km (flight economy)")),
            "commuting_km" | "komuter_km" => Some((0.17, "Motorcycle avg: 0.17 kgCO2/km")),
            "waste_landfill_ton" | "waste_ton" | "limbah_ton" => Some((1200.0, "IPCC default: 1.2 tCO2e/ton mixed waste")),
            "waste_incineration_ton" => Some((900.0, "IPCC: 0.9 tCO2e/ton incineration")),
            "water_supply_m3" | "air_m3" => Some((0.344, "Water supply: 0.344 kgCO2/m³")),
            "procurement_idr" | "purchased_goods_idr" => Some((0.0000001, "Proxy: 100 gCO2e/M IDR")),
            "freight_tonkm" | "angkutan_tonkm" => Some((0.062, "Road freight: 0.062 kgCO2/ton-km")),
            _ => None,
        }
    }

    // Calculate per scope
    let mut s1_total = 0.0_f64;
    let mut s2_total = 0.0_f64;
    let mut s3_total = 0.0_f64;

    let mut s1_details: Vec<(String, f64, f64, String)> = Vec::new();
    let mut s2_details: Vec<(String, f64, f64, String)> = Vec::new();
    let mut s3_details: Vec<(String, f64, f64, String)> = Vec::new();

    let mut errors: Vec<String> = Vec::new();

    for s in &scope1 {
        match get_ef(&s.source, &s.unit) {
            Some((ef, ref_str)) => {
                let emission = s.amount * ef / 1000.0; // tCO2e
                s1_total += emission;
                s1_details.push((s.source.clone(), s.amount, emission, ref_str.to_string()));
            }
            None => errors.push(format!("Scope 1: '{}' ({}) — faktor emisi tidak ditemukan", s.source, s.unit)),
        }
    }
    for s in &scope2 {
        match get_ef(&s.source, &s.unit) {
            Some((ef, ref_str)) => {
                let emission = s.amount * ef / 1000.0;
                s2_total += emission;
                s2_details.push((s.source.clone(), s.amount, emission, ref_str.to_string()));
            }
            None => errors.push(format!("Scope 2: '{}' ({}) — faktor emisi tidak ditemukan", s.source, s.unit)),
        }
    }
    for s in &scope3 {
        match get_ef(&s.source, &s.unit) {
            Some((ef, ref_str)) => {
                let emission = s.amount * ef / 1000.0;
                s3_total += emission;
                s3_details.push((s.source.clone(), s.amount, emission, ref_str.to_string()));
            }
            None => errors.push(format!("Scope 3: '{}' ({}) — faktor emisi tidak ditemukan", s.source, s.unit)),
        }
    }

    let grand_total = s1_total + s2_total + s3_total;

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  GHG PROTOCOL SCOPE 1/2/3\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: GHG Protocol Corporate Standard, Perpres 98/2021, IPCC 2006\n\n");

    if !errors.is_empty() {
        out.push_str("⚠️ PERINGATAN:\n");
        for e in &errors {
            out.push_str(&format!("  {}\n", e));
        }
        out.push_str("\n");
    }

    // Scope 1
    let s1_pct = if grand_total > 0.0 { s1_total / grand_total * 100.0 } else { 0.0 };
    out.push_str(&format!("SCOPE 1 — Emisi Langsung ({:.1}%)\n", s1_pct));
    out.push_str("  (Pembakaran bahan bakar, proses industri, fugitive)\n");
    for (src, amt, em, rf) in &s1_details {
        out.push_str(&format!("  {:30} {:>10} → {:>8.2} tCO2e  [{}]\n", src, fmt_num(*amt), em, rf));
    }
    out.push_str(&format!("  {:30} {:>21.2} tCO2e\n\n", "Subtotal Scope 1:", s1_total));

    // Scope 2
    let s2_pct = if grand_total > 0.0 { s2_total / grand_total * 100.0 } else { 0.0 };
    out.push_str(&format!("SCOPE 2 — Emisi Tidak Langsung Energi ({:.1}%)\n", s2_pct));
    out.push_str("  (Listrik PLN, pembelian steam/cooling)\n");
    for (src, amt, em, rf) in &s2_details {
        out.push_str(&format!("  {:30} {:>10} → {:>8.2} tCO2e  [{}]\n", src, fmt_num(*amt), em, rf));
    }
    out.push_str(&format!("  {:30} {:>21.2} tCO2e\n\n", "Subtotal Scope 2:", s2_total));

    // Scope 3
    let s3_pct = if grand_total > 0.0 { s3_total / grand_total * 100.0 } else { 0.0 };
    out.push_str(&format!("SCOPE 3 — Emisi Tidak Langsung Lainnya ({:.1}%)\n", s3_pct));
    out.push_str("  (Perjalanan dinas, limbah, rantai pasok, komuter)\n");
    for (src, amt, em, rf) in &s3_details {
        out.push_str(&format!("  {:30} {:>10} → {:>8.2} tCO2e  [{}]\n", src, fmt_num(*amt), em, rf));
    }
    out.push_str(&format!("  {:30} {:>21.2} tCO2e\n\n", "Subtotal Scope 3:", s3_total));

    // Grand total
    out.push_str(&format!("══════════════════════════════════════════\n  TOTAL EMISI GRK = {:.2} tCO2e\n══════════════════════════════════════════\n\n", grand_total));

    // Percentage breakdown
    out.push_str(&format!(
        "BREAKDOWN:\n  Scope 1: {:>8.2} tCO2e ({:.1}%)\n  Scope 2: {:>8.2} tCO2e ({:.1}%)\n  Scope 3: {:>8.2} tCO2e ({:.1}%)\n\n",
        s1_total, s1_pct, s2_total, s2_pct, s3_total, s3_pct
    ));

    // Intensity metrics
    out.push_str("METRIK INTENSITAS (isi sesuai konteks):\n");
    out.push_str(&format!("  Per karyawan (asumsi 100): {:.2} tCO2e/orang\n", grand_total / 100.0));
    out.push_str(&format!("  Per M IDR revenue (asumsi 10B): {:.2} tCO2e/M IDR\n\n", grand_total / 10000.0));

    // Carbon valuation
    let carbon_tax_idr = grand_total * 30_000.0; // Perpres 98/2021 ~Rp30,000/tCO2
    let carbon_market_usd = grand_total * 51.0; // EPA SCC
    out.push_str(&format!(
        "VALUASI KARBON:\n  Pajak karbon Indonesia (Perpres 98/2021): Rp {} (@ Rp 30,000/tCO2)\n  Social Cost of Carbon (EPA 2023):          USD {} (@ $51/tCO2)\n\n",
        fmt_num(carbon_tax_idr), fmt_num(carbon_market_usd)
    ));

    // Reduction opportunities
    out.push_str("PELUANG PENGURANGAN EMISI:\n");
    if s2_total > s1_total {
        out.push_str("  • Scope 2 dominan — pasang PLTS rooftop, PPA renewable energy\n");
    }
    if s1_total > 0.0 {
        out.push_str("  • Scope 1 — konversi bahan bakar fosil ke gas/listrik, efisiensi boiler\n");
    }
    if s3_total > 0.0 {
        out.push_str("  • Scope 3 — program WFH, waste diversion, green procurement\n");
    }
    out.push_str("  • Target SBTi: -42% emisi absolut by 2030 (1.5°C pathway)\n");
    out
}
