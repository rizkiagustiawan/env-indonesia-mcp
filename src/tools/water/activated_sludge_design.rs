//! Activated Sludge Process Design — Complete Design Package
//! Includes: SRT/MCRT design, F/M ratio, sludge production, oxygen demand,
//! secondary clarifier sizing, return sludge ratio.
//! Ref: Metcalf & Eddy (2014) Wastewater Engineering 5th Ed, Chapter 8
//! Ref: PermenLHK 68/2016 (Baku Mutu Air Limbah Domestik)

/// Complete activated sludge design
pub fn design(
    q_m3d: f64,      // influent flow (m³/day)
    bod_in: f64,     // influent BOD (mg/L)
    cod_in: f64,     // influent COD (mg/L)
    tss_in: f64,     // influent TSS (mg/L)
    tn_in: f64,      // influent TN (mg/L) — for nitrification check
    bod_target: f64, // effluent BOD target (mg/L)
    temp_c: f64,     // wastewater temperature (°C)
) -> String {
    // Kinetic coefficients (typical domestic wastewater, Metcalf & Eddy Table 8-14)
    let y = 0.6; // yield coefficient (g VSS/g BOD)
    let kd = 0.06; // endogenous decay (1/d)
    let mu_max = 6.0; // max specific growth rate (1/d)
    let ks = 60.0; // half-saturation constant (mg/L BOD)

    // Temperature correction
    let theta_mu: f64 = 1.07;
    let theta_kd: f64 = 1.04;
    let mu_max_t = mu_max * theta_mu.powf(temp_c - 20.0);
    let kd_t = kd * theta_kd.powf(temp_c - 20.0);

    // BOD removal
    let s0 = bod_in;
    let se = bod_target;
    let bod_removed = s0 - se; // mg/L

    // SRT (Solids Retention Time) — minimum for treatment
    // 1/SRT_min = Y * mu_max * Se / (Ks + Se) - kd
    let srt_min = 1.0 / (y * mu_max_t * se / (ks + se) - kd_t);
    let srt_design = (srt_min * 2.0).max(5.0).min(30.0); // safety factor 2x, clamp 5-30 days

    // Check nitrification SRT requirement
    let mu_n = 0.75 * 1.07_f64.powf(temp_c - 20.0); // nitrifier growth rate
    let srt_nitrification = 1.0 / (mu_n - 0.04); // minimum SRT for nitrification
    let nitrification_possible = srt_design > srt_nitrification;

    // MLSS concentration (typical 2000-4000 mg/L)
    let mlss = 3000.0; // mg/L, design value
    let mlvss = mlss * 0.75; // VSS/SS ratio ~0.75

    // HRT = SRT * Y * (S0 - Se) / (MLVSS * (1 + kd * SRT))
    let hrt_d = srt_design * y * bod_removed / (mlvss * (1.0 + kd_t * srt_design));
    let hrt_hr = hrt_d * 24.0;

    // Aeration tank volume
    let v_m3 = q_m3d * hrt_d;

    // F/M ratio
    let fm = q_m3d * s0 / (v_m3 * mlvss); // kg BOD/kg MLVSS/day

    // Sludge production
    let y_obs = y / (1.0 + kd_t * srt_design); // observed yield
    let px_vss = y_obs * q_m3d * bod_removed / 1000.0; // kg VSS/day
    let px_tss = px_vss / 0.75; // kg TSS/day

    // Oxygen demand
    let o2_bod = q_m3d * bod_removed / 1000.0 / 0.68; // kg O2/day (BOD basis, f=0.68)
    let o2_endogenous = 1.42 * kd_t * mlvss * v_m3 / 1000.0; // endogenous respiration
    let o2_nitrification = if nitrification_possible {
        4.57 * q_m3d * tn_in * 0.8 / 1000.0
    } else {
        0.0
    };
    let o2_total = o2_bod + o2_endogenous + o2_nitrification;

    // Secondary clarifier
    let surface_loading = 24.0; // m³/m²/day (typical for activated sludge)
    let clarifier_area = q_m3d / surface_loading;
    let clarifier_diameter = (4.0 * clarifier_area / std::f64::consts::PI).sqrt();

    // Return sludge ratio
    let svi = 150.0; // Sludge Volume Index (mL/g), typical
    let x_r = 1000.0 / svi * 1000.0; // return sludge concentration (mg/L)
    let r = mlss / (x_r - mlss); // recirculation ratio
    let q_r = r * q_m3d; // return sludge flow

    // Waste sludge
    let q_w = v_m3 * mlss / (srt_design * x_r * 1000.0) * 1000.0; // m³/day (approx)

    format!(
        "=== DESAIN ACTIVATED SLUDGE (IPAL) ===\n\
         Ref: Metcalf & Eddy (2014), PermenLHK 68/2016\n\n\
         INFLUENT:\n  Q = {:.1} m³/hari\n  BOD = {:.0} mg/L\n  COD = {:.0} mg/L\n  TSS = {:.0} mg/L\n  TN = {:.0} mg/L\n  Suhu = {:.1}°C\n\n\
         TARGET EFFLUEN:\n  BOD = {:.0} mg/L\n  BOD removal = {:.1}%\n\n\
         PARAMETER DESAIN:\n  SRT minimum = {:.1} hari\n  SRT desain = {:.1} hari (SF=2x)\n  HRT = {:.2} hari ({:.1} jam)\n  F/M = {:.3} kg BOD/kg MLVSS/hari{}\n  MLSS = {:.0} mg/L\n  MLVSS = {:.0} mg/L\n  Y_obs = {:.3} g VSS/g BOD\n\n\
         DIMENSI AERASI:\n  Volume = {:.1} m³\n  (Panjang × Lebar × Kedalaman disesuaikan)\n\n\
         PRODUKSI LUMPUR:\n  P_VSS = {:.1} kg VSS/hari\n  P_TSS = {:.1} kg TSS/hari\n\n\
         KEBUTUHAN OKSIGEN:\n  O₂ (BOD removal) = {:.1} kg/hari\n  O₂ (endogenous) = {:.1} kg/hari\n  O₂ (nitrifikasi) = {:.1} kg/hari\n  O₂ TOTAL = {:.1} kg/hari\n\n\
         KLARIFIER SEKUNDER:\n  Beban permukaan = {:.0} m³/m²/hari\n  Luas = {:.1} m²\n  Diameter = {:.1} m\n\n\
         RESIRKULASI LUMPUR:\n  SVI = {:.0} mL/g\n  Rasio (R) = {:.2}\n  Q_return = {:.1} m³/hari\n  Q_waste ≈ {:.2} m³/hari\n\n\
         NITRIFIKASI: {}\n",
        q_m3d, bod_in, cod_in, tss_in, tn_in, temp_c,
        bod_target, 100.0 * bod_removed / s0,
        srt_min, srt_design, hrt_d, hrt_hr,
        fm,
        if fm < 0.1 { " (Extended Aeration)" } else if fm < 0.3 { " (Conventional)" } else { " (High Rate)" },
        mlss, mlvss, y_obs,
        v_m3,
        px_vss, px_tss,
        o2_bod, o2_endogenous, o2_nitrification, o2_total,
        surface_loading, clarifier_area, clarifier_diameter,
        svi, r, q_r, q_w,
        if nitrification_possible {
            format!("MUNGKIN (SRT desain {:.1}d > SRT nitrifikasi {:.1}d)", srt_design, srt_nitrification)
        } else {
            format!("TIDAK (SRT desain {:.1}d < SRT nitrifikasi {:.1}d). Naikkan SRT.", srt_design, srt_nitrification)
        }
    )
}
