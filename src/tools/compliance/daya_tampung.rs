/// Daya Tampung Beban Pencemaran (DTBP)
/// Ref: PP 22/2021 (mencabut PP 82/2001)
///
/// Mass balance di titik campur sungai-limbah:
///   (Q·C_hulu + q·C_limbah) / (Q + q) <= C_bm
///
/// Debit limbah `q` menambah volume pengencer, jadi ia harus muncul di kedua
/// ruas. Versi sebelumnya mencampur dua konvensi — kapasitas beban dihitung
/// tanpa `q` (`Q·(C_bm − C_hulu)`) tetapi konsentrasi limbah maksimum dibagi
/// `q` — sehingga kedua keluaran tidak saling konsisten: memasukkan kembali
/// `C_limbah,maks` ke perhitungan DTBP tidak menghasilkan nol.
use crate::result_contract::ResultStatus;

/// Faktor konversi m³/s · mg/L -> kg/hari (86400 s/hari ÷ 1000 mg/g).
const KG_PER_DAY_FACTOR: f64 = 86.4;

/// Dasar statistik debit sungai yang dipakai sebagai debit rancangan.
///
/// DTBP yang dihitung pada debit rata-rata tahunan melebih-lebihkan kapasitas
/// asimilasi sungai, karena beban kritis terjadi saat debit rendah (kemarau).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignFlowBasis {
    /// Debit yang dilampaui 95% waktu — debit rendah, dasar yang lazim untuk DTBP.
    Q95,
    /// Debit rata-rata minimum 7 hari berturut dengan periode ulang 10 tahun.
    Q7Q10,
    /// Debit rata-rata tahunan — BUKAN debit rancangan untuk DTBP.
    MeanAnnual,
    /// Dasar statistik tidak dinyatakan pemanggil.
    Unknown,
}

impl DesignFlowBasis {
    pub fn parse(input: &str) -> Self {
        match input
            .trim()
            .to_ascii_lowercase()
            .replace([' ', '-', ',', '.'], "_")
            .as_str()
        {
            "q95" | "q_95" => DesignFlowBasis::Q95,
            "q7_10" | "q7q10" | "q_7_10" | "q7_q10" => DesignFlowBasis::Q7Q10,
            "mean" | "mean_annual" | "rata_rata" | "maf" => DesignFlowBasis::MeanAnnual,
            _ => DesignFlowBasis::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DesignFlowBasis::Q95 => "Q95 (debit dilampaui 95% waktu)",
            DesignFlowBasis::Q7Q10 => "Q7,10 (debit rendah 7-hari, ulang 10 tahun)",
            DesignFlowBasis::MeanAnnual => "debit rata-rata tahunan",
            DesignFlowBasis::Unknown => "tidak dinyatakan",
        }
    }

    /// Apakah dasar ini merupakan statistik debit rendah yang layak untuk DTBP.
    pub fn is_low_flow(self) -> bool {
        matches!(self, DesignFlowBasis::Q95 | DesignFlowBasis::Q7Q10)
    }
}

#[derive(Debug, Clone)]
pub struct DtbpAssessment {
    pub parameter: String,
    pub q_river_m3s: f64,
    pub q_waste_m3s: f64,
    pub c_upstream_mgl: f64,
    pub c_standard_mgl: f64,
    pub c_waste_mgl: f64,
    /// Beban maksimum yang masih menjaga C_campur <= baku mutu (kg/hari).
    pub allowable_load_kg_day: f64,
    /// Beban aktual dari buangan (kg/hari).
    pub waste_load_kg_day: f64,
    /// Sisa daya tampung = allowable − aktual (kg/hari).
    pub dtbp_kg_day: f64,
    /// Konsentrasi hasil pencampuran di hilir titik buang (mg/L).
    pub c_mix_mgl: f64,
    /// Konsentrasi limbah maksimum pada debit limbah saat ini (mg/L).
    /// `None` bila tidak ada debit limbah, sehingga tidak ada batas yang bermakna.
    pub max_waste_conc_mgl: Option<f64>,
    pub design_flow_basis: DesignFlowBasis,
    pub status: ResultStatus,
    pub limitations: Vec<String>,
}

impl DtbpAssessment {
    /// Kapasitas tersisa ada bila DTBP positif. Ini pernyataan neraca massa,
    /// bukan kesimpulan hukum tentang kepatuhan.
    pub fn has_remaining_capacity(&self) -> bool {
        self.dtbp_kg_day > 0.0
    }
}

pub fn assess(
    q_river_m3s: f64,
    c_upstream_mgl: f64,
    c_standard_mgl: f64,
    q_waste_m3s: f64,
    c_waste_mgl: f64,
    parameter: &str,
    design_flow_basis: DesignFlowBasis,
) -> Result<DtbpAssessment, String> {
    if !q_river_m3s.is_finite() || q_river_m3s <= 0.0 {
        return Err(format!(
            "ERROR [E102]: q_river_m3s harus > 0 dan finit. {}",
            q_river_m3s
        ));
    }
    if !q_waste_m3s.is_finite() || q_waste_m3s < 0.0 {
        return Err(format!(
            "ERROR [E102]: q_waste_m3s tidak boleh negatif dan harus finit. {}",
            q_waste_m3s
        ));
    }
    if !c_standard_mgl.is_finite() || c_standard_mgl <= 0.0 {
        return Err(format!(
            "ERROR [E102]: c_standard_mgl harus > 0 dan finit. {}",
            c_standard_mgl
        ));
    }
    if !c_upstream_mgl.is_finite() || c_upstream_mgl < 0.0 {
        return Err(format!(
            "ERROR [E102]: c_upstream_mgl tidak boleh negatif dan harus finit. {}",
            c_upstream_mgl
        ));
    }
    if !c_waste_mgl.is_finite() || c_waste_mgl < 0.0 {
        return Err(format!(
            "ERROR [E102]: c_waste_mgl tidak boleh negatif dan harus finit. {}",
            c_waste_mgl
        ));
    }

    let q_total = q_river_m3s + q_waste_m3s;

    // Neraca massa titik campur. Debit limbah ikut mengencerkan, jadi kapasitas
    // dihitung terhadap debit total, bukan debit sungai saja.
    let allowable_load_kg_day =
        (q_total * c_standard_mgl - q_river_m3s * c_upstream_mgl) * KG_PER_DAY_FACTOR;
    let waste_load_kg_day = q_waste_m3s * c_waste_mgl * KG_PER_DAY_FACTOR;
    let dtbp_kg_day = allowable_load_kg_day - waste_load_kg_day;

    let c_mix_mgl = (q_river_m3s * c_upstream_mgl + q_waste_m3s * c_waste_mgl) / q_total;

    let max_waste_conc_mgl = if q_waste_m3s > 0.0 {
        Some((q_total * c_standard_mgl - q_river_m3s * c_upstream_mgl) / q_waste_m3s)
    } else {
        None
    };

    let mut limitations = Vec::new();

    // Debit rancangan menentukan apakah angka ini dapat dipertanggungjawabkan.
    let mut status = if design_flow_basis.is_low_flow() {
        ResultStatus::ValidWithAssumptions
    } else {
        ResultStatus::ScreeningOnly
    };

    match design_flow_basis {
        DesignFlowBasis::MeanAnnual => limitations.push(
            "Debit rancangan adalah rata-rata tahunan. Beban kritis terjadi pada debit \
             rendah, sehingga kapasitas asimilasi di sini terlalu besar. Hitung Q95 atau \
             Q7,10 dari seri debit (tool `flow_duration_curve`) sebelum dipakai untuk \
             persetujuan teknis."
                .into(),
        ),
        DesignFlowBasis::Unknown => limitations.push(
            "Dasar statistik debit sungai tidak dinyatakan, jadi kapasitas asimilasi \
             tidak dapat diverifikasi. Nyatakan `design_flow_basis` sebagai q95 atau \
             q7_10, atau hitung dengan tool `flow_duration_curve`."
                .into(),
        ),
        DesignFlowBasis::Q95 | DesignFlowBasis::Q7Q10 => limitations.push(format!(
            "Debit rancangan dinyatakan sebagai {}. Nilai debitnya sendiri tidak \
             diverifikasi oleh tool ini.",
            design_flow_basis.label()
        )),
    }

    // Sungai sudah melampaui baku mutu sebelum ada buangan: tidak ada kapasitas
    // untuk dialokasikan, dan angka DTBP negatif bukan "kuota" apa pun.
    if c_upstream_mgl >= c_standard_mgl {
        limitations.push(format!(
            "Konsentrasi hulu ({:.3} mg/L) sudah >= baku mutu ({:.3} mg/L). Tidak ada \
             daya tampung untuk dialokasikan; setiap penambahan beban memperburuk \
             kondisi yang telah melampaui baku mutu.",
            c_upstream_mgl, c_standard_mgl
        ));
        status = ResultStatus::ScreeningOnly;
    }

    limitations.push(
        "Model pencampuran sempurna seketika, kondisi tunak, tanpa peluruhan/pengendapan \
         dan tanpa sumber lain di ruas yang sama. Sungai dengan banyak pembuang \
         memerlukan neraca multi-sumber."
            .into(),
    );

    Ok(DtbpAssessment {
        parameter: parameter.to_string(),
        q_river_m3s,
        q_waste_m3s,
        c_upstream_mgl,
        c_standard_mgl,
        c_waste_mgl,
        allowable_load_kg_day,
        waste_load_kg_day,
        dtbp_kg_day,
        c_mix_mgl,
        max_waste_conc_mgl,
        design_flow_basis,
        status,
        limitations,
    })
}

pub fn calculate(
    q_river_m3s: f64,
    c_upstream_mgl: f64,
    c_standard_mgl: f64,
    q_waste_m3s: f64,
    c_waste_mgl: f64,
    parameter: &str,
    design_flow_basis: Option<String>,
) -> String {
    let basis = design_flow_basis
        .as_deref()
        .map(DesignFlowBasis::parse)
        .unwrap_or(DesignFlowBasis::Unknown);

    let a = match assess(
        q_river_m3s,
        c_upstream_mgl,
        c_standard_mgl,
        q_waste_m3s,
        c_waste_mgl,
        parameter,
        basis,
    ) {
        Ok(a) => a,
        Err(e) => return e,
    };

    format_assessment(&a)
}

pub fn format_assessment(a: &DtbpAssessment) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Daya Tampung Beban Pencemaran\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PP No. 22 Tahun 2021\n\n");
    out.push_str(&format!("Parameter          : {}\n", a.parameter));
    out.push_str(&format!("Debit Sungai (Q)   : {:.4} m³/s\n", a.q_river_m3s));
    out.push_str(&format!(
        "Dasar debit Q      : {}\n",
        a.design_flow_basis.label()
    ));
    out.push_str(&format!("C Hulu             : {:.3} mg/L\n", a.c_upstream_mgl));
    out.push_str(&format!(
        "Baku Mutu (C_bm)   : {:.3} mg/L\n",
        a.c_standard_mgl
    ));
    out.push_str(&format!("Debit Limbah (q)   : {:.4} m³/s\n", a.q_waste_m3s));
    out.push_str(&format!(
        "C Limbah           : {:.3} mg/L\n\n",
        a.c_waste_mgl
    ));

    out.push_str("Neraca massa titik campur:\n");
    out.push_str("  (Q·C_hulu + q·C_limbah) / (Q+q) <= C_bm\n\n");
    out.push_str(&format!(
        "  Beban diizinkan = ((Q+q)·C_bm − Q·C_hulu) × {}\n",
        KG_PER_DAY_FACTOR
    ));
    out.push_str(&format!(
        "                  = (({:.4}+{:.4})×{:.3} − {:.4}×{:.3}) × {}\n",
        a.q_river_m3s,
        a.q_waste_m3s,
        a.c_standard_mgl,
        a.q_river_m3s,
        a.c_upstream_mgl,
        KG_PER_DAY_FACTOR
    ));
    out.push_str(&format!(
        "                  = {:.2} kg/hari\n\n",
        a.allowable_load_kg_day
    ));
    out.push_str(&format!(
        "  Beban limbah    = q × C_limbah × {} = {:.2} kg/hari\n\n",
        KG_PER_DAY_FACTOR, a.waste_load_kg_day
    ));
    out.push_str(&format!(
        "  DTBP = {:.2} − {:.2} = {:.2} kg/hari\n\n",
        a.allowable_load_kg_day, a.waste_load_kg_day, a.dtbp_kg_day
    ));

    out.push_str("Pemeriksaan langsung (angka yang dapat dicek):\n");
    out.push_str(&format!(
        "  C campuran hilir : {:.3} mg/L (baku mutu {:.3} mg/L)\n",
        a.c_mix_mgl, a.c_standard_mgl
    ));
    match a.max_waste_conc_mgl {
        Some(c) => out.push_str(&format!(
            "  C limbah maks    : {:.3} mg/L pada q = {:.4} m³/s\n\n",
            c, a.q_waste_m3s
        )),
        None => out.push_str(
            "  C limbah maks    : tidak terdefinisi (tidak ada debit limbah)\n\n",
        ),
    }

    out.push_str(&format!(
        "Sisa daya tampung  : {}\n",
        if a.has_remaining_capacity() {
            "ada (DTBP > 0)"
        } else {
            "tidak ada (DTBP <= 0)"
        }
    ));
    out.push_str(&format!(
        "Status hasil       : {}\n",
        serde_json::to_value(&a.status)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".into())
    ));
    out.push_str(
        "\nCatatan: ini pernyataan neraca massa, bukan kesimpulan hukum tentang\n\
         kepatuhan. Penetapan alokasi beban adalah kewenangan instansi berwenang.\n",
    );

    if !a.limitations.is_empty() {
        out.push_str("\nKeterbatasan:\n");
        for l in &a.limitations {
            out.push_str(&format!("  - {}\n", l));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const Q: f64 = 2.0;
    const C_UP: f64 = 1.0;
    const C_BM: f64 = 3.0;
    const QW: f64 = 0.5;

    fn run(c_waste: f64, basis: DesignFlowBasis) -> DtbpAssessment {
        assess(Q, C_UP, C_BM, QW, c_waste, "bod", basis).expect("valid input")
    }

    #[test]
    fn max_waste_concentration_accounts_for_the_waste_flow_as_diluent() {
        // Neraca titik campur: ((Q+q)·C_bm − Q·C_hulu)/q
        //                     = ((2.5)(3) − (2)(1))/0.5 = 11.0 mg/L
        // Versi lama memakai Q·(C_bm − C_hulu)/q = 8.0 mg/L, mengabaikan bahwa
        // debit limbah sendiri menambah volume pengencer. Selisihnya tepat C_bm.
        let a = run(100.0, DesignFlowBasis::Q95);
        let got = a.max_waste_conc_mgl.expect("waste flow present");
        assert!(
            (got - 11.0).abs() < 1e-9,
            "expected 11.0 mg/L from mixing-point balance, got {got}"
        );
        let legacy = Q * (C_BM - C_UP) / QW;
        assert!(
            (got - legacy - C_BM).abs() < 1e-9,
            "gap against the legacy formula must be exactly C_bm"
        );
    }

    #[test]
    fn discharging_at_the_maximum_concentration_exhausts_the_capacity_exactly() {
        // Invarian yang tidak dipenuhi kode lama: memasukkan kembali
        // C_limbah,maks harus menghabiskan DTBP tepat nol dan menempatkan
        // konsentrasi campuran persis di baku mutu.
        let probe = run(0.0, DesignFlowBasis::Q95);
        let c_max = probe.max_waste_conc_mgl.expect("waste flow present");

        let a = run(c_max, DesignFlowBasis::Q95);
        assert!(
            a.dtbp_kg_day.abs() < 1e-9,
            "DTBP at C_max must be zero, got {} kg/day",
            a.dtbp_kg_day
        );
        assert!(
            (a.c_mix_mgl - C_BM).abs() < 1e-9,
            "mixed concentration at C_max must equal the standard, got {}",
            a.c_mix_mgl
        );
        assert!(!a.has_remaining_capacity());
    }

    #[test]
    fn mixed_concentration_is_reported_so_the_load_number_can_be_checked() {
        // (2·1 + 0.5·100)/2.5 = 20.8 mg/L, jauh di atas baku mutu 3 mg/L.
        let a = run(100.0, DesignFlowBasis::Q95);
        assert!((a.c_mix_mgl - 20.8).abs() < 1e-9, "got {}", a.c_mix_mgl);
        assert!(a.dtbp_kg_day < 0.0, "over-standard discharge must show deficit");
    }

    #[test]
    fn allowable_load_and_dtbp_stay_consistent_with_the_mixed_concentration() {
        // Tanda DTBP dan posisi C_campur terhadap baku mutu tidak boleh
        // bertentangan pada rentang konsentrasi limbah mana pun.
        for c_waste in [0.0, 1.0, 3.0, 10.0, 11.0, 12.0, 50.0, 500.0] {
            let a = run(c_waste, DesignFlowBasis::Q95);
            let over_standard = a.c_mix_mgl > a.c_standard_mgl + 1e-12;
            let deficit = a.dtbp_kg_day < -1e-12;
            assert_eq!(
                over_standard, deficit,
                "c_waste={c_waste}: c_mix={} vs standard={}, dtbp={}",
                a.c_mix_mgl, a.c_standard_mgl, a.dtbp_kg_day
            );
        }
    }

    #[test]
    fn mean_annual_flow_is_downgraded_to_screening_only() {
        let a = run(1.0, DesignFlowBasis::MeanAnnual);
        assert_eq!(a.status, ResultStatus::ScreeningOnly);
        assert!(
            a.limitations.iter().any(|l| l.contains("rata-rata tahunan")),
            "must say why the basis is inadequate: {:?}",
            a.limitations
        );
    }

    #[test]
    fn unknown_flow_basis_is_screening_only_and_names_the_missing_input() {
        let a = run(1.0, DesignFlowBasis::Unknown);
        assert_eq!(a.status, ResultStatus::ScreeningOnly);
        assert!(
            a.limitations
                .iter()
                .any(|l| l.contains("design_flow_basis")),
            "must name the input to supply: {:?}",
            a.limitations
        );
    }

    #[test]
    fn low_flow_basis_is_the_only_one_that_carries_assumptions_rather_than_screening() {
        for basis in [DesignFlowBasis::Q95, DesignFlowBasis::Q7Q10] {
            assert_eq!(run(1.0, basis).status, ResultStatus::ValidWithAssumptions);
        }
    }

    #[test]
    fn upstream_already_over_standard_has_no_capacity_to_allocate() {
        let a = assess(2.0, 5.0, 3.0, 0.5, 1.0, "bod", DesignFlowBasis::Q95)
            .expect("valid input");
        assert_eq!(a.status, ResultStatus::ScreeningOnly);
        assert!(
            a.limitations.iter().any(|l| l.contains("sudah >= baku mutu")),
            "{:?}",
            a.limitations
        );
    }

    #[test]
    fn zero_waste_flow_has_no_meaningful_concentration_limit() {
        let a = assess(2.0, 1.0, 3.0, 0.0, 0.0, "bod", DesignFlowBasis::Q95)
            .expect("valid input");
        assert!(a.max_waste_conc_mgl.is_none());
        assert!((a.c_mix_mgl - 1.0).abs() < 1e-9);
    }

    #[test]
    fn design_flow_basis_parses_indonesian_and_shorthand_spellings() {
        assert_eq!(DesignFlowBasis::parse("Q95"), DesignFlowBasis::Q95);
        assert_eq!(DesignFlowBasis::parse("q7,10"), DesignFlowBasis::Q7Q10);
        assert_eq!(DesignFlowBasis::parse("Q7-10"), DesignFlowBasis::Q7Q10);
        assert_eq!(DesignFlowBasis::parse("rata-rata"), DesignFlowBasis::MeanAnnual);
        assert_eq!(DesignFlowBasis::parse("maf"), DesignFlowBasis::MeanAnnual);
        assert_eq!(DesignFlowBasis::parse("apa saja"), DesignFlowBasis::Unknown);
    }

    #[test]
    fn non_finite_input_is_rejected_rather_than_producing_nan_capacity() {
        assert!(assess(f64::NAN, 1.0, 3.0, 0.5, 1.0, "bod", DesignFlowBasis::Q95).is_err());
        assert!(assess(2.0, 1.0, f64::INFINITY, 0.5, 1.0, "bod", DesignFlowBasis::Q95).is_err());
        assert!(assess(2.0, 1.0, 3.0, 0.5, f64::NAN, "bod", DesignFlowBasis::Q95).is_err());
    }

    #[test]
    fn output_states_it_is_not_a_legal_conclusion() {
        let text = format_assessment(&run(1.0, DesignFlowBasis::Q95));
        assert!(text.contains("bukan kesimpulan hukum"));
        assert!(text.contains("C campuran hilir"));
        // Verdict emoji lama tidak boleh kembali.
        assert!(!text.contains('✅') && !text.contains('❌'));
    }
}
