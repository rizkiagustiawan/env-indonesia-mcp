# 🇮🇩 env-indonesia-mcp

**God Tier 10/10 Model Context Protocol (MCP) Server for Environmental Engineering in Indonesia.**

`env-indonesia-mcp` is an elite, physics-informed remote sensing and environmental calculation server. Designed to be consumed by LLM agents (like Zeroclaw), it bridges the gap between conversational AI and deterministic, regulatory-grade environmental science.

With **228 integrated tools**, this MCP server ensures that no AI hallucination violates the laws of thermodynamics, fluid dynamics, or Indonesian environmental regulations.

---

## 🌟 Key Features (Elite God-Tier Architecture)

### 1. The Physics-Informed Validator (`physics_validator.rs`)
LLMs are bad at math. This MCP acts as a rigid boundary. If an LLM attempts to pass `Runoff > Rainfall` or `COD < BOD`, the server hard-rejects the request with a detailed physics explanation (e.g., "Violates Law of Mass Conservation"). Includes built-in checks for:
- **PP 22/2021** (Air & Water Quality Standards).
- **KepMenLH 48/1996** (Noise Level Limits).
- **PermenLHK 14/2020** (ISPU / Air Quality Index).
- Thermodynamic constraints (e.g., Evapotranspiration bounds, DO saturation limits).

### 2. Global & National Satellite Integrations
- **Microsoft Planetary Computer STAC**: L-Band SAR (JAXA ALOS-2) integration to penetrate dense tropical canopies, overcoming Sentinel-1 (C-Band) limitations.
- **Google Earth Engine (GEE)**: Seamless headless extraction of Sentinel-2, Landsat, MODIS, and CHIRPS.
- **Indonesian Authentic APIs**:
  - **BIG (Badan Informasi Geospasial)**: Automated extraction of DEMNAS (8m resolution elevation) bypassing JWT token barriers.
  - **BNPB InaRISK**: Direct connections to Indonesia's disaster risk spatial database.
  - **BMKG & KLHK**: Real-time telemetry and hotspot monitoring.

### 3. Spatial Topology & Hydrological Routing
- **Gravity-Directed Acyclic Graphs (DAG)**: Extracts Z-values (elevation) from DEMNAS and translates them into gravitational flow networks using Manning's roughness coefficients.
- **Contaminant Plume Routing**: Predicts the exact downstream trajectory of Acid Mine Drainage (AMD) or tailings spills across map indices (e.g., in Sumbawa Barat).

### 4. 228 Regulatory Calculators & Workflows
Calculators strictly adhere to global scientific consensus and SNI:
- **Olofsson Accuracy Assessment**: Unbiased area estimates with 95% Confidence Intervals for Deforestation mapping.
- **2D Monte Carlo Risk Analysis (2D-MCA)**: Separates epistemic and aleatory uncertainty for Human Health Risk Assessments (HHRA).
- **Gaussian Plume Dispersion**: Translates LLM JSON inputs into rigorous atmospheric dispersion arrays.
- **Hydrology**: SCS-CN, Rational Method, Streeter-Phelps DO Sag Curves, RUSLE erosion modeling.

---

## 🛠️ Ecosystem Integration (Zeroclaw)
This MCP server is designed to act as the "Hands and Eyes" for the `zeroclaw` agent orchestrator.
- **`zeroclaw-ml`**: The local Rust deep learning engine handles regression and parameter calibration locally.
- **`geo_orchestrator`**: Breaks down massive spatial queries (e.g., all of Indonesia) into async, overlapping 50km tiles to prevent OOM and GEE timeouts.
- **`aermod_orchestrator`**: Translates LLM intents into US EPA FORTRAN `.inp` files for heavy containerized simulation.

---

## 📦 Installation & Usage

Add this server to your MCP client configuration (e.g., Claude Desktop, Zeroclaw, Cursor):

```json
{
  "mcpServers": {
    "env-indonesia": {
      "command": "cargo",
      "args": ["run", "--manifest-path", "/path/to/env-indonesia-mcp/Cargo.toml"]
    }
  }
}
```

## 🔐 Domain Lock Security
All spatial tools are hardware-locked to the geographical boundaries of Indonesia (`[-11.5, 95.0, 6.0, 141.5]`). Requests outside this bounding box are automatically rejected.

## 🤝 Scientific Attribution
All calculators and workflows return a standardized `ScientificResult` JSON object, ensuring every number output by the AI contains a transparent citation (e.g., *Ref: Kuichling 1889, Suripin 2004*).
