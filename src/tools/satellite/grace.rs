use reqwest::Client;

pub async fn query(client: &Client) -> String {
    let mut out = String::from("=== NASA GRACE / GRACE-FO (Gravity Recovery and Climate Experiment) ===\n\n");
    out.push_str("Sensor: Twin satellites measuring gravity anomalies\n");
    out.push_str("Produk Utama: Terrestrial Water Storage (TWS) Anomaly\n");
    out.push_str("Resolusi Spasial: ~300 km (Mascon / Gridded)\n\n");
    
    out.push_str("Aplikasi di NTB (digunakan di ntb-groundwater-monitor project):\n");
    out.push_str("- Perhitungan Groundwater Storage (GWS) = TWS - Soil Moisture (GLDAS)\n");
    out.push_str("- Memantau defisit air tanah di pulau Sumbawa selama musim kemarau panjang.\n");
    out.push_str("- Mendeteksi penurunan muka air tanah antropogenik vs variabilitas iklim natural.\n");
    out
}
