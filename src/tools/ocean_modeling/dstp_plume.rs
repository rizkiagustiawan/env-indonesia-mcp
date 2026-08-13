/// Deep-Sea Tailings Placement (DSTP) Plume Dispersion
/// Menghitung dispersi tailing di laut dalam untuk industri nikel
/// Ref: Shimmield et al. 2010 (SAMS DSTP Review); Reichelt-Brushett 2012

pub fn assess(
    discharge_depth_m: f64,
    tailings_volume_m3_day: f64,
    solid_fraction_pct: f64,
    ocean_current_speed_m_s: f64,
    settling_velocity_mm_s: f64,
) -> String {
    let mut out = String::from("=== Deep-Sea Tailings Placement (DSTP) Dispersion ===\n");
    out.push_str("Ref: Shimmield et al. (SAMS); Reichelt-Brushett 2012\n\n");

    if discharge_depth_m < 50.0 {
        out.push_str("⚠️ [CRITICAL WARNING] Kedalaman < 50m sangat berbahaya. DSTP mensyaratkan pelepasan di bawah zona euphotic (biasanya >100m - 200m) untuk mencegah upwelling racun.\n\n");
    }

    // Settling time calculation
    // Time to sink 100 meters down from discharge point (simplified)
    let v_settle_m_s = settling_velocity_mm_s / 1000.0;
    
    if v_settle_m_s <= 0.0 {
        return "ERROR: Settling velocity harus > 0.".into();
    }

    // Typical slope distance behavior
    let sink_time_100m_s = 100.0 / v_settle_m_s;
    let horizontal_drift_100m_drop = ocean_current_speed_m_s * sink_time_100m_s;

    // Plume footprint area estimate (simplified cone dispersion)
    // Area roughly scales with volume and inverse settling velocity
    let solids_m3_day = tailings_volume_m3_day * (solid_fraction_pct / 100.0);
    let plume_thickness_m = 0.05; // 5 cm daily deposition threshold
    let footprint_area_km2 = (solids_m3_day / plume_thickness_m) / 1_000_000.0;

    out.push_str(&format!("Parameter Operasional:\n"));
    out.push_str(&format!("  Kedalaman Pipa Pelepasan : {:.0} m\n", discharge_depth_m));
    out.push_str(&format!("  Volume Tailing           : {:.0} m³/hari\n", tailings_volume_m3_day));
    out.push_str(&format!("  Fraksi Padat             : {:.1}%\n", solid_fraction_pct));
    out.push_str(&format!("  Arus Laut                : {:.3} m/s\n", ocean_current_speed_m_s));
    out.push_str(&format!("  Kecepatan Pengendapan    : {:.2} mm/s\n\n", settling_velocity_mm_s));

    out.push_str("-- ESTIMASI DISPERSI PLUME (Subsurface) --\n\n");
    out.push_str(&format!("  Waktu endap tiap 100m  : {:.1} jam\n", sink_time_100m_s / 3600.0));
    out.push_str(&format!("  Drift horizontal /100m : {:.2} km (Jarak paparan pelagik)\n", horizontal_drift_100m_drop / 1000.0));
    
    let danger = horizontal_drift_100m_drop > 5000.0;
    if danger {
        out.push_str("  [BAHAYA] Arus terlalu kuat vs kecepatan endap. Plume akan menyebar sangat jauh dari outfall sebelum mencapai dasar laut (risiko ke perairan dangkal).\n\n");
    } else {
        out.push_str("  [OK] Plume cenderung mengendap secara vertikal/lokal.\n\n");
    }

    out.push_str("-- DAMPAK BENTHIK (Dasar Laut) --\n\n");
    out.push_str(&format!("  Est. Luas Zona Mati/hari : {:.2} km² (tertimbun tailing >5cm/hari)\n", footprint_area_km2));
    out.push_str(&format!("  Est. Luas Zona Mati/thn  : {:.1} km² (asumsi sebaran seragam)\n\n", footprint_area_km2 * 365.0));

    out.push_str("Risiko Tambahan (Konteks Nikel Morowali/Obi):\n");
    out.push_str("  1. Upwelling: Risiko tailing naik ke zona penangkapan ikan pelagis\n");
    out.push_str("  2. Toksisitas: Logam berat (Ni, Cr, Co) mencemari rantai makanan laut\n");
    out.push_str("  3. Ekologi: Kematian massal terumbu karang laut dalam & meiofauna\n\n");
    out.push_str("Rekomendasi:\n");
    out.push_str("  Gunakan Dry Stacking (Filter Press) di darat jika topografi memungkinkan, alih-alih DSTP.\n");

    out
}

#[cfg(test)]
mod tests {
    use super::assess;

    #[test]
    fn drift_computation() {
        // settle = 1 mm/s -> 100m drop takes 100,000 s. Current 0.1 m/s -> drift 10,000 m = 10 km
        let res = assess(200.0, 10000.0, 30.0, 0.1, 1.0);
        assert!(res.contains("10.00 km"), "Drift computation wrong: \n{}", res);
        assert!(res.contains("[BAHAYA]"));
    }
}
