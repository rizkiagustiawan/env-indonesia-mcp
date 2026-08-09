/// Analisis Dampak Lalu Lintas (Andalalin) — Permen PUPR 28/2015 + PKJI 2023
/// V/C ratio → LOS (Level of Service A-F)
/// Kapasitas = Co × FCLE × FCPA × FCHS × FCUK
/// Kecepatan arus bebas = (VBD + VBL) × FVB.HS × FVB.UK
/// Ref: Permen PUPR 28/2015; PKJI 2023 (Pedoman Kapasitas Jalan Indonesia)
pub fn assess(
    road_type: &str,
    lane_width_m: f64,
    volume_kend_per_jam: f64,
    emp_mp: f64, emp_ks: f64, emp_sm: f64, emp_bb: f64,
    vol_mp: f64, vol_ks: f64, vol_sm: f64, vol_bb: f64,
    khs: &str,
    shoulder_width_m: f64,
    city_population_million: f64,
    direction_split: f64,
) -> String {
    let mut out = String::from("=== Analisis Dampak Lalu Lintas (Andalalin) ===\n");
    out.push_str("Ref: Permen PUPR 28/2015; PKJI 2023 (Pedoman Kapasitas Jalan Indonesia)\n\n");

    let rt = road_type.to_lowercase();
    let divided = rt.contains("4/2") || rt.contains("6/2") || rt.contains("8/2") || rt.contains("satu_arah") || rt.contains("one_way");

    // ─── 1. KAPASITAS DASAR (Co) ───
    let co: f64 = if divided { 1700.0 } else { 2800.0 }; // SMP/jam per lajur (divided) or per 2 arah (undivided)

    // ─── 2. FAKTOR KOREKSI LEBAR LAJUR (FCLE) ───
    let fcle: f64 = if divided {
        match lane_width_m {
            w if w <= 3.0 => 0.92,
            w if w <= 3.25 => 0.96,
            w if w <= 3.5 => 1.00,
            w if w <= 3.75 => 1.04,
            _ => 1.08,
        }
    } else {
        match lane_width_m {
            w if w <= 5.0 => 0.56,
            w if w <= 6.0 => 0.87,
            w if w <= 7.0 => 1.00,
            w if w <= 8.0 => 1.14,
            w if w <= 9.0 => 1.25,
            w if w <= 10.0 => 1.29,
            w if w <= 11.0 => 1.34,
            _ => 1.34,
        }
    };

    // ─── 3. FAKTOR KOREKSI PEMISAH ARAH (FCPA) ───
    let fcpa: f64 = match direction_split {
        d if d <= 50.0 => 1.00,
        d if d <= 55.0 => 0.97,
        d if d <= 60.0 => 0.94,
        d if d <= 65.0 => 0.91,
        _ => 0.88,
    };

    // ─── 4. FAKTOR KOREKSI HAMBATAN SAMPING (FCHS) ───
    let khs_lower = khs.to_lowercase();
    let khs_weight: f64 = match khs_lower.as_str() {
        s if s.contains("sangat rendah") || s.contains("sr") => 1.0,
        s if s.contains("rendah") || s.contains("r") => 0.97,
        s if s.contains("sedang") || s.contains("s") => 0.93,
        s if s.contains("tinggi") || s.contains("t") => 0.88,
        s if s.contains("sangat tinggi") || s.contains("st") => 0.82,
        _ => 0.95,
    };
    // Shoulder correction
    let shoulder_factor: f64 = match shoulder_width_m {
        w if w <= 0.5 => 0.92,
        w if w <= 1.0 => 0.96,
        w if w <= 1.5 => 0.98,
        _ => 1.00,
    };
    let fchs = khs_weight * shoulder_factor;

    // ─── 5. FAKTOR KOREKSI UKURAN KOTA (FCUK) ───
    let fcuk: f64 = match city_population_million {
        p if p < 0.1 => 0.86,
        p if p < 0.5 => 0.90,
        p if p < 1.0 => 0.94,
        p if p < 3.0 => 1.00,
        _ => 1.04,
    };

    // ─── KAPASITAS JALAN ───
    let capacity = co * fcle * fcpa * fchs * fcuk;

    // ─── VOLUME (SMP/jam) ───
    let total_smp = vol_mp * emp_mp + vol_ks * emp_ks + vol_sm * emp_sm + vol_bb * emp_bb;
    let total_kend = vol_mp + vol_ks + vol_sm + vol_bb;

    // ─── V/C RATIO ───
    let vc_ratio = total_smp / capacity.max(1.0);

    // ─── LOS (Level of Service) ───
    let (los, los_desc) = match vc_ratio {
        v if v <= 0.20 => ("A", "Arus bebas, kecepatan tinggi, nyaman"),
        v if v <= 0.40 => ("B", "Arus stabil, kecepatan tinggi"),
        v if v <= 0.60 => ("C", "Arus stabil, kecepatan sedang"),
        v if v <= 0.75 => ("D", "Arus mendekati kapasitas, kecepatan menurun"),
        v if v <= 0.85 => ("E", "Arus tidak stabil, mendekati jenuh"),
        _ => ("F", "Arus jenuh/terhenti, antrian panjang"),
    };

    // ─── KECEPATAN ARUS BEBAS ───
    let vbd: f64 = if divided { 57.0 } else { 42.0 }; // rata-rata semua kendaraan
    let vbl: f64 = if divided {
        match lane_width_m {
            w if w <= 3.0 => -4.0, w if w <= 3.25 => -2.0, w if w <= 3.5 => 0.0,
            w if w <= 3.75 => 2.0, _ => 4.0,
        }
    } else {
        match lane_width_m {
            w if w <= 5.0 => -9.5, w if w <= 6.0 => -3.0, w if w <= 7.0 => 0.0,
            w if w <= 8.0 => 3.0, w if w <= 9.0 => 4.0, w if w <= 10.0 => 6.0, _ => 7.0,
        }
    };
    let fvbuuk: f64 = match city_population_million {
        p if p < 0.1 => 0.90, p if p < 0.5 => 0.93, p if p < 1.0 => 0.95,
        p if p < 3.0 => 1.00, _ => 1.03,
    };
    let vb = (vbd + vbl) * fchs * fvbuuk;

    // Kecepatan rata-rata (approx berdasarkan V/C ratio)
    let v_avg = if vc_ratio <= 0.2 { vb }
                else if vc_ratio <= 0.4 { vb * 0.85 }
                else if vc_ratio <= 0.6 { vb * 0.70 }
                else if vc_ratio <= 0.75 { vb * 0.55 }
                else if vc_ratio <= 0.85 { vb * 0.40 }
                else { vb * 0.25 };

    // Kepadatan
    let density = if v_avg > 0.0 { total_smp / v_avg } else { 0.0 };

    // ─── OUTPUT ───
    out.push_str(&format!("Tipe Jalan: {} ({})\n", road_type, if divided { "Terbagi" } else { "Tak Terbagi 2/2-TT" }));
    out.push_str(&format!("Lebar Lajur: {:.2} m\n", lane_width_m));
    out.push_str(&format!("Hambatan Samping: {} (bahu {:.1}m)\n", khs, shoulder_width_m));
    out.push_str(&format!("Ukuran Kota: {:.1} juta jiwa\n\n", city_population_million));

    out.push_str("═══ PERHITUNGAN KAPASITAS ═══\n");
    out.push_str(&format!("  Co (Kapasitas Dasar): {} SMP/jam\n", co));
    out.push_str(&format!("  FCLE (Lebar Lajur): {:.2}\n", fcle));
    out.push_str(&format!("  FCPA (Pemisah Arah): {:.2}\n", fcpa));
    out.push_str(&format!("  FCHS (Hambatan Samping): {:.2}\n", fchs));
    out.push_str(&format!("  FCUK (Ukuran Kota): {:.2}\n", fcuk));
    out.push_str(&format!("  >> KAPASITAS (C) = {:.0} × {:.2} × {:.2} × {:.2} × {:.2} = {:.0} SMP/jam\n\n",
        co, fcle, fcpa, fchs, fcuk, capacity));

    out.push_str("═══ VOLUME LALU LINTAS ═══\n");
    out.push_str(&format!("  Total Kendaraan: {:.0} kend/jam\n", total_kend));
    out.push_str(&format!("  EMP: MP={:.1}, KS={:.1}, SM={:.1}, BB={:.1}\n", emp_mp, emp_ks, emp_sm, emp_bb));
    out.push_str(&format!("  >> Total Volume = {:.0} SMP/jam\n\n", total_smp));

    out.push_str("═══ KINERJA JALAN ═══\n");
    out.push_str(&format!("  V/C Ratio = {:.0}/{:.0} = {:.3}\n", total_smp, capacity, vc_ratio));
    out.push_str(&format!("  >> Level of Service (LOS): {} — {}\n", los, los_desc));
    out.push_str(&format!("  Kecepatan Arus Bebas (VB): {:.1} km/jam\n", vb));
    out.push_str(&format!("  Kecepatan Rata-rata: {:.1} km/jam\n", v_avg));
    out.push_str(&format!("  Kepadatan: {:.1} SMP/km\n\n", density));

    // ─── Dampak & Mitigation ───
    out.push_str("═══ ANALISIS DAMPAK (AMDAL) ═══\n");
    match los {
        "A"|"B" => {
            out.push_str("  ✅ Dampak RINGAN — kinerja jalan masih baik\n");
            out.push_str("  Mitigasi: Monitoring volume berkala\n");
        }
        "C"|"D" => {
            out.push_str("  ⚠️ Dampak SEDANG — arus mulai terganggu\n");
            out.push_str("  Mitigasi:\n");
            out.push_str("  1. Manajemen lalu lintas (satu arah, putaran)\n");
            out.push_str("  2. Fasilitas pejalan kaki & lampu lalu lintas\n");
            out.push_str("  3. Pengaturan akses keluar-masuk kawasan\n");
        }
        "E"|"F" => {
            out.push_str("  ❌ Dampak BERAT — arus jenuh/terhenti\n");
            out.push_str("  Mitigasi:\n");
            out.push_str("  1. Pelebaran jalan / tambah lajur\n");
            out.push_str("  2. Jalan alternatif / bypass\n");
            out.push_str("  3. Transportasi publik (shuttle bus, BRT)\n");
            out.push_str("  4. Pengaturan jam operasional (off-peak)\n");
            out.push_str("  5. Traffic demand management (parking pricing)\n");
        }
        _ => {}
    }
    out.push('\n');

    // ─── Parking Demand ───
    out.push_str("═══ KEBUTUHAN PARKIR ═══\n");
    let parking_demand = total_kend * 0.7; // 70% of trip generation
    out.push_str(&format!("  Estimasi kendaraan ke kawasan: {:.0} ({:.0}% dari volume)\n", parking_demand, 70.0));
    out.push_str(&format!("  Kebutuhan slot parkir: {:.0} (asumsi durasi 2 jam)\n", parking_demand * 2.0 / 60.0 * 60.0));
    out.push_str("  Wajib sediakan: lahan parkir on-site, tidak di badan jalan\n\n");

    out.push_str("═══ PEMANTAUAN (RPL) ═══\n");
    out.push_str("  Parameter: Volume lalu lintas, V/C ratio, LOS, kecepatan\n");
    out.push_str("  Frekuensi: Tahunan (kondisi normal), bulanan (operasi awal)\n");
    out.push_str("  Lokasi: Ruas jalan terdampak + simpang utama\n\n");

    out.push_str("═══ PELAPORAN & IZIN ═══\n");
    out.push_str("  PP 22/2021 Pasal 124-131; Amdalnet + OSS\n");
    out.push_str("  Permen PUPR 28/2015: Andalalin wajib untuk AMDAL\n");
    out.push_str("  Permen LH 6/2026: Sanksi jika tidak patuh\n");

    out.push_str("\n  Ref: Permen PUPR 28/2015; PKJI 2023; PP 22/2021\n");
    out
}
