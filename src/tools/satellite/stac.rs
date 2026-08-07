use reqwest::Client;
use serde_json::Value;

const MPC_STAC_URL: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";
const EARTH_SEARCH_URL: &str = "https://earth-search.aws.element84.com/v1";

#[derive(Debug, Clone)]
pub enum StacApi {
    PlanetaryComputer,
    EarthSearch,
}

impl StacApi {
    fn base_url(&self) -> &str {
        match self {
            StacApi::PlanetaryComputer => MPC_STAC_URL,
            StacApi::EarthSearch => EARTH_SEARCH_URL,
        }
    }

    fn name(&self) -> &str {
        match self {
            StacApi::PlanetaryComputer => "Microsoft Planetary Computer",
            StacApi::EarthSearch => "Element 84 Earth Search",
        }
    }
}

fn parse_api(api_str: &str) -> StacApi {
    match api_str.trim().to_lowercase().as_str() {
        "mpc" | "planetary" | "planetarycomputer" | "microsoft" => StacApi::PlanetaryComputer,
        _ => StacApi::EarthSearch,
    }
}

fn parse_bbox(bbox_opt: &Option<String>) -> (f64, f64, f64, f64) {
    if let Some(ref bbox) = bbox_opt {
        let coords: Vec<f64> = bbox
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();
        if coords.len() == 4 {
            return (coords[0], coords[1], coords[2], coords[3]);
        }
    }
    (-11.5, 95.0, 6.0, 141.0)
}

pub async fn list_collections(client: &Client, api_str: &str) -> String {
    let api = parse_api(api_str);
    let url = format!("{}/collections", api.base_url());

    let mut out = format!("=== {} — Available Collections ===\n\n", api.name());

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(v) => {
                if let Some(collections) = v.get("collections").and_then(|c| c.as_array()) {
                    out.push_str(&format!("Total collections: {}\n\n", collections.len()));
                    for (i, col) in collections.iter().enumerate() {
                        let id = col.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        let title = col
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let license = col
                            .get("license")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        out.push_str(&format!(
                            "{:3}. {:40} | {:50} | license: {}\n",
                            i + 1,
                            id,
                            title,
                            license
                        ));
                    }
                } else {
                    out.push_str("No collections field in response.\n");
                }
            }
            Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}

pub async fn describe_collection(
    client: &Client,
    api_str: &str,
    collection_id: &str,
) -> String {
    let api = parse_api(api_str);
    let url = format!("{}/collections/{}", api.base_url(), collection_id);

    let mut out = format!(
        "=== {} — Collection: {} ===\n\n",
        api.name(),
        collection_id
    );

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(v) => {
                let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("N/A");
                let description = v
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("N/A");
                let license = v.get("license").and_then(|l| l.as_str()).unwrap_or("N/A");
                let providers = v.get("providers").and_then(|p| p.as_array());
                let extent = v.get("extent");
                let spatial = extent
                    .and_then(|e| e.get("spatial"))
                    .and_then(|s| s.get("bbox"))
                    .and_then(|b| b.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|bb| bb.as_array());
                let temporal = extent
                    .and_then(|e| e.get("temporal"))
                    .and_then(|t| t.get("interval"))
                    .and_then(|i| i.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|iv| iv.as_array());

                out.push_str(&format!("Title: {}\n", title));
                out.push_str(&format!("Description: {}\n", description));
                out.push_str(&format!("License: {}\n", license));

                if let Some(spatial) = spatial {
                    if spatial.len() >= 4 {
                        out.push_str(&format!(
                            "Spatial extent: [{}, {}, {}, {}]\n",
                            spatial[0].as_f64().unwrap_or(0.0),
                            spatial[1].as_f64().unwrap_or(0.0),
                            spatial[2].as_f64().unwrap_or(0.0),
                            spatial[3].as_f64().unwrap_or(0.0)
                        ));
                    }
                }

                if let Some(temporal) = temporal {
                    if temporal.len() >= 2 {
                        let start = temporal[0].as_str().unwrap_or("N/A");
                        let end = temporal[1].as_str().unwrap_or("present");
                        out.push_str(&format!("Temporal extent: {} to {}\n", start, end));
                    }
                }

                if let Some(providers) = providers {
                    let provider_names: Vec<&str> = providers
                        .iter()
                        .filter_map(|p| p.get("name").and_then(|n| n.as_str()))
                        .collect();
                    if !provider_names.is_empty() {
                        out.push_str(&format!("Providers: {}\n", provider_names.join(", ")));
                    }
                }

                if let Some(assets) = v.get("assets").and_then(|a| a.as_object()) {
                    out.push_str(&format!("\nAssets ({}):\n", assets.len()));
                    for (key, val) in assets.iter().take(20) {
                        let asset_title = val
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        let asset_type = val
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        out.push_str(&format!(
                            "  {:25} | {:40} | {}\n",
                            key, asset_title, asset_type
                        ));
                    }
                    if assets.len() > 20 {
                        out.push_str(&format!("  ... and {} more\n", assets.len() - 20));
                    }
                }

                if let Some(summaries) = v.get("summaries").and_then(|s| s.as_object()) {
                    out.push_str(&format!("\nSummaries:\n"));
                    for (key, val) in summaries.iter().take(10) {
                        let val_str = if let Some(n) = val.as_f64() {
                            format!("{}", n)
                        } else if let Some(s) = val.as_str() {
                            s.to_string()
                        } else {
                            val.to_string()
                        };
                        out.push_str(&format!("  {}: {}\n", key, val_str));
                    }
                }
            }
            Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}

pub async fn search(
    client: &Client,
    api_str: &str,
    collection: &str,
    bbox_opt: &Option<String>,
    datetime_opt: &Option<String>,
    limit: u32,
) -> String {
    let api = parse_api(api_str);
    let (s, w, n, e) = parse_bbox(bbox_opt);
    let bbox_str = format!("{},{},{},{}", w, s, e, n);
    let datetime = datetime_opt
        .as_deref()
        .unwrap_or("2024-01-01T00:00:00Z/2026-12-31T23:59:59Z");

    let _url = format!(
        "{}/search?collections={}&bbox={}&datetime={}&limit={}",
        api.base_url(),
        collection,
        bbox_str,
        datetime,
        limit
    );

    let mut out = format!("=== {} — STAC Search ===\n", api.name());
    out.push_str(&format!("Collection: {}\n", collection));
    out.push_str(&format!(
        "BBox: {:.2},{:.2},{:.2},{:.2} (W,S,E,N)\n",
        w, s, e, n
    ));
    out.push_str(&format!("Datetime: {}\n", datetime));
    out.push_str(&format!("Limit: {}\n\n", limit));

    let body = serde_json::json!({
        "collections": [collection],
        "bbox": [w, s, e, n],
        "datetime": datetime,
        "limit": limit,
    });

    let post_url = format!("{}/search", api.base_url());

    match client.post(&post_url).json(&body).send().await {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                out.push_str(&format!("HTTP Error: {}\n", status));
                if let Ok(text) = resp.text().await {
                    out.push_str(&format!("Response: {}\n", &text[..text.len().min(500)]));
                }
                return out;
            }
            match resp.json::<Value>().await {
                Ok(v) => {
                    let context = v.get("context");
                    let matched = context
                        .and_then(|c| c.get("matched"))
                        .and_then(|m| m.as_u64())
                        .unwrap_or(0);
                    let returned = context
                        .and_then(|c| c.get("returned"))
                        .and_then(|r| r.as_u64())
                        .unwrap_or(0);

                    out.push_str(&format!("Matched: {} | Returned: {}\n\n", matched, returned));

                    if let Some(features) = v.get("features").and_then(|f| f.as_array()) {
                        if features.is_empty() {
                            out.push_str("No scenes found. Try widening datetime range or bbox.\n");
                            return out;
                        }

                        for (i, feat) in features.iter().enumerate() {
                            let id = feat.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                            let props = feat.get("properties").unwrap_or(&Value::Null);
                            let datetime = props
                                .get("datetime")
                                .and_then(|d| d.as_str())
                                .unwrap_or("?");
                            let cloud_cover = props
                                .get("eo:cloud_cover")
                                .and_then(|c| c.as_f64())
                                .map(|c| format!("{:.1}%", c))
                                .unwrap_or_else(|| "N/A".to_string());
                            let platform = props
                                .get("platform")
                                .and_then(|p| p.as_str())
                                .unwrap_or("");
                            let constellation = props
                                .get("constellation")
                                .and_then(|c| c.as_str())
                                .unwrap_or("");

                            out.push_str(&format!(
                                "{}. ID: {}\n   DATE: {}\n   CLOUD: {}\n   PLATFORM: {} {}\n",
                                i + 1,
                                id,
                                datetime,
                                cloud_cover,
                                constellation,
                                platform
                            ));

                            if let Some(assets) = feat.get("assets").and_then(|a| a.as_object()) {
                                let asset_keys: Vec<&str> = assets.keys().map(|k| k.as_str()).collect();
                                out.push_str(&format!("   ASSETS: {}\n", asset_keys.join(", ")));

                                if let Some(visual) = assets.get("visual")
                                    .or_else(|| assets.get("rendered_preview"))
                                    .or_else(|| assets.get("thumbnail"))
                                    .or_else(|| assets.get("preview"))
                                {
                                    if let Some(href) = visual.get("href").and_then(|h| h.as_str()) {
                                        out.push_str(&format!("   PREVIEW: {}\n", href));
                                    }
                                }

                                if let Some(data) = assets.get("data")
                                    .or_else(|| assets.get("image"))
                                    .or_else(|| assets.get("tif"))
                                    .or_else(|| assets.get("tiff"))
                                {
                                    if let Some(href) = data.get("href").and_then(|h| h.as_str()) {
                                        out.push_str(&format!("   DATA URL: {}\n", href));
                                    }
                                }
                            }

                            if let Some(links) = feat.get("links").and_then(|l| l.as_array()) {
                                for link in links.iter() {
                                    let rel = link.get("rel").and_then(|r| r.as_str()).unwrap_or("");
                                    let href = link.get("href").and_then(|h| h.as_str()).unwrap_or("");
                                    if rel == "self" {
                                        out.push_str(&format!("   SELF: {}\n", href));
                                    }
                                }
                            }
                            out.push_str("\n");
                        }

                        if matched > returned {
                            out.push_str(&format!(
                                "... {} more scenes available. Increase limit or narrow bbox/datetime.\n",
                                matched - returned
                            ));
                        }
                    } else {
                        out.push_str("No features in response.\n");
                    }
                }
                Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
            }
        }
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}

pub async fn get_asset_url(
    client: &Client,
    api_str: &str,
    collection: &str,
    item_id: &str,
    asset_key: &str,
) -> String {
    let api = parse_api(api_str);
    let url = format!(
        "{}/collections/{}/items/{}",
        api.base_url(),
        collection,
        item_id
    );

    let mut out = format!(
        "=== {} — Asset URL ===\n",
        api.name()
    );
    out.push_str(&format!("Collection: {}\n", collection));
    out.push_str(&format!("Item: {}\n", item_id));
    out.push_str(&format!("Asset: {}\n\n", asset_key));

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(v) => {
                if let Some(assets) = v.get("assets").and_then(|a| a.as_object()) {
                    if let Some(asset) = assets.get(asset_key) {
                        let href = asset
                            .get("href")
                            .and_then(|h| h.as_str())
                            .unwrap_or("N/A");
                        let title = asset
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("N/A");
                        let type_ = asset
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("N/A");
                        let roles = asset
                            .get("roles")
                            .and_then(|r| r.as_array())
                            .map(|roles| {
                                roles
                                    .iter()
                                    .filter_map(|r| r.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                            .unwrap_or_default();

                        out.push_str(&format!("Title: {}\n", title));
                        out.push_str(&format!("Type: {}\n", type_));
                        out.push_str(&format!("Roles: {}\n", roles));
                        out.push_str(&format!("\nDOWNLOAD URL:\n{}\n", href));

                        if let Some(alternate) = asset.get("alternate").and_then(|a| a.as_object()) {
                            out.push_str("\nAlternate URLs:\n");
                            for (key, val) in alternate.iter() {
                                let alt_href = val
                                    .get("href")
                                    .and_then(|h| h.as_str())
                                    .unwrap_or("N/A");
                                out.push_str(&format!("  {}: {}\n", key, alt_href));
                            }
                        }
                    } else {
                        out.push_str(&format!(
                            "Asset '{}' not found. Available assets:\n",
                            asset_key
                        ));
                        for key in assets.keys() {
                            out.push_str(&format!("  - {}\n", key));
                        }
                    }
                } else {
                    out.push_str("No assets in item.\n");
                }
            }
            Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}
