/// SPPL Checker
/// Ref: PP 22/2021 Pasal 35-37

pub fn check(kegiatan: &str, is_wajib_amdal: bool, is_wajib_uklupl: bool) -> String {
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  SPPL Compliance Checker\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PP 22/2021 ttg PPLH\n\n");

    out.push_str(&format!("Kegiatan: {}\n\n", kegiatan));

    if is_wajib_amdal || is_wajib_uklupl {
        out.push_str("❌ Kegiatan Anda TIDAK BISA menggunakan SPPL.\n");
        if is_wajib_amdal {
            out.push_str("Status: WAJIB AMDAL (Dampak Penting & Skala Besar).\n");
        } else {
            out.push_str("Status: WAJIB UKL-UPL (Dampak Sedang).\n");
        }
    } else {
        out.push_str("✅ Kegiatan Anda WAJIB SPPL (Surat Pernyataan Kesanggupan Pengelolaan dan Pemantauan Lingkungan Hidup).\n\n");
        out.push_str("Persyaratan SPPL via OSS (Self-Declare):\n");
        out.push_str("  1. Identitas Pelaku Usaha\n");
        out.push_str("  2. Deskripsi singkat kegiatan dan lokasi\n");
        out.push_str("  3. Dampak lingkungan yang mungkin terjadi (ex: limbah padat, bising)\n");
        out.push_str("  4. Pernyataan komitmen pengelolaan lingkungan\n");
    }
    out
}
