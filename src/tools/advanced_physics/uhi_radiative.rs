/// Urban Heat Island (UHI) Radiative Transfer
/// Estimasi kenaikan suhu lingkungan akibat albedo & Sky View Factor

pub fn calculate_uhi(
    albedo_urban: f64, 
    sky_view_factor: f64, 
    solar_insolation_w: f64, 
    ambient_temp_c: f64
) -> String {
    if albedo_urban < 0.0 || albedo_urban > 1.0 {
        return "ERROR: Albedo harus di antara 0.0 (menyerap semua) dan 1.0 (memantulkan semua).".into();
    }
    if sky_view_factor <= 0.0 || sky_view_factor > 1.0 {
        return "ERROR: Sky View Factor (SVF) harus di antara 0.0 dan 1.0.".into();
    }

    // Pendekatan fisika sederhana:
    // Energi tertahan (Q_net) = (1 - Albedo) * Insolation
    // Radiasi terperangkap (Trapping) = (1 - SVF) * Q_net
    // Kenaikan suhu perkiraan = Trapping / (Konstanta Konveksi)
    // Anggap Konstanta konveksi h_c sekitar 20 W/m²K
    
    let q_net = (1.0 - albedo_urban) * solar_insolation_w;
    let heat_trapped = (1.0 - sky_view_factor) * q_net;
    let h_c = 20.0; 
    let delta_t = heat_trapped / h_c;

    let final_temp = ambient_temp_c + delta_t;

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  UHI Radiative Transfer Model\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Simplified Urban Surface Energy Balance\n\n");
    
    out.push_str(&format!("INPUT:\n  Albedo Perkotaan    = {:.2} ({}% pantul)\n  Sky View Factor (SVF) = {:.2}\n  Radiasi Matahari    = {:.0} W/m²\n  Suhu Udara Sekitar  = {:.1}°C\n\n", 
        albedo_urban, albedo_urban * 100.0, sky_view_factor, solar_insolation_w, ambient_temp_c));
        
    out.push_str("HASIL:\n");
    out.push_str(&format!("  Panas Terperangkap  = {:.1} W/m²\n", heat_trapped));
    out.push_str(&format!("  Kenaikan Suhu (ΔT)  = +{:.2}°C\n", delta_t));
    out.push_str(&format!("  Suhu Mikro (Aktual) = {:.1}°C\n\n", final_temp));

    if delta_t > 3.0 {
        out.push_str("⚠️ ANALISIS: Geometri urban (SVF kecil/gedung rapat) menyebabkan efek Urban Heat Island SANGAT KUAT.\n");
        out.push_str("   -> Solusi: Cat atap warna putih (tingkatkan albedo) atau tanam vegetasi atap.\n");
    } else {
        out.push_str("✅ ANALISIS: Kondisi termal perkotaan dalam batas wajar.\n");
    }

    out
}
