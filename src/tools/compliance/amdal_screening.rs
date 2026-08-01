/// AMDAL Screening Tool
/// Ref: PermenLHK 4/2021

pub fn screen(sector: &str, activity: &str, scale_value: f64, scale_unit: &str) -> String {
    let s = sector.to_lowercase();
    let a = activity.to_lowercase();

    if scale_value < 0.0 {
        return format!(
            "ERROR [E102]: Parameter tidak boleh negatif. {}",
            scale_value
        );
    }

    // (sector, activity) -> (amdal_threshold, ukl_upl_threshold, unit, activity_desc)
    // If scale >= amdal_threshold -> AMDAL
    // If scale >= ukl_upl_threshold -> UKL-UPL
    // Else -> SPPL
    struct Threshold {
        sector: &'static str,
        activity: &'static str,
        amdal: f64,
        uklupl: f64,
        unit: &'static str,
        desc: &'static str,
    }

    let thresholds = vec![
        // Pertambangan
        Threshold {
            sector: "pertambangan",
            activity: "mineral_logam",
            amdal: 200.0,
            uklupl: 50.0,
            unit: "ha",
            desc: "Eksploitasi mineral logam",
        },
        Threshold {
            sector: "pertambangan",
            activity: "batubara",
            amdal: 200.0,
            uklupl: 50.0,
            unit: "ha",
            desc: "Eksploitasi batubara",
        },
        Threshold {
            sector: "pertambangan",
            activity: "batu_gamping",
            amdal: 25.0,
            uklupl: 5.0,
            unit: "ha",
            desc: "Penambangan batu gamping/kapur",
        },
        // Kehutanan
        Threshold {
            sector: "kehutanan",
            activity: "hph",
            amdal: 100.0,
            uklupl: 0.0,
            unit: "ha",
            desc: "Hak Pengusahaan Hutan (HPH)",
        },
        Threshold {
            sector: "kehutanan",
            activity: "hutan_tanaman",
            amdal: 5000.0,
            uklupl: 1000.0,
            unit: "ha",
            desc: "Hutan Tanaman Industri (HTI)",
        },
        Threshold {
            sector: "kehutanan",
            activity: "pelepasan_kawasan",
            amdal: 200.0,
            uklupl: 50.0,
            unit: "ha",
            desc: "Pelepasan kawasan hutan",
        },
        // Industri
        Threshold {
            sector: "industri",
            activity: "semen",
            amdal: 1_000_000.0,
            uklupl: 100_000.0,
            unit: "ton/tahun",
            desc: "Industri semen",
        },
        Threshold {
            sector: "industri",
            activity: "pulp_kertas",
            amdal: 100_000.0,
            uklupl: 10_000.0,
            unit: "ton/tahun",
            desc: "Industri pulp & kertas",
        },
        Threshold {
            sector: "industri",
            activity: "tekstil",
            amdal: 20_000.0,
            uklupl: 5_000.0,
            unit: "ton/tahun",
            desc: "Industri tekstil/finishing",
        },
        Threshold {
            sector: "industri",
            activity: "sawit",
            amdal: 3000.0,
            uklupl: 500.0,
            unit: "ha",
            desc: "Perkebunan kelapa sawit (luas lahan)",
        },
        Threshold {
            sector: "industri",
            activity: "farmasi",
            amdal: 100_000.0,
            uklupl: 10_000.0,
            unit: "ton/tahun",
            desc: "Industri farmasi",
        },
        // Energi
        Threshold {
            sector: "energi",
            activity: "pltu",
            amdal: 100.0,
            uklupl: 10.0,
            unit: "MW",
            desc: "PLTU (Pembangkit Listrik Tenaga Uap)",
        },
        Threshold {
            sector: "energi",
            activity: "plta",
            amdal: 50.0,
            uklupl: 2.0,
            unit: "MW",
            desc: "PLTA (Pembangkit Listrik Tenaga Air)",
        },
        Threshold {
            sector: "energi",
            activity: "pltg",
            amdal: 100.0,
            uklupl: 10.0,
            unit: "MW",
            desc: "PLTG (Pembangkit Listrik Tenaga Gas)",
        },
        Threshold {
            sector: "energi",
            activity: "sutt",
            amdal: 150.0,
            uklupl: 0.0,
            unit: "kV",
            desc: "Saluran Udara Tegangan Tinggi",
        },
        // Transportasi
        Threshold {
            sector: "transportasi",
            activity: "jalan_tol",
            amdal: 5.0,
            uklupl: 1.0,
            unit: "km",
            desc: "Pembangunan jalan tol",
        },
        Threshold {
            sector: "transportasi",
            activity: "pelabuhan",
            amdal: 10_000.0,
            uklupl: 2_000.0,
            unit: "DWT",
            desc: "Pembangunan pelabuhan",
        },
        Threshold {
            sector: "transportasi",
            activity: "bandara",
            amdal: 2500.0,
            uklupl: 1000.0,
            unit: "m (runway)",
            desc: "Pembangunan bandara",
        },
        // Pariwisata
        Threshold {
            sector: "pariwisata",
            activity: "kawasan_wisata",
            amdal: 100.0,
            uklupl: 10.0,
            unit: "ha",
            desc: "Kawasan wisata",
        },
        Threshold {
            sector: "pariwisata",
            activity: "hotel_resort",
            amdal: 200.0,
            uklupl: 50.0,
            unit: "kamar",
            desc: "Hotel / resort",
        },
        // Pertanian
        Threshold {
            sector: "pertanian",
            activity: "irigasi",
            amdal: 2000.0,
            uklupl: 500.0,
            unit: "ha",
            desc: "Jaringan irigasi",
        },
        Threshold {
            sector: "pertanian",
            activity: "perkebunan",
            amdal: 3000.0,
            uklupl: 500.0,
            unit: "ha",
            desc: "Perkebunan (selain sawit)",
        },
        // Perikanan
        Threshold {
            sector: "perikanan",
            activity: "tambak",
            amdal: 50.0,
            uklupl: 10.0,
            unit: "ha",
            desc: "Budidaya tambak",
        },
        Threshold {
            sector: "perikanan",
            activity: "keramba",
            amdal: 2500.0,
            uklupl: 500.0,
            unit: "m²",
            desc: "Keramba jaring apung",
        },
        // Permukiman
        Threshold {
            sector: "permukiman",
            activity: "perumahan",
            amdal: 100.0,
            uklupl: 25.0,
            unit: "ha",
            desc: "Perumahan / real estate",
        },
        Threshold {
            sector: "permukiman",
            activity: "gedung",
            amdal: 10000.0,
            uklupl: 5000.0,
            unit: "m²",
            desc: "Gedung bertingkat (luas lantai)",
        },
    ];

    let found = thresholds.iter().find(|t| {
        (s.contains(t.sector) || t.sector.contains(&s))
            && (a.contains(t.activity) || t.activity.contains(&a))
    });

    let th = match found {
        Some(t) => t,
        None => {
            let mut out = format!(
                "ERROR: Kombinasi sektor '{}' dan aktivitas '{}' tidak ditemukan.\n\nSektor & aktivitas valid:\n",
                sector, activity
            );
            for t in &thresholds {
                out.push_str(&format!("  {} / {} ({})\n", t.sector, t.activity, t.desc));
            }
            return out;
        }
    };

    if scale_unit.to_lowercase() != th.unit.to_lowercase() {
        return format!(
            "ERROR: Satuan '{}' tidak sesuai. Untuk {} / {} gunakan satuan '{}'.",
            scale_unit, sector, activity, th.unit
        );
    }

    let (dokumen, kelas, penjelasan) = if scale_value >= th.amdal {
        (
            "AMDAL (Analisis Mengenai Dampak Lingkungan)",
            "Risiko Tinggi",
            "Wajib menyusun dokumen AMDAL. Perlu Komisi Penilai AMDAL.",
        )
    } else if th.uklupl > 0.0 && scale_value >= th.uklupl {
        (
            "UKL-UPL (Upaya Pengelolaan - Upaya Pemantauan LH)",
            "Risiko Menengah",
            "Wajib menyusun dokumen UKL-UPL.",
        )
    } else {
        (
            "SPPL (Surat Pernyataan Kesanggupan Pengelolaan & Pemantauan LH)",
            "Risiko Rendah",
            "Cukup self-declare melalui OSS-RBA.",
        )
    };

    let mut out = String::from(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  AMDAL Screening Tool\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    out.push_str("Ref: PermenLHK No. 4 Tahun 2021\n\n");
    out.push_str(&format!("Sektor    : {}\n", sector));
    out.push_str(&format!("Aktivitas : {} ({})\n", activity, th.desc));
    out.push_str(&format!(
        "Skala     : {:.2} {}\n\n",
        scale_value, scale_unit
    ));
    out.push_str("Ambang Batas:\n");
    out.push_str(&format!("  AMDAL    : ≥ {:.0} {}\n", th.amdal, th.unit));
    if th.uklupl > 0.0 {
        out.push_str(&format!("  UKL-UPL  : ≥ {:.0} {}\n", th.uklupl, th.unit));
    }
    out.push_str(&format!(
        "  SPPL     : < {:.0} {}\n\n",
        if th.uklupl > 0.0 { th.uklupl } else { th.amdal },
        th.unit
    ));
    out.push_str(&format!("Kelas Risiko    : {}\n", kelas));
    out.push_str(&format!("Dokumen Wajib   : {}\n", dokumen));
    out.push_str(&format!("Penjelasan      : {}\n", penjelasan));
    out
}
