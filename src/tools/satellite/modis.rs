use reqwest::Client;

pub async fn query(_client: &Client, product: &str) -> String {
    let prod = product.to_uppercase();
    let mut out = format!("=== NASA MODIS Product: {} ===\n\n", prod);
    
    if prod.contains("FIRE") || prod.contains("MCD14") {
        out.push_str("Product: MOD14/MYD14 (Thermal Anomalies & Fire)\n");
        out.push_str("Resolusi: 1km\n");
        out.push_str("Deskripsi: Mendeteksi kebakaran hutan/lahan. Digunakan sebagai backbone NASA FIRMS awal.\n");
    } else if prod.contains("NDVI") || prod.contains("MOD13") {
        out.push_str("Product: MOD13Q1 (Vegetation Indices 16-Day)\n");
        out.push_str("Resolusi: 250m\n");
        out.push_str("Deskripsi: NDVI dan EVI global setiap 16 hari. Sangat baik untuk memantau siklus tanam padi/jagung di seluruh Indonesia.\n");
    } else if prod.contains("LST") || prod.contains("MOD11") {
        out.push_str("Product: MOD11A1 (Land Surface Temperature & Emissivity Daily)\n");
        out.push_str("Resolusi: 1km\n");
        out.push_str("Deskripsi: Suhu permukaan daratan harian. Berguna untuk memantau kekeringan iklim dan anomali panas.\n");
    } else {
        out.push_str("Produk populer: MOD13Q1 (NDVI 250m), MOD11A1 (LST 1km), MCD64A1 (Burned Area 500m)\n");
    }
    
    out.push_str("\n⚠️ Tool ini menampilkan informasi referensi dataset, bukan query data real-time.\n");
    out.push_str("\nAkses Data:\n");
    out.push_str("- NASA LAADS DAAC: https://ladsweb.modaps.eosdis.nasa.gov/ (perlu Earthdata Login gratis)\n");
    out.push_str("- NASA CMR API: https://cmr.earthdata.nasa.gov/search/granules.json?short_name=MOD13Q1\n");
    out.push_str("- Google Earth Engine: ee.ImageCollection('MODIS/061/MOD13Q1')\n");
    out.push_str("Rekomendasi: Gunakan GEE untuk akses data aktual via tool band_math_gee.\n");
    out
}
