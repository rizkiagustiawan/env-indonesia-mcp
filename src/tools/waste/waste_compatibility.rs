/// Matriks Kompatibilitas Limbah B3
/// Ref: PP 101/2014, EPA Waste Compatibility Chart

pub fn check(waste_a: &str, waste_b: &str) -> String {
    let categories = [
        "acid",
        "base",
        "oxidizer",
        "reducer",
        "water_reactive",
        "flammable",
        "organic_solvent",
        "heavy_metal",
        "cyanide",
    ];

    let a_lower = waste_a.to_lowercase();
    let b_lower = waste_b.to_lowercase();

    let a_idx = categories.iter().position(|c| *c == a_lower.as_str());
    let b_idx = categories.iter().position(|c| *c == b_lower.as_str());

    if a_idx.is_none() {
        return format!(
            "ERROR: Kategori limbah '{}' tidak dikenal.\nKategori tersedia: {}",
            waste_a,
            categories.join(", ")
        );
    }
    if b_idx.is_none() {
        return format!(
            "ERROR: Kategori limbah '{}' tidak dikenal.\nKategori tersedia: {}",
            waste_b,
            categories.join(", ")
        );
    }

    let a = a_idx.unwrap();
    let b = b_idx.unwrap();

    // Compatibility matrix (symmetric)
    // C = compatible, H = incompatible_heat, T = incompatible_toxic_gas,
    // F = incompatible_fire, E = incompatible_explosion
    //                       acid  base  oxid  redu  watr  flam  org_s  h_met cyan
    let matrix: [[char; 9]; 9] = [
        ['C', 'H', 'H', 'H', 'H', 'C', 'C', 'T', 'T'], // acid
        ['H', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C'], // base
        ['H', 'C', 'C', 'F', 'C', 'F', 'F', 'C', 'E'], // oxidizer
        ['H', 'C', 'F', 'C', 'F', 'F', 'C', 'C', 'C'], // reducer
        ['H', 'C', 'C', 'F', 'C', 'F', 'C', 'C', 'C'], // water_reactive
        ['C', 'C', 'F', 'F', 'F', 'C', 'C', 'C', 'C'], // flammable
        ['C', 'C', 'F', 'C', 'C', 'C', 'C', 'C', 'C'], // organic_solvent
        ['T', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'T'], // heavy_metal
        ['T', 'C', 'E', 'C', 'C', 'C', 'C', 'T', 'C'], // cyanide
    ];

    let code = matrix[a][b];

    let (status, status_id, reaction, safety) = match code {
        'C' => (
            "KOMPATIBEL",
            "Aman untuk disimpan berdekatan",
            "Tidak ada reaksi berbahaya yang diperkirakan.",
            "Penyimpanan standar sesuai PP 101/2014.",
        ),
        'H' => (
            "TIDAK KOMPATIBEL — REAKSI EKSOTERMIK",
            "Menghasilkan panas berlebih",
            "Reaksi netralisasi atau eksotermik; pelepasan panas signifikan.\nProduk: panas, uap air, garam.",
            "Pisahkan penyimpanan min 5 m atau dengan dinding tahan api.\nSediakan alat pemadam kebakaran dan sistem pendingin.",
        ),
        'T' => (
            "TIDAK KOMPATIBEL — GAS BERACUN",
            "Menghasilkan gas toksik",
            "Reaksi menghasilkan gas beracun (contoh: HCN dari asam+sianida,\nH₂S dari asam+sulfida, gas logam berat).",
            "DILARANG disimpan dalam ruangan yang sama.\nPisahkan dengan dinding kedap gas.\nSediakan ventilasi dan detektor gas.\nAPD lengkap (SCBA) wajib saat penanganan.",
        ),
        'F' => (
            "TIDAK KOMPATIBEL — RISIKO KEBAKARAN",
            "Risiko kebakaran / nyala api",
            "Reaksi dapat menghasilkan panas cukup untuk menyalakan bahan mudah terbakar.\nOksidator + bahan organik = risiko kebakaran tinggi.",
            "DILARANG disimpan berdekatan.\nPisahkan dengan dinding tahan api min 2 jam.\nSediakan sprinkler otomatis dan APAR kelas B.\nLarangan merokok dan sumber api.",
        ),
        'E' => (
            "TIDAK KOMPATIBEL — RISIKO LEDAKAN",
            "Risiko ledakan",
            "Reaksi keras, berpotensi meledak.\nContoh: oksidator kuat + sianida = reaksi ledakan.",
            "DILARANG disimpan dalam satu area.\nJarak pisah minimum 20 m atau bangunan terpisah.\nNotifikasi DAMKAR dan protokol darurat wajib.\nPenanganan hanya oleh personel terlatih.",
        ),
        _ => (
            "TIDAK DIKETAHUI",
            "Status tidak dapat ditentukan",
            "Data kompatibilitas tidak tersedia.",
            "Lakukan uji kompatibilitas laboratorium sebelum penyimpanan.",
        ),
    };

    fn category_id(cat: &str) -> &'static str {
        match cat {
            "acid" => "Asam",
            "base" => "Basa",
            "oxidizer" => "Oksidator",
            "reducer" => "Reduktor",
            "water_reactive" => "Reaktif Air",
            "flammable" => "Mudah Terbakar",
            "organic_solvent" => "Pelarut Organik",
            "heavy_metal" => "Logam Berat",
            "cyanide" => "Sianida",
            _ => "Lainnya",
        }
    }

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("MATRIKS KOMPATIBILITAS LIMBAH B3\n");
    result.push_str("Ref: PP 101/2014, EPA Waste Compatibility\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str(&format!(
        "Limbah A : {} ({})\n",
        category_id(&a_lower),
        waste_a
    ));
    result.push_str(&format!(
        "Limbah B : {} ({})\n\n",
        category_id(&b_lower),
        waste_b
    ));

    result.push_str(&format!("STATUS: {}\n", status));
    result.push_str(&format!("        {}\n\n", status_id));

    result.push_str("REAKSI:\n");
    result.push_str(&format!("{}\n\n", reaction));

    result.push_str("TINDAKAN KESELAMATAN:\n");
    result.push_str(&format!("{}\n", safety));

    result.push_str("\nLEGENDA MATRIKS:\n");
    result.push_str("  C = Kompatibel (aman berdekatan)\n");
    result.push_str("  H = Reaksi eksotermik (panas)\n");
    result.push_str("  T = Gas beracun\n");
    result.push_str("  F = Risiko kebakaran\n");
    result.push_str("  E = Risiko ledakan\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}
