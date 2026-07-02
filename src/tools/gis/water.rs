pub fn quality(green: f64, red: f64, nir: f64, blue: Option<f64>) -> String {
    let ndwi = if (green + nir).abs() < 1e-10 { 0.0 } else { (green - nir) / (green + nir) };
    let is_water = ndwi > 0.0;
    
    // Turbidity proxy (red/green ratio)
    let turbidity_ratio = if green.abs() < 1e-10 { 0.0 } else { red / green };
    let turbidity_class = if turbidity_ratio < 0.5 { "Clear" } else if turbidity_ratio < 1.0 { "Moderate" } else { "Turbid" };

    // Chlorophyll-a proxy (blue/green or green/red)
    let chl_a_proxy = if let Some(b) = blue {
        if b.abs() < 1e-10 { 0.0 } else { green / b }
    } else {
        if red.abs() < 1e-10 { 0.0 } else { green / red }
    };
    let chl_class = if chl_a_proxy < 1.0 { "Low (oligotrophic)" } else if chl_a_proxy < 2.0 { "Moderate (mesotrophic)" } else { "High (eutrophic)" };

    format!(
        "=== Water Quality Analysis (Sentinel-2) ===\nBands: Green(B3)={:.4}, Red(B4)={:.4}, NIR(B8)={:.4}{}\n\nNDWI: {:.4} ({})\nTurbidity proxy (R/G): {:.3} — {}\nChlorophyll-a proxy: {:.3} — {}\n\nInterpretation:\n  NDWI > 0: Water detected\n  NDWI < 0: Non-water (land/vegetation)\n  Higher turbidity ratio = more suspended sediment\n  Higher chl-a proxy = more algal biomass (eutrophication risk)\n\nFor NTB applications:\n  - Danau Segara Anak (Rinjani crater lake)\n  - Gili Islands coral reef water quality\n  - Sungai Dodokan/Meninting water monitoring\n  - Coastal zone sediment plume detection",
        green, red, nir,
        blue.map(|b| format!(", Blue(B2)={:.4}", b)).unwrap_or_default(),
        ndwi, if is_water { "WATER" } else { "LAND" },
        turbidity_ratio, turbidity_class,
        chl_a_proxy, chl_class
    )
}
