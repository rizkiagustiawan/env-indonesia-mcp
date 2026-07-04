use reqwest::Client;

pub async fn query(_client: &Client) -> String {
    let mut out = String::from("=== Google Dynamic World (Near Real-Time Land Cover) ===\n\n");
    out.push_str("Source: Google / World Resources Institute (WRI)\n");
    out.push_str("Resolusi: 10m (derived dari Sentinel-2)\n\n");
    
    out.push_str("Kelas Tutupan Lahan (9 Kelas):\n");
    out.push_str("1. Water (Air)\n");
    out.push_str("2. Trees (Pohon/Hutan)\n");
    out.push_str("3. Grass (Rumput)\n");
    out.push_str("4. Flooded Vegetation (Vegetasi Rawa/Mangrove)\n");
    out.push_str("5. Crops (Pertanian/Sawah)\n");
    out.push_str("6. Shrub & Scrub (Semak Belukar)\n");
    out.push_str("7. Built (Bangunan/Perkotaan)\n");
    out.push_str("8. Bare (Lahan Kosong/Batu)\n");
    out.push_str("9. Snow & Ice (Salju - tidak relevan untuk Indonesia tropis)\n\n");
    
    out.push_str("Aplikasi di Indonesia:\n");
    out.push_str("- Memantau urban sprawl di seluruh Indonesia (urban sprawl Jakarta, deforestasi Kalimantan, sawah Jawa).\n");
    out.push_str("- Perubahan lahan pertanian (Crops) ke perumahan (Built).\n");
    out.push_str("- Identifikasi tutupan lahan secara cepat tanpa perlu melatih classifier dari nol.\n\n");
    out.push_str("⚠️ Tool ini menampilkan informasi referensi dataset, bukan query data real-time.\n\n");
    out.push_str("Akses Data:\n");
    out.push_str("- GEE: ee.ImageCollection('GOOGLE/DYNAMICWORLD/V1')\n");
    out.push_str("- Dynamic World Explorer: https://dynamicworld.app/\n");
    out.push_str("- Resolusi temporal: Near real-time (setiap ada Sentinel-2 scene baru)\n");
    out.push_str("Rekomendasi: Gunakan GEE atau tool landcover_classify untuk analisis tutupan lahan aktual.\n");
    out
}
