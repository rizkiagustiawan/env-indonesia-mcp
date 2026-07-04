/// Kelas Risiko Lingkungan (OSS)
/// Ref: PP 22/2023, PermenLHK 4/2021

pub fn determine(sector: &str, scale_description: &str, has_hazardous_waste: bool, near_protected_area: bool) -> String {
    let s = sector.to_lowercase();
    let scale = scale_description.to_lowercase();

    // Base risk from sector
    let sector_risk: i32 = match s.as_str() {
        "pertambangan" | "mining" => 4,
        "migas" | "oil_gas" => 4,
        "smelter" | "peleburan" => 4,
        "pltu" | "pembangkit_listrik" => 4,
        "kimia" | "chemical" | "petrokimia" => 4,
        "pulp_kertas" | "pulp" => 3,
        "semen" | "cement" => 3,
        "tekstil" | "textile" => 3,
        "sawit" | "kelapa_sawit" => 3,
        "farmasi" | "pharmaceutical" => 3,
        "rumah_sakit" | "hospital" => 3,
        "electroplating" => 3,
        "konstruksi" | "construction" => 2,
        "pariwisata" | "tourism" => 2,
        "hotel" | "penginapan" => 2,
        "peternakan" | "livestock" => 2,
        "pertanian" | "agriculture" => 2,
        "perikanan" | "fishery" => 2,
        "pergudangan" | "warehouse" => 1,
        "perdagangan" | "retail" | "toko" => 1,
        "jasa" | "services" => 1,
        "umkm" | "mikro" | "kecil" => 1,
        _ => 2, // default menengah
    };

    // Scale modifier
    let scale_mod: i32 = if scale.contains("besar") || scale.contains("large") || scale.contains(">") {
        1
    } else if scale.contains("kecil") || scale.contains("small") || scale.contains("mikro") {
        -1
    } else {
        0 // menengah/medium
    };

    // B3 and protected area modifiers
    let b3_mod = if has_hazardous_waste { 1 } else { 0 };
    let prot_mod = if near_protected_area { 1 } else { 0 };

    let total_risk = (sector_risk + scale_mod + b3_mod + prot_mod).clamp(1, 4);

    let (kelas, dokumen, deskripsi) = match total_risk {
        1 => (
            "Risiko Rendah",
            "SPPL (Surat Pernyataan Kesanggupan Pengelolaan dan Pemantauan LH)",
            "Usaha dengan dampak lingkungan rendah. Cukup self-declare melalui OSS.",
        ),
        2 => (
            "Risiko Menengah-Rendah",
            "UKL-UPL Standar",
            "Usaha dengan dampak lingkungan menengah-rendah. Mengisi formulir UKL-UPL standar melalui OSS.",
        ),
        3 => (
            "Risiko Menengah-Tinggi",
            "UKL-UPL (Upaya Pengelolaan Lingkungan - Upaya Pemantauan Lingkungan)",
            "Usaha dengan dampak lingkungan menengah-tinggi. Wajib menyusun dokumen UKL-UPL.",
        ),
        _ => (
            "Risiko Tinggi",
            "AMDAL (Analisis Mengenai Dampak Lingkungan)",
            "Usaha dengan dampak penting terhadap lingkungan. Wajib AMDAL + Komisi Penilai AMDAL.",
        ),
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Kelas Risiko Lingkungan (OSS)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PP 22/2023, PermenLHK No. 4 Tahun 2021\n\n");
    out.push_str(&format!("Input:\n"));
    out.push_str(&format!("  Sektor            : {}\n", sector));
    out.push_str(&format!("  Skala             : {}\n", scale_description));
    out.push_str(&format!("  Limbah B3         : {}\n", if has_hazardous_waste { "Ya" } else { "Tidak" }));
    out.push_str(&format!("  Dekat Kawasan Lindung: {}\n\n", if near_protected_area { "Ya" } else { "Tidak" }));
    out.push_str(&format!("Skor Risiko: {} / 4\n", total_risk));
    out.push_str(&format!("  Sektor: {} | Skala: {:+} | B3: {:+} | Kawasan Lindung: {:+}\n\n", sector_risk, scale_mod, b3_mod, prot_mod));
    out.push_str(&format!("Kelas: {}\n", kelas));
    out.push_str(&format!("Dokumen Lingkungan: {}\n", dokumen));
    out.push_str(&format!("Penjelasan: {}\n\n", deskripsi));
    out.push_str("Klasifikasi Risiko:\n");
    out.push_str("  Risiko Rendah           → SPPL\n");
    out.push_str("  Risiko Menengah-Rendah  → UKL-UPL Standar\n");
    out.push_str("  Risiko Menengah-Tinggi  → UKL-UPL\n");
    out.push_str("  Risiko Tinggi           → AMDAL\n");
    out
}
