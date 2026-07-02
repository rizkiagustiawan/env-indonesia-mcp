pub fn analyze(geojson_str: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(geojson_str) {
        Ok(gj) => {
            let mut out = String::from("=== GeoJSON Analysis ===\n");
            let gtype = gj.get("type").and_then(|t| t.as_str()).unwrap_or("Unknown");
            out.push_str(&format!("Type: {}\n", gtype));

            if gtype == "FeatureCollection" {
                if let Some(features) = gj.get("features").and_then(|f| f.as_array()) {
                    out.push_str(&format!("Features: {}\n", features.len()));
                    for (i, f) in features.iter().take(5).enumerate() {
                        let geom_type = f.get("geometry").and_then(|g| g.get("type")).and_then(|t| t.as_str()).unwrap_or("?");
                        let props = f.get("properties").cloned().unwrap_or(serde_json::json!({}));
                        out.push_str(&format!("  Feature {}: {} | props: {}\n", i, geom_type, 
                            serde_json::to_string(&props).unwrap_or_default().chars().take(100).collect::<String>()));
                    }
                }
            } else if gtype == "Feature" {
                let geom_type = gj.get("geometry").and_then(|g| g.get("type")).and_then(|t| t.as_str()).unwrap_or("?");
                out.push_str(&format!("Geometry: {}\n", geom_type));
                if let Some(coords) = gj.get("geometry").and_then(|g| g.get("coordinates")) {
                    out.push_str(&format!("Coordinates preview: {}\n", 
                        serde_json::to_string(coords).unwrap_or_default().chars().take(200).collect::<String>()));
                }
            }
            out
        }
        Err(e) => format!("Invalid GeoJSON: {}", e),
    }
}
