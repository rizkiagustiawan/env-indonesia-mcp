# M3 Spatial Bias Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement spatial bias validation metrics (spatial independence, temporal alignment, sensor disparity) as a foundational Rust module.

**Architecture:** Create a new `spatial_validation.rs` module. It exposes independent validation functions returning `Result<(), String>`.

**Tech Stack:** Rust 2021, `geo` crate.

## Global Constraints

- Do not modify existing tools yet; just provide the contract/validators.
- Code must compile without warnings.
- All code must be tested via TDD.

---

### Task 1: Sensor Disparity Guard

**Files:**
- Create: `src/tools/gis/spatial_validation.rs`
- Modify: `src/tools/gis/mod.rs` (create if needed, or link in `src/tools/mod.rs`)

**Interfaces:**
- Produces: `check_sensor_resolution(resolution_m: f64, area_sqm: f64) -> Result<(), String>`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Write minimal implementation**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit**

### Task 2: Temporal Alignment Guard

**Files:**
- Modify: `src/tools/gis/spatial_validation.rs`

**Interfaces:**
- Produces: `check_temporal_alignment(data_season: &str, target_season: &str) -> Result<(), String>`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Write minimal implementation**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit**

### Task 3: Spatial Independence Guard

**Files:**
- Modify: `src/tools/gis/spatial_validation.rs`

**Interfaces:**
- Produces: `check_spatial_independence(coords: &[(f64, f64)], min_distance_m: f64) -> Result<(), String>`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Write minimal implementation**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit**
