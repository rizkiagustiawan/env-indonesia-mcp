use reqwest::Client;

pub async fn query(client: &Client) -> String {
    let mut out = String::from("=== CHIRPS (Climate Hazards Group InfraRed Precipitation with Station data) ===\n\n");
    out.push_str("Produk: Curah hujan harian/dekadal/bulanan\n");
    out.push_str("Resolusi Spasial: 0.05° (~5 km)\n");
    out.push_str("Rentang Waktu: 1981 - Sekarang\n\n");
    
    out.push_str("Aplikasi untuk Lingkungan NTB:\n");
    out.push_str("- Pemantauan kekeringan (Standardized Precipitation Index / SPI).\n");
    out.push_str("- Identifikasi anomali curah hujan selama event El Nino (kering di Sumbawa) dan La Nina (banjir di Lombok).\n");
    out.push_str("- Integrasi dengan data Groundwater Storage (GWS).\n\n");
    
    out.push_str("Integrasi GEE:\n");
    out.push_str("ee.ImageCollection('UCSB-CHG/CHIRPS/DAILY')\n");
    out
}
