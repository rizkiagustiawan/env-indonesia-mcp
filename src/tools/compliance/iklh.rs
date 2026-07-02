/// IKLH Calculator — Indeks Kualitas Lingkungan Hidup
/// Ref: PermenLHK P.27/2021
/// IKLH = (IKA × 30%) + (IKU × 30%) + (IKTL × 40%)

pub fn calculate(ika: f64, iku: f64, iktl: f64) -> String {
    let mut out = String::from("=== IKLH Calculator ===\n");
    out.push_str("Ref: PermenLHK No. P.27/2021\n");
    out.push_str("IKLH = (IKA × 30%) + (IKU × 30%) + (IKTL × 40%)\n\n");

    if ika < 0.0 || ika > 100.0 { return format!("ERROR: IKA ({:.1}) harus 0-100.", ika); }
    if iku < 0.0 || iku > 100.0 { return format!("ERROR: IKU ({:.1}) harus 0-100.", iku); }
    if iktl < 0.0 || iktl > 100.0 { return format!("ERROR: IKTL ({:.1}) harus 0-100.", iktl); }

    let iklh = (ika * 0.30) + (iku * 0.30) + (iktl * 0.40);

    out.push_str(&format!("Input:\n  IKA (Indeks Kualitas Air) = {:.2}\n  IKU (Indeks Kualitas Udara) = {:.2}\n  IKTL (Indeks Kualitas Tutupan Lahan) = {:.2}\n\n", ika, iku, iktl));
    out.push_str(&format!("IKLH = ({:.2}×0.30) + ({:.2}×0.30) + ({:.2}×0.40) = {:.2}\n\n", ika, iku, iktl, iklh));

    let kategori = if iklh >= 80.0 { "SANGAT BAIK" } else if iklh >= 70.0 { "BAIK" } else if iklh >= 60.0 { "CUKUP" } else if iklh >= 50.0 { "KURANG" } else { "SANGAT KURANG" };
    out.push_str(&format!("Kategori: {} ({:.2})\n\n", kategori, iklh));
    out.push_str("Skala: 0-50 Sangat Kurang | 50-60 Kurang | 60-70 Cukup | 70-80 Baik | 80-100 Sangat Baik\n");
    out.push_str("\nCatatan:\n  IKA dihitung dari parameter BOD, COD, TSS, DO, Fosfat, Fecal Coliform.\n  IKU dihitung dari parameter SO2 dan NO2 (metode passive sampler).\n  IKTL dihitung dari luas tutupan lahan relatif terhadap luas wilayah.\n\n");
    out.push_str("Referensi: https://iklh.menlhk.go.id/\n");
    out
}
