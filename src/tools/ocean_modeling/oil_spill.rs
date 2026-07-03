use std::process::Command;

pub fn simulate_4d(volume_m3: f64, oil_type: &str, wind_speed: f64, wind_dir: f64, current_speed: f64, current_dir: f64, hours: u32, output: &str) -> String {
    // Oil spill drift: 3% wind + 100% current
    let drift_wind = 0.03 * wind_speed;
    let total_drift_x = (drift_wind * (wind_dir as f64).to_radians().sin() + current_speed * (current_dir as f64).to_radians().sin()) * (hours as f64) * 3600.0;
    let total_drift_y = (drift_wind * (wind_dir as f64).to_radians().cos() + current_speed * (current_dir as f64).to_radians().cos()) * (hours as f64) * 3600.0;
    let total_km = (total_drift_x * total_drift_x + total_drift_y * total_drift_y).sqrt() / 1000.0;

    // Evaporation (first order decay)
    let k_evap = match oil_type.to_lowercase().as_str() {
        "crude" | "mentah" => 0.02,
        "diesel" => 0.08,
        "gasoline" | "bensin" => 0.20,
        "bunker" | "hfo" => 0.005,
        _ => 0.02,
    };
    let volume_remaining = volume_m3 * (-k_evap * (hours as f64)).exp();
    let evaporated_pct = (1.0 - volume_remaining / volume_m3) * 100.0;

    // Slick area (Fay spreading)
    let area_m2 = 1000.0 * volume_remaining.powf(0.67) * ((hours as f64) * 3600.0).powf(0.33);
    let area_km2 = area_m2 / 1e6;

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Oil Spill Trajectory & Fate\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: NOAA GNOME simplified, Fay (1969) spreading\n");
    out.push_str("⚠️ Screening-level model. Gunakan GNOME/OSCAR untuk operasional.\n\n");
    out.push_str(&format!("INPUT:\n  Volume = {:.0} m³\n  Tipe minyak = {}\n  Angin = {:.1} m/s @ {:.0}°\n  Arus = {:.2} m/s @ {:.0}°\n  Durasi = {} jam\n\n",
        volume_m3, oil_type, wind_speed, wind_dir, current_speed, current_dir, hours));
    out.push_str(&format!("TRAJECTORY:\n  Drift: {:.1} km ({:.0}m E, {:.0}m N)\n  Wind contribution: {:.1} m/s (3% of wind)\n  Current contribution: {:.2} m/s\n\n",
        total_km, total_drift_x, total_drift_y, drift_wind, current_speed));
    out.push_str(&format!("WEATHERING:\n  Volume awal: {:.0} m³\n  Evaporasi ({}h): {:.1}%\n  Volume sisa: {:.1} m³\n  Area slick: {:.3} km²\n  k_evap: {:.3} /jam ({})\n\n",
        volume_m3, hours, evaporated_pct, volume_remaining, area_km2, k_evap, oil_type));

    // Timeline
    out.push_str("TIMELINE:\n");
    for t in [1, 3, 6, 12, 24, 48].iter() {
        if *t <= hours {
            let v = volume_m3 * (-k_evap * (*t as f64)).exp();
            let a = 1000.0 * v.powf(0.67) * ((*t as f64) * 3600.0).powf(0.33) / 1e6;
            let d = total_km * (*t as f64) / (hours as f64);
            out.push_str(&format!("  {}h: drift {:.1}km, volume {:.0}m³ ({:.0}%), area {:.3}km²\n",
                t, d, v, v/volume_m3*100.0, a));
        }
    }

    // ESI classification
    out.push_str("\nSENSITIVITAS PANTAI (ESI Indonesia):\n");
    out.push_str("  1A-2B: Pantai batu terjal (self-cleaning cepat)\n");
    out.push_str("  3A-4: Pantai pasir halus (rentan sedang)\n");
    out.push_str("  5-7: Pantai berlumpur (sulit dibersihkan)\n");
    out.push_str("  8-10: Mangrove/rawa (SANGAT SENSITIF — pembersihan bertahun-tahun)\n");
    out
}
