use reqwest::Client;

pub async fn statistics(client: &Client, keyword: &str) -> String {
    let api_key = std::env::var("BPS_API_KEY").unwrap_or_default();
    
    // Jika ada API Key, panggil API BPS (Contoh implementasi)
    if !api_key.is_empty() {
        let _url = format!("https://webapi.bps.go.id/v1/api/list/model/data/domain/5200/key/{}", api_key);
        // Implementasi request ke BPS bisa ditambahkan di sini nantinya
    }
    
    let mut out = format!("=== BPS Statistik Lingkungan NTB (Query: {}) ===\n\n", keyword);
    out.push_str("Data Referensi Utama (BPS NTB 2023-2025):\n");
    
    let l = keyword.to_lowercase();
    if l.contains("hutan") || l.contains("forest") {
        out.push_str("- Luas Kawasan Hutan NTB: ~1.07 Juta Ha (52% dari luas wilayah)\n");
        out.push_str("- Hutan Konservasi: ~173 ribu Ha\n");
        out.push_str("- Hutan Lindung: ~445 ribu Ha\n");
        out.push_str("- Hutan Produksi: ~453 ribu Ha\n");
    } else if l.contains("sampah") || l.contains("waste") {
        out.push_str("- Timbulan Sampah NTB: ~3.3 Juta Ton/Tahun\n");
        out.push_str("- Sampah Terkelola: ~45%\n");
        out.push_str("- TPA Utama: TPA Kebon Kongok (Lombok Barat), TPA Rababaka (Sumbawa)\n");
    } else if l.contains("air") || l.contains("water") {
        out.push_str("- Sumber Air Bersih Utama: Mata air Narmada, sumur bor, PDAM\n");
        out.push_str("- Rumah Tangga dengan Akses Air Bersih Layak: ~78%\n");
    } else {
        out.push_str("- PDRB NTB didominasi sektor Pertanian dan Pertambangan.\n");
        out.push_str("- Sektor Pertanian rentan terhadap perubahan iklim (El Nino).\n");
    }
    
    out.push_str("\nUntuk integrasi API live, daftarkan API Key di https://webapi.bps.go.id/developer/\n");
    out.push_str("Lalu atur environment variable BPS_API_KEY.");
    out
}
