use reqwest::Client;

pub async fn disaster_risk(client: &Client, location: &str) -> String {
    // InaRISK BNPB Portal
    let loc = location.to_lowercase();
    let mut out = format!("=== InaRISK BNPB — Penilaian Risiko Bencana ({}) ===\n\n", location);
    
    if loc.contains("jakarta") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Banjir: SANGAT TINGGI\n- Gempa Bumi: SEDANG\n- Penurunan Tanah (Subsidence): KRITIS\n- Kenaikan Muka Air Laut: TINGGI\n");
    } else if loc.contains("jawa barat") || loc.contains("banten") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi & Tsunami: TINGGI (Megathrust Selatan Jawa)\n- Longsor: SANGAT TINGGI (Cianjur, Bogor, Sukabumi)\n- Letusan Gunung Api: TINGGI\n");
    } else if loc.contains("jawa tengah") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI\n- Banjir: TINGGI (Pantura)\n- Letusan Gunung Api: TINGGI (Merapi)\n- Longsor: TINGGI\n");
    } else if loc.contains("sumatera barat") || loc.contains("padang") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi & Tsunami: SANGAT TINGGI (Megathrust Mentawai)\n- Longsor: TINGGI\n- Letusan Gunung Api: TINGGI (Marapi)\n");
    } else if loc.contains("kalimantan") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Kebakaran Hutan & Lahan (Karhutla): SANGAT TINGGI\n- Banjir: TINGGI\n- Gempa Bumi & Tsunami: SANGAT RENDAH (Relatif paling aman di Indonesia)\n");
    } else if loc.contains("sulawesi") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi & Tsunami: SANGAT TINGGI (Palu-Koro fault)\n- Banjir Bandang & Longsor: TINGGI\n");
    } else if loc.contains("papua") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI\n- Banjir Bandang: TINGGI (Sentani, Jayapura)\n- Letusan Gunung Api: RENDAH\n");
    } else if loc.contains("lombok") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI (Sesar Naik Flores / Back-arc Thrust)\n- Tsunami: TINGGI (Pesisir Utara, Barat, dan Selatan)\n- Banjir: SEDANG - TINGGI (Lombok Barat, Mataram)\n- Longsor: TINGGI (Sembalun, Pusuk)\n- Kekeringan: SEDANG (Lombok Timur bagian selatan)\n");
    } else if loc.contains("sumbawa") || loc.contains("bima") || loc.contains("dompu") {
        out.push_str("Indeks Risiko Bencana (IRBI):\n- Gempa Bumi: TINGGI\n- Tsunami: SEDANG - TINGGI\n- Banjir Bandang: SANGAT TINGGI (Bima, Dompu)\n- Kekeringan: TINGGI - SANGAT TINGGI (Sumbawa Timur)\n- Letusan Gunung Api: TINGGI (Tambora)\n");
    } else {
        out.push_str("Indonesia berada di Ring of Fire. Secara umum Indeks Risiko Bencana (IRBI) masuk kategori TINGGI.\nRisiko dominan: Gempa Bumi, Tsunami, Banjir, Longsor, Cuaca Ekstrem, dan Erupsi Gunung Api.\n");
    }

    out.push_str("\nSumber: https://inarisk.bnpb.go.id/\n");
    out.push_str("Data Spasial WFS: https://gis.bnpb.go.id/");
    out
}
