# Changelog

Semua perubahan penting pada proyek ini akan dicatat di file ini.

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/) dan project ini mengikuti [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
