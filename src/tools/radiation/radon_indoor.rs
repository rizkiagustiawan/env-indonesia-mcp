/// Estimasi Radon Indoor
/// Ref: WHO Handbook on Indoor Radon (2009), ICRP 126 (2014)

pub fn calculate(
    soil_radon_bq_m3: f64,
    floor_area_m2: f64,
    room_height_m: f64,
    ventilation_rate_ach: f64,
    floor_type: &str,
) -> String {
    if soil_radon_bq_m3 < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }
    if floor_area_m2 <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if room_height_m <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if ventilation_rate_ach <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    let ft_lower = floor_type.to_lowercase();

    // Entry rate coefficient (Bq/m²/s per Bq/m³ soil radon)
    let (entry_coeff, floor_name) = match ft_lower.as_str() {
        "concrete_slab" | "beton" => (0.001, "Plat beton (concrete slab)"),
        "basement" | "ruang_bawah_tanah" => (0.01, "Ruang bawah tanah (basement)"),
        "tanah" | "earth" | "dirt" => (0.05, "Lantai tanah (tanpa pelapis)"),
        "elevated" | "panggung" => (0.0001, "Rumah panggung (elevated)"),
        "tile" | "keramik" => (0.002, "Keramik di atas beton"),
        _ => {
            return format!(
                "ERROR: Tipe lantai '{}' tidak dikenal.\nPilihan: concrete_slab/beton, basement/ruang_bawah_tanah, tanah/earth, elevated/panggung, tile/keramik",
                floor_type
            );
        }
    };

    let room_volume = floor_area_m2 * room_height_m;

    // Entry rate E (Bq/s) = coefficient × soil_radon × floor_area
    let entry_rate_bq_s = entry_coeff * soil_radon_bq_m3 * floor_area_m2;

    // Ventilation rate λv (s⁻¹) = ACH / 3600
    let lambda_v = ventilation_rate_ach / 3600.0;

    // Radon decay constant λ_Rn = ln(2) / t½ = ln(2) / (3.8235 days × 86400 s)
    let lambda_rn = (2.0_f64).ln() / (3.8235 * 86400.0);

    // Steady-state indoor radon: C = E / ((λv + λ_Rn) × V)
    let indoor_radon = entry_rate_bq_s / ((lambda_v + lambda_rn) * room_volume);

    // Annual effective dose (mSv/year)
    // Dose conversion: 1 Bq/m³ → ~0.017 mSv/year (UNSCEAR, 7000 hr/yr indoor, F=0.4, DCF=9 nSv/(Bq·h/m³))
    let occupancy_hours = 7000.0; // indoor hours per year
    let dose_conversion = 9e-6; // mSv per (Bq/m³ × hour)
    let annual_dose_msv = indoor_radon * occupancy_hours * dose_conversion;

    // Compliance
    let who_compliant = indoor_radon <= 100.0;
    let icrp_compliant = indoor_radon <= 300.0;

    // Recommended ventilation for 100 Bq/m³ target
    let target_100 = 100.0;
    let required_lambda = entry_rate_bq_s / (target_100 * room_volume) - lambda_rn;
    let required_ach = if required_lambda > 0.0 {
        required_lambda * 3600.0
    } else {
        0.0
    };

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("ESTIMASI RADON INDOOR\n");
    result.push_str("Ref: WHO Indoor Radon Handbook (2009), ICRP 126\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("FORMULA: C = E / ((λv + λ_Rn) × V)\n\n");

    result.push_str("INPUT:\n");
    result.push_str(&format!(
        "• Radon tanah          : {:.0} Bq/m³\n",
        soil_radon_bq_m3
    ));
    result.push_str(&format!(
        "• Luas lantai          : {:.1} m²\n",
        floor_area_m2
    ));
    result.push_str(&format!(
        "• Tinggi ruangan       : {:.1} m\n",
        room_height_m
    ));
    result.push_str(&format!("• Volume ruangan       : {:.1} m³\n", room_volume));
    result.push_str(&format!(
        "• Laju ventilasi       : {:.2} ACH\n",
        ventilation_rate_ach
    ));
    result.push_str(&format!("• Tipe lantai          : {}\n\n", floor_name));

    result.push_str("PERHITUNGAN:\n");
    result.push_str(&format!(
        "• Koefisien masuk      : {:.4} (Bq/m²/s)/(Bq/m³)\n",
        entry_coeff
    ));
    result.push_str(&format!(
        "• Laju masuk (E)       : {:.4} Bq/s\n",
        entry_rate_bq_s
    ));
    result.push_str(&format!("• λv (ventilasi)       : {:.6e} s⁻¹\n", lambda_v));
    result.push_str(&format!(
        "• λ_Rn (peluruhan)     : {:.6e} s⁻¹\n\n",
        lambda_rn
    ));

    result.push_str("HASIL:\n");
    result.push_str(&format!(
        "• Radon indoor (C)     : {:.1} Bq/m³\n",
        indoor_radon
    ));
    result.push_str(&format!(
        "• Dosis efektif tahunan: {:.2} mSv/tahun\n\n",
        annual_dose_msv
    ));

    result.push_str("KEPATUHAN:\n");
    result.push_str(&format!(
        "• WHO (100 Bq/m³)      : {} {}\n",
        if who_compliant {
            "MEMENUHI ✓"
        } else {
            "TIDAK MEMENUHI ✗"
        },
        if !who_compliant {
            "— perlu tindakan"
        } else {
            ""
        }
    ));
    result.push_str(&format!(
        "• ICRP (300 Bq/m³)     : {} {}\n\n",
        if icrp_compliant {
            "MEMENUHI ✓"
        } else {
            "TIDAK MEMENUHI ✗"
        },
        if !icrp_compliant {
            "— tindakan segera"
        } else {
            ""
        }
    ));

    if !who_compliant && required_ach > 0.0 {
        result.push_str(&format!(
            "REKOMENDASI VENTILASI (target 100 Bq/m³): {:.2} ACH\n\n",
            required_ach
        ));
    }

    result.push_str("MITIGASI RADON:\n");
    result.push_str("• Perbaikan ventilasi (mekanik/alami)\n");
    result.push_str("• Pemasangan radon barrier pada lantai\n");
    result.push_str("• Sub-slab depressurization system\n");
    result.push_str("• Sealing retakan dan celah pada lantai/dinding\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}
