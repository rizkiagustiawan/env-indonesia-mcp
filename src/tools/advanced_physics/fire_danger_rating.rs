/// Fire Danger Rating System (FDRS) — Keetch-Byram Drought Index (KBDI)
/// Early Warning System untuk prediksi risiko Karhutla (hutan & gambut)
/// Ref: Keetch & Byram (1968); BMKG FDRS Indonesia; Yulianti et al. 2024

pub fn calculate(
    kbdi_yesterday: f64,
    max_temp_c: f64,
    mean_annual_precip_mm: f64,
    daily_precip_mm: f64,
    is_peatland: bool,
) -> String {
    let mut out = String::from("=== Fire Danger Rating System (FDRS) - KBDI ===\n");
    out.push_str("Ref: Keetch-Byram 1968; BMKG FDRS; Yulianti 2024 (Gambut)\n\n");

    if kbdi_yesterday < 0.0 || max_temp_c < 0.0 || mean_annual_precip_mm < 0.0 || daily_precip_mm < 0.0 {
        return "ERROR [E102]: Semua parameter input harus bernilai positif.".into();
    }

    // Convert to US customary units for standard KBDI formula
    // Temperature: C to F
    let max_temp_f = max_temp_c * 1.8 + 32.0;
    // Precipitation: mm to inches
    let map_in = mean_annual_precip_mm / 25.4;
    let dp_in = daily_precip_mm / 25.4;
    
    // Effective rainfall (subtract 0.2 inches canopy interception)
    let net_rain_in = if dp_in > 0.2 { dp_in - 0.2 } else { 0.0 };

    // Reduce KBDI by net rainfall (1 point = 0.01 inch)
    let kbdi_after_rain = (kbdi_yesterday - net_rain_in * 100.0).max(0.0);

    // Calculate daily drought factor (dQ) if max_temp > 50F
    let mut dq = 0.0;
    if max_temp_f > 50.0 {
        // Equation from Keetch & Byram (1968)
        let numerator = (800.0 - kbdi_after_rain) * (0.968 * (0.0486 * max_temp_f).exp() - 8.3);
        let denominator = 1.0 + 10.88 * (-0.0441 * map_in).exp();
        dq = (numerator / denominator) / 1000.0;
    }

    // New KBDI
    let mut kbdi_today = kbdi_after_rain + dq;
    kbdi_today = kbdi_today.min(800.0);

    // Classification based on Indonesian context (BMKG FDRS modification)
    let danger_level = if kbdi_today < 100.0 {
        "RENDAH (Aman)"
    } else if kbdi_today < 300.0 {
        "SEDANG (Normal)"
    } else if kbdi_today < 400.0 {
        "TINGGI (Waspada)"
    } else {
        "EKSTREM (Bahaya)"
    };

    // Peatland specific multiplier for risk
    let ignition_prob = if kbdi_today > 400.0 && is_peatland {
        "SANGAT TINGGI — Lahan gambut mengering melampaui ambang batas kritis (subsidence & deep smoldering fire risk)"
    } else if kbdi_today > 400.0 {
        "TINGGI — Risiko api permukaan menjalar cepat"
    } else if is_peatland && kbdi_today > 300.0 {
        "TINGGI — Gambut mulai rentan terbakar"
    } else {
        "RENDAH - NORMAL"
    };

    out.push_str(&format!("Kondisi Meteorologi:\n  Suhu Maksimum : {:.1} °C ({:.1} °F)\n  Curah Hujan   : {:.1} mm\n  Curah Hujan Tahunan: {:.0} mm\n  Lahan Gambut  : {}\n\n", 
        max_temp_c, max_temp_f, daily_precip_mm, mean_annual_precip_mm, if is_peatland {"Ya"} else {"Tidak"}));

    out.push_str("Perhitungan KBDI (Skala 0 - 800):\n");
    out.push_str(&format!("  KBDI kemarin    : {:.1}\n", kbdi_yesterday));
    out.push_str(&format!("  Faktor Hujan    : -{:.1} poin\n", net_rain_in * 100.0));
    out.push_str(&format!("  Faktor Kekeringan (dQ): +{:.1} poin\n", dq));
    out.push_str(&format!("  >> KBDI HARI INI : {:.1}\n\n", kbdi_today));

    out.push_str("-- STATUS PERINGATAN DINI (EARLY WARNING) --\n");
    out.push_str(&format!("  Tingkat Bahaya : {}\n", danger_level));
    out.push_str(&format!("  Risiko Ignisi  : {}\n\n", ignition_prob));

    out.push_str("Mitigasi yang Disarankan:\n");
    if kbdi_today > 400.0 {
        out.push_str("  • Patroli intensif 24/7 di kawasan rawan\n");
        out.push_str("  • Larangan total pembakaran lahan (zero burning policy)\n");
        out.push_str("  • Siagakan regu pemadam Manggala Agni\n");
        if is_peatland {
            out.push_str("  • Operasikan sumur bor & sekat kanal untuk pembasahan gambut (rewetting)\n");
        }
    } else if kbdi_today > 300.0 {
        out.push_str("  • Mulai tingkatkan frekuensi pemantauan titik api (hotspot)\n");
        out.push_str("  • Sosialisasikan pencegahan karhutla ke masyarakat\n");
    } else {
        out.push_str("  • Kondisi aman, teruskan pemantauan rutin\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::calculate;

    #[test]
    fn kbdi_computation_logic() {
        // Test rain reduction: 25.4 mm (1 inch) -> net rain 0.8 inches -> reduces KBDI by 80 points
        let res = calculate(500.0, 20.0, 2000.0, 25.4, false);
        // dq will be small at 20C (68F). KBDI ~ 500 - 80 + dq = 425.1
        assert!(res.contains("425.1"), "rain reduction logic wrong: \n{}", res);
    }
}
