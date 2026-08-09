/// Sanksi Administratif Bidang Lingkungan Hidup — Permen LH/BPLH 6/2026
/// 4 jenjang: Teguran → Paksaan Pemerintah → Denda (max Rp3M) → Pencabutan Izin
/// Denda = debit × konsentrasi × durasi_hari
/// Mencabut Permen LHK 14/2024; harmonisasi dengan PP 28/2025
/// Ref: Permen LH/BPLH 6/2026 (6 Jul 2026, 246 halaman); PP 28/2025; PP 22/2021
pub fn assess(
    violation_type: &str,
    has_persetujuan_lingkungan: bool,
    has_perizinan_berusaha: bool,
    nilai_investasi_rp: f64,
    debit_m3_day: f64,
    konsentrasi_pencemar_mg_l: f64,
    durasi_hari: u32,
) -> String {
    let mut out = String::from("=== Sanksi Administratif LH (Permen LH/BPLH 6/2026) ===\n");
    out.push_str("Ref: Permen LH 6/2026 (6 Jul 2026, 246 hal); PP 28/2025; PP 22/2021\n\n");

    out.push_str(&format!("Jenis Pelanggaran: {}\n", violation_type));
    out.push_str(&format!("Persetujuan Lingkungan: {}\n", if has_persetujuan_lingkungan { "Ada" } else { "TIDAK ADA" }));
    out.push_str(&format!("Perizinan Berusaha: {}\n", if has_perizinan_berusaha { "Ada" } else { "TIDAK ADA" }));
    out.push_str(&format!("Nilai Investasi: Rp {:.0}\n\n", nilai_investasi_rp));

    // ─── Skema Sanksi Berjenjang (Pasal Permen LH 6/2026) ───
    out.push_str("═══ SKEMA SANKSI BERJENJANG ═══\n\n");

    out.push_str("1. TEGURAN TERTULIS\n");
    out.push_str("   Pelanggaran ringan → perbaikan dalam 30 hari\n");
    out.push_str("   Jika selesai dalam 30 hari: tidak ada sanksi lanjutan\n\n");

    out.push_str("2. PAKSAAN PEMERINTAH\n");
    out.push_str("   Jika teguran diabaikan:\n");
    out.push_str("   - Hentikan sementara produksi\n");
    out.push_str("   - Tutup outlet/outfall limbah\n");
    out.push_str("   - Sita peralatan operasional\n\n");

    out.push_str("3. DENDA ADMINISTRATIF KUMULATIF\n");
    out.push_str("   Max Rp 3.000.000.000 per pelanggaran\n");
    out.push_str("   Denda keterlambatan harian: 1%-5%\n\n");

    out.push_str("4. PEMBEKUAN & PENCABUTAN IZIN\n");
    out.push_str("   Jika denda tidak dilunasi / pencemaran tidak dihentikan\n\n");

    // ─── Perhitungan Denda ───
    out.push_str("═══ PERHITUNGAN DENDA ═══\n\n");

    let mut denda = 0.0_f64;

    // Case 1: Tidak ada dokumen lingkungan
    if !has_persetujuan_lingkungan && !has_perizinan_berusaha {
        denda = nilai_investasi_rp * 0.05; // 5% dari nilai investasi
        out.push_str("Kategori: TIDAK ADA PB & TIDAK ADA Persetujuan Lingkungan\n");
        out.push_str(&format!("  Denda = 5% × Rp{:.0} = Rp {:.0}\n\n", nilai_investasi_rp, denda));
    } else if !has_persetujuan_lingkungan && has_perizinan_berusaha {
        denda = nilai_investasi_rp * 0.025; // 2.5%
        out.push_str("Kategori: ADA PB tapi TIDAK ADA Persetujuan Lingkungan\n");
        out.push_str(&format!("  Denda = 2.5% × Rp{:.0} = Rp {:.0}\n\n", nilai_investasi_rp, denda));
    } else {
        // Case 2: Pelanggaran baku mutu (ada dokumen tapi melanggar)
        out.push_str("Kategori: Pelanggaran Baku Mutu (ada dokumen lingkungan)\n");
        let denda_baku_mutu = debit_m3_day * konsentrasi_pencemar_mg_l * durasi_hari as f64;
        denda = denda_baku_mutu.min(3_000_000_000.0); // max Rp3M

        out.push_str(&format!("  Formula: Denda = Debit × Konsentrasi × Durasi\n"));
        out.push_str(&format!("  = {:.1} m³/hari × {:.2} mg/L × {} hari\n", debit_m3_day, konsentrasi_pencemar_mg_l, durasi_hari));
        out.push_str(&format!("  = Rp {:.0}\n", denda_baku_mutu));
        if denda_baku_mutu > 3_000_000_000.0 {
            out.push_str(&format!("  ⚠️ Denda mencapai batas maksimal Rp 3.000.000.000\n"));
        }
        out.push_str(&format!("  >> Denda: Rp {:.0}\n\n", denda));
    }

    // ─── Profil Risiko (Permen LH 6/2026 basis risiko) ───
    out.push_str("═══ PROFIL RISIKO (Basis Pengawasan) ═══\n");
    out.push_str("  Faktor profil risiko perusahaan:\n");
    out.push_str("  - Tingkat risiko kegiatan (rendah/menengah/tinggi)\n");
    out.push_str(&format!("  - Nilai investasi: Rp {:.0}\n", nilai_investasi_rp));
    out.push_str("  - Riwayat kepatuhan (PROPER)\n");
    out.push_str("  - Kompleksitas pengendalian pencemaran\n\n");

    out.push_str("  Pengawasan:\n");
    out.push_str("  - REGULER: laporan berkala via OSS + kunjungan virtual + inspeksi terjadwal\n");
    out.push_str("  - INSIDENTAL: mendadak, dipicu pengaduan/indikasi pelanggaran/instruksi Menteri\n");
    out.push_str("  - Prioritas: PROPER Merah, objek vital nasional, industri besar\n\n");

    // ─── Hak Pelaku Usaha ───
    out.push_str("═══ HAK PELAKU USAHA ═══\n");
    out.push_str("  Keberatan: maksimal 7 hari sejak keputusan diterima\n");
    out.push_str("  Ajukan ke instansi penerbit sanksi\n\n");

    // ─── Compliance Check ───
    out.push_str("═══ STATUS KEPATUHAN ═══\n");
    if denda > 0.0 {
        out.push_str(&format!("  ❌ PELANGGARAN TERKONFIRMASI\n"));
        out.push_str(&format!("  Denda: Rp {:.0}\n", denda));
        out.push_str(&format!("  Tindakan: Teguran → Paksaan → Denda → Pencabutan (jenjang))\n\n"));
    } else {
        out.push_str("  ✅ Tidak ada pelanggaran terdeteksi\n\n");
    }

    // ─── Mitigation ───
    out.push_str("═══ REKOMENDASI MITIGASI ═══\n");
    if !has_persetujuan_lingkungan {
        out.push_str("  1. Segera urus Persetujuan Lingkungan (AMDAL/UKL-UPL/DPLH via Amdalnet)\n");
        out.push_str("  2. Pastikan RKL-RPL diunggah ke OSS tepat waktu\n");
        out.push_str("  3. Audit kelengkapan dokumen lingkungan\n");
    } else {
        out.push_str("  1. Segera patuhi teguran tertulis (30 hari)\n");
        out.push_str("  2. Perbaiki IPAL/pengendalian pencemaran\n");
        out.push_str("  3. Evaluasi posisi PROPER\n");
        out.push_str("  4. Siapkan tim untuk mekanisme keberatan (7 hari)\n");
    }

    out.push_str("\n═══ PELAPORAN & IZIN ═══\n");
    out.push_str("  PP 22/2021 Pasal 124-131: Laporan kinerja LH semesteran\n");
    out.push_str("  Amdalnet + OSS terintegrasi (PP 28/2025)\n");
    out.push_str("  Permen LH 6/2026: Pengawasan berbasis risiko\n");
    out.push_str("  Kawasan khusus: IKN, KEK dalam cakupan pengawasan\n");

    out.push_str("\n  Ref: Permen LH/BPLH 6/2026; PP 28/2025; PP 22/2021\n");
    out
}
