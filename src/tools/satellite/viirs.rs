use reqwest::Client;

pub async fn query(client: &Client) -> String {
    let mut out = String::from("=== VIIRS (Visible Infrared Imaging Radiometer Suite) ===\n\n");
    out.push_str("Platform: Suomi NPP, NOAA-20, NOAA-21\n");
    out.push_str("Data Utama untuk NTB:\n\n");
    
    out.push_str("1. VNP14IMG (Active Fires)\n");
    out.push_str("   - Resolusi: 375m (Lebih tajam dari MODIS 1km)\n");
    out.push_str("   - Kemampuan: Bisa mendeteksi api yang lebih kecil di lahan pertanian NTB.\n\n");
    
    out.push_str("2. VNP46A1/A2 (Black Marble / Nighttime Lights)\n");
    out.push_str("   - Resolusi: 500m\n");
    out.push_str("   - Kemampuan: Mendeteksi lampu kota, aktivitas kapal penangkap ikan (squid boats) di perairan NTB malam hari, dan flare gas.\n\n");
    
    out.push_str("Akses via Google Earth Engine:\n");
    out.push_str("- ee.ImageCollection('NOAA/VIIRS/DNB/MONTHLY_V1/VCMSLCFG')\n");
    out
}
