//! AI Foundation Models for Earth Observation — NOT IMPLEMENTED.
//!
//! This module previously returned a `[SIMULASI]` string describing a 768-dim
//! embedding extraction that never happened. Read out of context, that output
//! looked like a result. It now returns an explicit error instead.
//!
//! Why it is not implemented: geospatial foundation model inference requires a
//! local GPU with PyTorch. Neither is present in this environment (`nvidia-smi`
//! absent, `import torch` fails). Sending imagery to a third-party inference
//! endpoint is not done without explicit operator consent.
//!
//! What the literature offers once a GPU exists:
//!   - Blumenstiel, Moor, Kienzler & Brunschwiler 2024, arXiv 2403.02059 —
//!     Prithvi for multi-spectral retrieval (6 bands): mAP 97.62% on
//!     BigEarthNet-43, 44.51% on ForestNet-12, with binarised embeddings giving
//!     32x compression at equal accuracy.
//!   - Jain, Marcos, Ienco, Interdonato & Berchoux, TimeSenCLIP,
//!     arXiv 2508.11919, DOI 10.1016/j.isprsjprs.2026.03.043 — argues existing
//!     RS vision-language models over-weight spatial context because they are
//!     adapted from very-high-resolution architectures, and are therefore a poor
//!     fit for medium-resolution imagery. This repository is entirely Sentinel-2
//!     (10-20 m) and Landsat (30 m), i.e. exactly the case they describe.

use serde_json::json;

/// Requested model families, for a clearer error message.
const KNOWN_MODEL_FAMILIES: &[&str] = &["prithvi", "timesenclip", "clay", "dinov2", "siglip"];

/// Returns a structured `not_implemented` error. No embedding is produced.
pub fn get_embeddings(lat: f64, lon: f64, model: &str) -> String {
    let recognised = KNOWN_MODEL_FAMILIES
        .iter()
        .any(|m| model.to_lowercase().contains(m));

    json!({
        "error": "E501",
        "status": "not_implemented",
        "parameter": "foundation_model_embedding",
        "value": serde_json::Value::Null,
        "message": "Ekstraksi embedding foundation model belum diimplementasikan. \
                    Tidak ada nilai yang dikembalikan.",
        "requested": {
            "model": model,
            "model_family_recognised": recognised,
            "lat": lat,
            "lon": lon
        },
        "blocker": {
            "reason": "Inferensi foundation model membutuhkan GPU lokal + PyTorch.",
            "gpu_detected": false,
            "torch_available": false,
            "third_party_inference": "tidak digunakan tanpa persetujuan eksplisit operator \
                                      (mengirim citra ke pihak ketiga)"
        },
        "when_unblocked": [
            "Prithvi retrieval multispektral — Blumenstiel et al. 2024, arXiv 2403.02059 \
             (mAP 97.62% BigEarthNet-43, 44.51% ForestNet-12, binarisasi 32x)",
            "TimeSenCLIP untuk resolusi menengah — Jain et al., arXiv 2508.11919, \
             DOI 10.1016/j.isprsjprs.2026.03.043"
        ]
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_not_implemented_never_a_value() {
        let out = get_embeddings(-8.46, 118.73, "prithvi");
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(v["status"], "not_implemented");
        assert_eq!(v["error"], "E501");
        assert!(v["value"].is_null(), "must not fabricate an embedding");
    }

    #[test]
    fn output_contains_no_simulation_wording() {
        let out = get_embeddings(0.0, 100.0, "anything").to_lowercase();
        assert!(!out.contains("simulasi"), "must not present itself as a simulation result");
        assert!(!out.contains("768"), "must not imply a produced embedding dimension");
    }

    #[test]
    fn records_whether_model_family_is_recognised() {
        let known: serde_json::Value =
            serde_json::from_str(&get_embeddings(0.0, 100.0, "Prithvi-100M")).unwrap();
        assert_eq!(known["requested"]["model_family_recognised"], true);

        let unknown: serde_json::Value =
            serde_json::from_str(&get_embeddings(0.0, 100.0, "no-such-model")).unwrap();
        assert_eq!(unknown["requested"]["model_family_recognised"], false);
    }
}
