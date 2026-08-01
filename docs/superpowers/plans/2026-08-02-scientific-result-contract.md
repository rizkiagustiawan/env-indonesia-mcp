# Scientific Result Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a reusable provenance and uncertainty envelope that prevents unsupported environmental claims.

**Architecture:** Add a standalone Rust module used by future tools and a backward-compatible Python provenance adapter. Validation is deterministic and fail-closed for finite values, uncertainty integrity, source freshness, stochastic seeds, and screening claims.

**Tech Stack:** Rust 2021, serde, serde_json, chrono, cargo test; Python standard library and unittest.

## Global Constraints

- Do not invent fallback values, telemetry, timestamps, or uncertainty.
- Preserve unrelated user changes in the worktree.
- Keep legacy tools compatible until individually migrated.
- Return `insufficient_data` when required evidence is absent.

### Task 1: Rust Contract RED/GREEN

**Files:**
- Create: `src/result_contract.rs`
- Modify: `src/main.rs`
- Test: inline `#[cfg(test)]` tests in `src/result_contract.rs`

- [ ] Write tests for valid results, non-finite values, malformed uncertainty, missing stochastic seed, unlabeled fallback sources, stale sources, and forbidden screening claims.
- [ ] Run `cargo test result_contract` and confirm the tests fail because the module is not implemented.
- [ ] Implement serde data types and `validate()` with deterministic error messages.
- [ ] Run `cargo test result_contract` and confirm all contract tests pass.

### Task 2: Python Provenance Adapter

**Files:**
- Modify: `src/tools/gis/provenance.py`
- Create: `tests/test_provenance_contract.py`

- [ ] Write failing tests for required metadata, source fallback labels, and JSON round-trip.
- [ ] Run `python -m unittest tests.test_provenance_contract` and confirm the expected failure.
- [ ] Add a strict metadata builder while preserving `create_provenance()` output compatibility.
- [ ] Run the Python tests and `python -m compileall -q src/tools`.

### Task 3: Verification

**Files:**
- No production files beyond Tasks 1-2.

- [ ] Run `cargo test`.
- [ ] Run `cargo build --release`.
- [ ] Run `python -m compileall -q src/tools`.
- [ ] Inspect `git diff` and confirm backup files and graph artifacts are untouched.
