pub fn calculate_emissions(
    grid_data_json: &str,
    u_wind: f64,
    v_wind: f64,
    dx_meters: f64,
    dy_meters: f64,
    lifetime_hours: f64,
) -> String {
    // Deserialize input: diharapkan 2D array berupa list of lists
    let parsed: Result<Vec<Vec<f64>>, _> = serde_json::from_str(grid_data_json);
    if parsed.is_err() {
        return "ERROR: Format grid_data_json tidak valid. Harap berikan array 2D konsentrasi."
            .into();
    }

    let omega = parsed.unwrap();
    if omega.is_empty() || omega[0].is_empty() {
        return "ERROR: Grid kosong.".into();
    }

    let rows = omega.len();
    let cols = omega[0].len();

    if rows < 3 || cols < 3 {
        return "ERROR: Ukuran grid minimal 3x3 untuk central difference.".into();
    }

    // Kecepatan peluruhan gas (Loss L)
    // L = Ω / lifetime (dalam detik)
    let tau_seconds = lifetime_hours * 3600.0;

    let mut emissions = vec![vec![0.0; cols]; rows];
    let mut max_emission = 0.0;
    let mut max_loc = (0, 0);

    // Hitung E = d(uΩ)/dx + d(vΩ)/dy + Ω/τ
    // Kita asumsikan medan angin (u, v) konstan di seluruh grid (pendekatan sederhana)
    for i in 1..(rows - 1) {
        for j in 1..(cols - 1) {
            // Turunan parsial pusat (Central Difference)
            // d(uΩ)/dx
            let dx_flux = u_wind * (omega[i][j + 1] - omega[i][j - 1]) / (2.0 * dx_meters);
            // d(vΩ)/dy (ingat: baris i bertambah ke bawah di matriks, jadi dy bisa negatif tergantung orientasi. Asumsi standar kartesian)
            let dy_flux = v_wind * (omega[i - 1][j] - omega[i + 1][j]) / (2.0 * dy_meters);

            // Penguraian kimia (Loss term)
            let loss = omega[i][j] / tau_seconds;

            let e = dx_flux + dy_flux + loss;
            emissions[i][j] = e;

            if e > max_emission {
                max_emission = e;
                max_loc = (i, j);
            }
        }
    }

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Flux Divergence Emission (FDA)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: E = ∇·(vΩ) + L (Beirle et al., 2019)\n\n");
    out.push_str(&format!("Resolusi Grid: {}x{}\n", rows, cols));
    out.push_str(&format!(
        "Angin: u={:.1} m/s, v={:.1} m/s\n",
        u_wind, v_wind
    ));
    out.push_str(&format!(
        "Siklus hidup polutan (τ): {:.1} jam\n\n",
        lifetime_hours
    ));

    out.push_str("HASIL ANALISIS:\n");
    out.push_str(&format!(
        "  Emisi Tertinggi terdeteksi di grid [{}, {}]\n",
        max_loc.0, max_loc.1
    ));
    out.push_str(&format!(
        "  Nilai Flux: {:.4} (unit/m²/s)\n\n",
        max_emission
    ));
    out.push_str("  Kecurigaan: Titik ini merupakan lokasi cerobong asap atau kebocoran masif (Point Source).\n");

    // Representasi ringkas matriks
    out.push_str("\nRepresentasi Top-Left 5x5 Emisi:\n");
    for i in 1..std::cmp::min(rows - 1, 6) {
        for j in 1..std::cmp::min(cols - 1, 6) {
            out.push_str(&format!("{:>8.2} ", emissions[i][j]));
        }
        out.push_str("\n");
    }

    out
}
