/// Flow Duration Curve — debit rancangan untuk daya tampung beban pencemaran
///
/// DTBP dihitung pada debit rendah, bukan debit rata-rata. Tool ini menurunkan
/// Q50/Q80/Q90/Q95 dari seri debit terukur memakai posisi plot Weibull.
use crate::result_contract::ResultStatus;

#[derive(Debug, Clone)]
pub struct FlowDurationCurve {
    pub n_records: usize,
    pub q50_m3s: Option<f64>,
    pub q80_m3s: Option<f64>,
    pub q90_m3s: Option<f64>,
    pub q95_m3s: Option<f64>,
    pub q_min_m3s: f64,
    pub q_max_m3s: f64,
    pub q_mean_m3s: f64,
    pub status: ResultStatus,
    pub limitations: Vec<String>,
}

/// Jumlah catatan minimum agar kurva punya bentuk, bukan sekadar urutan angka.
const MIN_RECORDS: usize = 2;

/// Satu tahun catatan harian. Di bawah ini musim tidak terwakili penuh.
const ONE_YEAR_DAILY: usize = 365;

/// Praktik baik analisis debit rendah menghendaki catatan panjang agar
/// variabilitas antar-tahun tertangkap.
const GOOD_PRACTICE_YEARS: usize = 10;

/// Persentil yang dilaporkan, sebagai peluang debit DILAMPAUI.
const TARGET_EXCEEDANCE: [f64; 4] = [0.50, 0.80, 0.90, 0.95];

/// Debit pada peluang terlampaui `p` memakai posisi plot Weibull.
///
/// Seri diurut menurun, peringkat `m = 1` adalah debit terbesar, dan
/// `p = m / (n + 1)`. Nilai `None` bila `p` di luar rentang posisi plot yang
/// tercakup catatan — melaporkannya berarti ekstrapolasi di luar data.
fn exceedance_flow(sorted_desc: &[f64], p: f64) -> Option<f64> {
    let n = sorted_desc.len();
    if n < MIN_RECORDS {
        return None;
    }
    // m = p (n + 1), peringkat pecahan dalam seri urut menurun.
    let m = p * (n as f64 + 1.0);
    if m < 1.0 || m > n as f64 {
        return None;
    }
    let lower_rank = m.floor() as usize; // >= 1
    let upper_rank = m.ceil() as usize; // <= n
    let lo = sorted_desc[lower_rank - 1];
    if lower_rank == upper_rank {
        return Some(lo);
    }
    let hi = sorted_desc[upper_rank - 1];
    let frac = m - lower_rank as f64;
    Some(lo + (hi - lo) * frac)
}

pub fn compute(flows_m3s: &[f64]) -> Result<FlowDurationCurve, String> {
    if flows_m3s.len() < MIN_RECORDS {
        return Err(format!(
            "ERROR [E102]: butuh minimal {} catatan debit, diberi {}.",
            MIN_RECORDS,
            flows_m3s.len()
        ));
    }
    for (i, q) in flows_m3s.iter().enumerate() {
        if !q.is_finite() {
            return Err(format!(
                "ERROR [E102]: debit ke-{} tidak finit ({}). Isi rumpang harus \
                 dihilangkan dari seri, bukan diisi angka semu.",
                i, q
            ));
        }
        if *q < 0.0 {
            return Err(format!(
                "ERROR [E102]: debit ke-{} negatif ({} m³/s). Debit sungai tidak \
                 boleh negatif.",
                i, q
            ));
        }
    }

    let n = flows_m3s.len();
    let mut sorted_desc = flows_m3s.to_vec();
    sorted_desc.sort_by(|a, b| b.partial_cmp(a).expect("finite values checked above"));

    let percentiles: Vec<Option<f64>> = TARGET_EXCEEDANCE
        .iter()
        .map(|p| exceedance_flow(&sorted_desc, *p))
        .collect();

    let q_max_m3s = sorted_desc[0];
    let q_min_m3s = sorted_desc[n - 1];
    let q_mean_m3s = flows_m3s.iter().sum::<f64>() / n as f64;

    let mut limitations = Vec::new();

    if percentiles.iter().any(Option::is_none) {
        let max_p = n as f64 / (n as f64 + 1.0);
        limitations.push(format!(
            "Catatan {} nilai hanya mencakup peluang terlampaui sampai {:.3} \
             (posisi plot Weibull n/(n+1)). Persentil di luar itu tidak dilaporkan \
             karena akan menjadi ekstrapolasi di luar data.",
            n, max_p
        ));
    }

    let status = if n < ONE_YEAR_DAILY {
        limitations.push(format!(
            "Catatan {} nilai lebih pendek dari satu tahun harian ({}), jadi musim \
             kemarau dan hujan belum terwakili. Debit rendah dari seri ini tidak \
             layak dipakai sebagai debit rancangan.",
            n, ONE_YEAR_DAILY
        ));
        ResultStatus::InsufficientData
    } else {
        limitations.push(format!(
            "Catatan {} nilai (~{:.1} tahun harian). Analisis debit rendah lazimnya \
             menghendaki minimal {} tahun agar variabilitas antar-tahun tertangkap.",
            n,
            n as f64 / ONE_YEAR_DAILY as f64,
            GOOD_PRACTICE_YEARS
        ));
        ResultStatus::ValidWithAssumptions
    };

    if q_min_m3s == 0.0 {
        let zero_days = flows_m3s.iter().filter(|q| **q == 0.0).count();
        limitations.push(format!(
            "Ada {} catatan berdebit nol. Pada sungai intermiten, daya tampung beban \
             pencemaran menjadi nol di periode itu — tidak ada aliran yang mengencerkan.",
            zero_days
        ));
    }

    limitations.push(
        "Kurva ini menggambarkan catatan yang diberikan, bukan kondisi mendatang. \
         Perubahan tata guna lahan, pengambilan air, dan iklim menggeser debit rendah. \
         Mutu dan homogenitas data debit tidak diperiksa oleh tool ini."
            .into(),
    );

    Ok(FlowDurationCurve {
        n_records: n,
        q50_m3s: percentiles[0],
        q80_m3s: percentiles[1],
        q90_m3s: percentiles[2],
        q95_m3s: percentiles[3],
        q_min_m3s,
        q_max_m3s,
        q_mean_m3s,
        status,
        limitations,
    })
}

pub fn calculate(flows_json: &str) -> String {
    let flows: Vec<f64> = match serde_json::from_str(flows_json) {
        Ok(v) => v,
        Err(e) => {
            return format!(
                "ERROR [E102]: gagal parse flows_json: {}.\nFormat: [12.4, 11.8, 9.2, ...] \
                 (m³/s, satu nilai per periode pencatatan).",
                e
            )
        }
    };

    let fdc = match compute(&flows) {
        Ok(f) => f,
        Err(e) => return e,
    };

    format_curve(&fdc)
}

pub fn format_curve(f: &FlowDurationCurve) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Flow Duration Curve\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Posisi plot: Weibull, p = m/(n+1)\n\n");
    out.push_str(&format!("Jumlah catatan : {}\n", f.n_records));
    out.push_str(&format!("Q maksimum     : {:.4} m³/s\n", f.q_max_m3s));
    out.push_str(&format!("Q rata-rata    : {:.4} m³/s\n", f.q_mean_m3s));
    out.push_str(&format!("Q minimum      : {:.4} m³/s\n\n", f.q_min_m3s));

    out.push_str("Debit menurut peluang terlampaui:\n");
    for (label, value) in [
        ("Q50 (median)", f.q50_m3s),
        ("Q80", f.q80_m3s),
        ("Q90", f.q90_m3s),
        ("Q95", f.q95_m3s),
    ] {
        match value {
            Some(q) => out.push_str(&format!("  {:<12} : {:.4} m³/s\n", label, q)),
            None => out.push_str(&format!(
                "  {:<12} : tidak dilaporkan (di luar rentang catatan)\n",
                label
            )),
        }
    }

    if let (Some(q95), true) = (f.q95_m3s, f.q_mean_m3s > 0.0) {
        if q95 > 0.0 {
            out.push_str(&format!(
                "\nRasio Q rata-rata / Q95 = {:.2}. Memakai debit rata-rata sebagai \
                 debit rancangan DTBP akan melebih-lebihkan kapasitas asimilasi \
                 sekitar sebanyak itu.\n",
                f.q_mean_m3s / q95
            ));
        } else {
            out.push_str(
                "\nQ95 = 0 m³/s: pada peluang terlampaui 95% sungai tidak mengalir, \
                 sehingga tidak ada daya tampung untuk dialokasikan.\n",
            );
        }
    }

    out.push_str(&format!(
        "\nStatus hasil   : {}\n",
        serde_json::to_value(&f.status)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".into())
    ));

    out.push_str("\nKeterbatasan:\n");
    for l in &f.limitations {
        out.push_str(&format!("  - {}\n", l));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Seri 1..=19 menempatkan setiap persentil Weibull tepat pada satu data,
    /// jadi nilai yang benar dapat dihitung tangan tanpa interpolasi.
    /// p = m/(n+1) = m/20; nilai peringkat-m terbesar = 20 − m.
    fn series_1_to_19() -> Vec<f64> {
        (1..=19).map(|v| v as f64).collect()
    }

    #[test]
    fn weibull_percentiles_land_exactly_on_ranked_observations() {
        let fdc = compute(&series_1_to_19()).expect("valid series");
        assert_eq!(fdc.n_records, 19);
        // p=0.50 -> m=10 -> nilai ke-10 terbesar = 10
        assert_eq!(fdc.q50_m3s, Some(10.0));
        // p=0.80 -> m=16 -> 4
        assert_eq!(fdc.q80_m3s, Some(4.0));
        // p=0.90 -> m=18 -> 2
        assert_eq!(fdc.q90_m3s, Some(2.0));
        // p=0.95 -> m=19 -> 1
        assert_eq!(fdc.q95_m3s, Some(1.0));
    }

    #[test]
    fn low_flow_percentiles_are_ordered_below_the_median() {
        let fdc = compute(&series_1_to_19()).expect("valid series");
        let q50 = fdc.q50_m3s.unwrap();
        let q80 = fdc.q80_m3s.unwrap();
        let q90 = fdc.q90_m3s.unwrap();
        let q95 = fdc.q95_m3s.unwrap();
        assert!(q95 <= q90 && q90 <= q80 && q80 <= q50, "{q95} {q90} {q80} {q50}");
    }

    #[test]
    fn q95_is_withheld_when_the_record_cannot_reach_that_exceedance() {
        // n=10 -> posisi plot Weibull terbesar = 10/11 = 0.909.
        // p=0.95 berada di luar rentang itu, jadi melaporkan Q95 berarti
        // ekstrapolasi. p=0.90 masih terkurung antara 9/11 dan 10/11.
        let fdc = compute(&(1..=10).map(|v| v as f64).collect::<Vec<_>>()).expect("valid");
        assert!(fdc.q95_m3s.is_none(), "Q95 must not be extrapolated");
        assert!(fdc.q90_m3s.is_some(), "Q90 is bracketed by the record");
        assert!(fdc.q50_m3s.is_some(), "median is well inside the record");
        assert!(
            fdc.limitations.iter().any(|l| l.contains("ekstrapolasi")),
            "must say why the percentile is missing: {:?}",
            fdc.limitations
        );
    }

    #[test]
    fn the_mean_is_far_above_the_low_flow_percentile_which_is_the_whole_point() {
        // Sungai musiman: banjir singkat, kemarau panjang. Debit rata-rata
        // melebih-lebihkan kapasitas asimilasi dibanding Q95.
        let mut flows = vec![2.0; 200];
        flows.extend(vec![120.0; 40]);
        let fdc = compute(&flows).expect("valid series");
        let q95 = fdc.q95_m3s.unwrap();
        assert!((q95 - 2.0).abs() < 1e-9, "Q95 should sit in the dry season, got {q95}");
        assert!(
            fdc.q_mean_m3s > 5.0 * q95,
            "mean {} should dwarf Q95 {}",
            fdc.q_mean_m3s,
            q95
        );
    }

    #[test]
    fn a_record_shorter_than_a_year_is_screening_only_at_best() {
        let fdc = compute(&series_1_to_19()).expect("valid series");
        assert_eq!(fdc.status, ResultStatus::InsufficientData);

        let year: Vec<f64> = (0..400).map(|i| 5.0 + (i % 7) as f64).collect();
        let long = compute(&year).expect("valid series");
        assert_eq!(long.status, ResultStatus::ValidWithAssumptions);
        assert!(
            long.limitations.iter().any(|l| l.contains("10 tahun")),
            "must state the record length good practice expects: {:?}",
            long.limitations
        );
    }

    #[test]
    fn zero_flow_days_are_kept_because_intermittent_rivers_are_real() {
        let mut flows = vec![0.0; 30];
        flows.extend((1..=90).map(|v| v as f64));
        let fdc = compute(&flows).expect("valid series");
        assert_eq!(fdc.q_min_m3s, 0.0);
        assert_eq!(fdc.q95_m3s, Some(0.0));
        assert!(
            fdc.limitations.iter().any(|l| l.contains("nol")),
            "zero-flow days change what DTBP means: {:?}",
            fdc.limitations
        );
    }

    #[test]
    fn negative_and_non_finite_flows_are_rejected() {
        assert!(compute(&[1.0, -2.0, 3.0]).is_err());
        assert!(compute(&[1.0, f64::NAN]).is_err());
        assert!(compute(&[1.0, f64::INFINITY]).is_err());
    }

    #[test]
    fn an_empty_or_tiny_series_is_an_error_not_a_number() {
        assert!(compute(&[]).is_err());
        assert!(compute(&[5.0]).is_err());
    }
}
