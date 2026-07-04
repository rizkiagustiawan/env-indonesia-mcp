use reqwest::Client;

pub async fn query(client: &Client, year: u32, month: u32) -> String {
    let url = format!("https://data.chc.ucsb.edu/products/CHIRPS-2.0/global_daily/tifs/p05/{}/", year);
    match client.get(&url).send().await {
        Ok(resp) => {
            let text = resp.text().await.unwrap_or_default();
            let mut out = format!("=== CHIRPS Rainfall Data ===\nSource: UCSB Climate Hazards (free, no auth)\nResolution: 0.05° (~5.5 km)\n\nAvailable files for {}/{}:\n\n", year, month);
            // Parse HTML directory listing for matching month
            let month_str = format!("{:02}", month);
            let mut count = 0;
            for line in text.lines() {
                if line.contains(&format!(".{}.{}", year, month_str)) && line.contains(".tif") {
                    if let Some(start) = line.find("chirps-v2.0.") {
                        if let Some(end) = line[start..].find("\"") {
                            let filename = &line[start..start+end];
                            out.push_str(&format!("  {}\n", filename));
                            count += 1;
                            if count >= 5 { break; }
                        }
                    }
                }
            }
            if count == 0 {
                out.push_str(&format!("  (Parsing directory listing — visit {} directly)\n", url));
            }
            out.push_str(&format!("\nDownload: wget {}<filename>\nCoverage: Global | Format: GeoTIFF (gzipped)\n", url));
            out
        }
        Err(e) => format!("ERROR: {}", e)
    }
}
