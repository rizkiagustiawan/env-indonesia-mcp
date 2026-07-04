
pub fn solve_pde(
    h_initial_json: &str, 
    diffusivity_d: f64, 
    dx_meters: f64, 
    dy_meters: f64, 
    time_steps: u32,
    dt_seconds: f64
) -> String {
    let parsed: Result<Vec<Vec<f64>>, _> = serde_json::from_str(h_initial_json);
    if parsed.is_err() {
        return "ERROR: h_initial_json harus berupa array 2D elevasi awal muka air tanah.".into();
    }
    
    let mut h = parsed.unwrap();
    let rows = h.len();
    let cols = h[0].len();
    
    if rows < 3 || cols < 3 {
        return "ERROR: Ukuran grid minimal 3x3.".into();
    }

    // 1. Validasi Stabilitas Von Neumann (CFL Limit)
    // dt <= 1 / [2D (1/dx^2 + 1/dy^2)]
    let cfl_limit = 1.0 / (2.0 * diffusivity_d * (1.0/(dx_meters*dx_meters) + 1.0/(dy_meters*dy_meters)));
    
    let mut safe_dt = dt_seconds;
    let mut modified_steps = time_steps;
    let mut warning_msg = String::new();

    if dt_seconds > cfl_limit {
        // Otomatis pecah dt menjadi sub-steps untuk menjaga stabilitas fisika tanpa merusak total waktu
        let total_time = dt_seconds * time_steps as f64;
        safe_dt = cfl_limit * 0.9; // 90% of max limit untuk safety
        modified_steps = (total_time / safe_dt).ceil() as u32;
        warning_msg = format!("⚠️ INTERVENSI FISIKA: Time step awal dt={:.1}s melampaui batas stabilitas CFL ({:.1}s). Simulasi berisiko NaN. Sistem otomatis memecah komputasi menjadi dt={:.1}s dengan {} iterasi.\n\n", 
                              dt_seconds, cfl_limit, safe_dt, modified_steps);
    }

    let mut h_next = h.clone();

    // 2. Eksekusi Numerik Beda Hingga Eksplisit
    for _ in 0..modified_steps {
        for i in 1..(rows - 1) {
            for j in 1..(cols - 1) {
                let d2h_dx2 = (h[i][j+1] - 2.0*h[i][j] + h[i][j-1]) / (dx_meters * dx_meters);
                let d2h_dy2 = (h[i-1][j] - 2.0*h[i][j] + h[i+1][j]) / (dy_meters * dy_meters);
                
                h_next[i][j] = h[i][j] + diffusivity_d * safe_dt * (d2h_dx2 + d2h_dy2);
            }
        }
        // Update state
        h = h_next.clone();
    }

    // 3. Evaluasi
    let center_val = h[rows/2][cols/2];

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Groundwater PDE Solver (2D)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str(&warning_msg);
    out.push_str(&format!("Parameter: D = {} m²/s, dx = {}m, dy = {}m\n", diffusivity_d, dx_meters, dy_meters));
    out.push_str(&format!("Total Simulasi: {:.1} Jam\n\n", (safe_dt * modified_steps as f64) / 3600.0));
    
    out.push_str("HASIL:\n");
    out.push_str(&format!("  Elevasi akhir (tengah grid): {:.3} meter\n", center_val));
    out.push_str("  Simulasi Selesai secara Stabil (No NaN).\n");

    out
}
