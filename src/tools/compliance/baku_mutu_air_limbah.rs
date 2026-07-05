/// Baku Mutu Air Limbah Industri
/// Ref: PermenLH 5/2014

pub fn check(industry: &str, parameter: &str, concentration: f64) -> String {
    let ind = industry.to_lowercase();
    let par = parameter.to_lowercase();

    if concentration < 0.0 && par != "ph" {
        return format!("ERROR [E102]: Parameter tidak boleh negatif. {}", concentration);
    }

    // (industry, parameter) -> (limit, unit)
    // Limits in mg/L unless otherwise noted; pH is unitless range
    let result: Option<(f64, f64, &str, bool)> = match (ind.as_str(), par.as_str()) {
        // Tekstil
        ("tekstil" | "textile", "bod")       => Some((60.0, 0.0, "mg/L", false)),
        ("tekstil" | "textile", "cod")       => Some((150.0, 0.0, "mg/L", false)),
        ("tekstil" | "textile", "tss")       => Some((50.0, 0.0, "mg/L", false)),
        ("tekstil" | "textile", "phenol")    => Some((0.5, 0.0, "mg/L", false)),
        ("tekstil" | "textile", "cr6" | "cr_total") => Some((1.0, 0.0, "mg/L", false)),
        ("tekstil" | "textile", "ph")        => Some((9.0, 6.0, "", true)),
        // Sawit (Kelapa Sawit / CPO)
        ("sawit" | "kelapa_sawit" | "cpo", "bod")  => Some((100.0, 0.0, "mg/L", false)),
        ("sawit" | "kelapa_sawit" | "cpo", "cod")  => Some((350.0, 0.0, "mg/L", false)),
        ("sawit" | "kelapa_sawit" | "cpo", "tss")  => Some((250.0, 0.0, "mg/L", false)),
        ("sawit" | "kelapa_sawit" | "cpo", "oil_grease" | "minyak_lemak") => Some((25.0, 0.0, "mg/L", false)),
        ("sawit" | "kelapa_sawit" | "cpo", "nh3n" | "ammonia") => Some((5.0, 0.0, "mg/L", false)),
        ("sawit" | "kelapa_sawit" | "cpo", "ph")   => Some((9.0, 6.0, "", true)),
        // Karet
        ("karet" | "rubber", "bod")       => Some((60.0, 0.0, "mg/L", false)),
        ("karet" | "rubber", "cod")       => Some((200.0, 0.0, "mg/L", false)),
        ("karet" | "rubber", "tss")       => Some((100.0, 0.0, "mg/L", false)),
        ("karet" | "rubber", "nh3n" | "ammonia") => Some((5.0, 0.0, "mg/L", false)),
        ("karet" | "rubber", "ph")        => Some((9.0, 6.0, "", true)),
        // Tapioka
        ("tapioka" | "tapioca", "bod")    => Some((75.0, 0.0, "mg/L", false)),
        ("tapioka" | "tapioca", "cod")    => Some((250.0, 0.0, "mg/L", false)),
        ("tapioka" | "tapioca", "tss")    => Some((100.0, 0.0, "mg/L", false)),
        ("tapioka" | "tapioca", "ph")     => Some((9.0, 6.0, "", true)),
        // Gula
        ("gula" | "sugar", "bod")         => Some((60.0, 0.0, "mg/L", false)),
        ("gula" | "sugar", "cod")         => Some((100.0, 0.0, "mg/L", false)),
        ("gula" | "sugar", "tss")         => Some((50.0, 0.0, "mg/L", false)),
        ("gula" | "sugar", "ph")          => Some((9.0, 6.0, "", true)),
        // Pulp & Kertas
        ("pulp_kertas" | "pulp" | "paper", "bod") => Some((70.0, 0.0, "mg/L", false)),
        ("pulp_kertas" | "pulp" | "paper", "cod") => Some((200.0, 0.0, "mg/L", false)),
        ("pulp_kertas" | "pulp" | "paper", "tss") => Some((70.0, 0.0, "mg/L", false)),
        ("pulp_kertas" | "pulp" | "paper", "ph")  => Some((9.0, 6.0, "", true)),
        // Farmasi
        ("farmasi" | "pharmaceutical", "bod") => Some((75.0, 0.0, "mg/L", false)),
        ("farmasi" | "pharmaceutical", "cod") => Some((150.0, 0.0, "mg/L", false)),
        ("farmasi" | "pharmaceutical", "tss") => Some((75.0, 0.0, "mg/L", false)),
        ("farmasi" | "pharmaceutical", "phenol") => Some((0.5, 0.0, "mg/L", false)),
        ("farmasi" | "pharmaceutical", "ph")  => Some((9.0, 6.0, "", true)),
        // Electroplating
        ("electroplating", "tss")          => Some((20.0, 0.0, "mg/L", false)),
        ("electroplating", "cr6")          => Some((0.1, 0.0, "mg/L", false)),
        ("electroplating", "cr_total")     => Some((0.5, 0.0, "mg/L", false)),
        ("electroplating", "cn" | "sianida") => Some((0.05, 0.0, "mg/L", false)),
        ("electroplating", "zn" | "seng")  => Some((5.0, 0.0, "mg/L", false)),
        ("electroplating", "cu" | "tembaga") => Some((2.0, 0.0, "mg/L", false)),
        ("electroplating", "ni" | "nikel") => Some((1.0, 0.0, "mg/L", false)),
        ("electroplating", "ph")           => Some((9.0, 6.0, "", true)),
        // Rumah Sakit
        ("rumah_sakit" | "hospital", "bod") => Some((30.0, 0.0, "mg/L", false)),
        ("rumah_sakit" | "hospital", "cod") => Some((80.0, 0.0, "mg/L", false)),
        ("rumah_sakit" | "hospital", "tss") => Some((30.0, 0.0, "mg/L", false)),
        ("rumah_sakit" | "hospital", "nh3n" | "ammonia") => Some((1.0, 0.0, "mg/L", false)),
        ("rumah_sakit" | "hospital", "ph")  => Some((9.0, 6.0, "", true)),
        // Hotel
        ("hotel", "bod")       => Some((30.0, 0.0, "mg/L", false)),
        ("hotel", "cod")       => Some((50.0, 0.0, "mg/L", false)),
        ("hotel", "tss")       => Some((30.0, 0.0, "mg/L", false)),
        ("hotel", "oil_grease" | "minyak_lemak") => Some((5.0, 0.0, "mg/L", false)),
        ("hotel", "ph")        => Some((9.0, 6.0, "", true)),
        // Peternakan
        ("peternakan" | "livestock", "bod") => Some((100.0, 0.0, "mg/L", false)),
        ("peternakan" | "livestock", "cod") => Some((200.0, 0.0, "mg/L", false)),
        ("peternakan" | "livestock", "tss") => Some((100.0, 0.0, "mg/L", false)),
        ("peternakan" | "livestock", "nh3n" | "ammonia") => Some((25.0, 0.0, "mg/L", false)),
        ("peternakan" | "livestock", "ph")  => Some((9.0, 6.0, "", true)),
        _ => None,
    };

    let (max_lim, min_lim, unit, is_range) = match result {
        Some(v) => v,
        None => return format!(
            "ERROR: Kombinasi industri '{}' dan parameter '{}' tidak ditemukan.\n\
             Industri: tekstil, sawit, karet, tapioka, gula, pulp_kertas, farmasi,\n\
             electroplating, rumah_sakit, hotel, peternakan\n\
             Parameter: BOD, COD, TSS, pH, oil_grease, phenol, Cr6, NH3N, CN, Zn, Cu, Ni",
            industry, parameter
        ),
    };

    let (status, pct_str) = if is_range {
        // pH range check
        let ok = concentration >= min_lim && concentration <= max_lim;
        let st = if ok { "Memenuhi Baku Mutu ✅" } else { "Melebihi Baku Mutu ❌" };
        (st, format!("Range: {}-{}", min_lim, max_lim))
    } else {
        let pct = (concentration / max_lim) * 100.0;
        let st = if concentration <= max_lim { "Memenuhi Baku Mutu ✅" } else { "Melebihi Baku Mutu ❌" };
        (st, format!("{:.1}% dari baku mutu", pct))
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Air Limbah Industri\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PermenLH No. 5 Tahun 2014\n\n");
    out.push_str(&format!("Industri    : {}\n", industry));
    out.push_str(&format!("Parameter   : {}\n", parameter));
    if is_range {
        out.push_str(&format!("Nilai       : {:.2}\n", concentration));
        out.push_str(&format!("Baku Mutu   : {:.1} - {:.1}\n", min_lim, max_lim));
    } else {
        out.push_str(&format!("Konsentrasi : {:.2} {}\n", concentration, unit));
        out.push_str(&format!("Baku Mutu   : {:.2} {}\n", max_lim, unit));
    }
    out.push_str(&format!("Persentase  : {}\n\n", pct_str));
    out.push_str(&format!("Status: {}\n", status));
    out
}
