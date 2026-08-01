use crate::indonesia;
use reqwest::Client;

pub async fn fire_hotspots(client: &Client, days: u32, bbox_opt: Option<String>) -> String {
    let days = days.min(10).max(1);
    // Default to Indonesia Bounding Box if not provided: [South, West, North, East]
    let bbox = bbox_opt.unwrap_or_else(|| "-11.5,95.0,6.0,141.0".to_string());

    let map_key = std::env::var("FIRMS_MAP_KEY").unwrap_or_else(|_| "FIRMS_MAP_KEY".to_string());

    // Try without MAP_KEY first (public endpoint) or with actual key
    let url = format!(
        "https://firms.modaps.eosdis.nasa.gov/api/area/csv/{}/VIIRS_SNPP_NRT/{}/{}",
        map_key, bbox, days
    );

    // Fallback: use web scraping approach
    let _alt_url = format!(
        "https://firms.modaps.eosdis.nasa.gov/api/country/csv/FIRMS_MAP_KEY/VIIRS_SNPP_NRT/IDN/{}",
        days
    );

    let mut out = format!(
        "=== NASA FIRMS Fire Hotspots — Indonesia ({} days) ===\n",
        days
    );
    out.push_str(&format!("Bounding Box: {}\n", bbox));
    out.push_str("Source: VIIRS S-NPP Near Real-Time\n");
    out.push_str(
        "Note: For full API access, register at https://firms.modaps.eosdis.nasa.gov/api/area/\n",
    );
    out.push_str("      Replace FIRMS_MAP_KEY in config with your key.\n\n");

    // Try the API call
    match client.get(&url).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Ok(body) if status.is_success() && body.contains("latitude") => {
                    let lines: Vec<&str> = body.lines().collect();
                    out.push_str(&format!(
                        "Total hotspots detected: {}\n\n",
                        lines.len().saturating_sub(1)
                    ));
                    for line in lines.iter().take(20) {
                        out.push_str(line);
                        out.push('\n');
                    }
                    if lines.len() > 20 {
                        out.push_str(&format!("... and {} more hotspots\n", lines.len() - 20));
                    }
                }
                Ok(_) => {
                    out.push_str("API key required. To enable:\n");
                    out.push_str("1. Register at https://firms.modaps.eosdis.nasa.gov/api/area/\n");
                    out.push_str("2. Get your MAP_KEY\n");
                    out.push_str("3. Set env: FIRMS_MAP_KEY=your_key\n");
                    out.push_str("\nAlternative: Use web interface at https://firms.modaps.eosdis.nasa.gov/map/\n");
                    out.push_str(&format!("Direct link for Indonesia: https://firms.modaps.eosdis.nasa.gov/map/#t:adv;d:today;l:noaa21-viirs-c2;@{},{},5z\n", indonesia::INDONESIA_CENTER[1], indonesia::INDONESIA_CENTER[0]));
                }
                Err(e) => out.push_str(&format!("Error: {}\n", e)),
            }
        }
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}
