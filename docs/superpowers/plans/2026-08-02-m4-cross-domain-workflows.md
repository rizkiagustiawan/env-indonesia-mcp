# M4 Cross-Domain Workflows Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a `workflows` module that orchestrates air and water tools while enforcing physical continuity checks.

**Architecture:** Create `src/tools/workflows/mod.rs` containing `water_quality.rs` and `air_dispersion.rs`. These orchestrators perform checks (like discharge volume vs river volume, and wind profile extrapolation) before combining tool outputs.

**Tech Stack:** Rust 2021

## Global Constraints

- Do not bypass physical reality (e.g., WWTP discharging more water than the river holds).
- Write failing tests first.
- Integrate with `ScientificResult` from M2.

---

### Task 1: Setup Workflows Module and Water Quality Orchestrator

**Files:**
- Create: `src/tools/workflows/mod.rs`
- Create: `src/tools/workflows/water_quality.rs`
- Modify: `src/tools/mod.rs`
- Modify: `src/server.rs` (to add the new endpoints)

**Interfaces:**
- Produces: `pub fn run_water_quality_assessment(wwtp_flow_m3d: f64, river_width_m: f64, river_depth_m: f64, river_velocity_ms: f64, ...) -> Result<ScientificResult, String>`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Write minimal implementation**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit**

### Task 2: Air Dispersion Orchestrator

**Files:**
- Create: `src/tools/workflows/air_dispersion.rs`
- Modify: `src/tools/workflows/mod.rs`

**Interfaces:**
- Produces: `pub fn run_air_dispersion_assessment(stack_height_m: f64, anemometer_height_m: f64, wind_speed_ms: f64, ...) -> Result<ScientificResult, String>`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Write minimal implementation**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit**
