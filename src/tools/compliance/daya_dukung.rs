/// Daya Dukung Lingkungan Hidup — Land, Water, Food
///
/// Ref: PermenLH 17/2009 menetapkan pendekatan ketersediaan-kebutuhan
/// (supply/demand). Tool ini menghitung rasio itu untuk tiga sumber daya.
///
/// Penting soal apa yang TIDAK dilakukan tool ini: nilai kebutuhan per kapita
/// (lahan, air, pangan) adalah masukan kebijakan yang berbeda antar tipologi
/// wilayah dan sumber acuan. Versi sebelumnya memakai 0.07 ha/kapita yang
/// di-hardcode tanpa rujukan, lalu langsung mencetak "DDL Terlampaui" untuk
/// seluruh Indonesia. Angka itu sekarang wajib dinyatakan pemanggil beserta
/// sumbernya, karena ia yang menentukan hasilnya, bukan perhitungannya.
use crate::honesty::{gate, DataAvailability, MaturityLevel};
use crate::result_contract::ResultStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Land,
    Water,
    Food,
}

impl Resource {
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "land" | "lahan" | "population" | "penduduk" => Some(Resource::Land),
            "water" | "air" => Some(Resource::Water),
            "food" | "pangan" => Some(Resource::Food),
            _ => None,
        }
    }

    fn supply_unit(self) -> &'static str {
        match self {
            Resource::Land => "ha",
            Resource::Water => "m³/tahun",
            Resource::Food => "ton/tahun",
        }
    }

    fn demand_unit(self) -> &'static str {
        match self {
            Resource::Land => "ha/kapita",
            Resource::Water => "m³/kapita/tahun",
            Resource::Food => "ton/kapita/tahun",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Resource::Land => "Lahan",
            Resource::Water => "Air",
            Resource::Food => "Pangan",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CarryingCapacity {
    pub resource: Resource,
    pub supply: f64,
    pub demand_per_capita: f64,
    /// Sumber acuan kebutuhan per kapita, dinyatakan pemanggil.
    pub demand_basis: String,
    pub population: f64,
    pub total_demand: f64,
    /// Jumlah penduduk yang dapat didukung pasokan ini.
    pub supported_population: f64,
    /// Rasio kebutuhan terhadap ketersediaan. > 1 berarti kebutuhan melampaui.
    pub demand_supply_ratio: f64,
    pub status: ResultStatus,
    pub limitations: Vec<String>,
}

impl CarryingCapacity {
    /// Kebutuhan melampaui ketersediaan menurut angka yang dimasukkan.
    /// Ini pernyataan aritmetika, bukan penetapan status DDL suatu wilayah.
    pub fn demand_exceeds_supply(&self) -> bool {
        self.demand_supply_ratio > 1.0
    }
}

pub fn assess(
    resource: Resource,
    supply: f64,
    population: f64,
    demand_per_capita: f64,
    demand_basis: &str,
) -> Result<CarryingCapacity, String> {
    if !supply.is_finite() || supply <= 0.0 {
        return Err(format!(
            "ERROR [E102]: ketersediaan {} harus > 0 dan finit. {}",
            resource.label(),
            supply
        ));
    }
    if !population.is_finite() || population <= 0.0 {
        return Err(format!(
            "ERROR [E102]: jumlah penduduk harus > 0 dan finit. {}",
            population
        ));
    }
    if !demand_per_capita.is_finite() || demand_per_capita <= 0.0 {
        return Err(format!(
            "ERROR [E102]: kebutuhan per kapita ({}) harus > 0 dan finit, dan wajib \
             dinyatakan. Angka ini yang menentukan hasil daya dukung, jadi tidak \
             disediakan sebagai nilai bawaan. {}",
            resource.demand_unit(),
            demand_per_capita
        ));
    }
    if demand_basis.trim().is_empty() {
        return Err(format!(
            "ERROR [E102]: `demand_basis` wajib diisi — sebutkan sumber acuan \
             kebutuhan {} per kapita (misal nomor peraturan, SNI, atau dokumen \
             perencanaan). Tanpa itu hasilnya tidak dapat diperiksa.",
            resource.label().to_lowercase()
        ));
    }

    let total_demand = demand_per_capita * population;
    let supported_population = supply / demand_per_capita;
    let demand_supply_ratio = total_demand / supply;

    // Kebutuhan per kapita berasal dari pemanggil, bukan dari observasi lapangan
    // maupun neraca sumber daya terukur. Itu menempatkan hasil di tangga
    // screening: aritmetikanya benar, dasarnya belum tervalidasi.
    let availability = DataAvailability {
        satellite_context: true,
        ..Default::default()
    };
    let decision = gate(MaturityLevel::Calibrated, &availability);

    let mut limitations = vec![
        format!(
            "Kebutuhan {} per kapita = {} {} dengan acuan \"{}\", dinyatakan pemanggil. \
             Hasil daya dukung ditentukan oleh angka ini; tool tidak memverifikasinya.",
            resource.label().to_lowercase(),
            demand_per_capita,
            resource.demand_unit(),
            demand_basis.trim()
        ),
        "Perhitungan ini rasio ketersediaan-kebutuhan agregat pada satu wilayah dan \
         satu titik waktu. Ia tidak memperhitungkan sebaran dalam wilayah, aliran \
         masuk-keluar antar-wilayah, mutu sumber daya, maupun dinamika musiman."
            .into(),
    ];

    if decision.blocked {
        limitations.push(format!(
            "Status dikunci pada tingkat ketersediaan data: kurang {}. Angka ini tidak \
             dapat dipakai sebagai penetapan daya dukung wilayah.",
            decision.missing.join(", ")
        ));
    }

    match resource {
        Resource::Land => limitations.push(
            "Pendekatan lahan menyamakan seluruh luas wilayah sebagai lahan yang \
             tersedia. Badan air, lereng terjal, kawasan lindung, dan lahan tidak \
             terbangun lain seharusnya dikeluarkan lebih dahulu."
                .into(),
        ),
        Resource::Water => limitations.push(
            "Ketersediaan air harus berbasis debit andalan, bukan potensi total. \
             Air permukaan dan air tanah punya keterbaruan berbeda dan tidak dapat \
             dijumlahkan begitu saja."
                .into(),
        ),
        Resource::Food => limitations.push(
            "Produksi pangan wilayah bukan pasokan yang tersedia bagi penduduk \
             wilayah itu: ada perdagangan antar-wilayah, susut pascapanen, dan \
             penggunaan non-pangan."
                .into(),
        ),
    }

    Ok(CarryingCapacity {
        resource,
        supply,
        demand_per_capita,
        demand_basis: demand_basis.trim().to_string(),
        population,
        total_demand,
        supported_population,
        demand_supply_ratio,
        status: crate::honesty::to_result_status(decision.allowed_level),
        limitations,
    })
}

pub fn calculate(
    approach: &str,
    population: f64,
    supply: f64,
    demand_per_capita: Option<f64>,
    demand_basis: Option<String>,
) -> String {
    let resource = match Resource::parse(approach) {
        Some(r) => r,
        None => {
            return format!(
                "ERROR [E102]: pendekatan '{}' tidak dikenal.\nPilihan: land/lahan, \
                 water/air, food/pangan",
                approach
            )
        }
    };

    let demand = match demand_per_capita {
        Some(v) => v,
        None => {
            return format!(
                "ERROR [E102]: `demand_per_capita` ({}) wajib diisi untuk pendekatan \
                 {}. Tidak ada nilai bawaan: kebutuhan per kapita adalah masukan \
                 kebijakan yang berbeda antar tipologi wilayah, dan ia yang \
                 menentukan hasilnya.",
                resource.demand_unit(),
                resource.label().to_lowercase()
            )
        }
    };

    let basis = demand_basis.unwrap_or_default();

    match assess(resource, supply, population, demand, &basis) {
        Ok(c) => format_capacity(&c),
        Err(e) => e,
    }
}

pub fn format_capacity(c: &CarryingCapacity) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Daya Dukung Lingkungan Hidup\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PermenLH No. 17 Tahun 2009 (pendekatan ketersediaan-kebutuhan)\n\n");
    out.push_str(&format!("Sumber daya       : {}\n", c.resource.label()));
    out.push_str(&format!(
        "Ketersediaan      : {:.4} {}\n",
        c.supply,
        c.resource.supply_unit()
    ));
    out.push_str(&format!("Penduduk          : {:.0} jiwa\n", c.population));
    out.push_str(&format!(
        "Kebutuhan/kapita  : {} {}\n",
        c.demand_per_capita,
        c.resource.demand_unit()
    ));
    out.push_str(&format!("Acuan kebutuhan   : {}\n\n", c.demand_basis));

    out.push_str("Perhitungan:\n");
    out.push_str(&format!(
        "  Total kebutuhan     = {} × {:.0} = {:.4} {}\n",
        c.demand_per_capita,
        c.population,
        c.total_demand,
        c.resource.supply_unit()
    ));
    out.push_str(&format!(
        "  Penduduk terdukung  = {:.4} / {} = {:.0} jiwa\n",
        c.supply, c.demand_per_capita, c.supported_population
    ));
    out.push_str(&format!(
        "  Rasio kebutuhan/ketersediaan = {:.4}\n\n",
        c.demand_supply_ratio
    ));

    out.push_str(&format!(
        "Menurut angka masukan: kebutuhan {} ketersediaan.\n",
        if c.demand_exceeds_supply() {
            "MELAMPAUI"
        } else {
            "masih di bawah"
        }
    ));
    out.push_str(&format!(
        "Status hasil      : {}\n",
        serde_json::to_value(&c.status)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".into())
    ));
    out.push_str(
        "\nCatatan: ini rasio ketersediaan-kebutuhan dari angka yang dimasukkan,\n\
         bukan penetapan status daya dukung wilayah. Penetapan DDL adalah\n\
         kewenangan instansi berwenang dan memerlukan neraca sumber daya terukur.\n",
    );

    out.push_str("\nKeterbatasan:\n");
    for l in &c.limitations {
        out.push_str(&format!("  - {}\n", l));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn land(supply_ha: f64, pop: f64, per_capita: f64) -> CarryingCapacity {
        assess(Resource::Land, supply_ha, pop, per_capita, "RTRW Kab X 2024")
            .expect("valid input")
    }

    #[test]
    fn per_capita_demand_has_no_default_because_it_decides_the_answer() {
        // Angka 0.07 ha/kapita yang dulu di-hardcode membalik kesimpulan hanya
        // dengan mengubah asumsi, pada wilayah dan penduduk yang sama.
        let strict = land(1000.0, 20_000.0, 0.07);
        let lenient = land(1000.0, 20_000.0, 0.04);
        assert!(strict.demand_exceeds_supply());
        assert!(!lenient.demand_exceeds_supply());

        // Karena itu tidak boleh ada nilai bawaan.
        let out = calculate("land", 20_000.0, 1000.0, None, Some("apa pun".into()));
        assert!(out.starts_with("ERROR [E102]"), "{out}");
        assert!(out.contains("wajib diisi"), "{out}");
    }

    #[test]
    fn the_basis_for_the_per_capita_demand_must_be_stated() {
        let err = assess(Resource::Land, 1000.0, 20_000.0, 0.07, "   ")
            .expect_err("empty basis must be rejected");
        assert!(err.contains("demand_basis"), "{err}");
    }

    #[test]
    fn the_stated_basis_is_carried_into_the_limitations_so_it_can_be_challenged() {
        let c = land(1000.0, 20_000.0, 0.07);
        assert!(
            c.limitations
                .iter()
                .any(|l| l.contains("RTRW Kab X 2024") && l.contains("0.07")),
            "{:?}",
            c.limitations
        );
    }

    #[test]
    fn ratio_and_supported_population_agree_with_each_other() {
        for (supply, pop, per_capita) in [
            (1000.0, 10_000.0, 0.05),
            (500.0, 50_000.0, 0.07),
            (2500.0, 1_000.0, 0.10),
        ] {
            let c = land(supply, pop, per_capita);
            let exceeds_by_ratio = c.demand_supply_ratio > 1.0;
            let exceeds_by_population = c.population > c.supported_population;
            assert_eq!(
                exceeds_by_ratio, exceeds_by_population,
                "ratio={} pop={} supported={}",
                c.demand_supply_ratio, c.population, c.supported_population
            );
        }
    }

    #[test]
    fn result_never_claims_validity_because_the_demand_figure_is_unverified() {
        let c = land(1000.0, 20_000.0, 0.07);
        assert_eq!(c.status, ResultStatus::ScreeningOnly);
        assert!(
            c.limitations.iter().any(|l| l.contains("dikunci")),
            "{:?}",
            c.limitations
        );
    }

    #[test]
    fn output_is_not_a_regulatory_determination_and_drops_the_verdict_emoji() {
        let text = format_capacity(&land(1000.0, 20_000.0, 0.07));
        assert!(text.contains("bukan penetapan status daya dukung"));
        assert!(!text.contains('✅') && !text.contains('❌'));
        assert!(!text.contains("DDL Terlampaui"));
    }

    #[test]
    fn each_resource_names_the_assumption_that_most_distorts_it() {
        let water = assess(Resource::Water, 1e6, 20_000.0, 21.9, "WHO basic need")
            .expect("valid");
        assert!(
            water.limitations.iter().any(|l| l.contains("debit andalan")),
            "{:?}",
            water.limitations
        );

        let food = assess(Resource::Food, 5000.0, 20_000.0, 0.3, "BPS konsumsi beras")
            .expect("valid");
        assert!(
            food.limitations.iter().any(|l| l.contains("perdagangan antar-wilayah")),
            "{:?}",
            food.limitations
        );

        let land_case = land(1000.0, 20_000.0, 0.07);
        assert!(
            land_case.limitations.iter().any(|l| l.contains("kawasan lindung")),
            "{:?}",
            land_case.limitations
        );
    }

    #[test]
    fn resource_names_accept_indonesian_and_the_legacy_population_spelling() {
        assert_eq!(Resource::parse("lahan"), Some(Resource::Land));
        // `population` adalah nama lama pendekatan lahan; tetap diterima.
        assert_eq!(Resource::parse("population"), Some(Resource::Land));
        assert_eq!(Resource::parse("penduduk"), Some(Resource::Land));
        assert_eq!(Resource::parse("air"), Some(Resource::Water));
        assert_eq!(Resource::parse("pangan"), Some(Resource::Food));
        assert_eq!(Resource::parse("energi"), None);
    }

    #[test]
    fn non_finite_and_non_positive_input_is_rejected() {
        assert!(assess(Resource::Land, 0.0, 100.0, 0.07, "x").is_err());
        assert!(assess(Resource::Land, 100.0, 0.0, 0.07, "x").is_err());
        assert!(assess(Resource::Land, 100.0, 100.0, 0.0, "x").is_err());
        assert!(assess(Resource::Land, f64::NAN, 100.0, 0.07, "x").is_err());
        assert!(assess(Resource::Land, 100.0, f64::INFINITY, 0.07, "x").is_err());
        assert!(assess(Resource::Land, 100.0, 100.0, f64::NAN, "x").is_err());
    }

    #[test]
    fn unknown_approach_lists_the_valid_options() {
        let out = calculate("energi", 100.0, 100.0, Some(1.0), Some("x".into()));
        assert!(out.contains("land/lahan"), "{out}");
        assert!(out.contains("water/air"), "{out}");
        assert!(out.contains("food/pangan"), "{out}");
    }
}
