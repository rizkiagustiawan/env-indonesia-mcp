use reqwest::Client;

pub async fn search(client: &Client, query: &str, limit: u32) -> String {
    let url = format!(
        "https://data.go.id/api/3/action/package_search?q={}&rows={}&fq=groups:lingkungan-dan-sumber-daya-alam",
        urlencoding(query), limit
    );

    let mut out = format!("=== Satu Data Indonesia — Search: '{}' ===\n", query);
    out.push_str("Source: data.go.id (CKAN API)\n\n");

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                if let Some(results) = v.get("result").and_then(|r| r.get("results")).and_then(|r| r.as_array()) {
                    out.push_str(&format!("Found: {} datasets\n\n", v.get("result").and_then(|r| r.get("count")).and_then(|c| c.as_u64()).unwrap_or(0)));
                    for (i, ds) in results.iter().enumerate() {
                        let title = ds.get("title").and_then(|t| t.as_str()).unwrap_or("?");
                        let org = ds.get("organization").and_then(|o| o.get("title")).and_then(|t| t.as_str()).unwrap_or("?");
                        let notes = ds.get("notes").and_then(|n| n.as_str()).unwrap_or("");
                        let url = ds.get("url").and_then(|u| u.as_str()).unwrap_or("");
                        out.push_str(&format!("{}. {}\n   Org: {}\n   {}\n   URL: {}\n\n", i+1, title, org, &notes[..notes.len().min(200)], url));
                    }
                } else {
                    out.push_str("No results or API format changed.\n");
                    out.push_str(&format!("Raw: {}\n", serde_json::to_string_pretty(&v).unwrap_or_default().chars().take(1000).collect::<String>()));
                }
            }
            Err(e) => out.push_str(&format!("Parse error: {}\n", e)),
        },
        Err(e) => out.push_str(&format!("Connection error: {}\n", e)),
    }
    out
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '~' => c.to_string(),
        ' ' => "+".into(),
        _ => format!("%{:02X}", c as u8),
    }).collect()
}
