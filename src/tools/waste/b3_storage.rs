/// Persyaratan TPS Limbah B3 (Tempat Penyimpanan Sementara)
/// Ref: PP 101/2014 tentang Pengelolaan Limbah B3

pub fn calculate(waste_type: &str, volume_m3_per_month: f64, density_kg_m3: f64) -> String {
    if volume_m3_per_month <= 0.0 { return "ERROR: Volume limbah per bulan harus > 0.".into(); }
    if density_kg_m3 <= 0.0 { return "ERROR: Densitas limbah harus > 0.".into(); }

    let wt_lower = waste_type.to_lowercase();

    let (type_name, max_storage_days_cat1, max_storage_days_cat2, stack_height_m, container_desc)
        = match wt_lower.as_str() {
        "padat" => (
            "Padat",
            90, 180,
            3.0,
            "Drum 200L (standar) atau kontainer tertutup",
        ),
        "cair" => (
            "Cair",
            90, 180,
            2.0, // single stack drums
            "Drum 200L HDPE/baja atau tangki IBC 1000L",
        ),
        "lumpur" => (
            "Lumpur (Sludge)",
            90, 180,
            2.0,
            "Drum 200L tertutup atau bak penampung berlapis",
        ),
        "gas" => (
            "Gas (Tabung Bertekanan)",
            90, 180,
            1.5, // single row
            "Tabung gas bertekanan standar DOT/SNI",
        ),
        _ => {
            return format!(
                "ERROR: Jenis limbah '{}' tidak dikenal.\nPilihan: padat, cair, lumpur, gas",
                waste_type
            );
        }
    };

    let mass_kg_per_month = volume_m3_per_month * density_kg_m3;
    let mass_ton_per_month = mass_kg_per_month / 1000.0;

    // Storage volume for max duration (kategori 2 = 180 days = 6 months)
    let max_stored_volume_m3 = volume_m3_per_month * (max_storage_days_cat2 as f64 / 30.0);
    let floor_area_m2 = max_stored_volume_m3 / stack_height_m;

    // Aisle factor: 40% additional for access
    let floor_area_with_aisle = floor_area_m2 * 1.4;

    // Containment: 110% of largest single container or 25% of total, whichever greater
    let drum_volume_m3 = 0.2; // 200L drum
    let containment_110: f64 = drum_volume_m3 * 1.1;
    let containment_25pct: f64 = max_stored_volume_m3 * 0.25;
    let containment_volume = containment_110.max(containment_25pct);

    // Number of drums (200L)
    let n_drums = (max_stored_volume_m3 / drum_volume_m3).ceil() as u64;

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("PERSYARATAN TPS LIMBAH B3\n");
    result.push_str("Ref: PP 101/2014 tentang Pengelolaan Limbah B3\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("INPUT:\n");
    result.push_str(&format!("• Jenis limbah         : {}\n", type_name));
    result.push_str(&format!("• Volume per bulan     : {:.2} m³/bulan\n", volume_m3_per_month));
    result.push_str(&format!("• Densitas             : {:.0} kg/m³\n", density_kg_m3));
    result.push_str(&format!("• Massa per bulan      : {:.2} ton/bulan\n\n", mass_ton_per_month));

    result.push_str("BATAS WAKTU PENYIMPANAN (PP 101/2014):\n");
    result.push_str(&format!("• Kategori 1 (akut/reaktif)     : {} hari\n", max_storage_days_cat1));
    result.push_str(&format!("• Kategori 2 (kronis/umum)      : {} hari\n\n", max_storage_days_cat2));

    result.push_str("KEBUTUHAN RUANG (maks {} hari):\n");
    result.push_str(&format!("• Volume tersimpan maks         : {:.2} m³\n", max_stored_volume_m3));
    result.push_str(&format!("• Kontainer                     : {}\n", container_desc));
    result.push_str(&format!("• Jumlah drum (200L) estimasi   : {} drum\n", n_drums));
    result.push_str(&format!("• Tinggi tumpukan maks          : {:.1} m\n", stack_height_m));
    result.push_str(&format!("• Luas lantai (netto)           : {:.1} m²\n", floor_area_m2));
    result.push_str(&format!("• Luas lantai (+ aisle 40%)     : {:.1} m²\n\n", floor_area_with_aisle));

    result.push_str("SISTEM PENAHAN TUMPAHAN (BUND):\n");
    result.push_str(&format!("• Volume bund minimum           : {:.2} m³\n", containment_volume));
    result.push_str("• Kriteria: 110% kontainer terbesar ATAU 25% total volume\n\n");

    result.push_str("PERSYARATAN KONSTRUKSI TPS B3:\n");
    result.push_str("• Lantai kedap (beton K-250 + coating epoxy)\n");
    result.push_str("• Atap penutup (melindungi dari hujan)\n");
    result.push_str("• Ventilasi memadai (min 6 ACH untuk cair/gas)\n");
    result.push_str("• Sistem drainase internal → bak penampung tumpahan\n");
    result.push_str("• Tidak terhubung ke saluran umum/lingkungan\n\n");

    result.push_str("PERSYARATAN KESELAMATAN:\n");
    result.push_str("• APAR sesuai jenis limbah (ABC, CO₂, foam)\n");
    result.push_str("• Spill kit dan absorben\n");
    result.push_str("• APD: sarung tangan, kacamata, masker, sepatu safety\n");
    result.push_str("• Shower darurat dan eyewash station\n");
    result.push_str("• SDS (Safety Data Sheet) untuk setiap jenis limbah\n\n");

    result.push_str("PERSYARATAN PELABELAN:\n");
    result.push_str("• Simbol B3 pada setiap kontainer\n");
    result.push_str("• Label: nama limbah, sumber, tanggal masuk, karakteristik\n");
    result.push_str("• Papan informasi darurat di pintu masuk\n");
    result.push_str("• Denah penyimpanan dan rute evakuasi\n\n");

    result.push_str("PELAPORAN:\n");
    result.push_str("• Neraca limbah B3 triwulanan ke DLHK\n");
    result.push_str("• Manifest limbah B3 setiap pengiriman\n");
    result.push_str("• Log book penerimaan dan pengeluaran harian\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}
