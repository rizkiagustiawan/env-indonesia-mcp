/// Source Apportionment — Chemical Mass Balance (CMB)
/// Memisahkan kontribusi sumber polusi PM2.5 (Kendaraan, PLTU, Debu, Pembakaran)
/// Ref: Vital Strategies (2025) Jakarta Source Apportionment; EPA CMB Model 8.2

pub fn assess(
    pm25_total_ug_m3: f64,
    so4_ug_m3: f64,
    no3_ug_m3: f64,
    ec_ug_m3: f64,      // Elemental Carbon
    oc_ug_m3: f64,      // Organic Carbon
    crustal_ug_m3: f64, // Si, Al, Ca, Fe
) -> String {
    let mut out = String::from("=== Source Apportionment PM2.5 (CMB Model) ===\n");
    out.push_str("Ref: EPA CMB 8.2; Vital Strategies (2025) Jakarta Study\n\n");

    if pm25_total_ug_m3 <= 0.0 {
        return "ERROR [E102]: Konsentrasi PM2.5 harus > 0.".into();
    }

    // Simplified Receptor Modeling (Chemical Mass Balance heuristic based on Jakarta signatures)
    // 1. Coal Power Plants (PLTU): strong SO4 signature
    let coal_factor = 1.37; // typical SO4 to ammonium sulfate ratio
    let source_coal = so4_ug_m3 * coal_factor;

    // 2. Vehicles (Transport): high NOx/NO3 and Elemental Carbon (EC)
    let transport_factor = 1.29; // NO3 to ammonium nitrate
    let source_transport = (no3_ug_m3 * transport_factor) + (ec_ug_m3 * 1.5); 

    // 3. Open Burning (Biomass/Trash): high Organic Carbon (OC) relative to EC
    let source_burning = if oc_ug_m3 > (ec_ug_m3 * 2.0) {
        (oc_ug_m3 - ec_ug_m3 * 1.5) * 1.6 // POM multiplier
    } else {
        oc_ug_m3 * 0.5
    };

    // 4. Dust/Construction (Crustal)
    let source_dust = crustal_ug_m3 * 1.1;

    // Mass closure check
    let sum_identified = source_coal + source_transport + source_burning + source_dust;
    let (normalized_coal, normalized_transport, normalized_burning, normalized_dust, unidentified) = 
        if sum_identified > pm25_total_ug_m3 {
            // Scale down if exceeds total
            let scale = pm25_total_ug_m3 / sum_identified;
            (source_coal * scale, source_transport * scale, source_burning * scale, source_dust * scale, 0.0)
        } else {
            (source_coal, source_transport, source_burning, source_dust, pm25_total_ug_m3 - sum_identified)
        };

    let pct = |val: f64| (val / pm25_total_ug_m3) * 100.0;

    out.push_str(&format!("Total PM2.5 Terukur : {:.1} µg/m³\n", pm25_total_ug_m3));
    out.push_str(&format!("Komposisi Kimia     : SO4={:.1}, NO3={:.1}, EC={:.1}, OC={:.1}, Debu Tanah={:.1} (µg/m³)\n\n", 
        so4_ug_m3, no3_ug_m3, ec_ug_m3, oc_ug_m3, crustal_ug_m3));

    out.push_str("-- ESTIMASI KONTRIBUSI SUMBER (Source Apportionment) --\n\n");
    out.push_str(&format!("  Kendaraan Bermotor  : {:>5.1} µg/m³ ({:>4.1}%)\n", normalized_transport, pct(normalized_transport)));
    out.push_str(&format!("  PLTU / Industri     : {:>5.1} µg/m³ ({:>4.1}%)\n", normalized_coal, pct(normalized_coal)));
    out.push_str(&format!("  Pembakaran Terbuka  : {:>5.1} µg/m³ ({:>4.1}%)\n", normalized_burning, pct(normalized_burning)));
    out.push_str(&format!("  Debu Jalan/Tanah    : {:>5.1} µg/m³ ({:>4.1}%)\n", normalized_dust, pct(normalized_dust)));
    out.push_str(&format!("  Tidak Teridentifikasi: {:>5.1} µg/m³ ({:>4.1}%)\n\n", unidentified, pct(unidentified)));

    out.push_str("-- REKOMENDASI KEBIJAKAN --\n");
    
    // Find dominant source
    let mut sources = vec![
        ("Kendaraan", normalized_transport),
        ("PLTU", normalized_coal),
        ("Pembakaran", normalized_burning),
        ("Debu", normalized_dust),
    ];
    sources.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let dominant = sources[0].0;

    out.push_str(&format!("  Sumber dominan adalah {}. Fokuskan regulasi pada sektor ini.\n", dominant));
    match dominant {
        "Kendaraan" => out.push_str("  Aksi: Terapkan LEZ (Low Emission Zone), uji emisi ketat, percepat elektrifikasi.\n"),
        "PLTU" => out.push_str("  Aksi: Audit kepatuhan emisi cerobong, pasang scrubber/FGD, percepat pensiun dini PLTU.\n"),
        "Pembakaran" => out.push_str("  Aksi: Larangan tegas pembakaran sampah terbuka, perbaiki layanan angkut sampah.\n"),
        _ => out.push_str("  Aksi: Pembersihan jalan, kontrol debu konstruksi.\n"),
    }

    out.push_str("\n  Note: Ini adalah model penyederhanaan kimiawi. Untuk hasil pro-justitia\n");
    out.push_str("        gunakan software EPA PMF 5.0 atau EPA CMB 8.2.\n");

    out
}

#[cfg(test)]
mod tests {
    use super::assess;

    #[test]
    fn test_cmb_mass_closure() {
        // High SO4 -> PLTU should dominate
        let res = assess(50.0, 20.0, 5.0, 2.0, 4.0, 5.0);
        assert!(res.contains("Sumber dominan adalah PLTU"));
        assert!(res.contains("Total PM2.5 Terukur : 50.0"));
    }
}
