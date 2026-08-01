use reqwest::Client;

pub async fn search(
    client: &Client,
    collection: &str,
    limit: u32,
    bbox_opt: Option<String>,
) -> String {
    let col_upper = collection.to_uppercase();
    let collection_id = match col_upper.as_str() {
        "SENTINEL-1" | "S1" => "SENTINEL-1",
        "SENTINEL-2" | "S2" => "SENTINEL-2",
        "SENTINEL-3" | "S3" => "SENTINEL-3",
        "SENTINEL-5P" | "S5P" => "SENTINEL-5P",
        other => other,
    };

    // Default to Indonesia if no BBOX provided (South, West, North, East)
    let bbox_str = bbox_opt.unwrap_or_else(|| "-11.5,95.0,6.0,141.0".to_string());
    let coords: Vec<f64> = bbox_str
        .split(',')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();
    let (s, w, n, e) = if coords.len() == 4 {
        (coords[0], coords[1], coords[2], coords[3])
    } else {
        (-11.5, 95.0, 6.0, 141.0) // Fallback Indonesia
    };

    let url = format!(
        "https://catalogue.dataspace.copernicus.eu/odata/v1/Products?$filter=Collection/Name eq '{}' and OData.CSC.Intersects(area=geography'SRID=4326;POLYGON(({} {},{} {},{} {},{} {},{} {}))')&$top={}&$orderby=ContentDate/Start desc",
        collection_id,
        w, s, e, s,
        e, n, w, n,
        w, s,
        limit
    );

    let mut out = format!("=== Copernicus {} — Spatial Search ===\n", collection_id);
    out.push_str(&format!(
        "Bounding box: {:.2}, {:.2}, {:.2}, {:.2}\n\n",
        s, w, n, e
    ));

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                if let Some(results) = v.get("value").and_then(|r| r.as_array()) {
                    out.push_str(&format!("Ditemukan {} citra:\n\n", results.len()));
                    for (i, r) in results.iter().enumerate() {
                        let name = r.get("Name").and_then(|n| n.as_str()).unwrap_or("?");
                        let date = r
                            .get("ContentDate")
                            .and_then(|d| d.get("Start"))
                            .and_then(|s| s.as_str())
                            .unwrap_or("?");
                        let size = r.get("ContentLength").and_then(|s| s.as_u64()).unwrap_or(0);
                        let id = r.get("Id").and_then(|i| i.as_str()).unwrap_or("?");

                        out.push_str(&format!(
                            "{}. {}\n   TANGGAL: {}\n   SIZE: {:.1} MB\n   ID: {}\n",
                            i + 1,
                            name,
                            date,
                            size as f64 / 1_048_576.0,
                            id
                        ));

                        // Generate direct viewer link for Sentinel-2
                        if collection_id == "SENTINEL-2" {
                            let center_lat = (s + n) / 2.0;
                            let center_lon = (w + e) / 2.0;
                            let date_short = date.split('T').next().unwrap_or(date);
                            let viewer_link = format!(
                                "https://browser.dataspace.copernicus.eu/?zoom=13&lat={}&lng={}&themeId=DEFAULT-THEME&datasetId=S2_L2A_CDAS&fromTime={}T00:00:00.000Z&toTime={}T23:59:59.999Z&layerId=1_TRUE_COLOR",
                                center_lat, center_lon, date_short, date_short
                            );
                            out.push_str(&format!(
                                "   🔗 PREVIEW / DOWNLOAD LANGSUNG: {}\n",
                                viewer_link
                            ));
                        }
                        out.push_str("\n");
                    }
                } else {
                    out.push_str("Tidak ada hasil.\n");
                }
            }
            Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}
