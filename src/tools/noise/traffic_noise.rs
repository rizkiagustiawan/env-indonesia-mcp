/// Traffic Noise Prediction (CoRTN Simplified / FHWA TNM Basic)
/// Ref: CoRTN (UK DoT, 1988), FHWA TNM 2.5, KepmenLH 48/1996

pub fn calculate(
    vehicles_per_hour: f64,
    speed_kmh: f64,
    distance_m: f64,
    heavy_vehicle_pct: f64,
    gradient_pct: f64,
    ground_type: &str,
    barrier_height_m: Option<f64>,
) -> String {
    if vehicles_per_hour <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if speed_kmh <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0 km/h.".into();
    }
    if distance_m <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0 m.".into();
    }
    if heavy_vehicle_pct < 0.0 || heavy_vehicle_pct > 100.0 {
        return "ERROR: Persentase kendaraan berat harus 0-100%.".into();
    }

    let q = vehicles_per_hour;
    let v = speed_kmh;

    // Basic reference noise level (CoRTN): L10 at 10m reference distance
    let l_basic = 42.2 + 10.0 * q.log10();

    // Speed correction
    let speed_corr = 33.0 * (v + 40.0 + 500.0 / v).log10() - 68.8;

    // Heavy vehicle correction (CoRTN): 10·log10(1 + 5P/V), P = % heavy vehicles
    let hv_corr = if heavy_vehicle_pct > 0.0 {
        10.0 * (1.0 + 5.0 * heavy_vehicle_pct / v).log10()
    } else {
        0.0
    };

    // Distance attenuation (from reference 13.5m)
    let dist_atten = if distance_m > 13.5 {
        -10.0 * (distance_m / 13.5).log10()
    } else {
        10.0 * (13.5 / distance_m).log10() // closer = louder
    };

    // Ground absorption correction
    let ground_corr = match ground_type.to_lowercase().as_str() {
        "hard" | "keras" | "aspal" | "beton" => 0.0,
        "soft" | "lunak" | "rumput" | "tanah" => {
            if distance_m > 20.0 { -3.0 } else { -1.5 }
        }
        "mixed" | "campuran" => {
            if distance_m > 20.0 { -1.5 } else { -0.75 }
        }
        _ => return format!("ERROR: Ground type '{}' tidak dikenal. Gunakan: hard/keras, soft/lunak, mixed/campuran.", ground_type),
    };

    // Gradient correction: +0.3 per % gradient
    let grad_corr = 0.3 * gradient_pct.abs();

    // Sum before barrier
    let l_no_barrier = l_basic + speed_corr + hv_corr + dist_atten + ground_corr + grad_corr;

    // Barrier insertion loss (Maekawa diffraction)
    let barrier_il = if let Some(h_barrier) = barrier_height_m {
        if h_barrier <= 0.0 {
            0.0
        } else {
            // Simplified Maekawa: IL ≈ 10×log10(3 + 20×N)
            // N = 2×δ/λ, δ = path difference, λ = wavelength (~0.68m for 500Hz)
            // Simplified: assume source height 0.5m, receiver 1.5m, barrier at midpoint
            let source_h = 0.5_f64;
            let receiver_h = 1.5_f64;
            let half_d = distance_m / 2.0;
            let d_over = (half_d * half_d + (h_barrier - source_h).powi(2)).sqrt()
                + (half_d * half_d + (h_barrier - receiver_h).powi(2)).sqrt();
            let d_direct = (distance_m * distance_m + (receiver_h - source_h).powi(2)).sqrt();
            let delta = d_over - d_direct;
            let lambda = 0.68; // 500 Hz characteristic
            let n = 2.0 * delta / lambda;
            if n > 0.0 {
                let il = 10.0 * (3.0 + 20.0 * n).log10();
                il.min(25.0) // practical max IL
            } else {
                0.0
            }
        }
    } else {
        0.0
    };

    let l_final = l_no_barrier - barrier_il;

    // Compliance check per KepmenLH 48/1996
    let zones: &[(&str, f64)] = &[
        ("Perumahan (Zona A)", 55.0),
        ("Perdagangan (Zona B)", 70.0),
        ("Industri (Zona C)", 73.0),
        ("Ruang Terbuka Hijau (Zona D)", 50.0),
        ("Rumah Sakit", 55.0),
        ("Sekolah", 55.0),
    ];

    // Required buffer distance for 55 dBA (residential)
    let target_db = 55.0;
    let mut buffer_dist = distance_m;
    if l_final > target_db {
        // Iterative: increase distance until noise drops below target
        let mut d = distance_m;
        loop {
            let atten = if d > 13.5 {
                -10.0 * (d / 13.5).log10()
            } else {
                0.0
            };
            let l_test =
                l_basic + speed_corr + hv_corr + atten + ground_corr + grad_corr - barrier_il;
            if l_test <= target_db || d > 10000.0 {
                break;
            }
            d *= 1.05;
        }
        buffer_dist = d;
    }

    let mut out = String::from("══════════════════════════════════════════════\n  PREDIKSI KEBISINGAN LALU LINTAS\n══════════════════════════════════════════════\n");
    out.push_str("Ref: CoRTN (UK DoT, 1988), FHWA TNM, KepmenLH 48/1996\n\n");

    out.push_str(&format!(
        "INPUT:\n  Volume lalu lintas  = {:.0} kendaraan/jam\n  Kecepatan           = {:.0} km/h\n  Jarak reseptor       = {:.1} m\n  Kendaraan berat      = {:.1}%\n  Gradien jalan        = {:.1}%\n  Tipe permukaan       = {}\n",
        q, v, distance_m, heavy_vehicle_pct, gradient_pct, ground_type
    ));
    if let Some(h) = barrier_height_m {
        out.push_str(&format!("  Tinggi barrier       = {:.1} m\n", h));
    } else {
        out.push_str("  Barrier              = Tidak ada\n");
    }

    out.push_str(&format!(
        "\nKOMPONEN PERHITUNGAN:\n  L dasar (42.2 + 10×log₁₀(Q))  = {:.1} dBA\n  Koreksi kecepatan               = {:+.1} dBA\n  Koreksi kendaraan berat          = {:+.1} dBA\n  Atenuasi jarak                   = {:+.1} dBA\n  Koreksi tanah                    = {:+.1} dBA\n  Koreksi gradien                  = {:+.1} dBA\n",
        l_basic, speed_corr, hv_corr, dist_atten, ground_corr, grad_corr
    ));
    if barrier_il > 0.0 {
        out.push_str(&format!(
            "  Insertion loss barrier           = -{:.1} dBA\n",
            barrier_il
        ));
    }

    out.push_str(&format!(
        "\n  L10 tanpa barrier = {:.1} dBA\n",
        l_no_barrier
    ));
    out.push_str(&format!("  L10 FINAL         = {:.1} dBA\n\n", l_final));

    // Compliance table
    out.push_str("KEPATUHAN KepmenLH 48/1996:\n");
    out.push_str(&format!(
        "  {:35} {:>8} {:>8} {:>10}\n",
        "Zona", "Baku Mutu", "Prediksi", "Status"
    ));
    out.push_str(&format!(
        "  {:35} {:>8} {:>8} {:>10}\n",
        "─".repeat(35),
        "─".repeat(8),
        "─".repeat(8),
        "─".repeat(10)
    ));
    for (name, limit) in zones {
        let status = if l_final <= *limit {
            "✓ PATUH"
        } else {
            "✗ MELEBIHI"
        };
        out.push_str(&format!(
            "  {:35} {:>6.0} dBA {:>6.1} dBA {:>10}\n",
            name, limit, l_final, status
        ));
    }

    if l_final > target_db {
        out.push_str(&format!(
            "\nJARAK BUFFER MINIMUM (zona perumahan {:.0} dBA):\n  {:.0} m dari tepi jalan\n",
            target_db, buffer_dist
        ));
    }

    out.push_str("\n══════════════════════════════════════════════\n");
    out
}

#[cfg(test)]
mod tests {
    use super::calculate;

    #[test]
    fn cortn_heavy_vehicle_correction() {
        // P=20%, V=50 → 10·log10(1 + 5·20/50) = 10·log10(3) ≈ +4.8 dB (was 8·log10(1.8) ≈ +2.0)
        let result = calculate(1000.0, 50.0, 100.0, 20.0, 0.0, "hard", None);
        assert!(result.contains("+4.8"), "CoRTN heavy-vehicle correction wrong:\n{result}");
        assert!(!result.contains("kendaraan berat          = +2.0"), "old buggy hv factor present:\n{result}");
    }
}
