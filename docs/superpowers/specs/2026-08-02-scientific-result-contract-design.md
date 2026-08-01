# Scientific Result Contract Design

## Goal

Add a shared, auditable result contract for environmental estimates without pretending that legacy tools have provenance or uncertainty they do not provide.

## Contract

The Rust contract has explicit status, numeric result, unit, uncertainty, provenance, assumptions, limitations, validation, and claims fields. A result may be `valid`, `valid_with_assumptions`, `screening_only`, `insufficient_data`, `out_of_domain`, or `validation_failed`.

Uncertainty is typed rather than always called a confidence interval. Supported representations include confidence interval, prediction interval, credible interval, bound, sensitivity range, and unavailable. Stochastic results must record a reproducible seed.

Provenance records source kind, identifier, acquisition timestamp, sensor, resolution, CRS, algorithm, parameters, and tool version. Fallback sources require an explicit reason. Missing provenance is not silently synthesized.

Claims are governed by status. Screening results cannot claim compliance, approval, safety, legality, or regulatory pass. Human verification is represented separately from machine validation.

## Migration Boundary

This milestone adds the reusable contract and a Python provenance adapter. Existing tools are not mass-migrated in one change. Future domain migrations must construct the contract from measured inputs and must return `insufficient_data` instead of inventing values.

## Verification

Unit tests cover finite values, interval ordering, seed requirements, fallback labeling, stale data, and screening claim restrictions. Serialization is tested as a stable JSON contract.
