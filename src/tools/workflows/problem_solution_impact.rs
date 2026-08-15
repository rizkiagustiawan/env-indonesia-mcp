/// Problem-Solution-Impact Orchestrator (SOP Workflow Generator)
/// Problem: Menghitung dampak polusi/bencana lingkungan dari nol sangat rawan halusinasi.
/// Method: Tool ini adalah META-ORCHESTRATOR. Tidak melakukan kalkulasi matematis. 
///         Berfungsi memberikan Standard Operating Procedure (SOP) langkah-demi-langkah
///         kepada LLM Agent tentang alat (MCP tool) mana saja yang harus dipanggil 
///         dan bagaimana merangkai datanya secara saintifik (Pro-Justitia).
///
/// Problem types: flood, fire, pollution_river, pollution_air, coastal_erosion, mining_impact.

pub fn orchestrate(
    problem_type: &str,
    location_name: &str,
    lat: f64,
    lon: f64,
    area_ha: f64,
    severity: &str,
) -> String {
    let mut out = String::new();
    out.push_str("===============================================================\n");
    out.push_str("  ENVIRONMENTAL INVESTIGATION SOP (Orchestrator Workflow)\n");
    out.push_str("===============================================================\n");
    out.push_str("PERHATIAN KEPADA AI AGENT:\n");
    out.push_str("JANGAN MENGARANG ANGKA! Ikuti langkah-langkah saintifik di bawah ini\n");
    out.push_str("dengan memanggil tool MCP yang disarankan secara berurutan.\n\n");

    out.push_str("Konteks Investigasi:\n");
    out.push_str(&format!("  Isu Lingkungan : {}\n", problem_type.to_uppercase()));
    out.push_str(&format!("  Lokasi         : {} ({:.4}, {:.4})\n", location_name, lat, lon));
    out.push_str(&format!("  Luas Area      : {:.1} ha\n", area_ha));
    out.push_str(&format!("  Severity (User): {}\n\n", severity));

    match problem_type.to_lowercase().as_str() {
        "flood" => flood_workflow(&mut out),
        "fire" => fire_workflow(&mut out),
        "pollution_river" => river_workflow(&mut out),
        "pollution_air" => air_workflow(&mut out),
        "coastal_erosion" => erosion_workflow(&mut out),
        "mining_impact" => mining_workflow(&mut out),
        other => {
            return format!(
                "ERROR: Isu '{}' belum memiliki SOP. Didukung: flood, fire, pollution_river, pollution_air, coastal_erosion, mining_impact.",
                other
            )
        }
    }

    out.push_str("===============================================================\n");
    out.push_str("PENTING: Gunakan output JSON dari tool sebelumnya sebagai input untuk tool selanjutnya.\n");
    out
}

fn flood_workflow(out: &mut String) {
    out.push_str("PHASE 1 - DIAGNOSIS (BANJIR)\n");
    out.push_str("  1. Panggil `bmkg_weather` untuk mengecek curah hujan (P) historis/prediksi di lokasi.\n");
    out.push_str("  2. Panggil `scs_cn` (Kurva Nomor SCS) untuk mengubah hujan (P) menjadi volume Runoff (Q) berdasarkan tutupan lahan.\n");
    out.push_str("  3. Jika ini daerah pesisir, panggil `tidal_flood_compound` untuk mengecek kombinasi pasang surut.\n\n");

    out.push_str("PHASE 2 - SOLUTION (HIDROLIKA)\n");
    out.push_str("  1. Panggil `swe_flood` (Shallow Water Equation 2D) dengan input debit (Q) dari Phase 1 untuk memetakan kedalaman & kecepatan genangan.\n");
    out.push_str("  2. Evaluasi luasan area menggunakan `osm_poi_query` untuk mencari infrastruktur kritis yang terendam.\n\n");

    out.push_str("PHASE 3 - IMPACT (VALUASI)\n");
    out.push_str("  1. Hitung kerugian aset berdasarkan kedalaman banjir.\n");
    out.push_str("  2. Panggil `restoration_cost` jika ada kerusakan infrastruktur tata air alami.\n");
}

fn fire_workflow(out: &mut String) {
    out.push_str("PHASE 1 - DIAGNOSIS (KARHUTLA & GAMBUT)\n");
    out.push_str("  1. Panggil `sipongi_fire` atau `firms_fire` untuk mendeteksi koordinat titik api (hotspot) aktual.\n");
    out.push_str("  2. Panggil `fire_danger_rating` (KBDI) menggunakan data cuaca BMKG untuk mengevaluasi kekeringan gambut.\n\n");

    out.push_str("PHASE 2 - SOLUTION (PEMADAMAN & SEBARAN)\n");
    out.push_str("  1. Panggil `fire_spread` (Rothermel CA) untuk memprediksi arah rambatan api 24 jam ke depan.\n");
    out.push_str("  2. Jika asap sangat tebal, panggil `haze_trajectory` untuk memprediksi sebaran asap lintas-batas (transboundary).\n\n");

    out.push_str("PHASE 3 - IMPACT (EMISI & KESEHATAN)\n");
    out.push_str("  1. Panggil `peat_co2` atau `forest_carbon` untuk menghitung tonase emisi Gas Rumah Kaca (GRK) akibat kebakaran.\n");
    out.push_str("  2. Panggil `health_impact_assessment` menggunakan estimasi PM2.5 dari `haze_trajectory` untuk menakar risiko ISPA.\n");
}

fn river_workflow(out: &mut String) {
    out.push_str("PHASE 1 - DIAGNOSIS (PENCEMARAN SUNGAI)\n");
    out.push_str("  1. Ambil data kualitas air (BOD, DO) dan debit dari lapangan (atau default konservatif).\n");
    out.push_str("  2. Panggil `baku_mutu_air_permukaan` untuk verifikasi legalitas Kelas Sungai (PP 22/2021).\n");
    out.push_str("  3. Jika limbah komposit, panggil `river_source_apportionment`.\n\n");

    out.push_str("PHASE 2 - SOLUTION (DAYA TAMPUNG & IPAL)\n");
    out.push_str("  1. Panggil `streeter_phelps` untuk mencari titik defisit Oksigen Kritis (DO Sag Curve).\n");
    out.push_str("  2. Panggil `daya_tampung` untuk menghitung kuota sisa pembuangan limbah (Mass Balance).\n");
    out.push_str("  3. Rekomendasikan teknologi IPAL (misal panggil `activated_sludge_design` atau `uasb_design`) untuk mencapai baku mutu efluen.\n\n");

    out.push_str("PHASE 3 - IMPACT (EKONOMI LINGKUNGAN)\n");
    out.push_str("  1. Panggil `sanksi_administratif` jika terjadi pelanggaran baku mutu efluen.\n");
    out.push_str("  2. Panggil `externality_cost` untuk menghitung kerugian sosial akibat sungai yang tercemar.\n");
}

fn air_workflow(out: &mut String) {
    out.push_str("PHASE 1 - DIAGNOSIS (POLUSI UDARA)\n");
    out.push_str("  1. Panggil `ispu` menggunakan data PM2.5/PM10/SO2/CO (misal dari sensor lapangan atau `waqi_air_quality`).\n");
    out.push_str("  2. Panggil `pm_source_apportionment` (CMB) untuk mengetahui apakah sumber dominan adalah PLTU, Kendaraan, atau Pembakaran.\n\n");

    out.push_str("PHASE 2 - SOLUTION (REGULATORY MODELING & APC)\n");
    out.push_str("  1. Jika PLTU/Industri: Panggil `aermod_generator` untuk menyusun file .inp guna simulasi dispersi tingkat lanjut (Tier-3/Pro-Justitia).\n");
    out.push_str("  2. Panggil `baku_mutu_emisi` untuk mengecek apakah konsentrasi di cerobong (stack) melanggar aturan.\n");
    out.push_str("  3. Rancang alat pengendali polusi dengan memanggil `scrubber`, `baghouse`, atau `esp`.\n\n");

    out.push_str("PHASE 3 - IMPACT (KESEHATAN PUBLIK)\n");
    out.push_str("  1. Panggil `health_impact_assessment` dengan parameter PM2.5 ambien dan jumlah populasi terpapar untuk menghitung Nilai Ekonomi Kematian Atributif (DALYs).\n");
}

fn erosion_workflow(out: &mut String) {
    out.push_str("PHASE 1 - DIAGNOSIS (ABRASI & SUBSIDENSI PESISIR)\n");
    out.push_str("  1. Panggil `jakarta_coastal_risk` (atau lokasi terkait) dengan data Laju Penurunan Tanah (Subsidence) dan Elevasi.\n");
    out.push_str("  2. Panggil `coastal_erosion` (Model Bruun & CERC) dengan data tinggi gelombang dan kenaikan muka air laut (SLR).\n\n");

    out.push_str("PHASE 2 - SOLUTION (MITIGASI PESISIR)\n");
    out.push_str("  1. Evaluasi apakah Sabuk Hijau (Mangrove) cukup, atau butuh tanggul laut (Seawall).\n");
    out.push_str("  2. Panggil `mangrove` untuk mendesain ketebalan hutan mangrove peredam gelombang.\n\n");

    out.push_str("PHASE 3 - IMPACT (KERUGIAN & RESTORASI)\n");
    out.push_str("  1. Hitung luasan daratan yang hilang (dari output coastal_erosion).\n");
    out.push_str("  2. Panggil `restoration_cost` untuk menghitung biaya perbaikan pesisir per hektar.\n");
}

fn mining_workflow(out: &mut String) {
    out.push_str("PHASE 1 - DIAGNOSIS (DAMPAK TAMBANG & LIMBAH B3)\n");
    out.push_str("  1. Panggil `mine_impact` untuk mendapatkan matriks risiko (Skrining Awal).\n");
    out.push_str("  2. Jika tambang Tembaga/Emas/Batu Bara: Panggil `acid_mine_drainage` (AMD) untuk potensi Air Asam Tambang.\n");
    out.push_str("  3. Jika tambang Nikel (Laterit): Panggil `phreeqc_leaching` untuk membuat model termodinamika pelindian Cr6+/Ni di tanah.\n\n");

    out.push_str("PHASE 2 - SOLUTION (REKLAMASI & TAILING)\n");
    out.push_str("  1. Panggil `tailings_management` untuk mengevaluasi opsi pembuangan (Dry Stacking vs TSF).\n");
    out.push_str("  2. JIKA rencana menggunakan laut (DSTP): Panggil `dstp_plume_dispersion` untuk memastikan area mati zona benthik.\n");
    out.push_str("  3. Panggil `mine_reclamation` untuk perancangan revegetasi lahan pasca-tambang.\n\n");

    out.push_str("PHASE 3 - IMPACT (BIAYA LINGKUNGAN)\n");
    out.push_str("  1. Panggil `restoration_cost` untuk estimasi reklamasi tambang per hektar.\n");
    out.push_str("  2. Jika ada logam berat bocor, panggil `heavy_metal_risk` (HHRA) untuk risiko karsinogenik warga sekitar.\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_problem_types_run() {
        let problems = vec![
            "flood",
            "fire",
            "pollution_river",
            "pollution_air",
            "coastal_erosion",
            "mining_impact",
        ];

        for prob in problems {
            let res = orchestrate(prob, "Jakarta", -6.2, 106.8, 100.0, "high");
            assert!(!res.contains("ERROR"), "Workflow {} failed: {}", prob, res);
            assert!(res.contains("PHASE 1"), "Missing phase 1 in {}", prob);
            assert!(res.contains("PHASE 2"), "Missing phase 2 in {}", prob);
            assert!(res.contains("PHASE 3"), "Missing phase 3 in {}", prob);
        }
    }
}
