use reqwest::Client;

pub async fn query(_client: &Client) -> String {
    let mut out = String::from("=== NASA GRACE / GRACE-FO (Gravity Recovery and Climate Experiment) ===\n\n");
    out.push_str("Sensor: Twin satellites measuring gravity anomalies\n");
    out.push_str("Produk Utama: Terrestrial Water Storage (TWS) Anomaly\n");
    out.push_str("Resolusi Spasial: ~300 km (Mascon / Gridded)\n\n");
    
    out.push_str("Aplikasi di Indonesia:\n");
    out.push_str("- Perhitungan Groundwater Storage (GWS) = TWS - Soil Moisture (GLDAS)\n");
    out.push_str("- Memantau defisit air tanah di Indonesia selama musim kemarau (terutama NTT, NTB, Jawa Timur).\n");
    out.push_str("- Mendeteksi penurunan muka air tanah antropogenik vs variabilitas iklim natural.\n\n");
    out.push_str("⚠️ Tool ini menampilkan informasi referensi dataset, bukan query data real-time.\n\n");
    out.push_str("Akses Data:\n");
    out.push_str("- NASA PO.DAAC: https://podaac.jpl.nasa.gov/ (perlu Earthdata Login gratis)\n");
    out.push_str("- GEE Mascon: ee.ImageCollection('NASA/GRACE/MASS_GRIDS/MASCON_CRI')\n");
    out.push_str("- GRACE Tellus: https://grace.jpl.nasa.gov/data/get-data/\n");
    out.push_str("Rekomendasi: Gunakan GEE untuk akses data aktual atau wrapper_groundwater untuk data Indonesia.\n");
    out
}
