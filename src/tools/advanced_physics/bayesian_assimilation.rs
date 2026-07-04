use std::time::{SystemTime, UNIX_EPOCH};

// Mengimplementasikan Linear Congruential Generator sederhana untuk menghindari dependensi `rand` tambahan jika tidak perlu
struct LCG { seed: u64 }
impl LCG {
    fn new() -> Self {
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        Self { seed: seed ^ 0x55AA_55AA_55AA_55AA }
    }
    fn next_float(&mut self) -> f64 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.seed >> 11) as f64 / (1u64 << 53) as f64
    }
}

pub fn assimilate_sensor_data(
    prior_particles_json: &str,
    sensor_reading: f64,
    sensor_noise_std: f64
) -> String {
    let parsed: Result<Vec<f64>, _> = serde_json::from_str(prior_particles_json);
    if parsed.is_err() {
        return "ERROR: prior_particles_json harus berupa list of floats [12.0, 11.5, ...].".into();
    }
    
    let particles = parsed.unwrap();
    let n = particles.len();
    if n < 10 {
        return "ERROR: Butuh minimal 10 partikel untuk asimilasi data.".into();
    }

    // 1. Update Weights berdasarkan fungsi likelihood (Gaussian)
    let mut weights = vec![0.0; n];
    let mut sum_w = 0.0;
    
    for i in 0..n {
        let diff = particles[i] - sensor_reading;
        // P(y | x) ~ exp(-0.5 * (diff / std)^2)
        let w = (-0.5 * (diff / sensor_noise_std).powi(2)).exp();
        weights[i] = w;
        sum_w += w;
    }

    // Normalisasi bobot
    if sum_w <= 1e-12 {
        return "ERROR: Semua partikel jauh dari observasi sensor (Degeneracy total). Sensor rusak atau model salah.".into();
    }
    for i in 0..n { weights[i] /= sum_w; }

    // 2. Systematic Resampling O(N)
    let mut lcg = LCG::new();
    let mut new_particles = vec![0.0; n];
    
    let r = lcg.next_float() / (n as f64);
    let mut c = weights[0];
    let mut i = 0;

    for j in 0..n {
        let u = r + (j as f64) / (n as f64);
        while u > c && i < n - 1 {
            i += 1;
            c += weights[i];
        }
        new_particles[j] = particles[i];
    }

    // 3. Kalkulasi Posterior Mean & Variance
    let mean: f64 = new_particles.iter().sum::<f64>() / (n as f64);
    let var: f64 = new_particles.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n as f64);
    let std_dev = var.sqrt();

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Bayesian Sensor Assimilation\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Metode: Particle Filter (Systematic Resampling)\n\n");
    out.push_str(&format!("Observasi Sensor IoT : {:.2}\n", sensor_reading));
    out.push_str(&format!("Noise Instrumen (σ)  : {:.2}\n", sensor_noise_std));
    out.push_str(&format!("Jumlah Partikel (N)  : {}\n\n", n));
    
    out.push_str("ESTIMASI POSTERIOR (Kebenaran Terkoreksi):\n");
    out.push_str(&format!("  Nilai Aktual       ≈ {:.3}\n", mean));
    out.push_str(&format!("  Ketidakpastian (±) ≈ {:.3}\n", std_dev));
    
    if (mean - sensor_reading).abs() > sensor_noise_std * 2.0 {
        out.push_str("\n⚠️ KESIMPULAN: Data sensor sangat menyimpang dari prior fisika. Sensor kemungkinan besar rusak/mengalami anomali drastis (Spike).\n");
    } else {
        out.push_str("\n✅ KESIMPULAN: Data asimilasi berhasil membersihkan noise minor dari sensor.\n");
    }

    out
}
