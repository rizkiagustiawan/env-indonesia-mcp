use reqwest::Client;

pub async fn info(client: &Client) -> String {
    let mut out = String::from("=== SRTM (Shuttle Radar Topography Mission) DEM ===\n\n");
    out.push_str("Digital Elevation Model untuk Provinsi NTB.\n");
    out.push_str("Resolusi: 30 meter (1 arc-second)\n\n");
    
    out.push_str("Titik-titik Ketinggian Ekstrem NTB:\n");
    out.push_str("- Puncak G. Rinjani (Lombok): ~3,726 mdpl\n");
    out.push_str("- Puncak G. Tambora (Sumbawa): ~2,850 mdpl\n");
    out.push_str("- Dataran rendah Mataram: ~10-25 mdpl\n\n");
    
    out.push_str("Penggunaan dalam Pipeline Geo-NTB:\n");
    out.push_str("1. Pemodelan Banjir: Menentukan Daerah Aliran Sungai (DAS) dan arah aliran air.\n");
    out.push_str("2. Koreksi Geometrik SAR: Mengoreksi foreshortening/layover pada citra Sentinel-1.\n");
    out.push_str("3. Evaluasi EBT: Analisis kelerengan (slope) dan hadap (aspect) untuk penempatan PLTS.\n");
    out
}
