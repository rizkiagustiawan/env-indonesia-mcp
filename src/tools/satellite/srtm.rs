use reqwest::Client;

pub async fn info(_client: &Client) -> String {
    let mut out = String::from("=== SRTM (Shuttle Radar Topography Mission) DEM ===\n\n");
    out.push_str("Digital Elevation Model untuk Indonesia.\n");
    out.push_str("Resolusi: 30 meter (1 arc-second)\n\n");
    
    out.push_str("Titik-titik Ketinggian Penting Indonesia:\n");
    out.push_str("- Puncak Jaya (Papua): ~4,884 mdpl\n");
    out.push_str("- G. Kerinci (Sumatera): ~3,805 mdpl\n");
    out.push_str("- G. Rinjani (Lombok): ~3,726 mdpl\n");
    out.push_str("- G. Semeru (Jawa): ~3,676 mdpl\n");
    out.push_str("- G. Merapi (Jawa): ~2,930 mdpl\n");
    out.push_str("- DAS utama: Kapuas, Mahakam, Barito, Citarum, Brantas\n\n");
    
    out.push_str("Penggunaan dalam Pipeline Environmental Indonesia:\n");
    out.push_str("1. Pemodelan Banjir: Menentukan Daerah Aliran Sungai (DAS) dan arah aliran air.\n");
    out.push_str("2. Koreksi Geometrik SAR: Mengoreksi foreshortening/layover pada citra Sentinel-1.\n");
    out.push_str("3. Evaluasi EBT: Analisis kelerengan (slope) dan hadap (aspect) untuk penempatan PLTS.\n\n");
    out.push_str("⚠️ Tool ini menampilkan informasi referensi dataset, bukan query data real-time.\n\n");
    out.push_str("Akses Data:\n");
    out.push_str("- USGS EarthExplorer: https://earthexplorer.usgs.gov/ (perlu login gratis)\n");
    out.push_str("- OpenTopography: https://opentopography.org/ (SRTM 30m gratis)\n");
    out.push_str("- GEE: ee.Image('USGS/SRTMGL1_003')\n");
    out.push_str("Rekomendasi: Gunakan tool dem_slope/dem_aspect/dem_hillshade untuk analisis DEM aktual via GEE.\n");
    out
}
