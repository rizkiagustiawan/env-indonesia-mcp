pub fn compute(nir: f64, red: f64) -> String {
    let ndvi = if (nir + red).abs() < 1e-10 {
        0.0
    } else {
        (nir - red) / (nir + red)
    };

    let category = match ndvi {
        v if v < -0.1 => "Water / Snow / Cloud",
        v if v < 0.1 => "Bare soil / Rock / Built-up",
        v if v < 0.2 => "Sparse vegetation / Degraded land",
        v if v < 0.4 => "Low vegetation / Grassland / Crops (early stage)",
        v if v < 0.6 => "Moderate vegetation / Shrubland / Crops (growing)",
        v if v < 0.8 => "Dense vegetation / Healthy forest / Crops (mature)",
        _ => "Very dense vegetation / Tropical rainforest",
    };

    let health = if ndvi > 0.6 {
        "Healthy"
    } else if ndvi > 0.3 {
        "Moderate"
    } else if ndvi > 0.1 {
        "Stressed"
    } else {
        "Non-vegetated"
    };

    format!(
        "=== NDVI Analysis ===\nNIR (B8): {:.4}\nRed (B4): {:.4}\nNDVI: {:.4}\nCategory: {}\nVegetation Health: {}\n\nScale:\n  -1.0 to 0.0: Water/Non-vegetated\n  0.0 to 0.2: Bare/Sparse\n  0.2 to 0.4: Low vegetation\n  0.4 to 0.6: Moderate vegetation\n  0.6 to 0.8: Dense vegetation\n  0.8 to 1.0: Very dense (tropical forest)\n\nFor Sentinel-2: B4=Red(665nm), B8=NIR(842nm)\nFormula: NDVI = (B8 - B4) / (B8 + B4)",
        nir, red, ndvi, category, health
    )
}
