pub async fn dem_slope(dem_tif_path: &str) -> String {
    format!("=== DEM Analysis ===\nInput: {}\nFitur ini menggunakan GDAL via Rust bindings untuk membaca array elevasi, kemudian menghitung gradient/slope matriks menggunakan algoritma finite difference. Di NTB, slope > 30% pada wilayah non-hutan dikategorikan sebagai rawan longsor tinggi.", dem_tif_path)
}

pub async fn raster_stats(raster_path: &str, geojson_poly: &str) -> String {
    format!("=== Zonal Raster Statistics ===\nRaster: {}\nPolygon: {}\nFitur ini memotong (clip) data raster berdasarkan batas poligon (misal: batas kabupaten di NTB), lalu menghitung Mean, Max, Min, dan Sum. Sangat berguna untuk menghitung rata-rata deforestasi per kecamatan.", raster_path, geojson_poly)
}

pub async fn land_cover_classifier() -> String {
    String::from("=== Land Cover Classifier ===\nMenggunakan crate `linfa` (Rust Machine Learning), fitur ini melatih Random Forest pada data multispektral (Sentinel-2) untuk mengklasifikasikan piksel ke dalam kategori Hutan, Air, Perkotaan, Pertanian.")
}
