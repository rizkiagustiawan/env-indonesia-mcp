from pathlib import Path


def test_stac_download_contract_mentions_hash_and_screening_boundary():
    stac_rs_path = Path("src/tools/satellite/stac.rs")
    source = stac_rs_path.read_text()
    assert "download_asset" in source, "Missing download_asset function in stac.rs"
    assert "sha256" in source, "Missing sha256 reference in stac.rs"
    assert "scientific interpretation was not performed" in source, "Missing screening boundary disclaimer in stac.rs"
    assert "dummy content" not in source
    assert "placeholder_or_actual_hash" not in source
    assert "ArtifactManifest::from_digest" in source
    assert "write_all" in source or "std::fs::write" in source
    assert "validate_download_bytes" in source
    assert "manifest.sha256" in source
