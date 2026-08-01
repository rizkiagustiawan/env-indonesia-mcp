use reqwest::Client;

pub async fn status(client: &Client) -> String {
    let url = "https://magma.vsi.esdm.go.id/api/v1/gunungapi/informasi";

    let mut out = String::from("=== Status Gunung Api Aktif Indonesia (MAGMA) ===\n\n");
    out.push_str("Data dari MAGMA Indonesia (magma.esdm.go.id). Jika API gagal, data fallback statis terakhir digunakan.\n\n");

    // Try live API first
    match client
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                if let Some(data) = v.get("data").and_then(|d| d.as_array()) {
                    out.push_str(&format!(
                        "(Data live dari API — {} gunung api)\n\n",
                        data.len()
                    ));
                    for item in data.iter().take(20) {
                        let name = item
                            .get("ga_nama_gapi")
                            .or_else(|| item.get("nama"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("?");
                        let status = item
                            .get("ga_status")
                            .or_else(|| item.get("status"))
                            .and_then(|s| s.as_str())
                            .unwrap_or("?");
                        let activity = item
                            .get("ga_aktivitas")
                            .or_else(|| item.get("aktivitas"))
                            .and_then(|a| a.as_str())
                            .unwrap_or("-");
                        let level = item
                            .get("ga_level")
                            .or_else(|| item.get("level"))
                            .and_then(|l| l.as_str())
                            .or_else(|| item.get("ga_level").and_then(|l| l.as_u64()).map(|_| ""))
                            .unwrap_or("");
                        out.push_str(&format!(
                            "- {} | Status: {} | Level: {} | Aktivitas: {}\n",
                            name, status, level, activity
                        ));
                    }
                    if data.len() > 20 {
                        out.push_str(&format!(
                            "\n... dan {} gunung api lainnya.\n",
                            data.len() - 20
                        ));
                    }
                    out.push_str("\nSumber: https://magma.vsi.esdm.go.id/\n");
                    return out;
                }
                // API returned JSON but unexpected structure — fall through to fallback
                out.push_str("(API response format tidak dikenali, menggunakan data fallback)\n\n");
            }
            Err(_) => {
                out.push_str("(API parse error, menggunakan data fallback)\n\n");
            }
        },
        Err(_) => {
            out.push_str("(API tidak dapat dihubungi, menggunakan data fallback statis)\n\n");
        }
    }

    // === FALLBACK: hardcoded data ===
    out.push_str("Indonesia memiliki 127 gunung api aktif. Berikut status yang menonjol:\n\n");

    out.push_str("LEVEL IV (AWAS - Sangat Berbahaya)\n");
    out.push_str("- G. Ruang (Sulawesi Utara): Erupsi eksplosif, awan panas, potensi tsunami.\n");
    out.push_str(
        "- G. Lewotobi Laki-laki (NTT): Erupsi, lontaran batu pijar, hujan abu lebat.\n\n",
    );

    out.push_str("LEVEL III (SIAGA)\n");
    out.push_str("- G. Merapi (Jawa Tengah/DIY): Guguran lava, awan panas.\n");
    out.push_str("- G. Semeru (Jawa Timur): Awan panas guguran (APG).\n");
    out.push_str("- G. Marapi (Sumatera Barat): Erupsi eksplosif intermiten.\n");
    out.push_str("- G. Anak Krakatau (Selat Sunda): Erupsi strombolian.\n");
    out.push_str("- G. Ibu (Sulawesi Utara): Erupsi strombolian & vulcanian.\n\n");

    out.push_str("LEVEL II (WASPADA)\n");
    out.push_str("- Termasuk G. Rinjani (Lombok), G. Kerinci (Jambi), G. Bromo (Jatim).\n\n");

    out.push_str("Status Khusus G. Tambora (Sumbawa): LEVEL I (NORMAL). Erupsi historis terbesar VEI 7 (1815).\n\n");

    out.push_str("Sumber: https://magma.vsi.esdm.go.id/\n");
    out
}
