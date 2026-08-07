use reqwest::Client;
use serde_json::Value;

const ERDDAP_BASE: &str = "https://coastwatch.pfeg.noaa.gov/erddap/griddap/NOAA_DHW.json";

const INDONESIA_REEF_SITES: &[(&str, f64, f64)] = &[
    ("Raja Ampat, Papua Barat", -0.5, 130.5),
    ("Bunaken, Sulut", 1.6, 124.8),
    ("Wakatobi, Sultra", -5.3, 123.7),
    ("Komodo, NTT", -8.5, 119.5),
    ("Bali (Nusa Penida)", -8.7, 115.5),
    ("Lombok (Gili)", -8.4, 116.0),
    ("Derawan, Kaltim", 2.3, 118.2),
    ("Togean, Sulteng", 0.1, 122.0),
    ("Banda, Maluku", -4.5, 129.9),
    ("Morotai, Maluku Utara", 2.5, 128.4),
    ("Anambas, Kepri", 3.2, 106.2),
    ("Karimunjawa, Jateng", -5.8, 110.5),
    ("Thousand Islands (Kepulauan Seribu)", -5.6, 106.8),
    ("Sumbawa", -8.5, 117.4),
    ("Flores (Maumere)", -8.6, 122.2),
    ("Timor (Kupang)", -10.2, 123.6),
];

pub async fn query_dhw(
    client: &Client,
    lat: f64,
    lon: f64,
) -> String {
    let now = chrono::Utc::now();
    let date_str = now.format("%Y-%m-%dT00:00:00Z").to_string();
    let month_ago = now - chrono::Duration::days(90);
    let month_str = month_ago.format("%Y-%m-%dT00:00:00Z").to_string();

    let url = format!(
        "{}?CRW_DHW%5B({}):1:({})%5D%5B({}):1:({})%5D%5B({}):1:({})%5D",
        ERDDAP_BASE, month_str, date_str,
        lat - 0.05, lat + 0.05,
        lon - 0.05, lon + 0.05
    );

    let mut out = format!("═══════════════════════════════════════════════\n");
    out.push_str("NOAA Coral Reef Watch — DHW (Degree Heating Weeks)\n");
    out.push_str(&format!("Lokasi: ({:.4}, {:.4})\n", lat, lon));
    out.push_str(&format!("Periode: {} to {}\n", month_str, date_str));
    out.push_str(&format!("Source: NOAA CRW v3.1 5km (ERDDAP)\n"));
    out.push_str(&format!("Ref: Goreau & Hayes 2024; Lachs et al. 2024; Festo et al. 2026\n\n"));

    match client.get(&url).send().await {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                if status.as_u16() == 302 || status.as_u16() == 301 {
                    out.push_str("INFO: NOAA ERDDAP redirecting ke mirror server.\n");
                    out.push_str("Coba akses langsung: https://pae-paha.pacioos.hawaii.edu/erddap/griddap/dhw_5km.json\n");
                } else {
                    out.push_str(&format!("HTTP Error: {}\n", status));
                }
                return out;
            }
            match resp.json::<Value>().await {
                Ok(v) => {
                    let table = v.get("table").and_then(|t| t.as_array());
                    if let Some(table) = table {
                        let mut dhw_values: Vec<(String, f64)> = Vec::new();
                        let mut latest_dhw: f64 = 0.0;
                        let mut latest_baa: i64 = 0;
                        let mut latest_sst: f64 = 0.0;
                        let mut latest_hotspot: f64 = 0.0;

                        for row in table.iter().rev().take(30).rev() {
                            if let Some(arr) = row.as_array() {
                                let timestamp = arr.get(0)
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("?");
                                let dhw = arr.get(1)
                                    .and_then(|d| d.as_f64())
                                    .or_else(|| arr.get(1).and_then(|d| d.as_str()).and_then(|s| s.parse::<f64>().ok()))
                                    .unwrap_or(0.0);
                                let baa = arr.get(3)
                                    .and_then(|b| b.as_f64())
                                    .unwrap_or(0.0) as i64;
                                let sst = arr.get(7)
                                    .and_then(|s| s.as_f64())
                                    .unwrap_or(0.0);
                                let hotspot = arr.get(5)
                                    .and_then(|h| h.as_f64())
                                    .unwrap_or(0.0);

                                if dhw > 0.0 || sst > 0.0 {
                                    dhw_values.push((timestamp.to_string(), dhw));
                                    if latest_dhw == 0.0 {
                                        latest_dhw = dhw;
                                        latest_baa = baa;
                                        latest_sst = sst;
                                        latest_hotspot = hotspot;
                                    }
                                }
                            }
                        }

                        if dhw_values.is_empty() {
                            out.push_str("Tidak ada data DHW untuk lokasi/periode ini.\n");
                            out.push_str("Kemungkinan: lokasi bukan reef, atau data belum update.\n");
                            return out;
                        }

                        out.push_str(&format!("DHW terbaru: {:.2} °C-weeks\n", latest_dhw));
                        out.push_str(&format!("SST terbaru: {:.2} °C\n", latest_sst));
                        out.push_str(&format!("Hotspot: {:.2} °C\n", latest_hotspot));
                        out.push_str(&format!("Bleaching Alert Area: {}\n\n", baa_label(latest_baa)));

                        let trend = dhw_values.iter().rev().take(7).collect::<Vec<_>>();
                        if trend.len() >= 2 {
                            out.push_str("Trend DHW (7 data terbaru):\n");
                            for (ts, val) in trend {
                                let short_ts = ts.split('T').next().unwrap_or(ts);
                                out.push_str(&format!("  {} → {:.2} °C-weeks\n", short_ts, val));
                            }
                            out.push_str("\n");
                        }

                        out.push_str(&dhw_assessment(latest_dhw));
                        out.push_str(&dhw_recommendation(latest_dhw));

                        out.push_str("\n───────────────────────────────────────────────\n");
                        out.push_str("ALERT LEVELS (NOAA CRW):\n");
                        out.push_str("  0 = No Stress\n");
                        out.push_str("  1 = Watch (heat stress possible)\n");
                        out.push_str("  2 = Warning (bleaching likely)\n");
                        out.push_str("  3 = Alert Level 1 (bleaching expected)\n");
                        out.push_str("  4 = Alert Level 2 (mortality likely)\n\n");

                        out.push_str("THRESHOLDS:\n");
                        out.push_str("  DHW > 4 °C-weeks → significant bleaching\n");
                        out.push_str("  DHW > 8 °C-weeks → widespread bleaching + mortality\n\n");

                        out.push_str("LIMITATION:\n");
                        out.push_str("  - 5km resolution: reef <1km tidak ter-resolve\n");
                        out.push_str("  - Indo-Pacific corals mungkin lebih tolerant (+1.7°C-weeks, Lachs 2024)\n");
                        out.push_str("  - DHW = cumulative 12-week, bukan instantaneous\n");
                        out.push_str("  - 2024 = fourth global mass bleaching event confirmed\n");
                        out.push_str("═══════════════════════════════════════════════\n");
                    } else {
                        out.push_str("Format ERDDAP tidak expected (no table array).\n");
                        out.push_str(&format!("Response keys: {}\n",
                            v.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>().join(", ")).unwrap_or("N/A".to_string())));
                    }
                }
                Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
            }
        }
        Err(e) => {
            out.push_str(&format!("Connection error: {}\n", e));
            out.push_str("NOAA ERDDAP mungkin timeout atau redirect ke mirror.\n");
            out.push_str("Coba: https://pae-paha.pacioos.hawaii.edu/erddap/griddap/dhw_5km.json\n");
        }
    }
    out
}

fn baa_label(baa: i64) -> &'static str {
    match baa {
        0 => "No Stress",
        1 => "Watch",
        2 => "Warning",
        3 => "Alert Level 1 (Bleaching Expected)",
        4 => "Alert Level 2 (Mortality Likely)",
        _ => "Unknown",
    }
}

fn dhw_assessment(dhw: f64) -> String {
    if dhw == 0.0 {
        return "STATUS: No thermal stress detected.\n".to_string();
    }
    if dhw < 4.0 {
        return format!("STATUS: Low stress ({:.2} °C-weeks). Monitoring recommended.\n", dhw);
    }
    if dhw < 8.0 {
        return format!("STATUS: ⚠️ Bleaching likely ({:.2} °C-weeks). Significant bleaching expected in sensitive species.\n", dhw);
    }
    if dhw < 12.0 {
        return format!("STATUS: 🔴 Widespread bleaching ({:.2} °C-weeks). Mortality likely in susceptible species.\n", dhw);
    }
    format!("STATUS: 🔴🔴 Severe mortality ({:.2} °C-weeks). Mass mortality expected.\n", dhw)
}

fn dhw_recommendation(dhw: f64) -> String {
    let mut r = String::from("\nRECOMMENDATION:\n");
    if dhw == 0.0 {
        r.push_str("  - Routine monitoring. No action needed.\n");
    } else if dhw < 4.0 {
        r.push_str("  - Increase monitoring frequency (weekly).\n");
        r.push_str("  - Check SST trend — if rising, prepare bleaching response plan.\n");
    } else if dhw < 8.0 {
        r.push_str("  - Alert dive operators and MPA managers.\n");
        r.push_str("  - Reduce dive tourism pressure on affected reefs.\n");
        r.push_str("  - Deploy in-water temperature loggers for validation.\n");
    } else {
        r.push_str("  - Activate emergency response: coral rescue, shade structures.\n");
        r.push_str("  - Close affected reef sites to tourism.\n");
        r.push_str("  - Document bleaching extent for post-event assessment.\n");
        r.push_str("  - Notify KLHK and regional coral monitoring network.\n");
    }
    r
}

pub fn reef_sites_list() -> String {
    let mut out = String::from("═══════════════════════════════════════════════\n");
    out.push_str("Indonesia Coral Reef Monitoring Sites\n");
    out.push_str("Source: NOAA Coral Reef Watch 5km\n");
    out.push_str("═══════════════════════════════════════════════\n\n");
    out.push_str(&format!("{:<5} {:<35} {:<12} {:<12}\n", "#", "Site", "Lat", "Lon"));
    out.push_str(&"-".repeat(64).to_string());
    out.push_str("\n");
    for (i, (name, lat, lon)) in INDONESIA_REEF_SITES.iter().enumerate() {
        out.push_str(&format!(
            "{:<5} {:<35} {:<12.4} {:<12.4}\n",
            i + 1, name, lat, lon
        ));
    }
    out.push_str(&format!("\nTotal: {} reef sites\n", INDONESIA_REEF_SITES.len()));
    out.push_str("\nUsage: kirim koordinat reef ke coral_dhw_alert untuk query DHW real-time.\n");
    out
}
