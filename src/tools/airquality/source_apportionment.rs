/// Source Apportionment — Chemical Mass Balance (CMB)
/// Memisahkan kontribusi sumber polusi PM2.5 (Kendaraan, PLTU, Debu, Pembakaran)
/// Ref: Vital Strategies (2025) Jakarta Source Apportionment; EPA CMB Model 8.2

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Positive Matrix Factorization (PMF) parameter set.
/// Ref: Paatero & Tapper 1994; EPA PMF 5.0 (EPA/600/R-14/108).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PmfParam {
    #[schemars(description = "Observed concentration matrix X as JSON 2D array [n_samples][m_species]")]
    pub x_json: String,
    #[schemars(description = "Uncertainty matrix sigma as JSON 2D array [n_samples][m_species]")]
    pub sigma_json: String,
    #[schemars(description = "Number of factors (sources) p >= 1")]
    pub n_factors: usize,
    #[schemars(description = "Max iterations (default 200)")]
    pub max_iter: u32,
}

fn matmul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let k = a[0].len();
    let m = b[0].len();
    let mut c = vec![vec![0.0; m]; n];
    for (i, row) in a.iter().enumerate() {
        for j in 0..m {
            let mut acc = 0.0;
            for (t, &av) in row.iter().enumerate() {
                acc += av * b[t][j];
            }
            c[i][j] = acc;
        }
    }
    c
}

/// Weighted non-negative matrix factorization (PMF via multiplicative ALS).
/// Minimises Q = Σ ((X - G·F)/σ)² subject to G ≥ 0, F ≥ 0.
/// Returns (G [n×p], F [p×m], Q final).
fn weighted_nmf(
    x: &[Vec<f64>],
    sigma: &[Vec<f64>],
    p: usize,
    max_iter: u32,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>, f64) {
    let n = x.len();
    let m = x[0].len();

    // Weights W = 1/σ² (clamped to avoid div-by-zero).
    let w: Vec<Vec<f64>> = sigma
        .iter()
        .map(|row| row.iter().map(|&s| 1.0 / (s * s).max(1e-12)).collect())
        .collect();

    // Deterministic non-negative init.
    let mut g = vec![vec![0.0; p]; n];
    let mut f = vec![vec![0.0; m]; p];
    for (i, row) in x.iter().enumerate() {
        let base = row.iter().sum::<f64>() / (m as f64 * p as f64).max(1.0);
        for k in 0..p {
            g[i][k] = base.max(0.01) + 0.001 * (k as f64 + 1.0);
        }
    }
    for (k, frow) in f.iter_mut().enumerate() {
        for (j, fv) in frow.iter_mut().enumerate() {
            *fv = 1.0 / p as f64 + 0.001 * (j as f64 + 1.0);
        }
    }

    for _ in 0..max_iter {
        let gf = matmul(&g, &f);

        // Update G given F: G ← G ⊙ ( (W⊙X)F^T ⊘ (W⊙(GF))F^T )
        for i in 0..n {
            for k in 0..p {
                let mut num = 0.0;
                let mut den = 0.0;
                for j in 0..m {
                    num += w[i][j] * x[i][j] * f[k][j];
                    den += w[i][j] * gf[i][j] * f[k][j];
                }
                g[i][k] *= num / den.max(1e-12);
            }
        }

        let gf = matmul(&g, &f);

        // Update F given G: F ← F ⊙ ( G^T(W⊙X) ⊘ G^T(W⊙(GF)) )
        for k in 0..p {
            for j in 0..m {
                let mut num = 0.0;
                let mut den = 0.0;
                for i in 0..n {
                    num += g[i][k] * w[i][j] * x[i][j];
                    den += g[i][k] * w[i][j] * gf[i][j];
                }
                f[k][j] *= num / den.max(1e-12);
            }
        }
    }

    // Final goodness-of-fit Q.
    let gf = matmul(&g, &f);
    let mut q = 0.0;
    for i in 0..n {
        for j in 0..m {
            let e = (x[i][j] - gf[i][j]) / sigma[i][j];
            q += e * e;
        }
    }
    (g, f, q)
}

/// PMF source apportionment. Returns formatted report with factor profiles (F)
/// and source contributions (G), plus goodness-of-fit Q and Q/Qexp ratio.
pub fn pmf_apportionment(p: &PmfParam) -> String {
    let x: Vec<Vec<f64>> = match serde_json::from_str(&p.x_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: x_json harus berupa array 2D numerik [samples][species].".into(),
    };
    let sigma: Vec<Vec<f64>> = match serde_json::from_str(&p.sigma_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: sigma_json harus berupa array 2D numerik.".into(),
    };

    if x.is_empty() || x[0].is_empty() {
        return "ERROR: matriks X kosong.".into();
    }
    if x.len() != sigma.len() || x[0].len() != sigma[0].len() {
        return "ERROR: dimensi X dan sigma harus sama.".into();
    }
    if p.n_factors < 1 || p.n_factors > x.len().min(x[0].len()) {
        return "ERROR: n_factors harus antara 1 dan min(n_samples, n_species).".into();
    }

    let (g, f, q) = weighted_nmf(&x, &sigma, p.n_factors, p.max_iter.max(1));

    let n = x.len();
    let m = x[0].len();
    // Q_expected = n*m - p*(n+m) (degrees of freedom, Paatero & Tapper)
    let q_expected = (n * m) as f64 - (p.n_factors as f64) * ((n + m) as f64);

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  PMF Source Apportionment (Weighted NMF)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Paatero & Tapper 1994; EPA PMF 5.0\n\n");

    // Normalise each factor profile to sum 1 for interpretability (scaling absorbed into G).
    // Report F as fraction of each species per factor.
    out.push_str("PROFIL FAKTOR (F, fraksi spesies per sumber):\n");
    for k in 0..p.n_factors {
        let row_sum: f64 = f[k].iter().sum::<f64>().max(1e-12);
        let pct: Vec<String> = f[k].iter().map(|&v| format!("{:.2}", v / row_sum * 100.0)).collect();
        out.push_str(&format!("  Faktor {}: [{}] (%)\n", k + 1, pct.join(", ")));
    }

    out.push_str("\nKONTRIBUSI SUMBER (G, konsentrasi per sampel):\n");
    let mut col_sums = vec![0.0; p.n_factors];
    for (i, row) in g.iter().enumerate() {
        for k in 0..p.n_factors {
            col_sums[k] += row[k];
        }
        let vals: Vec<String> = row.iter().map(|&v| format!("{:.2}", v)).collect();
        out.push_str(&format!("  Sampel {}: [{}]\n", i + 1, vals.join(", ")));
    }

    out.push_str("\nStatistik:\n");
    out.push_str(&format!("  Q (goodness-of-fit) = {:.2}\n", q));
    out.push_str(&format!("  Q_expected ≈ {:.1}\n", q_expected));
    let ratio = if q_expected > 0.0 { q / q_expected } else { 0.0 };
    out.push_str(&format!("  Q/Qexp = {:.3} {}\n", ratio, if ratio < 2.0 { "(faktor optimal)" } else { "(faktor terlalu banyak/sedikit)" }));

    out
}

pub fn assess(
    pm25_total_ug_m3: f64,
    so4_ug_m3: f64,
    no3_ug_m3: f64,
    ec_ug_m3: f64,      // Elemental Carbon
    oc_ug_m3: f64,      // Organic Carbon
    crustal_ug_m3: f64, // Si, Al, Ca, Fe
) -> String {
    let mut out = String::from("=== Source Apportionment PM2.5 (CMB Model) ===\n");
    out.push_str("Ref: EPA CMB 8.2; Vital Strategies (2025) Jakarta Study\n\n");

    if pm25_total_ug_m3 <= 0.0 {
        return "ERROR [E102]: Konsentrasi PM2.5 harus > 0.".into();
    }

    // Simplified Receptor Modeling (Chemical Mass Balance heuristic based on Jakarta signatures)
    // 1. Coal Power Plants (PLTU): strong SO4 signature
    let coal_factor = 1.37; // typical SO4 to ammonium sulfate ratio
    let source_coal = so4_ug_m3 * coal_factor;

    // 2. Vehicles (Transport): high NOx/NO3 and Elemental Carbon (EC)
    let transport_factor = 1.29; // NO3 to ammonium nitrate
    let source_transport = (no3_ug_m3 * transport_factor) + (ec_ug_m3 * 1.5); 

    // 3. Open Burning (Biomass/Trash): high Organic Carbon (OC) relative to EC
    let source_burning = if oc_ug_m3 > (ec_ug_m3 * 2.0) {
        (oc_ug_m3 - ec_ug_m3 * 1.5) * 1.6 // POM multiplier
    } else {
        oc_ug_m3 * 0.5
    };

    // 4. Dust/Construction (Crustal)
    let source_dust = crustal_ug_m3 * 1.1;

    // Mass closure check
    let sum_identified = source_coal + source_transport + source_burning + source_dust;
    let (normalized_coal, normalized_transport, normalized_burning, normalized_dust, unidentified) = 
        if sum_identified > pm25_total_ug_m3 {
            // Scale down if exceeds total
            let scale = pm25_total_ug_m3 / sum_identified;
            (source_coal * scale, source_transport * scale, source_burning * scale, source_dust * scale, 0.0)
        } else {
            (source_coal, source_transport, source_burning, source_dust, pm25_total_ug_m3 - sum_identified)
        };

    let pct = |val: f64| (val / pm25_total_ug_m3) * 100.0;

    out.push_str(&format!("Total PM2.5 Terukur : {:.1} µg/m³\n", pm25_total_ug_m3));
    out.push_str(&format!("Komposisi Kimia     : SO4={:.1}, NO3={:.1}, EC={:.1}, OC={:.1}, Debu Tanah={:.1} (µg/m³)\n\n", 
        so4_ug_m3, no3_ug_m3, ec_ug_m3, oc_ug_m3, crustal_ug_m3));

    out.push_str("-- ESTIMASI KONTRIBUSI SUMBER (Source Apportionment) --\n\n");
    out.push_str(&format!("  Kendaraan Bermotor  : {:>5.1} µg/m³ ({:>4.1}%)\n", normalized_transport, pct(normalized_transport)));
    out.push_str(&format!("  PLTU / Industri     : {:>5.1} µg/m³ ({:>4.1}%)\n", normalized_coal, pct(normalized_coal)));
    out.push_str(&format!("  Pembakaran Terbuka  : {:>5.1} µg/m³ ({:>4.1}%)\n", normalized_burning, pct(normalized_burning)));
    out.push_str(&format!("  Debu Jalan/Tanah    : {:>5.1} µg/m³ ({:>4.1}%)\n", normalized_dust, pct(normalized_dust)));
    out.push_str(&format!("  Tidak Teridentifikasi: {:>5.1} µg/m³ ({:>4.1}%)\n\n", unidentified, pct(unidentified)));

    out.push_str("-- REKOMENDASI KEBIJAKAN --\n");
    
    // Find dominant source
    let mut sources = vec![
        ("Kendaraan", normalized_transport),
        ("PLTU", normalized_coal),
        ("Pembakaran", normalized_burning),
        ("Debu", normalized_dust),
    ];
    sources.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let dominant = sources[0].0;

    out.push_str(&format!("  Sumber dominan adalah {}. Fokuskan regulasi pada sektor ini.\n", dominant));
    match dominant {
        "Kendaraan" => out.push_str("  Aksi: Terapkan LEZ (Low Emission Zone), uji emisi ketat, percepat elektrifikasi.\n"),
        "PLTU" => out.push_str("  Aksi: Audit kepatuhan emisi cerobong, pasang scrubber/FGD, percepat pensiun dini PLTU.\n"),
        "Pembakaran" => out.push_str("  Aksi: Larangan tegas pembakaran sampah terbuka, perbaiki layanan angkut sampah.\n"),
        _ => out.push_str("  Aksi: Pembersihan jalan, kontrol debu konstruksi.\n"),
    }

    out.push_str("\n  Note: Ini adalah model penyederhanaan kimiawi. Untuk hasil pro-justitia\n");
    out.push_str("        gunakan software EPA PMF 5.0 atau EPA CMB 8.2.\n");

    out
}

#[cfg(test)]
mod tests {
    use super::assess;

    #[test]
    fn test_cmb_mass_closure() {
        // High SO4 -> PLTU should dominate
        let res = assess(50.0, 20.0, 5.0, 2.0, 4.0, 5.0);
        assert!(res.contains("Sumber dominan adalah PLTU"));
        assert!(res.contains("Total PM2.5 Terukur : 50.0"));
    }
}

#[cfg(test)]
mod pmf_tests {
    use super::{weighted_nmf, pmf_apportionment, PmfParam};

    #[test]
    fn pmf_reconstructs_synthetic_data() {
        // X = G_true · F_true (2 sources, 5 samples, 4 species).
        let g_true = vec![
            vec![8.0, 2.0],
            vec![7.0, 3.0],
            vec![6.0, 4.0],
            vec![5.0, 5.0],
            vec![4.0, 6.0],
        ];
        let f_true = vec![
            vec![0.8, 0.1, 0.05, 0.05], // source 1 profile
            vec![0.1, 0.7, 0.1, 0.1],   // source 2 profile
        ];
        let x: Vec<Vec<f64>> = g_true.iter().map(|row| {
            (0..4).map(|j| row[0]*f_true[0][j] + row[1]*f_true[1][j]).collect()
        }).collect();
        let sigma = vec![vec![0.1; 4]; 5];

        let (g, f, q) = weighted_nmf(&x, &sigma, 2, 500);

        // Reconstruction error must be small relative to data magnitude.
        let mut recon_err = 0.0;
        let mut data_norm = 0.0;
        for i in 0..5 {
            for j in 0..4 {
                let recon = (0..2).map(|k| g[i][k]*f[k][j]).sum::<f64>();
                recon_err += (recon - x[i][j]).powi(2);
                data_norm += x[i][j].powi(2);
            }
        }
        let rel_err = (recon_err / data_norm).sqrt();
        assert!(rel_err < 0.05, "PMF reconstruction relative error {:.4} too large", rel_err);
        assert!(q.is_finite());
    }

    #[test]
    fn pmf_converges_q_decreases() {
        let x = vec![
            vec![6.4, 2.2, 0.6, 0.6],
            vec![5.9, 2.4, 0.65, 0.65],
            vec![5.2, 2.7, 0.72, 0.72],
            vec![4.6, 3.0, 0.78, 0.78],
            vec![4.0, 3.3, 0.85, 0.85],
        ];
        let sigma = vec![vec![0.1; 4]; 5];
        let (_g, _f, q1) = weighted_nmf(&x, &sigma, 2, 1);
        let (_g2, _f2, q2) = weighted_nmf(&x, &sigma, 2, 200);
        assert!(q2 < q1, "Q should decrease with iterations: {} -> {}", q1, q2);
    }

    #[test]
    fn pmf_rejects_bad_input() {
        let p = PmfParam {
            x_json: "[[1,2],[3,4]]".into(),
            sigma_json: "[[0.1]]".into(),
            n_factors: 1,
            max_iter: 10,
        };
        assert!(pmf_apportionment(&p).contains("ERROR"));
    }
}
