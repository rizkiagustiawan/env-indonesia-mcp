/// Soil Quality & Texture Classification
/// Ref: USDA Soil Texture Triangle, FAO Guidelines

pub fn classify_texture(sand_pct: f64, silt_pct: f64, clay_pct: f64) -> String {
    let total = sand_pct + silt_pct + clay_pct;
    if (total - 100.0).abs() > 1.0 { return format!("ERROR: Sand+Silt+Clay = {:.1}%, harus = 100%.", total); }
    if sand_pct < 0.0 || silt_pct < 0.0 || clay_pct < 0.0 { return "ERROR [E102]: Parameter tidak boleh negatif.".into(); }

    // USDA texture triangle decision tree
    let kelas = if clay_pct >= 40.0 && sand_pct <= 45.0 && silt_pct < 40.0 { "Clay" }
    else if clay_pct >= 40.0 && silt_pct >= 40.0 { "Silty Clay" }
    else if clay_pct >= 35.0 && sand_pct > 45.0 { "Sandy Clay" }
    else if clay_pct >= 27.0 && clay_pct < 40.0 && sand_pct > 20.0 && sand_pct <= 45.0 { "Clay Loam" }
    else if clay_pct >= 27.0 && clay_pct < 40.0 && sand_pct <= 20.0 { "Silty Clay Loam" }
    else if clay_pct >= 20.0 && clay_pct < 35.0 && sand_pct > 45.0 && silt_pct < 28.0 { "Sandy Clay Loam" }
    else if clay_pct >= 7.0 && clay_pct < 27.0 && silt_pct >= 28.0 && silt_pct < 50.0 && sand_pct <= 52.0 { "Loam" }
    else if silt_pct >= 80.0 { "Silt" }
    else if silt_pct >= 50.0 && clay_pct < 27.0 { "Silt Loam" }
    else if sand_pct >= 85.0 && clay_pct < 10.0 { "Sand" }
    else if sand_pct >= 70.0 && clay_pct < 15.0 { "Loamy Sand" }
    else if sand_pct >= 43.0 && clay_pct < 20.0 && silt_pct < 50.0 { "Sandy Loam" }
    else { "Loam (default)" };

    let mut out = String::from("=== Soil Texture Classification (USDA) ===\n\n");
    out.push_str(&format!("INPUT: Sand={:.0}%, Silt={:.0}%, Clay={:.0}%\n", sand_pct, silt_pct, clay_pct));
    out.push_str(&format!("KELAS TEKSTUR: {}\n\n", kelas));

    // Interpretasi untuk Indonesia
    let drainage = if sand_pct > 60.0 { "Cepat (risiko kekeringan)" } else if clay_pct > 40.0 { "Lambat (risiko genangan)" } else { "Sedang (baik)" };
    out.push_str(&format!("Drainase: {}\n", drainage));
    out
}

pub fn assess_ph(ph: f64) -> String {
    if ph < 0.0 || ph > 14.0 { return format!("ERROR: pH {} tidak valid (0-14).", ph); }
    let cat = if ph < 4.5 { "Sangat Masam" } else if ph < 5.5 { "Masam" } else if ph < 6.5 { "Agak Masam" } else if ph < 7.5 { "Netral (optimal pertanian)" } else if ph < 8.5 { "Agak Basa" } else { "Basa" };
    format!("=== Soil pH Assessment ===\npH = {:.2}\nKategori: {}\nOptimal pertanian: 6.0-7.5\n", ph, cat)
}
