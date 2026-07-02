use reqwest::Client;

pub async fn query(client: &Client) -> String {
    let mut out = String::from("=== ERA5 Climate Reanalysis (ECMWF) ===\n\n");
    out.push_str("Source: Copernicus Climate Change Service (C3S)\n");
    out.push_str("Resolusi: ~31 km (0.25°)\n");
    out.push_str("Rentang Waktu: 1940 - Sekarang (Hourly data)\n\n");
    
    out.push_str("Parameter Lingkungan Kunci untuk NTB:\n");
    out.push_str("- 2m Temperature (Tren pemanasan global lokal)\n");
    out.push_str("- Total Precipitation (Pola curah hujan ekstrem)\n");
    out.push_str("- 10m U & V Wind Components (Potensi energi angin darat/lepas pantai)\n");
    out.push_str("- Surface Solar Radiation Downwards (Potensi energi surya PLTS)\n");
    out.push_str("- Volumetric Soil Water (Kekeringan agrikultur)\n\n");
    
    out.push_str("Kelebihan:\n");
    out.push_str("Satu-satunya dataset global komprehensif yang bisa menunjukkan tren iklim jangka panjang (climate change trend) secara fisik konsisten di wilayah NTB tanpa terputus.\n");
    out
}
