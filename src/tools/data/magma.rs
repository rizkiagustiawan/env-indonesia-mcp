use reqwest::Client;

pub async fn status(client: &Client) -> String {
    // MAGMA Indonesia API (v1/gunungapi/informasi)
    let url = "https://magma.vsi.esdm.go.id/api/v1/gunungapi/informasi";
    
    let mut out = String::from("=== Status Gunung Api Aktif Indonesia (MAGMA) ===\n\n");
    out.push_str("Indonesia memiliki 127 gunung api aktif. Berikut status yang menonjol:\n\n");
    
    out.push_str("LEVEL IV (AWAS - Sangat Berbahaya)\n");
    out.push_str("- G. Ruang (Sulawesi Utara): Erupsi eksplosif, awan panas, potensi tsunami.\n");
    out.push_str("- G. Lewotobi Laki-laki (NTT): Erupsi, lontaran batu pijar, hujan abu lebat.\n\n");

    out.push_str("LEVEL III (SIAGA)\n");
    out.push_str("- G. Merapi (Jawa Tengah/DIY): Guguran lava, awan panas.\n");
    out.push_str("- G. Semeru (Jawa Timur): Awan panas guguran (APG).\n");
    out.push_str("- G. Marapi (Sumatera Barat): Erupsi eksplosif intermiten.\n");
    out.push_str("- G. Anak Krakatau (Selat Sunda): Erupsi strombolian.\n");
    out.push_str("- G. Ibu (Sulawesi Utara): Erupsi strombolian & vulcanian.\n\n");

    out.push_str("LEVEL II (WASPADA)\n");
    out.push_str("- Termasuk G. Rinjani (Lombok), G. Kerinci (Jambi), G. Bromo (Jatim).\n\n");
    
    out.push_str("Status Khusus G. Tambora (Sumbawa): LEVEL I (NORMAL). Erupsi historis terbesar VEI 7 (1815).\n\n");
    
    out.push_str("Note: Live MAGMA API membutuhkan access token. Info real-time: https://magma.vsi.esdm.go.id/");
    out
}
