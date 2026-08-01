//! Noise Barrier Insertion Loss — Maekawa (1968) + ISO 9613-2
//! IL = 10·log₁₀(3 + 20N) where N = Fresnel number
//! Ref: Maekawa (1968), ISO 9613-2:1996, KepmenLH 48/1996

pub fn calculate(
    source_height_m: f64,       // height of noise source above ground
    receiver_height_m: f64,     // height of receiver above ground
    barrier_height_m: f64,      // height of barrier
    source_to_barrier_m: f64,   // horizontal distance source to barrier
    barrier_to_receiver_m: f64, // horizontal distance barrier to receiver
    frequency_hz: f64,          // dominant frequency (Hz)
    source_db: f64,             // source noise level (dBA)
) -> String {
    let c = 343.0; // speed of sound (m/s)
    let wavelength = c / frequency_hz;

    // Path difference (δ) — Maekawa method
    // A = direct path: source → receiver
    // B = diffracted path: source → barrier top → receiver

    let d_sr = ((source_to_barrier_m + barrier_to_receiver_m).powi(2)
        + (source_height_m - receiver_height_m).powi(2))
    .sqrt();

    let d_sb = (source_to_barrier_m.powi(2) + (barrier_height_m - source_height_m).powi(2)).sqrt();
    let d_br =
        (barrier_to_receiver_m.powi(2) + (barrier_height_m - receiver_height_m).powi(2)).sqrt();

    let delta = d_sb + d_br - d_sr; // path difference (m)

    // Fresnel number
    let n = 2.0 * delta / wavelength;

    // Maekawa insertion loss
    let il = if n > 0.0 {
        10.0 * (3.0 + 20.0 * n).log10()
    } else if n > -0.2 {
        0.0 // barrier below line of sight but marginal
    } else {
        0.0 // no barrier effect
    };

    // Kurze-Anderson correction (more accurate for N > 1)
    let il_ka = if n > 0.0 {
        let sqrt_2pn = (2.0 * std::f64::consts::PI * n).sqrt();
        20.0 * (sqrt_2pn / (sqrt_2pn.tanh())).log10() + 5.0
    } else {
        0.0
    };

    // Use Kurze-Anderson for N > 1 (more accurate), Maekawa for N ≤ 1
    let il_final = if n > 1.0 {
        il_ka.min(25.0)
    } else {
        il.min(25.0)
    };
    let received_db = source_db - il_final;

    // Distance attenuation (geometric spreading)
    let _distance_loss = 20.0 * (d_sr.log10()) - 20.0 * 1.0_f64.log10(); // ref 1m

    format!(
        "=== NOISE BARRIER INSERTION LOSS ===\n\
         Ref: Maekawa (1968), ISO 9613-2, KepmenLH 48/1996\n\n\
         INPUT:\n  Sumber: tinggi = {:.1}m, jarak ke barrier = {:.1}m\n  Penerima: tinggi = {:.1}m, jarak dari barrier = {:.1}m\n  Barrier: tinggi = {:.1}m\n  Frekuensi = {:.0} Hz, λ = {:.3}m\n  Sumber = {:.1} dBA\n\n\
         KALKULASI:\n  Path difference δ = {:.4} m\n  Fresnel number N = {:.3}\n  IL (Maekawa) = {:.1} dB\n  IL (Kurze-Anderson) = {:.1} dB\n  IL digunakan = {:.1} dB\n\n\
         HASIL:\n  Tingkat bising di penerima = {:.1} dBA\n  Reduksi = {:.1} dB\n\n\
         Efektivitas: {}\n\
         Baku mutu perumahan (KepmenLH 48/1996): 55 dBA → {}",
        source_height_m, source_to_barrier_m, receiver_height_m, barrier_to_receiver_m,
        barrier_height_m, frequency_hz, wavelength, source_db,
        delta, n, il, il_ka, il_final,
        received_db, il_final,
        if il_final > 15.0 { "SANGAT EFEKTIF (>15 dB)" } else if il_final > 10.0 { "EFEKTIF (10-15 dB)" } else if il_final > 5.0 { "CUKUP (5-10 dB)" } else { "KURANG EFEKTIF (<5 dB)" },
        if received_db <= 55.0 { "MEMENUHI" } else { "MELEBIHI" }
    )
}
