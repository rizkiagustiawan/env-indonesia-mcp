//! Extended DO Model — Streeter-Phelps + Nitrification + SOD + Photosynthesis
//! D(x) = BOD_sag + NBOD_sag + SOD_sag - Photosynthesis + Re-aeration
//! Ref: Chapra (2008) Surface Water Quality Modeling, O'Connor & Dobbins (1958)

pub fn calculate(
    // BOD parameters
    k1: f64,
    l0: f64, // CBOD decay rate (1/d), initial CBOD (mg/L)
    // Nitrification
    kn: f64,
    n0: f64, // NBOD decay rate (1/d), initial NBOD (mg/L)
    // Reaeration
    k2: f64,
    // SOD (Sediment Oxygen Demand)
    sod_gm2d: f64, // g O2/m²/day
    depth_m: f64,  // average depth
    // Photosynthesis/Respiration
    p_mgl_d: f64, // photosynthesis rate (mg/L/day)
    r_mgl_d: f64, // respiration rate (mg/L/day)
    // Initial conditions
    do_initial: f64, // initial DO (mg/L)
    do_sat: f64,     // DO saturation (mg/L)
    // Transport
    velocity_ms: f64,
    distance_km: f64,
    temp_c: f64,
) -> String {
    let d0 = do_sat - do_initial; // initial deficit
    let v_kmd = velocity_ms * 86.4; // m/s to km/day

    // Temperature corrections (van't Hoff-Arrhenius)
    let theta_k1: f64 = 1.047;
    let theta_kn: f64 = 1.08;
    let theta_k2: f64 = 1.024;
    let k1_t = k1 * theta_k1.powf(temp_c - 20.0);
    let kn_t = kn * theta_kn.powf(temp_c - 20.0);
    let k2_t = k2 * theta_k2.powf(temp_c - 20.0);

    // SOD volumetric rate
    let sod_vol = sod_gm2d / depth_m; // mg/L/day

    // Net photosynthesis
    let net_photo = p_mgl_d - r_mgl_d; // mg/L/day (positive = O2 gain)

    let mut result = format!(
        "=== EXTENDED DO MODEL ===\n\
         Ref: Chapra (2008), O'Connor & Dobbins (1958)\n\n\
         PARAMETER (koreksi suhu {}°C):\n  k1 (CBOD) = {:.4}/d → {:.4}/d\n  kn (NBOD) = {:.4}/d → {:.4}/d\n  k2 (reaeration) = {:.4}/d → {:.4}/d\n  SOD = {:.2} g/m²/d = {:.2} mg/L/d\n  P-R = {:.2} - {:.2} = {:.2} mg/L/d\n  L0 = {:.1} mg/L, N0 = {:.1} mg/L\n  DO_sat = {:.2} mg/L, DO_0 = {:.2} mg/L\n  v = {:.3} m/s = {:.2} km/d\n\n",
        temp_c, k1, k1_t, kn, kn_t, k2, k2_t,
        sod_gm2d, sod_vol, p_mgl_d, r_mgl_d, net_photo,
        l0, n0, do_sat, do_initial, velocity_ms, v_kmd
    );

    result.push_str(&format!(
        "{:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}\n",
        "km", "hari", "CBOD", "NBOD", "D_cbod", "D_total", "DO", "Status"
    ));

    let steps = 50;
    let dx = distance_km / steps as f64;
    let mut min_do = do_initial;
    let mut critical_x = 0.0_f64;

    for i in 0..=steps {
        let x = i as f64 * dx;
        let t = if v_kmd > 0.0 { x / v_kmd } else { 0.0 };

        // CBOD remaining
        let lt = l0 * (-k1_t * t).exp();

        // NBOD remaining
        let nt = n0 * (-kn_t * t).exp();

        // Deficit from CBOD (classic Streeter-Phelps)
        let d_cbod = if (k2_t - k1_t).abs() > 0.001 {
            (k1_t * l0 / (k2_t - k1_t)) * ((-k1_t * t).exp() - (-k2_t * t).exp())
        } else {
            k1_t * l0 * t * (-k1_t * t).exp()
        };

        // Deficit from NBOD
        let d_nbod = if (k2_t - kn_t).abs() > 0.001 {
            (kn_t * n0 / (k2_t - kn_t)) * ((-kn_t * t).exp() - (-k2_t * t).exp())
        } else {
            kn_t * n0 * t * (-kn_t * t).exp()
        };

        // Deficit from SOD (constant source)
        let d_sod = if k2_t > 0.0 {
            sod_vol / k2_t * (1.0 - (-k2_t * t).exp())
        } else {
            0.0
        };

        // Deficit reduction from photosynthesis
        let d_photo = if k2_t > 0.0 {
            net_photo / k2_t * (1.0 - (-k2_t * t).exp())
        } else {
            0.0
        };

        // Initial deficit decay
        let d_init = d0 * (-k2_t * t).exp();

        // Total deficit
        let d_total = d_cbod + d_nbod + d_sod - d_photo + d_init;
        let do_val = (do_sat - d_total).max(0.0);

        if do_val < min_do {
            min_do = do_val;
            critical_x = x;
        }

        let status = if do_val < 2.0 {
            "KRITIS"
        } else if do_val < 4.0 {
            "RENDAH"
        } else if do_val < 6.0 {
            "CUKUP"
        } else {
            "BAIK"
        };

        if i % (steps / 10).max(1) == 0 || i == steps {
            result.push_str(&format!(
                "{:>8.2} {:>8.3} {:>8.2} {:>8.2} {:>8.3} {:>8.3} {:>8.2} {:>8}\n",
                x, t, lt, nt, d_cbod, d_total, do_val, status
            ));
        }
    }

    result.push_str(&format!(
        "\nTITIK KRITIS:\n  DO minimum = {:.2} mg/L pada jarak {:.2} km\n\
         \nCEK BAKU MUTU (PP 22/2021):\n  Kelas I (≥6 mg/L): {}\n  Kelas II (≥4 mg/L): {}\n  Kelas III (≥3 mg/L): {}\n  Kelas IV (≥0 mg/L): MEMENUHI\n",
        min_do, critical_x,
        if min_do >= 6.0 { "MEMENUHI" } else { "TIDAK" },
        if min_do >= 4.0 { "MEMENUHI" } else { "TIDAK" },
        if min_do >= 3.0 { "MEMENUHI" } else { "TIDAK" },
    ));

    result
}
