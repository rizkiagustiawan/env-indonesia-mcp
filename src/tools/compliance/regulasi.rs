/// Regulasi Lingkungan Indonesia Lookup
/// Multi-regulation reference engine

pub fn lookup(topic: &str) -> String {
    let t = topic.to_lowercase();

    struct Reg {
        nomor: &'static str,
        judul: &'static str,
        topik: &'static [&'static str],
    }

    let regs: Vec<Reg> = vec![
        Reg { nomor: "UU 32/2009", judul: "Perlindungan dan Pengelolaan Lingkungan Hidup (PPLH)", topik: &["lingkungan", "umum", "pplh", "amdal", "ukl_upl", "izin", "sanksi", "b3", "limbah"] },
        Reg { nomor: "UU 6/2023", judul: "Cipta Kerja (Perpu 2/2022 → UU)", topik: &["oss", "perizinan", "izin", "umum", "cipta_kerja", "amdal", "ukl_upl"] },
        Reg { nomor: "PP 22/2021", judul: "Penyelenggaraan Perlindungan dan Pengelolaan LH", topik: &["lingkungan", "umum", "pplh", "amdal", "ukl_upl", "b3", "limbah", "izin"] },
        Reg { nomor: "PP 22/2023", judul: "Perubahan PP 22/2021 — Perizinan Berusaha Berbasis Risiko", topik: &["oss", "perizinan", "risiko", "amdal", "ukl_upl", "izin"] },
        Reg { nomor: "PP 41/1999", judul: "Pengendalian Pencemaran Udara", topik: &["udara", "emisi", "ambien", "pencemaran_udara"] },
        Reg { nomor: "PP 82/2001", judul: "Pengelolaan Kualitas Air dan Pengendalian Pencemaran Air", topik: &["air", "sungai", "pencemaran_air", "baku_mutu_air", "limbah"] },
        Reg { nomor: "PP 101/2014", judul: "Pengelolaan Limbah Bahan Berbahaya dan Beracun", topik: &["b3", "limbah", "limbah_b3", "hazardous"] },
        Reg { nomor: "PermenLHK 4/2021", judul: "Daftar Usaha/Kegiatan yang Wajib AMDAL, UKL-UPL, atau SPPL", topik: &["amdal", "ukl_upl", "sppl", "screening", "risiko", "izin"] },
        Reg { nomor: "PermenLHK 5/2021", judul: "Tata Cara Penerbitan Persetujuan Teknis dan Surat Kelayakan Operasional Pengelolaan Limbah B3", topik: &["b3", "limbah", "limbah_b3", "izin"] },
        Reg { nomor: "PermenLHK 15/2019", judul: "Baku Mutu Emisi Pembangkit Listrik Tenaga Termal", topik: &["emisi", "udara", "pltu", "pembangkit", "cerobong"] },
        Reg { nomor: "PermenLH 5/2014", judul: "Baku Mutu Air Limbah", topik: &["air", "limbah", "air_limbah", "industri", "pencemaran_air"] },
        Reg { nomor: "PermenLHK 68/2016", judul: "Baku Mutu Air Limbah Domestik", topik: &["air", "limbah", "domestik", "air_limbah"] },
        Reg { nomor: "KepmenLH 48/1996", judul: "Baku Tingkat Kebisingan", topik: &["kebisingan", "noise", "bising"] },
        Reg { nomor: "KepmenLH 49/1996", judul: "Baku Tingkat Getaran", topik: &["getaran", "vibration", "vibrasi"] },
        Reg { nomor: "KepmenLH 50/1996", judul: "Baku Tingkat Kebauan", topik: &["kebauan", "bau", "odor"] },
        Reg { nomor: "KepmenLH 115/2003", judul: "Pedoman Penentuan Status Mutu Air (Metode STORET & Indeks Pencemaran)", topik: &["air", "storet", "indeks_pencemaran", "mutu_air"] },
        Reg { nomor: "PermenLHK 73/2019", judul: "ISPU (Indeks Standar Pencemar Udara)", topik: &["udara", "ispu", "kualitas_udara", "aqi"] },
        Reg { nomor: "PermenLHK 102/2018", judul: "Tata Cara Pelaksanaan dan Pelaporan Inventarisasi Gas Rumah Kaca Nasional", topik: &["grk", "ghg", "karbon", "emisi", "iklim", "carbon"] },
        Reg { nomor: "PermenLHK P.14/2020", judul: "IKLH (Indeks Kualitas Lingkungan Hidup)", topik: &["iklh", "ika", "iku", "iktl", "kualitas_lingkungan"] },
        Reg { nomor: "PermenLHK P.1/2021", judul: "PROPER (Program Penilaian Peringkat Kinerja Perusahaan dalam Pengelolaan LH)", topik: &["proper", "peringkat", "kinerja"] },
        Reg { nomor: "PermenLH 17/2009", judul: "Pedoman Penentuan Daya Dukung Lingkungan Hidup dalam Penataan Ruang Wilayah", topik: &["daya_dukung", "tata_ruang", "carrying_capacity"] },
        Reg { nomor: "PP 27/2012", judul: "Izin Lingkungan", topik: &["izin", "amdal", "ukl_upl", "lingkungan"] },
        Reg { nomor: "PermenLHK 22/2021", judul: "Baku Mutu Air Laut", topik: &["laut", "air_laut", "marine", "pesisir"] },
        Reg { nomor: "UU 18/2008", judul: "Pengelolaan Sampah", topik: &["sampah", "waste", "tpa", "pengelolaan_sampah"] },
        Reg { nomor: "PP 81/2012", judul: "Pengelolaan Sampah Rumah Tangga dan Sampah Sejenis", topik: &["sampah", "domestik", "rumah_tangga"] },
        Reg { nomor: "UU 41/1999", judul: "Kehutanan", topik: &["hutan", "kehutanan", "forest", "kawasan_hutan"] },
        Reg { nomor: "PP 23/2021", judul: "Penyelenggaraan Kehutanan", topik: &["hutan", "kehutanan", "forest", "tata_guna_hutan"] },
    ];

    let matches: Vec<&Reg> = regs
        .iter()
        .filter(|r| {
            r.topik.iter().any(|tp| t.contains(tp) || tp.contains(&t))
                || r.judul.to_lowercase().contains(&t)
                || r.nomor.to_lowercase().contains(&t)
        })
        .collect();

    if matches.is_empty() {
        let mut out = String::from("=== Regulasi Lingkungan Indonesia ===\n\n");
        out.push_str(&format!(
            "Topik '{}' tidak ditemukan. Topik yang tersedia:\n\n",
            topic
        ));
        out.push_str("  air, udara, limbah, b3, amdal, ukl_upl, sppl, kebisingan,\n");
        out.push_str("  getaran, kebauan, emisi, laut, sampah, hutan, karbon/ghg,\n");
        out.push_str("  ispu, storet, proper, iklh, oss, perizinan, izin, domestik\n");
        return out;
    }

    let mut out = String::from("=== Regulasi Lingkungan Indonesia ===\n");
    out.push_str(&format!("Topik: '{}'\n\n", topic));
    out.push_str(&format!(
        "Ditemukan {} regulasi terkait:\n\n",
        matches.len()
    ));

    for (i, r) in matches.iter().enumerate() {
        out.push_str(&format!("  {}. {} — {}\n", i + 1, r.nomor, r.judul));
    }

    out.push_str("\nCatatan: Daftar ini bersifat indikatif. Periksa versi terbaru di JDIH KLHK.\n");
    out
}
