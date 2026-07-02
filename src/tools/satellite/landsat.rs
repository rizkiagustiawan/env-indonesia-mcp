use reqwest::Client;

pub async fn search(client: &Client, year: &str) -> String {
    let mut out = format!("=== USGS Landsat Archive Search (NTB) - Year {} ===\n\n", year);
    out.push_str("Source: USGS EarthExplorer M2M API / Google Earth Engine\n");
    out.push_str("Collections: Landsat 5 (TM), Landsat 7 (ETM+), Landsat 8/9 (OLI/TIRS)\n\n");
    
    out.push_str("Data Availability for NTB:\n");
    if year.starts_with("19") || year == "2000" || year == "2001" || year == "2002" {
        out.push_str("- Landsat 5/7 coverage available. Resolusi: 30m Multispectral, 120m/60m Thermal.\n");
        out.push_str("- Note: Landsat 7 SLC-off data setelah May 2003 memiliki gap (striping).\n");
    } else {
        out.push_str("- Landsat 8/9 coverage available. Resolusi: 30m Multispectral, 15m Pan, 100m Thermal.\n");
        out.push_str("- Revisit time kombinasi L8/L9: 8 hari.\n");
    }
    
    out.push_str("\nAplikasi untuk Lingkungan NTB:\n");
    out.push_str("1. Analisis perubahan garis pantai (1970an - sekarang)\n");
    out.push_str("2. Land Surface Temperature (LST) untuk Urban Heat Island di Mataram\n");
    out.push_str("3. Sejarah deforestasi di Rinjani / Tambora\n");
    
    out
}
