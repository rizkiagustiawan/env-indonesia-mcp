/// SCS Triangular Unit Hydrograph
/// tp = D/2 + 0.6×tc, Qp = 0.208 × A / tp, tb = 2.67 × tp
/// Ref: USDA SCS (1972), NEH Part 630

pub fn calculate(a_km2: f64, tc_hours: f64, d_hours: f64) -> String {
    let mut out = String::from("=== SCS Triangular Unit Hydrograph ===\n");
    out.push_str("Ref: USDA SCS (1972), NEH Part 630\n\n");

    if a_km2 <= 0.0 { return "ERROR: Luas DAS (A) harus > 0.".into(); }
    if tc_hours <= 0.0 { return "ERROR: Waktu konsentrasi (tc) harus > 0.".into(); }
    if d_hours <= 0.0 { return "ERROR: Durasi hujan efektif (D) harus > 0.".into(); }

    // Time to peak
    let t_lag = 0.6 * tc_hours; // SCS lag time
    let tp = d_hours / 2.0 + t_lag;

    // Peak discharge (m³/s per mm of excess rainfall)
    let qp = 0.208 * a_km2 / tp;

    // Base time
    let tb = 2.67 * tp;

    // Volume check (area under triangle = 0.5 × Qp × tb should equal A × 1mm = A × 1000 m³)
    let volume_check = 0.5 * qp * tb * 3600.0; // m³ (converting hours to seconds)
    let expected_volume = a_km2 * 1e6 * 0.001; // 1mm over A km² = A × 1000 m³

    out.push_str(&format!("Input:\n  Luas DAS (A) = {:.2} km²\n  Waktu konsentrasi (tc) = {:.2} jam\n  Durasi hujan efektif (D) = {:.2} jam\n\n", a_km2, tc_hours, d_hours));

    out.push_str("Perhitungan:\n");
    out.push_str(&format!("  t_lag = 0.6 × tc = 0.6 × {:.2} = {:.2} jam\n", tc_hours, t_lag));
    out.push_str(&format!("  tp = D/2 + t_lag = {:.2}/2 + {:.2} = {:.2} jam\n", d_hours, t_lag, tp));
    out.push_str(&format!("  Qp = 0.208 × A / tp = 0.208 × {:.2} / {:.2} = {:.4} m³/s/mm\n", a_km2, tp, qp));
    out.push_str(&format!("  tb = 2.67 × tp = 2.67 × {:.2} = {:.2} jam\n\n", tp, tb));

    out.push_str("Hasil:\n");
    out.push_str(&format!("  Waktu puncak (tp) = {:.2} jam\n", tp));
    out.push_str(&format!("  Debit puncak (Qp) = {:.4} m³/s per mm hujan efektif\n", qp));
    out.push_str(&format!("  Waktu dasar (tb) = {:.2} jam\n", tb));
    out.push_str(&format!("  Volume check: {:.0} m³ vs {:.0} m³ (expected)\n\n", volume_check, expected_volume));

    // UH ordinates (triangular)
    out.push_str("Ordinat Unit Hydrograph:\n");
    out.push_str(&format!("  {:>8} | {:>10}\n", "t (jam)", "Q (m³/s)"));
    let dt = tb / 20.0;
    let mut t = 0.0;
    while t <= tb + dt * 0.5 {
        let q = if t <= tp {
            qp * t / tp
        } else if t <= tb {
            qp * (1.0 - (t - tp) / (tb - tp))
        } else {
            0.0
        };
        out.push_str(&format!("  {:>8.2} | {:>10.4}\n", t, q.max(0.0)));
        t += dt;
    }

    out
}
