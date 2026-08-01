/// Mangrove Index (NDMI/NDWI Gao) + Carbon Stock
/// Ref: Gao (1996), Komiyama et al. (2005)

pub fn ndmi(nir_b8a: f64, swir_b11: f64) -> String {
    if (nir_b8a + swir_b11).abs() < 1e-10 {
        return "ERROR: Pembagi nol (NIR + SWIR = 0).".into();
    }
    let ndmi = (nir_b8a - swir_b11) / (nir_b8a + swir_b11);

    let cat = if ndmi < -0.2 {
        "Tanah kering/batu"
    } else if ndmi < 0.0 {
        "Vegetasi kering/stress"
    } else if ndmi < 0.2 {
        "Vegetasi rendah kelembaban"
    } else if ndmi < 0.4 {
        "Vegetasi sehat (mangrove sedang)"
    } else {
        "Vegetasi sangat lembab (mangrove sehat/rawa)"
    };

    let mut out = String::from("=== Mangrove NDMI (Gao 1996) ===\n");
    out.push_str("⚠️ NDMI Gao ≠ NDWI McFeeters. Jangan dicampur.\n");
    out.push_str("Band: Sentinel-2 B8A (865nm NIR) & B11 (1610nm SWIR)\n\n");
    out.push_str(&format!("NIR (B8A) = {:.4}\nSWIR (B11) = {:.4}\nNDMI = (NIR-SWIR)/(NIR+SWIR) = {:.4}\nKategori: {}\n", nir_b8a, swir_b11, ndmi, cat));
    out
}

pub fn carbon_stock(dbh_cm: f64, wood_density: f64, n_trees_per_ha: f64) -> String {
    if dbh_cm <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if wood_density <= 0.0 || wood_density > 1.5 {
        return format!(
            "ERROR: Wood density {} g/cm³ di luar rentang (0-1.5).",
            wood_density
        );
    }

    // Komiyama et al. (2005) allometric
    let agb_kg = 0.251 * wood_density * dbh_cm.powf(2.46);
    let agb_ton_ha = agb_kg * n_trees_per_ha / 1000.0;
    let carbon_ton_ha = agb_ton_ha * 0.47; // 47% biomassa = karbon
    let co2e_ton_ha = carbon_ton_ha * 3.67; // C → CO2 factor

    let mut out = String::from("=== Mangrove Carbon Stock ===\n");
    out.push_str("Ref: Komiyama et al. (2005), Aquatic Botany 89\n\n");
    out.push_str(&format!("INPUT:\n  DBH = {:.1} cm\n  Wood density (ρ) = {:.3} g/cm³\n  Kerapatan = {:.0} pohon/ha\n\n", dbh_cm, wood_density, n_trees_per_ha));
    out.push_str(&format!("HASIL:\n  AGB per pohon = {:.2} kg\n  AGB per hektar = {:.2} ton/ha\n  Karbon (47%) = {:.2} tC/ha\n  CO₂e = {:.2} tCO₂/ha\n", agb_kg, agb_ton_ha, carbon_ton_ha, co2e_ton_ha));
    out
}
