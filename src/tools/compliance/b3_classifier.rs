/// Klasifikasi Limbah B3 (Bahan Berbahaya & Beracun)
/// Ref: PP 101/2014 jo. PP 22/2021, Lampiran I-IX

pub fn classify(waste_type: &str) -> String {
    let w = waste_type.to_lowercase();
    let mut out = String::from("=== Klasifikasi Limbah B3 ===\n");
    out.push_str("Ref: PP 101/2014 jo. PP 22/2021, UU 32/2009\n\n");

    let (kategori, kode, penjelasan, penanganan) = if w.contains("aki") || w.contains("baterai") || w.contains("battery") {
        ("KATEGORI 1 — B3 AKUT", "A101d", "Baterai/aki bekas mengandung Pb (timbal) dan H2SO4.", "Wajib dikumpulkan oleh pengumpul berizin. Dilarang dibuang ke TPA. Manifes B3 wajib.")
    } else if w.contains("oli") || w.contains("pelumas") || w.contains("lubricant") {
        ("KATEGORI 1 — B3", "B105d", "Oli/pelumas bekas mengandung logam berat dan PAH.", "Wajib diolah di pengolah berizin (re-refining/insinerasi). Penyimpanan max 90 hari.")
    } else if w.contains("pestisida") || w.contains("insektisida") || w.contains("herbisida") {
        ("KATEGORI 1 — B3 AKUT", "A103d", "Pestisida kadaluarsa atau sisa kemasan.", "Wajib insinerasi pada suhu > 1200°C. Sangat berbahaya bagi ekosistem akuatik.")
    } else if w.contains("merkuri") || w.contains("mercury") || w.contains("hg") {
        ("KATEGORI 1 — B3 AKUT", "A104d", "Limbah mengandung merkuri (Hg). Sangat toksik dan bioakumulatif.", "Stabilisasi/solidifikasi + penimbunan akhir di TPS LB3. Minamata Convention.")
    } else if w.contains("medis") || w.contains("rumah sakit") || w.contains("medical") {
        ("KATEGORI 1 — B3 INFEKSIUS", "A337-1", "Limbah medis infeksius (jarum, darah, organ, obat kadaluarsa).", "Wajib autoklaf/insinerasi. Dilarang dicampur sampah domestik. PermenLHK 56/2015.")
    } else if w.contains("sludge") || w.contains("lumpur") {
        ("KATEGORI 2 — B3", "B106d", "Lumpur IPAL mengandung logam berat dan zat organik persisten.", "Uji TCLP wajib. Jika lolos → non-B3. Jika tidak → pengolahan khusus.")
    } else if w.contains("abu") || w.contains("fly ash") || w.contains("bottom ash") {
        ("KATEGORI 2 — B3 (dengan pengecualian)", "B409", "Fly ash/bottom ash PLTU batubara.", "PP 22/2021 pasal khusus: bisa dimanfaatkan untuk bahan bangunan jika lolos uji TCLP (PermenLHK P.8/2021).")
    } else if w.contains("elektronik") || w.contains("e-waste") || w.contains("pcb") {
        ("KATEGORI 1 — B3", "A108d", "Limbah elektronik (PCB, CRT, chip) mengandung Pb, Hg, Cd, BFR.", "Wajib dikumpulkan pengumpul berizin. Extended Producer Responsibility (EPR).")
    } else {
        ("BELUM TERKLASIFIKASI", "—", "Jenis limbah tidak dikenali dalam database. Lakukan uji karakteristik (mudah meledak, mudah menyala, reaktif, infeksius, korosif, toksik) sesuai PP 101/2014 Lampiran III.", "Hubungi Dinas LHK setempat untuk uji laboratorium TCLP, LD50, dan karakteristik B3.")
    };

    out.push_str(&format!("Jenis Limbah: {}\n\n", waste_type));
    out.push_str(&format!("Klasifikasi: {}\n", kategori));
    out.push_str(&format!("Kode Limbah: {}\n", kode));
    out.push_str(&format!("Penjelasan: {}\n", penjelasan));
    out.push_str(&format!("Penanganan: {}\n\n", penanganan));
    out.push_str("Karakteristik B3 (PP 101/2014):\n");
    out.push_str("  1. Mudah meledak (explosive)\n  2. Mudah menyala (flammable)\n  3. Reaktif (reactive)\n  4. Infeksius (infectious)\n  5. Korosif (corrosive, pH<2 atau >12.5)\n  6. Beracun (toxic, uji TCLP/LD50)\n");
    out
}
