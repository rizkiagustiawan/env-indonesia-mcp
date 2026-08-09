# Changelog

Semua perubahan penting pada proyek ini akan dicatat di file ini.

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/) dan project ini mengikuti [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2026-08-10

### Quality Overhaul — Formula Fixes, ML Honesty, Validation Framework

Versi ini berfokus pada **kualitas, kejujuran ilmiah, dan kepatuhan regulasi**, bukan penambahan fitur. Dua belas bug formula diperbaiki (masing-masing disertai self-check test), 15 tool ML dilabeli ulang secara jujur, 6 stub ditandai, 10 tool baru ditambahkan, dan framework validasi model baru dibangun.

### Added — +10 New Tools (323→333)

**Validation Framework (2 tools, `src/validation/`):**
- `validate_model` — Model validation metrics: RMSE, MAE, MBE, R², NSE (Nash-Sutcliffe), KGE (Kling-Gupta), PBIAS
- `validation_badge` — Quality badge (excellent/good/satisfactory/unsatisfactory) berdasarkan threshold Moriasi et al.

**Indonesia Gap Tools (5 tools):**
- `haze_trajectory` — Transboundary haze trajectory (karhutla antar-negara)
- `jakarta_coastal_risk` — Integrated coastal risk Jakarta (subsidence + SLR + banjir rob)
- `river_source_apportionment` — Citarum river source apportionment
- `coastal_erosion` — Pantura (Pantai Utara Jawa) coastal erosion
- `sanitation_impact` — BABS/STBM sanitation impact assessment

**Health & Restoration (2 tools):**
- `health_impact_assessment` (HIA) — CRF → DALYs → economic cost (PM2.5 mortality/morbidity)
- `restoration_cost` — Mangrove/peatland/river/mine/coral restoration + carbon BCR

**Workflow (1 tool):**
- `problem_solution_impact` — Workflow orchestrator (6 problem types, 3 phases: problem→solution→impact)

### Added — Validation Framework (`src/validation/`)
- Modul validasi model baru: RMSE, MAE, MBE, R², NSE, KGE, PBIAS
- `validation_badge` — klasifikasi kualitas berdasarkan threshold literatur (Moriasi et al.)
- MCP tool `validate_model` exposed untuk konsumsi LLM

### Changed — ML Honesty Labeling (15 tools)
Akurasi paper riset (R²=0.997, 99.03%, 95.6%, F1=96%) yang sebelumnya tersaji sebagai performa tool kini dipindahkan ke label **"Literature Reference (NOT this tool's performance)"**. Tool di-*rename* untuk mencerminkan implementasi sebenarnya:
- `pinn_water` → **Physics-Constrained Finite Difference** (bukan PINN)
- `wwtp_digital_twin` XGBoost → **heuristic sensitivity** (bukan XGBoost)
- `trigrs` CNN-LSTM → **logistic regression** (bukan CNN-LSTM)
- `microplastic_detect`, `flood_sar_mapping`, `fire_spread` → label honest
- (+9 tool lainnya dilabeli ulang)

### Changed — Stub Cleanup (6 tools)
Tool berikut ditandai **STUB/PLACEHOLDER** secara eksplisit (tidak over-claim fungsionalitas):
- `report_parser`, `brin_spacemap`, `workflows/air_dispersion`, `workflows/water_quality`, `satellite/peatland`, `planetary_computer`

### Fixed — Formula Bugs (12 bugs + 67 self-check tests)
Setiap perbaikan disertai **self-check test** (67 total test, semua pass):

| # | Tool | Bug | Fix |
|---|------|-----|-----|
| 1 | `reaeration_coefficient` | Velocity unit bukan m/s | Konversi ke m/s |
| 2 | `forest_carbon` | DBH unit bukan cm (Chave) | Konversi ke cm |
| 3 | `awd_ghg_calculator` | N2O double-count + IPCC 2019 EF1FR | Hapus double-count, pakai EF1FR |
| 4 | `pfas_transport` | `erfc` missing `e^(-x²)` term | Tambah `e^(-x²)` |
| 5 | `contaminant_transport_1d` | `erfc` missing `e^(-x²)` term | Tambah `e^(-x²)` |
| 6 | `contaminant_transport_2d` | `erfc` missing `e^(-x²)` term | Tambah `e^(-x²)` |
| 7 | `pfas_scwo` | Konversi ppb→ng/L salah | Koreksi ppb→ng/L |
| 8 | `pfas_electro_nf` | Defluorination 100× overstated | Koreksi faktor 100× |
| 9 | `pollution_index` | Nemerow RMS + DO inverse salah | Koreksi RMS & DO inverse |
| 10 | `cyclone` | d50 inlet width formula | Koreksi d50 |
| 11 | `pump_treat` | Javandel capture zone formula | Koreksi Javandel |
| 12 | `river_quality` | Sign error | Koreksi tanda |

### Fixed — Formula Bugs (lanjutan)
- `swe_solver` — HLL solver rewrite (Riemann flux)
- `buffer_capacity` — van't Hoff temperature correction

### Fixed — Crash Bugs
- `emp_generator` — UTF-8 panic (ditambah `safe_truncate`)
- `tcfd` — emoji di output (disesuaikan ke severity match Indonesia)
- `carbon.rs` — `\n` + price formatting
- `b3_storage` — empty `{}` response
- `coords` — UTM hemisphere detection (N/S)

### Fixed — Regulatory References
| Regulasi | Sebelum | Sesudah |
|----------|---------|---------|
| ISPU | PermenLHK 73/2019 | **PermenLHK 14/2020** |
| IKLH | P.14/2020 | Regulasi yang benar (bukan P.14/2020) |
| Air laut | KepMen 22/2021 | **KepMen 51/2004** |
| Baku mutu air | PP 82/2001 (dicabut) | **PP 22/2021** |
| EPA PFAS MCL | PFNA/PFHxS/GenX MCL | **Rescinded May 2025** (hanya PFOA/PFOS) |

### Verified — Standards & Papers
- **ISO 14001:2026** — diverifikasi nyata (dipublikasi 15 Apr 2026, iso.org/DNV/BSI)
- **Falakh 2026** — diverifikasi
- **Altarazi 2026** — diverifikasi

### Skipped
- **Fase C (Python ML bridge)** — ditunda: tidak ada data training untuk konteks Indonesia. Tidak dilakukan over-claim kemampuan ML.

### Changed — Umum
- **Tool count**: 323 → 333 MCP tools (+10)
- **Test count**: 36 → 108 ( semua pass)
- **Self-check tests**: 67 test formula baru (satu per formula yang diperbaiki)
- **Direktori baru**: `src/validation/` (validation framework)

### Quality Assurance
- ✅ Setiap formula yang diperbaiki disertai self-check test (67 test, semua pass)
- ✅ Framework validasi model (RMSE/MAE/MBE/R²/NSE/KGE/PBIAS + badge)
- ✅ ML honesty — 15 tool dilabeli ulang, tidak ada akurasi paper yang dipinjam
- ✅ 6 stub ditandai eksplisit (STUB/PLACEHOLDER)
- ✅ Referensi regulasi diverifikasi ke sumber primer

## [1.2.0] - 2026-08-09

### Added — Phase 1: +21 Tools (269→290)
- `contaminant_transport_1d` / `contaminant_transport_2d` — Advection-dispersion transport model
- `vapor_intrusion` — Vapor intrusion into buildings (Johnson & Ettinger)
- `river_quality_model` — Streeter-Phelps + nitrogen cycle
- `reaeration_coefficient` — 17 reaeration formulas (O'Connor, Churchill, Owens, etc.)
- `sediment_oxygen_demand` — SOD estimation
- `chlorophyll_a_prediction` — Eutrophication prediction
- `mbr_design` — Membrane Bioreactor design
- `sbr_design` — Sequencing Batch Reactor design
- `aop_design` — Advanced Oxidation Process (UV/H2O2, O3, Fenton)
- `nutrient_removal` — BNR (nitrification/denitrification, EBPR)
- `struvite_precipitation` — Phosphorus recovery
- `chlorine_demand` — Chlorine disinfection design
- `buffer_capacity` — Water buffer capacity
- `indoor_air_quality` — IAQ assessment
- `stack_height_gep` — GEP stack height (USEPA)
- `fugitive_dust_ap42` — AP-42 fugitive dust
- `pome_calculator` — Palm Oil Mill Effluent
- `mdl_calculator` — Method Detection Limit
- `holding_time_checker` — Sample preservation
- `calibration_verification` — Instrument calibration

### Added — 2026 Regulasi Update: +13 Tools (290→303)
- `baku_mutu_air_permukaan` — PP 22/2021 surface water quality standards
- `sanksi_administratif_lh` — UUPLH administrative sanctions
- `ndc_mrv_tracker` — NDC Measurement/Reporting/Verification
- `traffic_impact_andal` — Traffic impact assessment for AMDAL
- `mine_reclamation_plan` — Mine reclamation (Permen ESDM)
- `remediation_target` — Soil/groundwater remediation targets
- `oil_spill_response` — Oil spill response planning
- `aquaculture_waste_load` — Aquaculture waste load
- `carbon_stock_forest` — Forest carbon stock
- `carbon_registry` — Carbon credit registry
- `pesticide_runoff_risk` — Pesticide runoff risk
- `tailings_management` — Tailings management (OCED/ICMM)

### Added — Cutting-Edge 2026: +18 Emerging Tech Tools (303→321)
**PFAS Bundle (6 tools):**
- `pfas_transport` — SDEM transport model (Yaroshchuk 2016), Langmuir AWI (Brusseau 2025)
- `pfas_scwo` — Supercritical water oxidation, bond cleavage C-S>C-C>C-F (Prasetya 2025)
- `pfas_foam` — Foam fractionation, Langmuir AWI (ITRC 2025)
- `pfas_screening` — EPA MCL May 2025, 2026 treatment comparison
- `pfas_electro_nf` — Electro-NF, modified SDEM + external field (Hua 2026, J. HazMat 141395)

**AI/ML Bundle (6 tools):**
- `pinn_water` — Physics-Informed Neural Network for water quality
- `hybrid_physics_ml` — Hybrid physics-ML dispersion
- `pm_forecast` — PM2.5 forecasting
- `watershed_twin` — Watershed digital twin
- `wwtp_digital_twin` — WWTP twin, XGBoost+SHAP (Nourani 2025), LSTM-GRU (Xiong 2025), carbon reduction (Yun 2025)
- `ml_dispersion` — ML-augmented dispersion

**Remote Sensing Bundle (4 tools):**
- `blue_carbon_mrv` — InVEST 4-pool carbon (AGC/BGC/SOC/DOC), GBDT canopy (Liu 2026), RAP-CNN (Zhuang 2026), blockchain MRV (Hirlekar 2026)
- `edna_biodiversity` — 3-level Bayesian occupancy (ψ/θ/p), multi-marker (12S/16S/COI/18S) (Schütz 2025, Ivanova 2025)
- `tropomi_emission` — Sentinel-5P TROPOMI CH4
- `satellite_compliance` — Satellite-based compliance monitoring

**Emerging Bundle (3 tools):**
- `blockchain_credit` — Blockchain carbon credit
- `nano_treatment` — Nanotechnology water treatment
- `microplastic_detect` — CNN1D+AE 99.03% (Yan 2026), SERS+1D-CNN (Ma 2026), hyperspectral F1=0.963 (Nayani 2026)

### Added — Deep Retrofit: +2 New Tools (321→323)
- `fire_suppression` — Aerial intervention optimizer, water vs retardant, two-stage optimization (Matei 2026)
- `awd_ghg_calculator` — AWD GHG meta-analysis CH₄ -64.5%, N₂O +18.7%, GWP -42.1% (Rafy 2025, 47 studies), DNDC SSP (Tang 2025)

### Changed — Deep Retrofit 9 Domains (14 tools retrofitted with 2025-2026 research)
- **`fire_spread`** — Neural-CA 3-state (U/B/R), Poisson ignition, wind/slope kernels (Zhenirovskyy 2026), aleatoric+epistemic ILR UQ (Matei 2026, arXiv:2606.13633)
- **`edna_biodiversity`** — 3-level Bayesian occupancy (ψ/θ/p), multi-marker, replicate optimization (Schütz 2025), Bayesian+GAM abundance (Ivanova 2025)
- **`pfas_transport`** — SDEM model `j=-P(dC/dx+zC·dφ/dx)` (Yaroshchuk 2016), Langmuir AWI (Brusseau 2025), E-NF results (Hua 2026)
- **`pfas_scwo`** — Bond cleavage order C-S>C-C>C-F (Prasetya 2025), radical mechanisms, MAT/EDP/APPJ alternatives
- **`pfas_foam`** — Langmuir AWI mechanism, ITRC 2025, 2026 treatment comparison table
- **`pfas_screening`** — EPA MCL confirmed May 2025, 2026 treatment comparison, Indonesia context
- **`microplastic_detect`** — CNN1D+AE 99.03% (Yan 2026), SERS+1D-CNN (Ma 2026), hyperspectral F1=0.963 (Nayani 2026), electrochemical AUC=0.98
- **`trigrs`** — CNN-LSTM 95.6% (Teng 2026), SHAP XAI (Oh 2026), spatial CV vs R-CV bias 6-18% (Kumar 2025, 43 cit), GBDT thresholds
- **`flood_sar_mapping`** — 2026 SOTA DL: Siamese U-Net F1=96% (Kacmaz 2026), TLE-FEDformer 98.1%, LightFloodNet 1.57M params, CMFS-UNet Mamba mIoU=79.4%
- **`blue_carbon_mrv`** — InVEST 4-pool (AGC/BGC/SOC/DOC), GBDT canopy height (Liu 2026), RAP-CNN (Zhuang 2026), blockchain MRV (Hirlekar 2026)
- **`wwtp_digital_twin`** — XGBoost+SHAP (Nourani 2025, 34 cit), LSTM-GRU hybrid (Xiong 2025, 23 cit), carbon reduction 788 tCO₂/y (Yun 2025), TreeSHAP O(TLD²) algorithm

### Changed — Retrofit Phase 1 & 2 (12 tools with 2026 compliance footer)
- `river_quality`, `mbr_design`, `sbr_design`, `pome`, `contaminant_transport_1d/2d`, `fugitive_dust` — added 2026 compliance context
- `uasb_design`, `trickling_filter`, `constructed_wetland`, `anaerobic_digestion` — added 2026 compliance context
- **`baku_mutu_emisi`** — updated to Permen LH 5/2026
- **`baku_mutu_air_limbah`** — updated to Permen LH 12/2025
- **`baku_mutu_domestik`** — updated to Permen LH 11/2025

### Changed — Umum
- **Tool count**: 228 → 323 MCP tools (+95 tools)
- **Server.rs**: ~4214 → ~6230+ baris
- **Param structs**: 191 → ~280+ terdeskripsi otomatis
- **Kategori direktori**: 20 → 22 (tambah `emerging/`, `amdal/`)
- **File emerging**: 20 file (PFAS×6, AI/ML×6, remote sensing×4, emerging×4)
- **File calculators**: 79 file (termasuk `awd_ghg.rs`)
- **File advanced_physics**: 13 file (termasuk `fire_spread.rs`, `fire_suppression.rs`, `trigrs.rs`)
- **File satellite**: 20+ file (termasuk `flood_sar.rs` dengan 2026 DL methods)

### Scientific Compliance
- ✅ Setiap output retrofit menyertakan sitasi DOI/arXiv ID/paper sumber
- ✅ Kuantifikasi ketidakpastian: Monte Carlo (10,000+ iterasi), ILR transform, Bayesian credible interval, Beta distribution
- ✅ Batasan honest — "simplified, not actual CNN/MCMC" bila aproksimasi
- ✅ Tidak ada over-claim — hasil empiris dikaitkan dengan paper spesifik
- ✅ Konteks regulasi Indonesia — Permen LH 6/2026, 8/2026, 10/2026, 11/2025, 12/2025

### Commits
- `a15a9e7` — Phase 1: +21 tools (269→290)
- `0ef5264` — 2026 Regulasi Update: +13 tools (290→303)
- `30c1607` — Retrofit Phase 1: 6 tools + baku_mutu_domestik 11/2025
- `7257440` — Retrofit Phase 2: 6 tools + baku_mutu_emisi 5/2026, air_limbah 12/2025
- `432adb0` — Cutting-Edge 2026: +18 emerging tech tools (303→321)
- `73d39df` — Deep retrofit 9 domains with 2025-2026 research (Phase 1-5,7,9)
- `23918e9` — Deep retrofit Phase 3c+8: PFAS SCWO/foam/screening + WWTP TreeSHAP
- `1429b8a` — Deep retrofit Phase 6: flood_sar 2026 SOTA DL methods

## [1.1.0] - 2026-08-07

### Added
- **AMDAL Pipeline Engine** (`src/amdal_pipeline.rs`): registry 20 peta lingkungan, orkestrasi paralel via `rayon`, rendering PNG via `plotters`, output JSON manifest
- **7 kalkulator Rust** terintegrasi ke pipeline: Noise, Dispersion, Flood (SWE), Subsidence, Penman-Monteith, SCS-CN, Streeter-Phelps, RUSLE, Monte Carlo, Biodiversity
- **Engine enum** (Rust/Python/Hybrid) untuk mode eksekusi pipeline fleksibel
- **CLI flags** di `src/main.rs`: `--pipeline --lat --lon --buffer` dan `--test-tool [name] [json]` (4 tools ter-wire)
- **Inline test `result_contract`** di `main.rs`: validasi ScientificResult (nilai finite, uncertainty bounds, seed stokastik, fallback reason, stale source)
- **SSE strip proxy** (port 20129): strip `data: [DONE]` dari 9router agar kompatibel dengan ZeroClaw LLM provider
- **Dependensi baru**: `rayon = "1"`, `plotters = "0.3"` di Cargo.toml
- **Dokumentasi**: README.md lengkap (574 baris) — arsitektur, katalog 228 tools, konfigurasi ZeroClaw, DEMNAS, troubleshooting
- **Config ZeroClaw**: `tool_result_retrim_chars` 4000→512000, `keep_recent` 8→50, `tool_timeout_secs` 600→1800, provider URI 20128→20129
- **4 agen lingkungan** di ZeroClaw: `manager_amdal`, `gis_expert`, `physics_modeler`, `esg_auditor`

### Changed
- **Tool count**: 219 → 228 MCP tools (191 struct parameter terdeskripsi)
- **Batasan output tool** dinaikkan signifikan agar hasil besar (peta, JSON) tidak terpotong
- **Timeout tool MCP** dinaikkan hingga 30 menit untuk task berat (GEE, DEMNAS)
- **LLM provider** dialihkan via SSE strip proxy untuk stabilitas streaming

### Fixed
- **Bug pre-existing** `server.rs:2903`: `generate_4d_timelapse` missing 4 arguments
- **Telegram 409 conflict**: menghentikan & menonaktifkan 5 service non-lingkungan yang memakai token bot yang sama
- **DEMNAS 8m**: login verified, 4 tile terunduh (170MB) — fix deteksi tile + implementasi metode Horn's local untuk slope

### Removed
- **Non-environmental services** (~5GB): `zeroclaw-trader/`, `zeroclaw-saham/`, `zeroclaw-stockbit/`, `zeroclaw-agent/`, `crypto_research_papers.md`
- **Agent non-lingkungan** `kaorimiyazuno` dan cron `stock_analysis`, `crypto_analysis` dari config ZeroClaw

### Tested
- 153/154 tools PASS (99.4%) via MCP stdio batch test
- 1 FAIL: `ndvi_timeseries` — timeout GEE API > 60s (diharapkan, API lambat)
- Daemon end-to-end verified: Telegram → daemon → LLM (3s) → reply, zero errors

## [1.0.0] - 2026-07-05

### Added
- 219 MCP tools covering 20 environmental engineering domains
- 100% Indonesia domain lock (38 provinces, coordinate validation)
- 63 BMKG city codes verified
- GEE integration for GIS/RS (Dynamic World, SAR, Sentinel-2)
- AMDAL document generation (PDF)
- 30+ Indonesian environmental regulations embedded
- ARKL Indonesia with Kemenkes 2012 defaults
- Nearest coral reef and MPA spatial query
- Error code integration [E101]-[E502]
