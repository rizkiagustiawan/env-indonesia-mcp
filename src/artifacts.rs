use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub artifact_id: String,
    pub source_url: String,
    pub collection: String,
    pub item_id: String,
    pub asset_key: String,
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: String,
    pub retrieved_at: String,
    pub crs: Option<String>,
    pub license: Option<String>,
}

impl ArtifactManifest {
    pub fn from_bytes(
        artifact_id: &str,
        source_url: &str,
        collection: &str,
        item_id: &str,
        asset_key: &str,
        media_type: &str,
        bytes: &[u8],
        crs: Option<&str>,
        license: Option<&str>,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);

        Self {
            artifact_id: artifact_id.to_string(),
            source_url: source_url.to_string(),
            collection: collection.to_string(),
            item_id: item_id.to_string(),
            asset_key: asset_key.to_string(),
            media_type: media_type.to_string(),
            byte_length: bytes.len() as u64,
            sha256: format!("{:.64x}", hasher.finalize()),
            retrieved_at: Utc::now().to_rfc3339(),
            crs: crs.map(str::to_string),
            license: license.map(str::to_string),
        }
    }

    pub fn from_digest(
        artifact_id: String,
        source_url: String,
        collection: String,
        item_id: String,
        asset_key: String,
        media_type: String,
        byte_length: u64,
        sha256: String,
        crs: Option<String>,
        license: Option<String>,
    ) -> Self {
        Self {
            artifact_id,
            source_url,
            collection,
            item_id,
            asset_key,
            media_type,
            byte_length,
            sha256,
            retrieved_at: Utc::now().to_rfc3339(),
            crs,
            license,
        }
    }

    pub fn validate_identity(
        collection: &str,
        item_id: &str,
        asset_key: &str,
    ) -> Result<(), String> {
        for (name, value) in [("collection", collection), ("item_id", item_id), ("asset_key", asset_key)] {
            if value.trim().is_empty() {
                return Err(format!("{} must not be empty", name));
            }
            if value.chars().any(|character| character.is_control()) {
                return Err(format!("{} must not contain control characters", name));
            }
            if value.contains('/')
                || value.contains('\\')
                || value.contains('?')
                || value.contains('#')
                || value.contains('%')
                || value == "."
                || value == ".."
            {
                return Err(format!("{} must not contain path traversal", name));
            }
        }
        Ok(())
    }

    pub fn write_json(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    pub fn write_json_create_new(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::ArtifactManifest;

    #[test]
    fn manifest_hashes_bytes_and_round_trips_json() {
        let manifest = ArtifactManifest::from_bytes(
            "artifact-1",
            "https://example.test/a.tif",
            "sentinel-2-l2a",
            "item-1",
            "red",
            "image/tiff",
            b"abc",
            Some("EPSG:4326"),
            Some("CC-BY-4.0"),
        );
        assert_eq!(manifest.byte_length, 3);
        assert_eq!(
            manifest.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let json = serde_json::to_string(&manifest).unwrap();
        let decoded: ArtifactManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, manifest);
    }

    #[test]
    fn manifest_rejects_empty_asset_identity() {
        let error = ArtifactManifest::validate_identity("", "item", "asset").unwrap_err();
        assert!(error.contains("collection"));
    }

    #[test]
    fn manifest_writes_json_to_a_nested_directory() {
        let directory = std::env::temp_dir()
            .join(format!(
                "env-indonesia-mcp-artifacts-{}",
                std::process::id()
            ))
            .join("nested");
        let path = directory.join("manifest.json");
        let manifest = ArtifactManifest::from_bytes(
            "artifact-1",
            "https://example.test/a.tif",
            "sentinel-2-l2a",
            "item-1",
            "red",
            "image/tiff",
            b"abc",
            None,
            None,
        );

        manifest.write_json(&path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let decoded: ArtifactManifest = serde_json::from_str(&contents).unwrap();
        assert_eq!(decoded, manifest);
        std::fs::remove_dir_all(directory.parent().unwrap()).unwrap();
    }

    #[test]
    fn manifest_create_new_does_not_overwrite_existing_json() {
        let directory = std::env::temp_dir().join(format!(
            "env-indonesia-mcp-artifacts-create-new-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("manifest.json");
        std::fs::write(&path, b"original").unwrap();
        let manifest = ArtifactManifest::from_bytes(
            "artifact-1",
            "https://example.test/a.tif",
            "sentinel-2-l2a",
            "item-1",
            "red",
            "image/tiff",
            b"abc",
            None,
            None,
        );

        assert!(manifest.write_json_create_new(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        std::fs::remove_dir_all(directory).unwrap();
    }
}
