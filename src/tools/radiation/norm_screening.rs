/// Skrining NORM (Naturally Occurring Radioactive Material)
/// Ref: PerKa BAPETEN No. 4/2013, IAEA Safety Guide RS-G-1.7

pub fn screen(material: &str, activity_bq_g: f64) -> String {
    if activity_bq_g < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }

    struct NormMaterial {
        name: &'static str,
        display: &'static str,
        typical_range_bq_g: (f64, f64),
        primary_nuclide: &'static str,
        source: &'static str,
    }

    let materials = [
        NormMaterial {
            name: "tin_slag",
            display: "Terak Timah (Tin Slag)",
            typical_range_bq_g: (0.1, 5.0),
            primary_nuclide: "Ra-228, Th-228",
            source: "Peleburan timah Bangka-Belitung",
        },
        NormMaterial {
            name: "monazite",
            display: "Monazit",
            typical_range_bq_g: (10.0, 200.0),
            primary_nuclide: "Th-232, U-238",
            source: "Penambangan pasir mineral",
        },
        NormMaterial {
            name: "zircon_sand",
            display: "Pasir Zirkon",
            typical_range_bq_g: (1.0, 10.0),
            primary_nuclide: "U-238, Th-232",
            source: "Penambangan pasir mineral",
        },
        NormMaterial {
            name: "coal_ash",
            display: "Abu Batubara (Fly/Bottom Ash)",
            typical_range_bq_g: (0.05, 1.0),
            primary_nuclide: "Ra-226, K-40",
            source: "PLTU batubara",
        },
        NormMaterial {
            name: "phosphogypsum",
            display: "Fosfogipsum",
            typical_range_bq_g: (0.1, 2.0),
            primary_nuclide: "Ra-226",
            source: "Industri pupuk fosfat",
        },
        NormMaterial {
            name: "bauxite_residue",
            display: "Residu Bauksit (Red Mud)",
            typical_range_bq_g: (0.1, 3.0),
            primary_nuclide: "Ra-226, Th-232",
            source: "Pengolahan aluminium",
        },
        NormMaterial {
            name: "oil_scale",
            display: "Kerak Minyak (Oil Pipe Scale)",
            typical_range_bq_g: (0.5, 50.0),
            primary_nuclide: "Ra-226, Ra-228",
            source: "Industri minyak & gas",
        },
        NormMaterial {
            name: "fertilizer",
            display: "Pupuk Fosfat",
            typical_range_bq_g: (0.01, 0.5),
            primary_nuclide: "U-238, Ra-226",
            source: "Industri pupuk",
        },
    ];

    let mat_lower = material.to_lowercase();
    let mat_info = materials.iter().find(|m| m.name == mat_lower.as_str());

    // BAPETEN clearance levels
    let clearance_general = 1.0; // Bq/g (general)
    let clearance_bulk = 10.0; // Bq/g (bulk/construction)

    let above_general = activity_bq_g > clearance_general;
    let above_bulk = activity_bq_g > clearance_bulk;

    let classification = if above_bulk {
        "MEMERLUKAN IZIN — Aktivitas melebihi tingkat pembebasan untuk material curah"
    } else if above_general {
        "PERLU EVALUASI LANJUT — Di atas clearance umum, di bawah clearance curah"
    } else {
        "DIBEBASKAN (EXEMPT) — Aktivitas di bawah tingkat pembebasan"
    };

    // Dose estimate: annual effective dose from 2000 hr/yr occupational exposure
    // Simplified: external dose ≈ activity × 0.001 mSv/yr per Bq/g (rough factor)
    // Inhalation: additional factor for dust
    let external_dose = activity_bq_g * 0.001 * 2000.0 / 2000.0; // normalized
    let inhalation_dose = activity_bq_g * 0.005; // dust inhalation contribution
    let total_annual_dose = external_dose + inhalation_dose;

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("SKRINING NORM (Naturally Occurring Radioactive Material)\n");
    result.push_str("Ref: PerKa BAPETEN No. 4/2013, IAEA RS-G-1.7\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("INPUT:\n");
    if let Some(info) = mat_info {
        result.push_str(&format!("• Material             : {}\n", info.display));
        result.push_str(&format!("• Sumber               : {}\n", info.source));
        result.push_str(&format!(
            "• Nuklida utama        : {}\n",
            info.primary_nuclide
        ));
        result.push_str(&format!(
            "• Rentang tipikal      : {:.2} – {:.2} Bq/g\n",
            info.typical_range_bq_g.0, info.typical_range_bq_g.1
        ));
    } else {
        result.push_str(&format!(
            "• Material             : {} (tidak ada data tipikal)\n",
            material
        ));
    }
    result.push_str(&format!(
        "• Aktivitas terukur    : {:.4} Bq/g\n\n",
        activity_bq_g
    ));

    result.push_str("TINGKAT PEMBEBASAN (PerKa BAPETEN No. 4/2013):\n");
    result.push_str(&format!(
        "• Umum (general)       : {:.1} Bq/g  {}\n",
        clearance_general,
        if !above_general {
            "✓ LULUS"
        } else {
            "✗ MELEBIHI"
        }
    ));
    result.push_str(&format!(
        "• Curah/konstruksi     : {:.1} Bq/g {}\n\n",
        clearance_bulk,
        if !above_bulk {
            "✓ LULUS"
        } else {
            "✗ MELEBIHI"
        }
    ));

    result.push_str(&format!("KLASIFIKASI: {}\n\n", classification));

    result.push_str("ESTIMASI DOSIS (pekerja, 2000 jam/tahun):\n");
    result.push_str(&format!(
        "• Dosis eksternal      : {:.4} mSv/tahun\n",
        external_dose
    ));
    result.push_str(&format!(
        "• Dosis inhalasi debu  : {:.4} mSv/tahun\n",
        inhalation_dose
    ));
    result.push_str(&format!(
        "• Total estimasi       : {:.4} mSv/tahun\n",
        total_annual_dose
    ));
    result.push_str(&format!(
        "• Batas pekerja        : 20 mSv/tahun {}\n\n",
        if total_annual_dose <= 20.0 {
            "✓"
        } else {
            "✗ MELEBIHI"
        }
    ));

    if above_general {
        result.push_str("TINDAKAN YANG DIPERLUKAN:\n");
        if above_bulk {
            result.push_str("• Pelaporan ke BAPETEN wajib\n");
            result.push_str("• Izin pemanfaatan zat radioaktif diperlukan\n");
            result.push_str("• Program proteksi radiasi harus diterapkan\n");
            result.push_str("• Pemantauan dosis pekerja (TLD/OSL)\n");
            result.push_str("• Pengelolaan limbah radioaktif sesuai PP 61/2013\n");
        } else {
            result.push_str("• Evaluasi dosis lebih detail diperlukan\n");
            result.push_str("• Pertimbangkan penerapan prinsip ALARA\n");
            result.push_str("• Monitoring berkala aktivitas material\n");
            result.push_str("• Konsultasi dengan Petugas Proteksi Radiasi (PPR)\n");
        }
    } else {
        result.push_str("Material dibebaskan dari pengawasan regulasi BAPETEN.\n");
        result.push_str("Dapat dikelola sebagai material non-radioaktif.\n");
    }

    result.push_str("\nMATERIAL NORM DI INDONESIA:\n");
    for m in &materials {
        result.push_str(&format!(
            "  • {:30} : {:.2}–{:.2} Bq/g ({})\n",
            m.display, m.typical_range_bq_g.0, m.typical_range_bq_g.1, m.primary_nuclide
        ));
    }
    result.push_str("══════════════════════════════════════════════\n");

    result
}
