/// Physics-Informed Advanced Validator V3
/// Menyuntikkan hukum fisika tingkat lanjut:
/// 1. TROPOMI Error Bounds (Conformal Prediction Bias)
/// 2. Planetary Boundary Layer (PBL) Inversion
/// 3. Bingham Plastic Rheology (Debris Flow)
/// 4. Tropical Atmospheric Kinetics (Photolysis)

pub fn validate_advanced_physics(
    gas_type: &str,
    concentration: f64,
    time_of_day: &str,
    fluid_type: &str,
    slope_angle_deg: f64,
    depth_m: f64,
) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Advanced Physics Validator V3\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    let mut validations = Vec::new();

    let gas_upper = gas_type.to_uppercase();

    // 1. TROPOMI UQ (Uncertainty Quantification)
    // Sensor satelit TROPOMI mengalami underestimasi 20-40% di daerah tropis karena efek albedo awan/aerosol
    if gas_upper == "NO2" && concentration > 0.0 {
        let true_lower = concentration / (1.0 - 0.20);
        let true_upper = concentration / (1.0 - 0.40);
        validations.push(format!(
            "[TROPOMI UQ] Sensor satelit mendeteksi NO2: {:.1} µg/m³. Namun, karena underestimation 20%-40% di wilayah tropis, nilai *ground-truth* diproyeksikan berada di rentang: {:.1} - {:.1} µg/m³.",
            concentration, true_lower, true_upper
        ));
    }

    // 2. PBL Inversion (Monin-Obukhov)
    let time_lower = time_of_day.to_lowercase();
    if time_lower == "night" || time_lower == "malam" {
        validations.push(
            "[METEOROLOGI PBL] Waktu MALAM. Inversi suhu permukaan di ekuator akan menekan Ketinggian Campur (Mixing Height) ke level kritis (<300m). Polutan dari cerobong akan gagal terdispersi dan memicu fenomena Fumigasi (terperangkap di zona pernapasan warga)."
            .to_string()
        );
    } else if time_lower == "day" || time_lower == "siang" {
        validations.push(
            "[METEOROLOGI PBL] Waktu SIANG. Konveksi termal kuat akan mengangkat Mixing Height ke 1000m-2500m. Dispersi polutan sangat baik."
            .to_string()
        );
    }

    // 3. Debris Flow (Bingham Plastic Rheology)
    let fluid_lower = fluid_type.to_lowercase();
    if fluid_lower == "mud" || fluid_lower == "lumpur" || fluid_lower == "debris" {
        // T = rho * g * d * sin(theta)
        let rho = 1500.0; // densitas lumpur pekat ~1500 kg/m3
        let g = 9.81;
        let tau = rho * g * depth_m * slope_angle_deg.to_radians().sin();
        let yield_stress = 100.0; // Pa (batas minimum aliran lumpur pekat)

        validations.push(format!(
            "[RHEOLOGI BINGHAM] Tegangan Geser (Shear Stress) lereng {:.1}° dengan kedalaman {:.1}m adalah {:.1} Pa.",
            slope_angle_deg, depth_m, tau
        ));

        if tau < yield_stress {
            validations.push(format!(
                "⛔ [RHEOLOGI] Tegangan Geser ({:.1} Pa) < Yield Stress ({} Pa). Aliran debris/lumpur TIDAK AKAN BERGERAK secara fisik karena viskoplastisitas tanah mengatasi gaya gravitasi.",
                tau, yield_stress
            ));
        } else {
            validations.push(format!(
                "⚠️ [RHEOLOGI] Tegangan Geser ({:.1} Pa) > Yield Stress ({} Pa). LUMPUR AKAN MENGALIR DERAS (Debris Flow Hazard).",
                tau, yield_stress
            ));
        }
    } else if fluid_lower == "water" || fluid_lower == "air" {
        validations.push("[HIDRODINAMIKA] Tipe fluida: Air (Newtonian Fluid). Bergerak bebas tanpa Yield Stress.".to_string());
    }

    // 4. Gas Kinetics (Fotolisis & Reaksi Radikal OH Tropis)
    if gas_upper == "NO2" {
        validations.push(
            "[KINETIKA ATMOSFER] Analisis Gas NO2 di Tropis. Radiasi UV ekuatorial dan konsentrasi radikal Hidroksil (OH-) sangat tinggi. Usia paruh (lifetime) gas NO2 jatuh secara drastis menjadi hanya 2-6 jam. Jangkauan penyebarannya sangat lokal."
            .to_string()
        );
    } else if gas_upper == "CH4" || gas_upper == "METHANE" {
        validations.push(
            "[KINETIKA ATMOSFER] Analisis Metana (CH4). Usia paruh gas di ekuator sekitar 8-9 tahun. Sangat persisten sebagai Gas Rumah Kaca."
            .to_string()
        );
    }

    if validations.is_empty() {
        out.push_str("Status: Parameter tidak men-trigger aturan fisika V3 (UQ, Rheologi, atau Kinetika Ekuator).\n");
    } else {
        for (i, v) in validations.iter().enumerate() {
            out.push_str(&format!("{}. {}\n\n", i + 1, v));
        }
    }

    out
}
