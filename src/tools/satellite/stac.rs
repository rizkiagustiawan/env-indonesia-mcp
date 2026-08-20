use reqwest::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::Duration;
use std::io::Write;
use crate::artifacts::ArtifactManifest;
use crate::result_contract::{ArtifactLineage, CrsReference, Provenance, ResultStatus, ScientificResult};

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

pub fn safe_asset_filename(value: &str) -> String {
    let safe: String = value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' })
        .collect();
    if safe.is_empty() { "unnamed".to_string() } else { safe }
}

fn is_tiff_content_type(content_type: &str) -> bool {
    matches!(
        content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase().as_str(),
        "image/tiff" | "image/geotiff" | "application/geotiff" | "application/x-geotiff"
    )
}

fn is_generic_content_type(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .eq_ignore_ascii_case("application/octet-stream")
}

fn has_tiff_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(b"II*\0")
        || bytes.starts_with(b"II+\0")
        || bytes.starts_with(b"MM\0*")
        || bytes.starts_with(b"MM\0+")
}

fn validate_identity_component(name: &str, value: &str) -> Result<(), String> {
    ArtifactManifest::validate_identity(
        if name == "collection" { value } else { "valid" },
        if name == "item_id" { value } else { "valid" },
        if name == "asset_key" { value } else { "valid" },
    )?;
    if value.chars().any(|character| character.is_control())
        || value.contains('/')
        || value.contains('\\')
        || value == "."
        || value == ".."
    {
        return Err(format!("{} contains path or control traversal", name));
    }
    Ok(())
}

fn validate_asset_href(api: &StacApi, href: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(href).map_err(|_| "asset href must be an absolute URL".to_string())?;
    if url.scheme() != "https" {
        return Err("asset href must use HTTPS".to_string());
    }
    if url.port_or_known_default() != Some(443) {
        return Err("asset href must use the default HTTPS port".to_string());
    }
    let host = url.host_str().ok_or_else(|| "asset href must have a host".to_string())?;
    let api_host = reqwest::Url::parse(api.base_url())
        .ok()
        .and_then(|base| base.host_str().map(str::to_string))
        .ok_or_else(|| "STAC API host is invalid".to_string())?;
    let documented_storage_host = [
        "blob.core.windows.net",
        "amazonaws.com",
        "element84.com",
        "planetarycomputer.microsoft.com",
    ]
    .iter()
    .any(|suffix| host == *suffix || host.ends_with(&format!(".{}", suffix)));
    if host != api_host && !documented_storage_host {
        return Err(format!("asset href host '{}' is not allowlisted", host));
    }
    Ok(url)
}

pub fn validate_download_bytes(
    asset_media_type: Option<&str>,
    response_media_type: Option<&str>,
    bytes: &[u8],
) -> Result<(), String> {
    let response_media_type = response_media_type
        .ok_or_else(|| "download response is missing a Content-Type header".to_string())?;
    let tiff_magic = has_tiff_magic(bytes);
    let response_is_tiff = is_tiff_content_type(response_media_type)
        || (is_generic_content_type(response_media_type) && tiff_magic);
    let asset_is_tiff = asset_media_type.is_none_or(|value| {
        is_tiff_content_type(value) || (is_generic_content_type(value) && tiff_magic)
    });
    if !response_is_tiff || !asset_is_tiff {
        return Err("downloaded asset does not have a TIFF content type".to_string());
    }

    if !tiff_magic {
        return Err("downloaded asset does not have TIFF magic bytes".to_string());
    }
    Ok(())
}

fn asset_client(api: &StacApi) -> Result<Client, String> {
    let redirect_api = api.clone();
    Client::builder()
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            if validate_asset_href(&redirect_api, attempt.url().as_str()).is_ok() {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .build()
        .map_err(|error| format!("asset HTTP client could not be built: {}", error))
}

fn structured_error(code: &str, message: impl Into<String>) -> String {
    serde_json::json!({
        "status": "error",
        "error": {
            "code": code,
            "message": message.into()
        }
    })
    .to_string()
}

fn item_metadata_string(item: &Value, asset: &Value, key: &str) -> Option<String> {
    asset.get(key)
        .and_then(Value::as_str)
        .or_else(|| item.get(key).and_then(Value::as_str))
        .map(str::to_string)
}

fn item_crs(item: &Value, asset: &Value) -> Option<String> {
    item_metadata_string(item, asset, "proj:code")
        .or_else(|| item_metadata_string(item, asset, "crs"))
        .or_else(|| item.get("properties").and_then(|properties| {
            properties
                .get("proj:code")
                .and_then(Value::as_str)
                .or_else(|| properties.get("crs").and_then(Value::as_str))
                .map(str::to_string)
        }))
        .or_else(|| {
            asset.get("proj:epsg")
                .and_then(Value::as_i64)
                .or_else(|| item.get("proj:epsg").and_then(Value::as_i64))
                .or_else(|| item.get("properties")
                    .and_then(|properties| properties.get("proj:epsg"))
                    .and_then(Value::as_i64))
                .map(|epsg| format!("EPSG:{}", epsg))
        })
}

pub async fn download_asset(
    _client: &Client,
    api_str: &str,
    collection: &str,
    item_id: &str,
    asset_key: &str,
    output_dir: &str,
) -> Result<String, String> {
    if output_dir.trim().is_empty() {
        return Err(structured_error("invalid_output_directory", "output_dir must not be empty"));
    }
    for (name, value) in [("collection", collection), ("item_id", item_id), ("asset_key", asset_key)] {
        if let Err(error) = validate_identity_component(name, value) {
            return Err(structured_error("invalid_identity", error));
        }
    }
    const MAX_ASSET_SIZE: u64 = 512 * 1024 * 1024;
    let api = parse_api(api_str);
    let url = format!(
        "{}/collections/{}/items/{}",
        api.base_url(),
        collection,
        item_id
    );

    let hardened_client = asset_client(&api).map_err(|error| structured_error("asset_client_failed", error))?;
    let response = hardened_client
        .get(&url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| structured_error("item_request_failed", format!("STAC item request failed: {}", e)))?;
    if !response.status().is_success() {
        return Err(structured_error(
            "item_http_error",
            format!("STAC item request returned HTTP {}", response.status()),
        ));
    }
    let item = response.json::<Value>().await.map_err(|e| {
        structured_error("item_parse_failed", format!("STAC item JSON could not be parsed: {}", e))
    })?;

    let assets = item
        .get("assets")
        .and_then(Value::as_object)
        .ok_or_else(|| structured_error("missing_assets", "STAC item has no assets object"))?;
    let asset = assets
        .get(asset_key)
        .ok_or_else(|| structured_error("asset_not_found", format!("Asset '{}' was not found", asset_key)))?;
    let href = asset
        .get("href")
        .and_then(Value::as_str)
        .ok_or_else(|| structured_error("missing_asset_href", "STAC asset has no href"))?;
    let href_url = validate_asset_href(&api, href)
        .map_err(|error| structured_error("invalid_asset_href", error))?;
    let media_type = asset.get("type").and_then(Value::as_str);
    let license = item_metadata_string(&item, asset, "license");
    let crs = item_crs(&item, asset);
    let crs_reference = crs
        .as_deref()
        .map(CrsReference::new)
        .transpose()
        .map_err(|error| structured_error("invalid_crs", error))?;

    let asset_response = hardened_client
        .get(href)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| structured_error("asset_request_failed", format!("Asset request failed: {}", e)))?;
    if !asset_response.status().is_success() {
        return Err(structured_error(
            "asset_http_error",
            format!("Asset request returned HTTP {}", asset_response.status()),
        ));
    }
    if let Some(length) = asset_response.content_length() {
        if length > MAX_ASSET_SIZE {
            return Err(structured_error("asset_too_large", format!("Asset content-length {} exceeds {} bytes", length, MAX_ASSET_SIZE)));
        }
    }
    let response_media_type = asset_response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());
    let response_media_type = response_media_type.map(str::to_string);
    let safe_item = safe_asset_filename(item_id);
    let safe_asset = safe_asset_filename(asset_key);
    let raster_path = format!("{}/{}_{}.tif", output_dir, safe_item, safe_asset);
    let manifest_path = format!("{}.manifest.json", raster_path);
    std::fs::create_dir_all(output_dir)
        .map_err(|e| structured_error("output_directory_failed", format!("Output directory could not be created: {}", e)))?;
    let mut raster_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&raster_path)
        .map_err(|e| structured_error("raster_create_failed", format!("Raster file could not be created: {}", e)))?;
    let download_result: Result<String, String> = (async {
        let mut asset_response = asset_response;
        let mut hasher = Sha256::new();
        let mut byte_length = 0u64;
        let mut magic = Vec::with_capacity(4);
        while let Some(chunk) = asset_response.chunk().await.map_err(|e| {
            structured_error("asset_read_failed", format!("Asset chunk could not be read: {}", e))
        })? {
            byte_length = byte_length.saturating_add(chunk.len() as u64);
            if byte_length > MAX_ASSET_SIZE {
                return Err(structured_error("asset_too_large", format!("Asset exceeds {} bytes", MAX_ASSET_SIZE)));
            }
            if magic.len() < 4 {
                magic.extend_from_slice(&chunk[..chunk.len().min(4 - magic.len())]);
            }
            hasher.update(&chunk);
            raster_file.write_all(&chunk).map_err(|e| {
                structured_error("raster_write_failed", format!("Raster bytes could not be written: {}", e))
            })?;
        }
        if byte_length == 0 {
            return Err(structured_error("empty_asset", "Asset response was empty"));
        }
        validate_download_bytes(media_type, response_media_type.as_deref(), &magic)
            .map_err(|e| structured_error("invalid_tiff", e))?;

        let artifact_id = format!("{}_{}", safe_item, safe_asset);
        let sha256 = format!("{:.64x}", hasher.finalize());
        let manifest = ArtifactManifest::from_digest(
            artifact_id,
            href_url.to_string(),
            collection.to_string(),
            item_id.to_string(),
            asset_key.to_string(),
            response_media_type.as_deref().or(media_type).unwrap_or("image/tiff").to_string(),
            byte_length,
            sha256,
            crs.clone(),
            license,
        );
        ArtifactManifest::validate_identity(collection, item_id, asset_key)
            .map_err(|error| structured_error("invalid_identity", error))?;
        manifest.write_json_create_new(&manifest_path).map_err(|e| {
            structured_error("manifest_write_failed", format!("Artifact manifest could not be written: {}", e))
        })?;
        let mut result = ScientificResult::new("stac_asset", byte_length as f64, "bytes")
            .with_status(ResultStatus::ScreeningOnly)
            .with_provenance(Provenance::new("stac", href_url.as_str(), &manifest.retrieved_at))
            .with_artifact_lineage(ArtifactLineage::new(
                &manifest.artifact_id,
                &manifest.source_url,
                manifest.byte_length,
                &manifest.sha256,
                &manifest.retrieved_at,
            ).with_identity(collection, item_id, asset_key))
            .with_artifact_paths(&raster_path, &manifest_path)
            .with_limitation("scientific interpretation was not performed");
        if let Some(crs) = crs_reference {
            result = result.with_crs(crs);
        }
        result.validate().map_err(|error| structured_error("result_validation_failed", error))?;
        serde_json::to_string_pretty(&result)
            .map_err(|e| structured_error("response_serialization_failed", format!("Download response could not be serialized: {}", e)))
    })
    .await;

    match download_result {
        Ok(json) => Ok(json),
        Err(error) => {
            let _ = std::fs::remove_file(&raster_path);
            let _ = std::fs::remove_file(&manifest_path);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{safe_asset_filename, validate_download_bytes};

    #[test]
    fn validates_little_and_big_endian_tiff_magic() {
        assert!(validate_download_bytes(Some("image/tiff"), Some("image/tiff"), b"II*\0data").is_ok());
        assert!(validate_download_bytes(Some("image/tiff"), Some("image/tiff"), b"MM\0*data").is_ok());
        assert!(validate_download_bytes(Some("image/tiff"), Some("image/tiff"), b"not-tiff").is_err());
    }

    #[test]
    fn rejects_non_tiff_content_types() {
        assert!(validate_download_bytes(Some("image/png"), Some("image/png"), b"II*\0data").is_err());
    }

    #[test]
    fn accepts_generic_content_type_when_tiff_magic_is_present() {
        assert!(validate_download_bytes(
            Some("application/octet-stream"),
            Some("application/octet-stream"),
            b"II*\0data",
        ).is_ok());
    }

    #[test]
    fn requires_response_content_type() {
        assert!(validate_download_bytes(Some("image/tiff"), None, b"II*\0data").is_err());
    }

    #[test]
    fn rejects_generic_content_type_without_tiff_magic() {
        assert!(validate_download_bytes(
            Some("application/octet-stream"),
            Some("application/octet-stream"),
            b"not-tiff",
        ).is_err());
    }

    #[test]
    fn rejects_traversal_and_untrusted_asset_hosts() {
        assert!(super::validate_identity_component("item_id", "../item").is_err());
        assert!(super::validate_asset_href(&super::StacApi::EarthSearch, "http://example.test/a.tif").is_err());
        assert!(super::validate_asset_href(&super::StacApi::EarthSearch, "https://evil.example/a.tif").is_err());
        assert!(super::validate_asset_href(&super::StacApi::EarthSearch, "https://example.com/a.tif").is_err());
    }

    #[test]
    fn sanitizes_asset_path_components() {
        assert_eq!(safe_asset_filename("../scene/red band"), ".._scene_red_band");
    }
}
