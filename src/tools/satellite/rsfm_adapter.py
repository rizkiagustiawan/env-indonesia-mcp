"""Evidence-first contract for remote-sensing foundation-model inference.

The baseline is deliberately small and deterministic.  It is a benchmark for
future TerraFM/SkySense/Prithvi adapters, not a claim that thresholding is a
foundation model or that its output is locally validated.
"""

from __future__ import annotations

import json
from typing import Any, Mapping

import numpy as np


SUPPORTED_TASKS = {"flood_segmentation"}
SUPPORTED_BACKENDS = {"baseline", "pretrained"}


def _invalid(
    error: str,
    task: str,
    backend: str,
    provenance: Mapping[str, Any] | None,
) -> dict[str, Any]:
    return {
        "status": "invalid_input",
        "task": task,
        "backend": backend,
        "error": error,
        "validation": "screening_only",
        "provenance": dict(provenance or {}),
        "limitations": ["Input was rejected before inference; no prediction was generated."],
    }


def _validate_inputs(inputs: Mapping[str, Any]) -> tuple[dict[str, np.ndarray] | None, str | None]:
    if not isinstance(inputs, Mapping) or not inputs:
        return None, "inputs must be a non-empty mapping of modality arrays"

    arrays: dict[str, np.ndarray] = {}
    shape = None
    for name, value in inputs.items():
        try:
            raw = np.asarray(value)
        except (TypeError, ValueError) as exc:
            return None, f"modality {name!r} is not numeric: {exc}"
        if not np.issubdtype(raw.dtype, np.number) or np.issubdtype(raw.dtype, np.bool_):
            return None, f"modality {name!r} must be numeric"
        array = raw.astype(float, copy=False)
        if array.ndim != 2:
            return None, f"modality {name!r} must be a two-dimensional array"
        if array.size == 0:
            return None, f"modality {name!r} must be non-empty"
        if shape is None:
            shape = array.shape
        elif array.shape != shape:
            return None, "all modality arrays must have the same shape"
        if not np.isfinite(array).all():
            return None, f"modality {name!r} must contain only finite values"
        arrays[name] = array
    return arrays, None


def _validate_provenance(provenance: Mapping[str, Any] | None) -> str | None:
    if provenance is None:
        return None
    if not isinstance(provenance, Mapping):
        return "provenance must be a mapping"
    try:
        json.dumps(dict(provenance), allow_nan=False)
    except (TypeError, ValueError) as exc:
        return f"provenance must be JSON-safe: {exc}"
    return None


def _baseline_flood_mask(arrays: Mapping[str, np.ndarray]) -> np.ndarray:
    """Return a conservative deterministic water-change proxy.

    VV/VH are interpreted in dB.  When only VV exists, the VV threshold is
    used.  With both polarisations, both must indicate low backscatter.  This
    keeps the adapter useful as a comparison baseline without pretending to
    reproduce a trained multimodal model.
    """
    vv = arrays.get("vv")
    vh = arrays.get("vh")
    if vv is None and vh is None:
        raise ValueError("baseline flood segmentation requires vv or vh")
    vv_mask = vv < -15.0 if vv is not None else None
    vh_mask = vh < -20.0 if vh is not None else None
    if vv_mask is None:
        return vh_mask
    if vh_mask is None:
        return vv_mask
    return vv_mask & vh_mask


def run_rsfm_inference(
    inputs: Mapping[str, Any],
    task: str = "flood_segmentation",
    backend: str = "baseline",
    provenance: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    """Run a validated RS inference backend and return a JSON-safe result.

    ``pretrained`` is intentionally a contract placeholder until model weights
    and a runtime are supplied.  It must not silently fall back to the
    threshold baseline because that would make experiment provenance false.
    """
    provenance_error = _validate_provenance(provenance)
    if provenance_error:
        return _invalid(provenance_error, task, backend, None)
    if task not in SUPPORTED_TASKS:
        return _invalid(
            f"unsupported task {task!r}; supported tasks: {sorted(SUPPORTED_TASKS)}",
            task,
            backend,
            provenance,
        )
    if backend not in SUPPORTED_BACKENDS:
        return _invalid(
            f"unsupported backend {backend!r}; supported backends: {sorted(SUPPORTED_BACKENDS)}",
            task,
            backend,
            provenance,
        )

    arrays, error = _validate_inputs(inputs)
    if error:
        return _invalid(error, task, backend, provenance)
    assert arrays is not None

    if backend == "pretrained":
        return {
            "status": "insufficient_data",
            "task": task,
            "backend": backend,
            "missing": ["weights", "runtime"],
            "validation": "screening_only",
            "provenance": dict(provenance or {}),
            "limitations": [
                "Pretrained weights and runtime were not supplied; no model output was generated.",
            ],
        }

    try:
        prediction = _baseline_flood_mask(arrays)
    except ValueError as exc:
        return _invalid(str(exc), task, backend, provenance)

    return {
        "status": "ok",
        "task": task,
        "backend": backend,
        "prediction": prediction.tolist(),
        "shape": list(prediction.shape),
        "positive_fraction": float(prediction.mean()),
        "validation": "screening_only",
        "provenance": dict(provenance or {}),
        "limitations": [
            "Baseline threshold output is not a trained foundation-model result.",
            "Output is not validated against local flood observations.",
        ],
    }
