/// Water Footprint Calculator
/// Ref: ISO 14046, Water Footprint Network (Hoekstra et al., 2011)

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

pub fn calculate(product: &str, quantity: f64, unit: &str) -> String {
    if quantity <= 0.0 { return "ERROR: Quantity harus > 0.".into(); }

    // Water footprint database (L per unit)
    // Format: (blue, green, grey, unit_desc, source)
    let (blue, green, grey, unit_desc, source) = match product.to_lowercase().as_str() {
        "rice" | "beras" | "padi" => (
            341.0, 1710.0, 449.0,
            "kg", "Mekonnen & Hoekstra (2011), global avg"
        ),
        "palm_oil" | "sawit" | "minyak_sawit" => (
            20.0, 4550.0, 430.0,
            "kg", "Mekonnen & Hoekstra (2011)"
        ),
        "rubber" | "karet" => (
            100.0, 14800.0, 1100.0,
            "kg", "Mekonnen & Hoekstra (2011)"
        ),
        "coffee" | "kopi" => (
            1100.0, 15300.0, 2500.0,
            "kg", "Mekonnen & Hoekstra (2011)"
        ),
        "beef" | "sapi" | "daging_sapi" => (
            550.0, 14200.0, 650.0,
            "kg", "Mekonnen & Hoekstra (2012)"
        ),
        "chicken" | "ayam" | "daging_ayam" => (
            313.0, 3545.0, 442.0,
            "kg", "Mekonnen & Hoekstra (2012)"
        ),
        "egg" | "telur" => (
            244.0, 2592.0, 464.0,
            "kg", "Mekonnen & Hoekstra (2012)"
        ),
        "milk" | "susu" => (
            86.0, 863.0, 72.0,
            "L", "Mekonnen & Hoekstra (2012)"
        ),
        "cotton" | "kapas" => (
            4482.0, 4235.0, 1283.0,
            "kg", "Mekonnen & Hoekstra (2011)"
        ),
        "paper" | "kertas" => (
            768.0, 8282.0, 950.0,
            "kg", "Van Oel & Hoekstra (2012)"
        ),
        "steel" | "baja" => (
            3400.0, 0.0, 9600.0,
            "ton", "WSA (2019)"
        ),
        "cement" | "semen" => (
            130.0, 0.0, 670.0,
            "ton", "Gerbens-Leenes et al. (2009)"
        ),
        "electricity_coal" | "listrik_batubara" => (
            1.5, 0.0, 0.5,
            "kWh", "Mekonnen et al. (2015)"
        ),
        "electricity_gas" | "listrik_gas" => (
            0.4, 0.0, 0.1,
            "kWh", "Mekonnen et al. (2015)"
        ),
        "tobacco" | "tembakau" => (
            205.0, 2375.0, 45.0,
            "kg", "Mekonnen & Hoekstra (2011), NTB crop"
        ),
        "corn" | "jagung" => (
            81.0, 947.0, 194.0,
            "kg", "Mekonnen & Hoekstra (2011)"
        ),
        "sugar" | "gula" | "tebu" => (
            57.0, 1168.0, 275.0,
            "kg", "Mekonnen & Hoekstra (2011)"
        ),
        _ => {
            return format!(
                "ERROR: Produk '{}' tidak ditemukan.\n\nProduk tersedia:\n  Pertanian: rice/beras, palm_oil/sawit, rubber/karet, coffee/kopi, tobacco/tembakau, corn/jagung, sugar/gula\n  Peternakan: beef/sapi, chicken/ayam, egg/telur, milk/susu\n  Tekstil: cotton/kapas\n  Industri: paper/kertas, steel/baja, cement/semen\n  Energi: electricity_coal, electricity_gas",
                product
            );
        }
    };

    let total_wf_per_unit = blue + green + grey;
    let total_wf = total_wf_per_unit * quantity;
    let blue_total = blue * quantity;
    let green_total = green * quantity;
    let grey_total = grey * quantity;

    // Sustainability assessment
    let blue_pct = if total_wf > 0.0 { blue_total / total_wf * 100.0 } else { 0.0 };
    let assessment = if blue_pct > 40.0 {
        "⚠️ TINGGI — Konsumsi air biru tinggi, tekanan pada sumber daya air permukaan/tanah"
    } else if blue_pct > 20.0 {
        "🟡 SEDANG — Ketergantungan moderat pada irigasi/air ledeng"
    } else {
        "🟢 RENDAH — Dominan air hujan (hijau), beban relatif rendah"
    };

    // Context comparisons
    let person_daily_drink = 2.0; // L/day
    let days_equiv = total_wf / person_daily_drink;

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  WATER FOOTPRINT (Jejak Air)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: ISO 14046, Water Footprint Network\n\n");

    out.push_str(&format!(
        "INPUT:\n  Produk   : {}\n  Jumlah   : {:.2} {}\n  Sumber   : {}\n\n",
        product, quantity, unit, source
    ));

    out.push_str(&format!(
        "WATER FOOTPRINT PER {} {}:\n  Blue (air biru)   = {} L  (air permukaan/tanah yang dikonsumsi)\n  Green (air hijau) = {} L  (air hujan yang digunakan)\n  Grey (air abu)    = {} L  (air untuk asimilasi polutan)\n  TOTAL             = {} L\n\n",
        unit_desc, "", fmt_num(blue), fmt_num(green), fmt_num(grey), fmt_num(total_wf_per_unit)
    ));

    out.push_str(&format!(
        "TOTAL WATER FOOTPRINT ({:.2} {}):\n  Blue  = {} L ({:.1}%)\n  Green = {} L ({:.1}%)\n  Grey  = {} L ({:.1}%)\n  TOTAL = {} L ({:.1} m³)\n\n",
        quantity, unit,
        fmt_num(blue_total), if total_wf > 0.0 { blue_total / total_wf * 100.0 } else { 0.0 },
        fmt_num(green_total), if total_wf > 0.0 { green_total / total_wf * 100.0 } else { 0.0 },
        fmt_num(grey_total), if total_wf > 0.0 { grey_total / total_wf * 100.0 } else { 0.0 },
        fmt_num(total_wf), total_wf / 1000.0
    ));

    out.push_str(&format!("KONTEKS:\n  Setara {:.0} hari minum (@ 2 L/hari)\n", days_equiv));
    out.push_str(&format!("  Setara {:.1} kolam renang Olympic (2,500 m³)\n\n", total_wf / 1000.0 / 2500.0));

    out.push_str(&format!("PENILAIAN KEBERLANJUTAN:\n  Porsi air biru: {:.1}%\n  {}\n\n", blue_pct, assessment));

    out.push_str("Catatan Indonesia:\n");
    out.push_str("  - Musim kemarau: defisit air di NTT, NTB, Jawa Timur — irigasi kritis\n");
    out.push_str("  - Grey WF tinggi = beban polutan pada badan air penerima\n");
    out.push_str("  - Water stress bervariasi: tinggi di Jawa, rendah di Kalimantan/Papua\n");
    out
}
