/// Physics-Informed Validator Engine untuk Environmental Engineering Indonesia
/// Berdasarkan: USDA TR-55, FAO-56, RUSLE (Renard 1997), Streeter-Phelps (1925),
/// PP 22/2021, PermenLHK 68/2016, KepMenLH 48/1996, CERC SPM 1984, APHA Standards
/// 
/// Modul ini adalah "jantung" Physics-Informed Agent (PINN).
/// Semua output LLM yang menyangkut data lingkungan WAJIB divalidasi di sini.

use serde::{Deserialize, Serialize};
use rmcp::schemars::{self, JsonSchema};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ValidatorParam {
    // === REMOTE SENSING ===
    #[schemars(description = "NDVI value (-1.0 to 1.0)")]
    pub ndvi: Option<f64>,
    #[schemars(description = "Surface reflectance / albedo (0.0 to 1.0)")]
    pub reflectance: Option<f64>,
    #[schemars(description = "Cloud fraction (0.0 to 1.0)")]
    pub cloud_cover: Option<f64>,
    #[schemars(description = "Aerosol Optical Depth (>= 0)")]
    pub aod: Option<f64>,

    // === ATMOSFER & DISPERSI ===
    #[schemars(description = "Pollutant concentration in µg/m³ (>= 0)")]
    pub pollutant_concentration: Option<f64>,
    #[schemars(description = "Wind speed in m/s (>= 0.28 for dispersion)")]
    pub wind_speed: Option<f64>,
    #[schemars(description = "PM2.5 concentration µg/m³")]
    pub pm25: Option<f64>,
    #[schemars(description = "PM10 concentration µg/m³")]
    pub pm10: Option<f64>,
    #[schemars(description = "NO2 concentration µg/m³")]
    pub no2: Option<f64>,
    #[schemars(description = "SO2 concentration µg/m³")]
    pub so2: Option<f64>,
    #[schemars(description = "ISPU index value (0-500)")]
    pub ispu: Option<f64>,

    // === HIDROLOGI ===
    #[schemars(description = "SCS Curve Number (0-100)")]
    pub cn: Option<f64>,
    #[schemars(description = "Runoff coefficient C (0-1)")]
    pub c_runoff: Option<f64>,
    #[schemars(description = "Evapotranspiration ET0 mm/day (0-15 tropis)")]
    pub et0: Option<f64>,
    #[schemars(description = "Crop coefficient Kc (0.1-1.5)")]
    pub kc: Option<f64>,
    #[schemars(description = "Manning's n roughness (0.03-0.15 tropis)")]
    pub mannings_n: Option<f64>,
    #[schemars(description = "Rainfall P in mm (>= 0)")]
    pub rainfall_mm: Option<f64>,
    #[schemars(description = "Runoff Q in mm (>= 0, must be <= rainfall)")]
    pub runoff_mm: Option<f64>,
    #[schemars(description = "Source elevation (m) for flow direction check")]
    pub elevation_source: Option<f64>,
    #[schemars(description = "Target elevation (m) for flow direction check")]
    pub elevation_target: Option<f64>,

    // === KUALITAS AIR ===
    #[schemars(description = "Dissolved Oxygen mg/L (0-14.6)")]
    pub do_mgl: Option<f64>,
    #[schemars(description = "BOD mg/L (>= 0)")]
    pub bod: Option<f64>,
    #[schemars(description = "COD mg/L (>= 0, must be >= BOD)")]
    pub cod: Option<f64>,
    #[schemars(description = "pH (0-14)")]
    pub ph: Option<f64>,
    #[schemars(description = "TSS mg/L (>= 0)")]
    pub tss: Option<f64>,
    #[schemars(description = "Water temperature °C for DO saturation check")]
    pub water_temp_c: Option<f64>,
    #[schemars(description = "Deoxygenation rate k1 (/day, 0.01-0.7)")]
    pub k1: Option<f64>,
    #[schemars(description = "Reaeration rate k2 (/day, 0.1-5.0, must > k1 for recovery)")]
    pub k2: Option<f64>,

    // === EROSI TANAH (RUSLE) ===
    #[schemars(description = "Rainfall erosivity R (MJ.mm/ha.hr.yr, 0-15000)")]
    pub r_erosivity: Option<f64>,
    #[schemars(description = "Soil erodibility K (0-1)")]
    pub k_erodibility: Option<f64>,
    #[schemars(description = "Cover management C factor (0-1)")]
    pub c_cover: Option<f64>,
    #[schemars(description = "Conservation practice P factor (0-1)")]
    pub p_practice: Option<f64>,

    // === KEBISINGAN ===
    #[schemars(description = "Noise level in dB (0-194)")]
    pub noise_db: Option<f64>,

    // === GELOMBANG & PESISIR ===
    #[schemars(description = "Significant wave height Hs in m (>= 0)")]
    pub wave_height: Option<f64>,
    #[schemars(description = "Wave period T in seconds (> 0)")]
    pub wave_period: Option<f64>,
    #[schemars(description = "Current speed in m/s (>= 0)")]
    pub current_speed: Option<f64>,
}

pub fn validate(p: ValidatorParam) -> String {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut info: Vec<String> = Vec::new();

    // ========== REMOTE SENSING ==========
    if let Some(v) = p.ndvi {
        if v < -1.0 || v > 1.0 {
            errors.push(format!("[RADIOMETRI] NDVI={:.4} TIDAK VALID. Rentang matematis mutlak: -1.0 sampai 1.0. Rumus: (NIR-Red)/(NIR+Red).", v));
        } else {
            let cat = if v < 0.0 { "Air/Awan/Salju" } else if v < 0.2 { "Tanah gundul/batu" } else if v < 0.4 { "Vegetasi jarang" } else if v < 0.6 { "Vegetasi sedang" } else { "Vegetasi lebat/hutan" };
            info.push(format!("[RADIOMETRI] NDVI={:.4} → {}", v, cat));
        }
    }
    if let Some(v) = p.reflectance {
        if v < 0.0 || v > 1.0 {
            errors.push(format!("[RADIOMETRI] Reflektansi={:.4} TIDAK VALID. Jika negatif: koreksi atmosferik overcorrected (aerosol terlalu tinggi). Rentang fisik: 0.0-1.0.", v));
        }
    }
    if let Some(v) = p.cloud_cover {
        if v < 0.0 || v > 1.0 {
            errors.push(format!("[METEOROLOGI] Cloud Fraction={:.2} TIDAK VALID. Harus 0.0 (cerah) sampai 1.0 (tertutup total).", v));
        } else if v > 0.2 {
            warnings.push(format!("[TROPOMI] Cloud Fraction={:.2} (>0.2). Data gas NO₂/CH₄ dari Sentinel-5P TIDAK RELIABLE pada piksel ini. Awan menghalangi pembacaan kolom vertikal permukaan (ghost-column problem). Gunakan SAR (Sentinel-1) sebagai alternatif.", v));
        }
    }
    if let Some(v) = p.aod {
        if v < 0.0 {
            errors.push(format!("[RADIOMETRI] AOD={:.3} TIDAK VALID. Aerosol Optical Depth tidak bisa negatif.", v));
        } else if v > 2.0 {
            warnings.push(format!("[RADIOMETRI] AOD={:.3} sangat tinggi. Koreksi atmosferik Sen2Cor/6SV kemungkinan menghasilkan error besar pada band biru/hijau.", v));
        }
    }

    // ========== ATMOSFER & DISPERSI ==========
    if let Some(v) = p.pollutant_concentration {
        if v < 0.0 {
            errors.push(format!("[KONSERVASI MASSA] Konsentrasi polutan={:.2} µg/m³ TIDAK MUNGKIN negatif. Pelanggaran Hukum Konservasi Massa.", v));
        }
    }
    if let Some(v) = p.wind_speed {
        if v < 0.0 {
            errors.push(format!("[FISIKA] Kecepatan angin={:.2} m/s negatif TIDAK VALID.", v));
        } else if v < 0.28 {
            errors.push(format!("[AERMOD/GAUSSIAN] Kecepatan angin={:.2} m/s < 0.28 m/s. Model dispersi atmosfer TIDAK VALID pada calm wind. Gaussian plume menghasilkan pembagian mendekati nol → konsentrasi tak hingga (singularitas).", v));
        }
    }
    if let Some(v) = p.pm25 {
        if v < 0.0 { errors.push(format!("[UDARA] PM2.5={:.1} negatif TIDAK VALID.", v)); }
        else if v > 65.0 { warnings.push(format!("[BAKU MUTU PP 22/2021] PM2.5={:.1} µg/m³ MELEBIHI baku mutu 24 jam (65 µg/m³).", v)); }
    }
    if let Some(v) = p.pm10 {
        if v < 0.0 { errors.push(format!("[UDARA] PM10={:.1} negatif TIDAK VALID.", v)); }
        else if v > 150.0 { warnings.push(format!("[BAKU MUTU PP 22/2021] PM10={:.1} µg/m³ MELEBIHI baku mutu 24 jam (150 µg/m³).", v)); }
    }
    if let Some(v) = p.no2 {
        if v < 0.0 { errors.push(format!("[UDARA] NO2={:.1} negatif TIDAK VALID.", v)); }
        else if v > 200.0 { warnings.push(format!("[BAKU MUTU PP 22/2021] NO2={:.1} µg/m³ MELEBIHI baku mutu 1 jam (200 µg/m³).", v)); }
    }
    if let Some(v) = p.so2 {
        if v < 0.0 { errors.push(format!("[UDARA] SO2={:.1} negatif TIDAK VALID.", v)); }
        else if v > 75.0 { warnings.push(format!("[BAKU MUTU PP 22/2021] SO2={:.1} µg/m³ MELEBIHI baku mutu 24 jam (75 µg/m³).", v)); }
    }
    if let Some(v) = p.ispu {
        if v < 0.0 || v > 500.0 {
            errors.push(format!("[ISPU] Nilai={:.0} di luar rentang valid 0-500 (PermenLHK 14/2020).", v));
        } else {
            let cat = if v <= 50.0 { "BAIK (Hijau)" } else if v <= 100.0 { "SEDANG (Biru)" } else if v <= 200.0 { "TIDAK SEHAT (Kuning)" } else if v <= 300.0 { "SANGAT TIDAK SEHAT (Merah)" } else { "BERBAHAYA (Hitam)" };
            info.push(format!("[ISPU] Nilai={:.0} → Kategori: {}", v, cat));
        }
    }

    // ========== HIDROLOGI ==========
    if let Some(v) = p.cn {
        if v < 0.0 || v > 100.0 {
            errors.push(format!("[SCS-CN] Curve Number={:.1} TIDAK VALID. Rentang: 0 (infiltrasi sempurna) sampai 100 (impervious). Ref: USDA TR-55.", v));
        }
    }
    if let Some(v) = p.c_runoff {
        if v < 0.0 || v > 1.0 {
            errors.push(format!("[RASIONAL] Koefisien limpasan C={:.2} TIDAK VALID. Rentang: 0 (tanpa limpasan) sampai 1 (semua menjadi limpasan).", v));
        }
    }
    if let Some(v) = p.et0 {
        if v < 0.0 {
            errors.push(format!("[PENMAN-MONTEITH] ET0={:.2} mm/hari TIDAK VALID. Evapotranspirasi tidak boleh negatif.", v));
        } else if v > 15.0 {
            errors.push(format!("[PENMAN-MONTEITH] ET0={:.2} mm/hari MELEBIHI batas fisik tropis (~15 mm/hari). Cek input radiasi netto (Rn) dan kecepatan angin (u2). Ref: FAO-56.", v));
        }
    }
    if let Some(v) = p.kc {
        if v < 0.1 || v > 1.5 {
            errors.push(format!("[FAO-56] Koefisien tanaman Kc={:.2} TIDAK VALID. Rentang: 0.1 (tanaman dormant) sampai 1.5 (sawah tergenang). Ref: FAO Irrigation Paper 56.", v));
        }
    }
    if let Some(v) = p.mannings_n {
        if v < 0.01 || v > 0.20 {
            errors.push(format!("[HIDROLOGI] Manning's n={:.3} di luar rentang wajar. Lahan terbuka tropis: 0.030, Hutan hujan lebat: 0.150. Ref: Chow (1959).", v));
        }
    }
    // Runoff tidak boleh melebihi curah hujan
    if let (Some(p_rain), Some(q)) = (p.rainfall_mm, p.runoff_mm) {
        if q < 0.0 { errors.push(format!("[HIDROLOGI] Runoff={:.1} mm TIDAK BOLEH negatif.", q)); }
        if q > p_rain {
            errors.push(format!("[KONSERVASI MASSA] Runoff={:.1} mm MELEBIHI curah hujan={:.1} mm. Limpasan tidak bisa melebihi input presipitasi (Hukum Konservasi Massa).", q, p_rain));
        }
    }
    // Air mengalir ke bawah (gravitasi)
    if let (Some(src), Some(tgt)) = (p.elevation_source, p.elevation_target) {
        if tgt > src {
            errors.push(format!("[GRAVITASI] Aliran dari elevasi {:.1}m ke {:.1}m MELANGGAR hukum gravitasi. Air permukaan mengalir dari elevasi tinggi ke rendah (Saint-Venant). Cek ulang DEM.", src, tgt));
        }
    }

    // ========== KUALITAS AIR ==========
    if let Some(v) = p.do_mgl {
        if v < 0.0 {
            errors.push(format!("[KIMIA AIR] DO={:.2} mg/L TIDAK BOLEH negatif.", v));
        } else if v > 14.6 {
            warnings.push(format!("[KIMIA AIR] DO={:.2} mg/L melebihi saturasi pada 0°C (14.6 mg/L). Supersaturasi mungkin terjadi akibat fotosintesis algae, tapi perlu verifikasi.", v));
        }
    }
    // Cek DO vs suhu air (saturasi turun seiring suhu naik)
    if let (Some(d), Some(t)) = (p.do_mgl, p.water_temp_c) {
        if t > 0.0 {
            // Rumus empiris sederhana saturasi DO (approx)
            let do_sat = 14.6 - 0.394 * t + 0.00714 * t * t - 0.0000646 * t * t * t;
            if d > do_sat * 1.5 {
                warnings.push(format!("[KIMIA AIR] DO={:.2} mg/L pada suhu {:.1}°C. Saturasi DO pada suhu ini ~{:.2} mg/L. Nilai melebihi 150% saturasi — kemungkinan error sensor atau blooming algae.", d, t, do_sat));
            }
        }
    }
    if let Some(v) = p.bod {
        if v < 0.0 { errors.push(format!("[KIMIA AIR] BOD={:.2} mg/L TIDAK BOLEH negatif.", v)); }
    }
    if let Some(v) = p.cod {
        if v < 0.0 { errors.push(format!("[KIMIA AIR] COD={:.2} mg/L TIDAK BOLEH negatif.", v)); }
    }
    // COD harus >= BOD (karena COD mengukur SEMUA oksidasi)
    if let (Some(b), Some(c)) = (p.bod, p.cod) {
        if c < b {
            errors.push(format!("[KIMIA AIR] COD={:.2} < BOD={:.2}. SECARA KIMIA TIDAK MUNGKIN. COD selalu ≥ BOD karena COD mengukur semua oksidasi (kimiawi + biologis). Cek ulang data laboratorium.", c, b));
        }
    }
    if let Some(v) = p.ph {
        if v < 0.0 || v > 14.0 {
            errors.push(format!("[KIMIA AIR] pH={:.2} TIDAK VALID. Rentang: 0-14.", v));
        }
    }
    if let Some(v) = p.tss {
        if v < 0.0 { errors.push(format!("[KIMIA AIR] TSS={:.1} mg/L TIDAK BOLEH negatif.", v)); }
    }
    // Streeter-Phelps: k2 harus > k1 untuk pemulihan sungai
    if let (Some(k1_v), Some(k2_v)) = (p.k1, p.k2) {
        if k1_v < 0.01 || k1_v > 0.7 {
            errors.push(format!("[STREETER-PHELPS] k1={:.3} /hari di luar rentang (0.01-0.7). Ref: deoxygenation rate standar pada 20°C.", k1_v));
        }
        if k2_v < 0.1 || k2_v > 5.0 {
            errors.push(format!("[STREETER-PHELPS] k2={:.3} /hari di luar rentang (0.1-5.0). Ref: reaeration rate tergantung kondisi sungai.", k2_v));
        }
        if k2_v <= k1_v {
            errors.push(format!("[STREETER-PHELPS] k2={:.3} ≤ k1={:.3}. Sungai TIDAK BISA self-purify. DO akan terus turun menuju kondisi anoksik (kematian biota). Skenario ini menandakan pencemaran berat yang membutuhkan intervensi.", k2_v, k1_v));
        }
    }

    // ========== EROSI TANAH (RUSLE) ==========
    if let Some(v) = p.r_erosivity {
        if v < 0.0 { errors.push(format!("[RUSLE] R-erosivity={:.0} TIDAK BOLEH negatif.", v)); }
        else if v > 15000.0 { warnings.push(format!("[RUSLE] R={:.0} sangat tinggi (>15000). Tipikal Indonesia: 2000-8000 MJ.mm/ha.hr.yr. Cek rumus Bols (1978).", v)); }
    }
    if let Some(v) = p.k_erodibility {
        if v < 0.0 || v > 1.0 { errors.push(format!("[RUSLE] K-erodibility={:.3} TIDAK VALID. Rentang: 0 (sangat tahan) sampai 1 (sangat erodibel).", v)); }
    }
    if let Some(v) = p.c_cover {
        if v < 0.0 || v > 1.0 { errors.push(format!("[RUSLE] C-cover={:.3} TIDAK VALID. 0=hutan primer tropis, 1=tanah gundul.", v)); }
    }
    if let Some(v) = p.p_practice {
        if v < 0.0 || v > 1.0 { errors.push(format!("[RUSLE] P-practice={:.3} TIDAK VALID. 0=teras bangku sempurna, 1=tanpa konservasi.", v)); }
    }

    // ========== KEBISINGAN ==========
    if let Some(v) = p.noise_db {
        if v < 0.0 { errors.push(format!("[AKUSTIK] Kebisingan={:.1} dB negatif TIDAK VALID.", v)); }
        else if v > 194.0 { errors.push(format!("[AKUSTIK] Kebisingan={:.1} dB melebihi batas fisik maksimum gelombang suara di udara (194 dB). Ref: pressure wave = 1 atm.", v)); }
        else if v > 70.0 { warnings.push(format!("[BAKU MUTU KepMenLH 48/1996] Kebisingan={:.1} dB MELEBIHI baku mutu zona industri (70 dB).", v)); }
        else if v > 55.0 { warnings.push(format!("[BAKU MUTU KepMenLH 48/1996] Kebisingan={:.1} dB MELEBIHI baku mutu zona permukiman siang (55 dB).", v)); }
    }

    // ========== GELOMBANG & PESISIR ==========
    if let Some(v) = p.wave_height {
        if v < 0.0 { errors.push(format!("[OSEANOGRAFI] Tinggi gelombang Hs={:.2} m negatif TIDAK VALID.", v)); }
    }
    if let Some(v) = p.wave_period {
        if v <= 0.0 { errors.push(format!("[OSEANOGRAFI] Periode gelombang T={:.2} s harus positif (>0).", v)); }
    }
    if let Some(v) = p.current_speed {
        if v < 0.0 { errors.push(format!("[OSEANOGRAFI] Kecepatan arus={:.2} m/s negatif TIDAK VALID.", v)); }
    }

    // ========== FORMAT OUTPUT ==========
    let mut out = String::from("=== PHYSICS-INFORMED VALIDATOR ENGINE ===\n");
    out.push_str("Referensi: USDA TR-55, FAO-56, RUSLE, Streeter-Phelps,\n");
    out.push_str("PP 22/2021, PermenLHK 68/2016, KepMenLH 48/1996\n\n");

    if errors.is_empty() && warnings.is_empty() {
        out.push_str("✅ VALID: Seluruh parameter mematuhi hukum fisika dan baku mutu Indonesia.\n");
    }

    if !errors.is_empty() {
        out.push_str(&format!("❌ {} PELANGGARAN FISIKA DITEMUKAN:\n", errors.len()));
        for (i, e) in errors.iter().enumerate() {
            out.push_str(&format!("  {}. {}\n", i + 1, e));
        }
        out.push_str("\n⛔ ANALISIS DITOLAK. Perbaiki parameter di atas sebelum melanjutkan.\n");
    }

    if !warnings.is_empty() {
        out.push_str(&format!("\n⚠️ {} PERINGATAN:\n", warnings.len()));
        for (i, w) in warnings.iter().enumerate() {
            out.push_str(&format!("  {}. {}\n", i + 1, w));
        }
    }

    if !info.is_empty() {
        out.push_str(&format!("\nℹ️ INFO:\n"));
        for inf in &info {
            out.push_str(&format!("  {}\n", inf));
        }
    }

    out
}
