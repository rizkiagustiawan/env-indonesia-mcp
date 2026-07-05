/// Stabilitas Lereng TPA (Simplified Bishop / Infinite Slope)
/// Ref: PermenPU 3/2013, Das (2010) Principles of Geotechnical Engineering

pub fn calculate(slope_angle_deg: f64, height_m: f64, unit_weight_kn_m3: f64, cohesion_kpa: f64, friction_deg: f64, pore_pressure_ratio: f64) -> String {
    if slope_angle_deg <= 0.0 || slope_angle_deg >= 90.0 {
        return "ERROR: Sudut lereng harus antara 0 dan 90 derajat (eksklusif).".into();
    }
    if height_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if unit_weight_kn_m3 <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if cohesion_kpa < 0.0 { return "ERROR [E102]: Parameter tidak boleh negatif.".into(); }
    if friction_deg < 0.0 || friction_deg > 60.0 {
        return "ERROR: Sudut geser internal harus antara 0 dan 60 derajat.".into();
    }
    if pore_pressure_ratio < 0.0 || pore_pressure_ratio > 1.0 {
        return "ERROR: Rasio tekanan pori (ru) harus antara 0 dan 1.".into();
    }

    let alpha = slope_angle_deg.to_radians();
    let phi = friction_deg.to_radians();

    // Infinite slope analysis:
    // FoS = c'/(γ×H×sin(α)×cos(α)) + tan(φ')/tan(α) - ru×tan(φ')/tan(α)
    // FoS = c'/(γ×H×sin(α)×cos(α)) + (1 - ru)×tan(φ')/tan(α)
    let sin_a = alpha.sin();
    let cos_a = alpha.cos();
    let tan_a = alpha.tan();
    let tan_phi = phi.tan();

    let cohesion_term = if (sin_a * cos_a) > 1e-12 {
        cohesion_kpa / (unit_weight_kn_m3 * height_m * sin_a * cos_a)
    } else {
        f64::INFINITY
    };

    let friction_term = if tan_a.abs() > 1e-12 {
        (1.0 - pore_pressure_ratio) * tan_phi / tan_a
    } else {
        f64::INFINITY
    };

    let fos = cohesion_term + friction_term;

    // Status per PermenPU 3/2013
    let (status_static, status_label) = if fos >= 1.5 {
        ("AMAN", "FoS ≥ 1.5 — Lereng stabil dengan margin keamanan tinggi")
    } else if fos >= 1.3 {
        ("AMAN", "FoS ≥ 1.3 — Memenuhi syarat minimum statik (PermenPU)")
    } else if fos >= 1.1 {
        ("MARGINAL", "FoS ≥ 1.1 — Memenuhi syarat seismik, tidak memenuhi statik")
    } else if fos >= 1.0 {
        ("TIDAK AMAN", "FoS 1.0–1.1 — Di bawah batas minimum, perlu perbaikan")
    } else {
        ("TIDAK AMAN", "FoS < 1.0 — Lereng tidak stabil, LONGSOR potensial")
    };

    // Recommended maximum slope angle for FoS = 1.3
    // Solve: 1.3 = c'/(γ×H×sin(α)×cos(α)) + (1-ru)×tan(φ)/tan(α)
    // Iterative approach
    let mut recommended_angle = slope_angle_deg;
    if fos < 1.3 {
        for test_deg in (5..=85).rev() {
            let test_rad = (test_deg as f64).to_radians();
            let sa = test_rad.sin();
            let ca = test_rad.cos();
            let ta = test_rad.tan();
            let c_t = if (sa * ca) > 1e-12 { cohesion_kpa / (unit_weight_kn_m3 * height_m * sa * ca) } else { f64::INFINITY };
            let f_t = if ta.abs() > 1e-12 { (1.0 - pore_pressure_ratio) * tan_phi / ta } else { f64::INFINITY };
            let test_fos = c_t + f_t;
            if test_fos >= 1.3 {
                recommended_angle = test_deg as f64;
                break;
            }
        }
    }

    // Critical height for FoS = 1.0 at given angle
    // 1.0 = c'/(γ×Hc×sin(α)×cos(α)) + (1-ru)×tan(φ)/tan(α)
    let friction_only = friction_term; // independent of H
    let h_critical = if friction_only < 1.0 && (sin_a * cos_a) > 1e-12 {
        cohesion_kpa / ((1.0 - friction_only) * unit_weight_kn_m3 * sin_a * cos_a)
    } else {
        f64::INFINITY // friction alone provides FoS ≥ 1.0
    };

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("ANALISIS STABILITAS LERENG TPA\n");
    result.push_str("Ref: PermenPU 3/2013, Das (2010)\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("Metode: Infinite Slope (Simplified)\n\n");

    result.push_str("INPUT:\n");
    result.push_str(&format!("• Sudut lereng (α)     : {:.1}°\n", slope_angle_deg));
    result.push_str(&format!("• Tinggi lereng (H)    : {:.1} m\n", height_m));
    result.push_str(&format!("• Berat isi (γ)        : {:.1} kN/m³\n", unit_weight_kn_m3));
    result.push_str(&format!("• Kohesi (c')          : {:.1} kPa\n", cohesion_kpa));
    result.push_str(&format!("• Sudut geser (φ')     : {:.1}°\n", friction_deg));
    result.push_str(&format!("• Rasio tekanan pori   : {:.2}\n\n", pore_pressure_ratio));

    result.push_str("FORMULA:\n");
    result.push_str("FoS = c'/(γ×H×sin(α)×cos(α)) + (1-ru)×tan(φ')/tan(α)\n\n");

    result.push_str("KOMPONEN FoS:\n");
    result.push_str(&format!("• Kontribusi kohesi    : {:.4}\n", cohesion_term));
    result.push_str(&format!("• Kontribusi gesekan   : {:.4}\n", friction_term));
    result.push_str(&format!("• FoS TOTAL            : {:.4}\n\n", fos));

    result.push_str(&format!("STATUS: {} — {}\n\n", status_static, status_label));

    result.push_str("KRITERIA PermenPU 3/2013:\n");
    result.push_str(&format!("• FoS minimum statik   : 1.3 {}\n",
        if fos >= 1.3 { "✓" } else { "✗ TIDAK MEMENUHI" }));
    result.push_str(&format!("• FoS minimum seismik  : 1.1 {}\n\n",
        if fos >= 1.1 { "✓" } else { "✗ TIDAK MEMENUHI" }));

    if fos < 1.3 {
        result.push_str(&format!("REKOMENDASI SUDUT LERENG (FoS ≥ 1.3): {:.0}° (1V:{:.1}H)\n",
            recommended_angle, 1.0 / recommended_angle.to_radians().tan()));
    }

    if h_critical.is_finite() {
        result.push_str(&format!("TINGGI KRITIS (FoS = 1.0): {:.1} m\n", h_critical));
    } else {
        result.push_str("TINGGI KRITIS: Tidak terbatas (gesekan saja sudah stabil)\n");
    }

    result.push_str("\nPARAMETER TIPIKAL SAMPAH PERKOTAAN:\n");
    result.push_str("• γ  : 8–14 kN/m³ (tergantung umur & komposisi)\n");
    result.push_str("• c' : 0–25 kPa\n");
    result.push_str("• φ' : 20–35°\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}
