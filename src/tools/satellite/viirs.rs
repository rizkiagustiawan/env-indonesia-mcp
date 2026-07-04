use reqwest::Client;

pub async fn query(_client: &Client) -> String {
    let mut out = String::from("=== VIIRS (Visible Infrared Imaging Radiometer Suite) ===\n\n");
    out.push_str("Platform: Suomi NPP, NOAA-20, NOAA-21\n");
    out.push_str("Data Utama untuk Indonesia:\n\n");
    
    out.push_str("1. VNP14IMG (Active Fires)\n");
    out.push_str("   - Resolusi: 375m (Lebih tajam dari MODIS 1km)\n");
    out.push_str("   - Kemampuan: Bisa mendeteksi api yang lebih kecil di lahan pertanian Indonesia.\n\n");
    
    out.push_str("2. VNP46A1/A2 (Black Marble / Nighttime Lights)\n");
    out.push_str("   - Resolusi: 500m\n");
    out.push_str("   - Kemampuan: Mendeteksi lampu kota, aktivitas kapal penangkap ikan (squid boats) di perairan Indonesia malam hari, dan flare gas.\n\n");
    
    out.push_str("\n⚠️ Tool ini menampilkan informasi referensi dataset, bukan query data real-time.\n\n");
    out.push_str("Akses Data:\n");
    out.push_str("- NASA LAADS DAAC: https://ladsweb.modaps.eosdis.nasa.gov/ (perlu Earthdata Login gratis)\n");
    out.push_str("- GEE Active Fires: ee.ImageCollection('NASA/VIIRS/002/VNP14IMG')\n");
    out.push_str("- GEE Nighttime Lights: ee.ImageCollection('NOAA/VIIRS/DNB/MONTHLY_V1/VCMSLCFG')\n");
    out.push_str("- NASA FIRMS (real-time fire): https://firms.modaps.eosdis.nasa.gov/\n");
    out.push_str("Rekomendasi: Gunakan GEE untuk akses data aktual.\n");
    out
}
