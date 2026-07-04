/// Perhitungan Perisai Radiasi (Radiation Shielding)
/// Ref: ICRP 103 (2007), NCRP 147 (2004), BAPETEN

pub fn calculate(initial_intensity: f64, material: &str, thickness_cm: f64, source: &str) -> String {
    if initial_intensity <= 0.0 { return "ERROR: Intensitas awal harus > 0.".into(); }
    if thickness_cm < 0.0 { return "ERROR: Ketebalan perisai tidak boleh negatif.".into(); }

    // HVL values in cm for various materials and sources
    // (Half Value Layer — ketebalan yang mengurangi intensitas 50%)
    let src_lower = source.to_lowercase();
    let mat_lower = material.to_lowercase();

    struct HvlEntry {
        source: &'static str,
        lead: f64,
        concrete: f64,
        water: f64,
        steel: f64,
        earth: f64,
    }

    let hvl_table = [
        HvlEntry { source: "cs137", lead: 0.65, concrete: 4.8, water: 8.5, steel: 1.6, earth: 6.5 },
        HvlEntry { source: "co60",  lead: 1.2,  concrete: 6.2, water: 11.0, steel: 2.1, earth: 8.0 },
        HvlEntry { source: "i131",  lead: 0.3,  concrete: 2.5, water: 5.0,  steel: 0.8, earth: 3.5 },
        HvlEntry { source: "ra226", lead: 1.4,  concrete: 7.0, water: 12.0, steel: 2.5, earth: 9.0 },
        HvlEntry { source: "tc99m", lead: 0.03, concrete: 1.0, water: 2.5,  steel: 0.3, earth: 1.5 },
    ];

    let entry = match hvl_table.iter().find(|e| e.source == src_lower.as_str()) {
        Some(e) => e,
        None => {
            return format!(
                "ERROR: Sumber radiasi '{}' tidak dikenal.\nPilihan: Cs137, Co60, I131, Ra226, Tc99m",
                source
            );
        }
    };

    let hvl = match mat_lower.as_str() {
        "lead" | "timbal" | "pb" => entry.lead,
        "concrete" | "beton" => entry.concrete,
        "water" | "air" => entry.water,
        "steel" | "baja" => entry.steel,
        "earth" | "tanah" => entry.earth,
        _ => {
            return format!(
                "ERROR: Material perisai '{}' tidak dikenal.\nPilihan: lead/timbal, concrete/beton, water/air, steel/baja, earth/tanah",
                material
            );
        }
    };

    // I = I₀ × (1/2)^(x/HVL)
    let n_hvl = thickness_cm / hvl;
    let transmitted = initial_intensity * (0.5_f64).powf(n_hvl);
    let attenuation_factor = transmitted / initial_intensity;
    let reduction_pct = (1.0 - attenuation_factor) * 100.0;

    // Additional thickness to reach target (0.5 µSv/hr for public)
    let target_msv_hr = 0.0005; // 0.5 µSv/hr = 0.0005 mSv/hr
    let additional_thickness = if transmitted > target_msv_hr {
        let needed_total = hvl * (initial_intensity / target_msv_hr).log2();
        (needed_total - thickness_cm).max(0.0)
    } else {
        0.0
    };

    let material_id = match mat_lower.as_str() {
        "lead" | "timbal" | "pb" => "Timbal (Pb)",
        "concrete" | "beton" => "Beton",
        "water" | "air" => "Air",
        "steel" | "baja" => "Baja",
        "earth" | "tanah" => "Tanah",
        _ => material,
    };

    let source_name = match src_lower.as_str() {
        "cs137" => "Cesium-137 (Cs-137)",
        "co60" => "Cobalt-60 (Co-60)",
        "i131" => "Iodine-131 (I-131)",
        "ra226" => "Radium-226 (Ra-226)",
        "tc99m" => "Technetium-99m (Tc-99m)",
        _ => source,
    };

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("PERHITUNGAN PERISAI RADIASI\n");
    result.push_str("Ref: ICRP 103, NCRP 147, BAPETEN\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("FORMULA: I = I₀ × (½)^(x/HVL)\n\n");

    result.push_str("INPUT:\n");
    result.push_str(&format!("• Intensitas awal (I₀) : {:.4} mSv/jam\n", initial_intensity));
    result.push_str(&format!("• Sumber radiasi       : {}\n", source_name));
    result.push_str(&format!("• Material perisai     : {}\n", material_id));
    result.push_str(&format!("• Ketebalan            : {:.2} cm\n", thickness_cm));
    result.push_str(&format!("• HVL                  : {:.2} cm\n\n", hvl));

    result.push_str("HASIL:\n");
    result.push_str(&format!("• Jumlah HVL           : {:.2}\n", n_hvl));
    result.push_str(&format!("• Intensitas transmisi : {:.6} mSv/jam\n", transmitted));
    result.push_str(&format!("                       = {:.4} µSv/jam\n", transmitted * 1000.0));
    result.push_str(&format!("• Faktor atenuasi      : {:.6}\n", attenuation_factor));
    result.push_str(&format!("• Reduksi              : {:.2}%\n\n", reduction_pct));

    if additional_thickness > 0.0 {
        result.push_str(&format!(
            "KETEBALAN TAMBAHAN untuk target 0.5 µSv/jam: {:.2} cm {}\n\n",
            additional_thickness, material_id
        ));
    } else {
        result.push_str("TARGET: Intensitas sudah di bawah 0.5 µSv/jam (batas publik) ✓\n\n");
    }

    result.push_str(&format!("TABEL HVL — {} (cm):\n", source_name));
    result.push_str(&format!("  Timbal  : {:.2} cm\n", entry.lead));
    result.push_str(&format!("  Beton   : {:.2} cm\n", entry.concrete));
    result.push_str(&format!("  Air     : {:.2} cm\n", entry.water));
    result.push_str(&format!("  Baja    : {:.2} cm\n", entry.steel));
    result.push_str(&format!("  Tanah   : {:.2} cm\n", entry.earth));
    result.push_str("══════════════════════════════════════════════\n");

    result
}
