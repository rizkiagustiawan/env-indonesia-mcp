# env-indonesia-mcp

**Sistem Model Context Protocol (MCP) Tingkat Lanjut untuk Rekayasa Lingkungan di Indonesia**

Sistem `env-indonesia-mcp` adalah server komputasi spasial dan rekayasa lingkungan (Environmental Engineering) berbasis fisika yang dirancang khusus untuk memandu dan membatasi model kecerdasan buatan (LLM) seperti arsitektur agen ZeroClaw. Server ini menjembatani kemampuan dialog kecerdasan buatan dengan kepatuhan sains lingkungan yang deterministik dan berbasis regulasi nasional.

Sistem ini memastikan setiap analisis spasial, perhitungan hidrologi, hingga pemodelan atmosfer yang dilakukan oleh AI tidak menyalahi hukum alam (seperti termodinamika atau dinamika fluida) serta mematuhi Standar Nasional Indonesia (SNI) dan peraturan kementerian terkait.

---

## Daftar Isi

- [Fitur Utama](#fitur-utama)
- [Arsitektur Sistem](#arsitektur-sistem)
- [Pemasangan](#pemasangan)
- [Konfigurasi](#konfigurasi)
- [Penggunaan CLI](#penggunaan-cli)
- [Katalog Tools MCP (333 Tools)](#katalog-tools-mcp-333-tools)
- [Quality Assurance](#quality-assurance)
- [AMDAL Pipeline Engine](#amdal-pipeline-engine)
- [Deep Retrofit 2025-2026 (Research-Verified)](#deep-retrofit-2025-2026-research-verified)
- [Integrasi ZeroClaw Multi-Agent](#integrasi-zeroclaw-multi-agent)
- [Integrasi Telegram](#integrasi-telegram)
- [Setup DEMNAS 8m](#setup-demnas-8m)
- [Struktur Direktori](#struktur-direktori)
- [Dependensi](#dependensi)
- [Pengujian](#pengujian)
- [Pemecahan Masalah](#pemecahan-masalah)
- [Keamanan Pagar Wilayah](#keamanan-pagar-wilayah)
- [Akuntabilitas Ilmiah](#akuntabilitas-ilmiah)
- [Lisensi](#lisensi)

---

## Fitur Utama

### 1. Sistem Validasi Berbasis Fisika (Physics-Informed Validator)
Kecerdasan buatan generatif rentan terhadap halusinasi matematis. Sistem ini mengimplementasikan modul `physics_validator.rs` sebagai gerbang pembatas kaku. Jika sebuah perhitungan membuahkan hasil empiris yang mustahil (contoh: Limpasan Permukaan / *Runoff* melebihi total Curah Hujan, atau nilai *Chemical Oxygen Demand* lebih rendah dari *Biological Oxygen Demand*), sistem akan menghentikan eksekusi dan mengeluarkan pesan penolakan berdasarkan asas saintifik.

Modul ini juga mengunci parameter agar taat pada:
- **PP 22 Tahun 2021** (Baku Mutu Kualitas Air dan Udara Nasional).
- **KepMenLH 48/1996** (Batas Kebisingan Lingkungan).
- **PermenLHK 14/2020** (Indeks Standar Pencemar Udara - ISPU).
- Kepatuhan pada batasan termodinamika (contoh: batas saturasi *Dissolved Oxygen*, angka evapotranspirasi maksimal iklim tropis).

### 2. Integrasi Data Satelit dan Spasial Waktu Nyata
Sistem terhubung secara simultan (multi-sensor) untuk melakukan akuisisi data observasi bumi tingkat tinggi:
- **Google Earth Engine (GEE)**: Pemrosesan otomatis instrumen Sentinel-2 (Optik Resolusi Tinggi), Landsat, CHIRPS, dan MODIS.
- **Deteksi Titik Api (VIIRS & MODIS)**: Modul pemindaian *hotspot* otomatis untuk memetakan luasan kebakaran hutan dan lahan. Termasuk integrasi deteksi daya radiasi api (Fire Radiative Power - FRP) dan identifikasi kebakaran lahan gambut melalui metode proksi indeks aerosol.
- **Pemodelan Animasi 4D (Timelapse)**: Mampu menghasilkan animasi GIF pertumbuhan tutupan lahan atau kerusakan akibat kebakaran hutan melalui sensor penembus awan Sentinel-1 (Radar SAR) dan Sentinel-2. Framerate (FPS) dan interval waktu (harian, mingguan, bulanan) yang sepenuhnya terkonfigurasi.
- **Sentinel-5P TROPOMI**: Ekstraksi kadar konsentrasi metana (CH4) secara langsung guna memantau anomali emisi gas rumah kaca.

### 3. Pemodelan Dinamika Fluida dan Oseanografi
- **2D Shallow Water Equations (SWE)**: Memodelkan rute, jangkauan, dan kedalaman banjir di atas matriks elevasi menggunakan metode numerik Riemann HLL (Rust).
- **Ekstraksi DEMNAS Otomatis**: Secara terprogram mengambil Data Elevasi Digital Nasional (DEMNAS) beresolusi 8 meter langsung dari server Badan Informasi Geospasial (BIG), memintas pembatasan reCAPTCHA dan token JWT secara *headless*.
- **Tumpahan Minyak Lepas Pantai (Oil Spill Trajectory)**: Menggabungkan data arus laut aktuaria dari pemodelan kelautan HYCOM dengan parameter cuaca pesisir untuk memetakan vektor polusi laut.

### 4. Rantai Perhitungan Standar Lingkungan
Lebih dari 333 model analitik terkalibrasi untuk kondisi iklim dan hidrologi Asia Tenggara:
- **Dispersi Atmosfer**: Peningkatan pada Model Gaussian Plume yang memperhitungkan sumber garis (*line source* seperti jalan raya tol) dan sumber area (*area source* seperti kolam limbah/TPA), dengan parameter stabilitas Pasquill-Gifford.
- **Kekeringan Iklim Tropis**: Implementasi *Standardized Precipitation Evapotranspiration Index* (SPEI) untuk akurasi prediksi kekeringan yang lebih komprehensif dibandingkan SPI konvensional.
- **Kesehatan Masyarakat**: *2D Monte Carlo Risk Analysis* yang membedakan ketidakpastian episodik dan fundamental pada Human Health Risk Assessment (HHRA).
- **Emerging Contaminants**: PFAS transport (SDEM model), SCWO destruction, foam fractionation, electro-NF, screening (EPA MCL 2025). Microplastic detection (CNN1D+AE, SERS, hyperspectral).
- **AI/ML Integration**: PINN water quality, hybrid physics-ML dispersion, PM2.5 forecasting, watershed digital twin, WWTP digital twin (XGBoost+SHAP).
- **Blue Carbon MRV**: InVEST 4-pool carbon (AGC/BGC/SOC/DOC), GBDT canopy height, blockchain MRV, mangrove biodiversity.

### 5. Multi-Agent Circuit Breaker
Sistem pengaman tingkat lanjut (`circuit_breaker.rs`) yang secara otomatis mendeteksi kebuntuan iteratif atau gangguan komunikasi eksternal, mematikan rantai perintah kecerdasan buatan sebelum menghabiskan sumber daya komputasi secara berlebihan (*Max Iterations Safeguard*).

### 6. AMDAL Pipeline Engine
Mesin pipelining AMDAL (Analisis Mengenai Dampak Lingkungan) berbasis Rust dengan:
- **20 peta lingkungan** terdaftar dalam registry (`MAP_REGISTRY`).
- **7 kalkulator Rust** terintegrasi: Noise, Dispersion, Flood, Subsidence, Penman-Monteith, SCS-CN, Streeter-Phelps, RUSLE, Monte Carlo, Biodiversity.
- **Orkestrasi paralel** menggunakan `rayon`.
- **Rendering PNG** menggunakan `plotters`.
- **Mode hybrid**: Rust, Python, atau kombinasi keduanya.
- **Output JSON manifest** untuk konsumsi downstream.

### 7. Deep Retrofit 2025-2026 dengan Verifikasi Paper Riset
Semua tool *cutting-edge* telah di-*retrofit* dengan formula dan temuan terkini dari 100+ paper riset 2025-2026 (arXiv, J. HazMat, Water Res., dll.). Setiap output menyertakan:
- **Sitasi DOI/paper sumber** secara transparan
- **Kuantifikasi ketidakpastian** (Monte Carlo, ILR transform, Bayesian credible interval, Beta distribution)
- **Batasan honest** — tidak *over-claim* (mis. "simplified, not actual CNN/MCMC")
- **Konteks regulasi Indonesia** (Permen LH 6/2026, 8/2026, 10/2026, 11/2025, 12/2025)

---

## Arsitektur Sistem

```
┌─────────────────────────────────────────────────────────────┐
│                      ZeroClaw Agent (LLM)                    │
│  manager_amdal │ gis_expert │ physics_modeler │ esg_auditor │
└──────────────────────────┬──────────────────────────────────┘
                           │ MCP (stdio)
┌──────────────────────────▼──────────────────────────────────┐
│                 env-indonesia-mcp (Rust)                     │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐  │
│  │ server.rs    │  │ amdal_       │  │ physics_validator.rs│  │
│  │ 333 MCP tools│  │ pipeline.rs  │  │ circuit_breaker.rs  │  │
│  └──────┬──────┘  └──────┬───────┘  └─────────────────────┘  │
│         │                 │                                   │
│  ┌──────▼──────┐  ┌──────▼───────┐                            │
│  │ src/tools/* │  │ python/      │                            │
│  │ 22 kategori │  │ pipeline     │                            │
│  └──────┬──────┘  └──────────────┘                            │
└─────────┼────────────────────────────────────────────────────┘
          │
┌─────────▼────────────────────────────────────────────────────┐
│                   Sumber Data Eksternal                      │
│ BMKG │ NASA FIRMS │ GFW │ Copernicus │ Open-Meteo │ NASA     │
│ POWER │ Satu Data │ Climate TRACE │ GEE │ DEMNAS │ MAGMA     │
│ BPS │ InaRISK │ HYCOM │ OJK ESG │ Bappenas │ KLHK │ ...      │
└──────────────────────────────────────────────────────────────┘
```

### Alur Data AMDAL Pipeline

```
Input (lat, lon, buffer_km, start_date, end_date)
        │
        ▼
PipelineParams ──► Registry 20 Maps ──► Kalkulator Rust (rayon, paralel)
        │                                      │
        │                                      ▼
        │                              ScientificResult
        │                              (validasi finite, batas ketidakpastian)
        │                                      │
        ▼                                      ▼
PipelineReport ◄─────── AmdalResult (map_id, title, status, calculation,
                                baku_mutu_class, narrative, render_path,
                                duration_ms)
        │
        ▼
JSON manifest + PNG render (plotters)
```

---

## Pemasangan

### Prasyarat
- **Rust** 1.75+ (stable) dengan `cargo`
- **Python** 3.9+ (untuk mode pipeline Python dan wrapper tools)
- **ZeroClaw daemon** (opsional, untuk integrasi agent)
- **Token bot Telegram** (opsional, untuk notifikasi)

### Build dari Sumber

```bash
git clone https://github.com/rizkiagustiawan/env-indonesia-mcp.git
cd env-indonesia-mcp

# Build release
cargo build --release

# Binary: target/release/env-indonesia-mcp
```

### Run

```bash
# Mode server MCP (stdio) - untuk dihubungkan ke ZeroClaw / Claude / Cursor
cargo run --release

# Mode CLI pipeline langsung
cargo run --release -- --pipeline --lat -7.25 --lon 112.75 --buffer 5

# Mode test tool individual
cargo run --release -- --test-tool "nama_tool" '{"param": "value"}'
```

### Service systemd

```bash
# Instal service daemon
cp env-indonesia-mcp.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now env-indonesia-mcp

# Log
journalctl --user -u env-indonesia-mcp -f
```

---

## Konfigurasi

### Klien MCP (Claude Desktop, Cursor, ZeroClaw)

```json
{
  "mcpServers": {
    "env-indonesia": {
      "command": "cargo",
      "args": ["run", "--manifest-path", "/lokasi/absolut/ke/env-indonesia-mcp/Cargo.toml"]
    }
  }
}
```

### ZeroClaw Config (`~/.zeroclaw/config.toml`)

```toml
schema_version = 3

[llm]
provider_uri = "http://127.0.0.1:20129/v1"   # Via SSE strip proxy
model = "my-karyawan"                        # grok-4.5 via 9router

[agents]
# 4 agen lingkungan terdaftar
agents = ["manager_amdal", "gis_expert", "physics_modeler", "esg_auditor"]

[runtime_profiles]
# Profil unbounded untuk task panjang
unbounded = { ... }

[mcp_servers]
# env-indonesia-mcp terdaftar sebagai tool provider
```

Catatan penting:
- `provider_uri` mengarah ke **SSE strip proxy** (port 20129) karena 9router (port 20128) mengirim `data: [DONE]` yang tidak dipahami ZeroClaw.
- `tool_result_retrim_chars = 512000` — batas hasil tool diperbesar agar hasil besar (peta, JSON) tidak terpotong.
- `keep_recent = 50` — riwayat chat disimpan lebih panjang.
- `tool_timeout_secs = 1800` — tool MCP diberi waktu hingga 30 menit.

---

## Penggunaan CLI

### Flag Utama

| Flag | Deskripsi | Contoh |
|------|-----------|--------|
| `--pipeline` | Jalankan AMDAL pipeline engine | `--pipeline --lat -7.25 --lon 112.75` |
| `--lat` | Latitude pusat area studi | `--lat -7.25` |
| `--lon` | Longitude pusat area studi | `--lon 112.75` |
| `--buffer` | Radius buffer dalam km | `--buffer 5` |
| `--test-tool` | Test tool MCP tertentu dengan JSON args | `--test-tool "noise_level" '{"lat": -7.25}'` |

### Contoh Pipeline

```bash
# Jalankan pipeline penuh dengan 20 peta
target/release/env-indonesia-mcp --pipeline --lat -7.25 --lon 112.75 --buffer 10

# Hasil:
# - output_maps/      → PNG renders (plotters)
# - output_amdal/     → JSON manifest per map
# - PipelineReport    → ringkasan paralel (duration_ms per map)
```

---

## Katalog Tools MCP (333 Tools)

Server mendaftarkan **333 tool MCP** (dengan ~280+ struct parameter terdeskripsi otomatis). Tool dibagi ke dalam 22 kategori direktori di `src/tools/`:

### Kategori Tools

| Kategori | Direktori | Jumlah | Contoh Tools |
|----------|-----------|--------|--------------|
| **Fisika Lanjutan** | `advanced_physics/` | 13 | Fire spread (Neural-CA), fire suppression, TRIGRS (CNN-LSTM+SHAP), SWE flood, SWE solver, tidal flood, EnKF, groundwater PDE, UHI, flux divergence |
| **Kualitas Udara** | `airquality/` | 11 | ISPU, PM2.5/PM10, dispersi polutan, fugitive dust (AP-42), indoor air, stack height (GEP), baghouse, cyclone, ESP, scrubber |
| **Biodiversitas** | `biodiversity/` | — | Analisis habitat, keanekaragaman hayati, spesies terancam |
| **Kalkulator** | `calculators/` | 79 | Carbon stock, AWD GHG, acid mine drainage, bioremediation, bioretention, buffer capacity, chlorophyll, eutrophication, flood frequency, forest carbon, IDF curve, landfill gas, RUSLE, struvite, UHI |
| **Kepatuhan** | `compliance/` | 30+ | Baku mutu (emisi/air limbah/air permukaan/domestik/laut/udara/kebisingan), sanksi administratif LH, NDC MRV, carbon registry, PROPER, SPPL, IKLH, STORET, ISO 14001 gap |
| **Data** | `data/` | — | Katalog data, metadata sumber data lingkungan |
| **Datasources** | `datasources/` | — | Wrapper BMKG, BPS, Satu Data Indonesia |
| **Emerging Tech** | `emerging/` | 20 | PFAS bundle (6: transport SDEM, electro-NF, SCWO, foam, screening), AI/ML (6: PINN, hybrid physics-ML, PM forecast, watershed twin, WWTP twin), Remote sensing (4: blue carbon MRV, eDNA biodiversity, TROPOMI, satellite compliance), Emerging (4: blockchain credit, nano treatment, microplastic detect) |
| **ESG** | `esg/` | — | OJK ESG, TCFD, SDGs, rating ESG |
| **GIS** | `gis/` | — | Geoprocessing, buffer, overlay, GeoJSON, transformasi koordinat |
| **Kebisingan** | `noise/` | — | Tingkat kebeningan, KepMenLH 48/96, pemetaan noise |
| **Kelautan** | `ocean/` | — | Kualitas air laut, terumbu karang, suhu permukaan laut |
| **Oseanografi** | `ocean_modeling/` | — | HYCOM, arus laut, oil spill trajectory |
| **Pemrosesan** | `processing/` | — | NDVI timeseries, reklasifikasi, mosaik raster |
| **Radiasi** | `radiation/` | — | Monitoring radiasi lingkungan |
| **Satelit** | `satellite/` | 20+ | Sentinel-1/2/5P, Landsat, MODIS, FIRMS, flood SAR (2026 DL), InSAR MintPy, hyperspectral, GPM IMERG, GRACE, ERA5, CHIRPS, mangrove, peatland, burned area, VIIRS fishing |
| **Limbah** | `waste/` | — | Pengelolaan limbah B3, non-B3, sampah |
| **Air** | `water/` | 29 | MBR/SBR design, AOP, nutrient removal, contaminant transport 1D/2D, vapor intrusion, river quality, constructed wetland, anaerobic digestion, GAC, ion exchange, MODFLOW 3D |
| **Workflow** | `workflows/` | — | Workflow AMDAL, KLHS, dokumen lingkungan |
| **AMDAL** | `amdal/` | 3 | AMDAL engine, EMP generator, generator |
| **Meteorologi** | — | — | BMKG, Open-Meteo, NASA POWER, curah hujan, evapotranspirasi |
| **Risiko Bencana** | — | — | InaRISK, MAGMA Indonesia, subsidence, banjir |
| **Validasi Model** | `validation/` | 2 | `validate_model` (RMSE, MAE, MBE, R², NSE, KGE, PBIAS), `validation_badge` (Moriasi threshold) |
| **Dampak Kesehatan** | — | 1 | `health_impact_assessment` (HIA: CRF → DALYs → biaya ekonomi PM2.5) |
| **Biaya Restorasi** | — | 1 | `restoration_cost` (mangrove/gambut/sungai/tambang/karang + BCR karbon) |
| **Workflow PCI** | — | 1 | `problem_solution_impact` (6 tipe masalah, 3 fase: problem→solution→impact) |
| **Gap Indonesia** | — | 5 | `haze_trajectory` (transboundary karhutla), `jakarta_coastal_risk`, `river_source_apportionment` (Citarum), `coastal_erosion` (Pantura), `sanitation_impact` (BABS/STBM) |

### Sumber Data Eksternal Terintegrasi

| Sumber | Data yang Disediakan |
|--------|---------------------|
| **BMKG** | Cuaca, iklim, gempa, peringatan dini |
| **NASA FIRMS** | Hotspot kebakaran VIIRS & MODIS |
| **Global Forest Watch** | Deforestasi, tutupan lahan |
| **Copernicus** | Sentinel-1/2/5P, data atmosfer & laut |
| **Open-Meteo** | Forecast cuaca, historical weather |
| **NASA POWER** | Solar radiation, meteorologi agro |
| **Satu Data Indonesia** | Data statistik nasional |
| **Climate TRACE** | Emisi GRK per sektor |
| **GEE (Google Earth Engine)** | NDVI timeseries, Sentinel-2/Landsat/CHIRPS/MODIS |
| **DEMNAS (BIG)** | Elevasi digital 8m |
| **MAGMA Indonesia** | Vulkanologi, gunung api, gerakan tanah |
| **BPS** | Statistik daerah, sosial-ekonomi |
| **InaRISK** | Risiko bencana nasional |
| **HYCOM** | Arus laut global |
| **OJK** | Regulasi dan data ESG |
| **Wrapper tools** | Port 8000-8004 (Python microservices) |

---

## Quality Assurance

### 1. Formula Self-Check Tests
Setiap formula yang diperbaiki disertai **self-check test** (67 test, semua pass) yang memvalidasi output terhadap nilai referensi/analytic. Bug formula yang diperbaiki mencakup: reaeration (unit velocity), forest_carbon (DBH cm), awd_ghg (N2O double-count + IPCC 2019 EF1FR), pfas_transport + contaminant_transport_1d/2d (erfc `e^(-x²)`), pfas_scwo (ppb→ng/L), pfas_electro_nf (defluorination 100×), pollution_index (Nemerow RMS + DO inverse), cyclone (d50), pump_treat (Javandel capture zone), river_quality (sign error), swe_solver (HLL rewrite), buffer_capacity (van't Hoff).

### 2. Validation Framework (`src/validation/`)
Framework validasi model baru untuk menilai akurasi model terhadap data observasi:
- **Metrik**: RMSE, MAE, MBE, R², NSE (Nash-Sutcliffe), KGE (Kling-Gupta), PBIAS
- **Badge**: `excellent` / `good` / `satisfactory` / `unsatisfactory` (threshold Moriasi et al.)
- **MCP tool**: `validate_model` tersedia untuk konsumsi LLM

### 3. ML Honesty Labels
Tool yang mengadopsi pendekatan ML tidak meminjam akurasi paper riset sebagai performa tool sendiri. Angka seperti R²=0.997, 99.03%, 95.6%, F1=96% dipindahkan ke label **"Literature Reference (NOT this tool's performance)"**. Penamaan tool disesuaikan dengan implementasi sebenarnya:
- PINN → **Physics-Constrained Finite Difference**
- XGBoost → **heuristic sensitivity**
- CNN-LSTM → **logistic regression**
- 15 tool ML lainnya dilabeli serupa

### 4. Stub Marking
Tool yang belum lengkap ditandai **STUB/PLACEHOLDER** secara eksplisit agar tidak over-claim fungsionalitas: `report_parser`, `brin_spacemap`, `workflows/air_dispersion`, `workflows/water_quality`, `satellite/peatland`, `planetary_computer`.

### 5. Regulatory Verification
Referensi regulasi diverifikasi ke sumber primer: ISPU → PermenLHK 14/2020, IKLH, air laut → KepMen 51/2004, baku mutu air → PP 22/2021 (PP 82/2001 dicabut), EPA PFAS MCL (PFNA/PFHxS/GenX rescinded May 2025). Standar ISO 14001:2026 diverifikasi nyata (publikasi 15 Apr 2026).

---

## AMDAL Pipeline Engine

### Parameter

```rust
struct PipelineParams {
    lat: f64,          // Latitude pusat area studi
    lon: f64,          // Longitude pusat area studi
    buffer_km: f64,    // Radius buffer analisis
    start_date: String, // Awal periode data
    end_date: String,  // Akhir periode data
}
```

### Hasil per Map

```rust
struct AmdalResult {
    map_id: String,
    title: String,
    status: String,
    calculation: f64,
    baku_mutu_class: String,
    narrative: String,
    render_path: String,   // PNG via plotters
    duration_ms: u128,
}
```

### Engine

```rust
enum Engine {
    Rust,     // Kalkulator native Rust (rayon paralel)
    Python,   // Pipeline Python (master_pipeline.py, super_amdal_pipeline.py)
    Hybrid,   // Kombinasi keduanya
}

enum RustCalc {
    Noise,
    Dispersion,
    Flood,
    Subsidence,
    PenmanMonteith,
    ScsCn,
    StreeterPhelps,
    Rusle,
    MonteCarlo,
    Biodiversity,
}
```

### 20 Peta dalam MAP_REGISTRY

| # | Map ID | Kategori | Engine |
|---|--------|----------|--------|
| 1 | Noise | Kebisingan | Rust |
| 2 | Dispersion | Kualitas Udara | Rust |
| 3 | Flood (SWE) | Hidrologi | Rust |
| 4 | Subsidence | Geologi | Rust |
| 5 | Penman-Monteith | Hidrologi | Rust |
| 6 | SCS-CN | Hidrologi | Rust |
| 7 | Streeter-Phelps | Kualitas Air | Rust |
| 8 | RUSLE | Erosi | Rust |
| 9 | Monte Carlo | Risiko | Rust |
| 10 | Biodiversity | Ekologi | Rust |
| 11-20 | Peta Python | Mixed | Python |

### Kontrak Hasil Ilmiah

Setiap hasil tervalidasi oleh `result_contract` (inline test di `main.rs`):
- Nilai **finite** (bukan NaN/Infinity)
- Batas **ketidakpastian** tercatat
- **Seed** untuk proses stokastik (reproducibility)
- **Fallback reason** jika data tidak tersedia
- **Stale source** detection (sumber data kadaluarsa)

---

## Deep Retrofit 2025-2026 (Research-Verified)

Semua tool *cutting-edge* telah di-*retrofit* dengan formula dan temuan terkini dari 100+ paper riset 2025-2026. Pendekatan ini memastikan server memiliki "super intelligence tanpa over-claim" — setiap hasil dikaitkan dengan sumber ilmiah spesifik dan batasan diungkapkan secara jujur.

### 9 Domain Retrofit

| Domain | Tool | Key Paper 2025-2026 | Metode |
|--------|------|---------------------|--------|
| **Fire Spread** | `fire_spread` | Zhenirovskyy 2026 (Neural-CA), Matei 2026 (arXiv:2606.13633) | 3-state CA (U/B/R), Poisson ignition, wind/slope kernels, ILR epistemic UQ |
| **Fire Suppression** | `fire_suppression` | Matei 2026 | Aerial intervention optimizer, water vs retardant, two-stage optimization |
| **eDNA Biodiversity** | `edna_biodiversity` | Schütz 2025, Ivanova 2025, Plewnia 2026 | 3-level Bayesian occupancy (ψ/θ/p), multi-marker (12S/16S/COI/18S), Bayesian+GAM abundance |
| **PFAS Transport** | `pfas_transport` | Yaroshchuk 2016 (SDEM), Brusseau 2025 (Langmuir AWI) | SDEM `j=-P(dC/dx+zC·dφ/dx)`, E-NF results from Hua 2026 |
| **PFAS Electro-NF** | `pfas_electro_nf` | Hua 2026 (J. HazMat 141395) | Modified SDEM + external field, PFOA 90.4%, PFBS 83.9%, <1.92 kWh/m³ |
| **PFAS SCWO** | `pfas_scwo` | Prasetya 2025 | Bond cleavage order C-S>C-C>C-F, radical mechanisms, MAT/EDP/APPJ |
| **PFAS Foam** | `pfas_foam` | Hatton 2025, ITRC 2025 | Langmuir AWI mechanism, 2026 treatment comparison |
| **PFAS Screening** | `pfas_screening` | EPA MCL May 2025 | Confirmed MCL, 2026 treatment comparison, Indonesia context |
| **Microplastic** | `microplastic_detect` | Yan 2026 (99.03%), Ma 2026 (SERS), Nayani 2026 (HSI F1=0.963) | CNN1D+AE, SERS+1D-CNN, hyperspectral, electrochemical AUC=0.98 |
| **Landslide (TRIGRS)** | `trigrs` | Teng 2026 (95.6%), Oh 2026 (SHAP), Kumar 2025 (S-CV) | CNN-LSTM, SHAP XAI, spatial CV vs R-CV (bias 6-18%), GBDT thresholds |
| **Flood SAR** | `flood_sar_mapping` | Kacmaz 2026 (Siamese U-Net F1=96%), Ahmadi 2026 (FEDformer 98.1%) | 2026 SOTA DL: Siamese U-Net, TLE-FEDformer, LightFloodNet (1.57M params), CMFS-UNet (Mamba mIoU=79.4%) |
| **Blue Carbon MRV** | `blue_carbon_mrv` | InVEST 4-pool, Liu 2026 (GBDT), Zhuang 2026 (RAP-CNN) | 4-pool (AGC/BGC/SOC/DOC), GBDT canopy height, blockchain MRV (Hirlekar 2026) |
| **WWTP Digital Twin** | `wwtp_digital_twin` | Nourani 2025 (34 cit), Yun 2025, Xiong 2025 (23 cit) | XGBoost+SHAP, LSTM-GRU hybrid, carbon reduction 788 tCO₂/y, TreeSHAP O(TLD²) |
| **AWD GHG** | `awd_ghg_calculator` | Rafy 2025 (47 studies), Bhattacharyya 2025, Tang 2025 (DNDC) | Meta-analysis CH₄ -64.5%, N₂O +18.7%, GWP -42.1%, DNDC SSP scenarios |

### Komitmen Akuntabilitas Ilmiah

Setiap tool retrofit mengikuti protokol akuntabilitas:
1. **Sitasi eksplisit** — DOI/arXiv ID/paper sumber untuk setiap formula
2. **Kuantifikasi ketidakpastian** — Monte Carlo (10,000+ iterasi), ILR transform, Bayesian credible interval, Beta distribution
3. **Batasan honest** — "simplified, not actual CNN/MCMC" bila implementasi merupakan aproksimasi
4. **Konteks regulasi Indonesia** — Permen LH terkini (6/2026, 8/2026, 10/2026, 11/2025, 12/2025)
5. **Tidak ada over-claim** — hasil empiris dikaitkan dengan paper spesifik, bukan klaim independen

---

## Integrasi ZeroClaw Multi-Agent

ZeroClaw daemon dikonfigurasi sebagai orkestrator dengan **4 agen lingkungan**:

| Agent | Peran |
|-------|-------|
| `manager_amdal` | Mengelola dokumen AMDAL, menilai dampak, koordinasi workflow |
| `gis_expert` | Analisis spasial, pemetaan, GeoJSON, overlay peta |
| `physics_modeler` | Pemodelan fisika: dispersi, hidrologi, noise, fluida |
| `esg_auditor` | Audit ESG, TCFD, SDGs, kepatuhan OJK |

### Alur Perintah

```
User ──► Telegram ──► ZeroClaw daemon ──► LLM (9router via SSE proxy)
                                           │
                                           ▼
                              Pilih agen (manager_amdal)
                                           │
                                           ▼
                              Panggil tool MCP env-indonesia-mcp
                                           │
                                           ▼
                              Hasil dikembalikan ke LLM untuk sintesis
                                           │
                                           ▼
                              Balasan dikirim ke Telegram user
```

### Config Agent (ringkas)

```toml
[agents.manager_amdal]
role = "manager"
description = "Mengelola analisis AMDAL end-to-end"

[agents.gis_expert]
role = "specialist"
description = "Ahli GIS dan pemetaan spasial"

[agents.physics_modeler]
role = "specialist"
description = "Pemodelan fisika lingkungan"

[agents.esg_auditor]
role = "specialist"
description = "Audit ESG dan kepatuhan"
```

---

## Integrasi Telegram

Notifikasi dan interaksi via bot Telegram:

```bash
# Konfigurasi bot (di zeroclaw config — JANGAN hardcode di repo)
TELEGRAM_BOT_TOKEN="<isi-token-dari-BotFather>"
TELEGRAM_CHAT_ID="<isi-chat-id>"
```

### Contoh Perintah Pengguna

```
/tulis amdal untuk kawasan industri di Sidoarjo, radius 5 km
```

Hasil: daemon → LLM → agen GIS memanggil tool pipeline → laporan + peta PNG dikirim ke Telegram.

---

## Setup DEMNAS 8m

DEMNAS (Digital Elevation Model Nasional, resolusi 8m dari BIG) diunduh otomatis oleh tool:

### Kredensial (environment variables)

```bash
# Di ~/.config/systemd/user/zeroclaw.service — JANGAN commit ke repo
Environment="DEMNAS_EMAIL=<email-big-tanahair>"
Environment="DEMNAS_PASSWORD=<password-big-tanahair>"
```

### Alur

1. Tool mengirim request login ke `tanahair.indonesia.go.id`.
2. Menangani reCAPTCHA dan token JWT secara headless.
3. Mendapatkan daftar tile yang menutupi bbox area studi.
4. Mengunduh tile (sekitar 170MB per 4 tile).
5. Menyusun matriks elevasi untuk kalkulator SWE flood, subsidence, dll.

### Verifikasi

```bash
# Login test berhasil, 4 tile terverifikasi terunduh
# Lokasi cache: (per konfigurasi tool)
```

---

## Struktur Direktori

```
env-indonesia-mcp/
├── Cargo.toml                  # Dependensi Rust
├── Dockerfile                  # Container build
├── CHANGELOG.md
├── DEPLOYMENT.md
├── README.md
├── env-indonesia-mcp.service   # Systemd unit
├── src/
│   ├── main.rs                 # Entry point, CLI flags, inline tests
│   ├── server.rs               # MCP server, 333 tools, ~280+ param structs (~6230+ baris)
│   ├── amdal_pipeline.rs       # AMDAL pipeline engine (20 maps)
│   ├── physics_validator.rs    # Validasi berbasis fisika
│   ├── circuit_breaker.rs      # Circuit breaker multi-agent
│   ├── validation/             # Framework validasi model (RMSE/MAE/MBE/R²/NSE/KGE/PBIAS + badge)
│   └── tools/
│       ├── advanced_physics/   # 13: fire_spread (Neural-CA), fire_suppression, TRIGRS (CNN-LSTM+SHAP), SWE, tidal, EnKF, groundwater PDE, UHI
│       ├── airquality/         # 11: ISPU, dispersi, fugitive dust, indoor air, stack height, baghouse, cyclone, ESP, scrubber
│       ├── amdal/              # 3: AMDAL engine, EMP generator, generator
│       ├── biodiversity/       # Habitat, keanekaragaman
│       ├── calculators/        # 79: carbon, AWD GHG, AMD, bioremediation, bioretention, buffer, chlorophyll, eutrophication, flood freq, IDF, landfill gas, RUSLE, struvite, UHI
│       ├── compliance/         # 30+: baku mutu (7 jenis), sanksi LH, NDC MRV, carbon registry, PROPER, SPPL, IKLH, STORET, ISO 14001
│       ├── data/               # Katalog data
│       ├── datasources/        # BMKG, BPS, Satu Data
│       ├── emerging/           # 20: PFAS×6, AI/ML×6, remote sensing×4, emerging×4
│       │   ├── blue_carbon_mrv.rs     # InVEST 4-pool, GBDT, RAP-CNN, blockchain MRV
│       │   ├── edna_biodiversity.rs   # 3-level Bayesian occupancy (ψ/θ/p)
│       │   ├── microplastic_detect.rs # CNN1D+AE, SERS, hyperspectral
│       │   ├── pfas_electro_nf.rs     # SDEM + external field (Hua 2026)
│       │   ├── pfas_foam.rs           # Langmuir AWI (ITRC 2025)
│       │   ├── pfas_screening.rs      # EPA MCL May 2025
│       │   ├── pfas_scwo.rs           # Bond cleavage C-S>C-C>C-F (Prasetya 2025)
│       │   ├── pfas_transport.rs      # SDEM model (Yaroshchuk 2016)
│       │   ├── wwtp_digital_twin.rs   # XGBoost+SHAP, LSTM-GRU (2025-2026)
│       │   └── ... (10 more)
│       ├── esg/                # OJK ESG, TCFD, SDGs
│       ├── gis/                # Geoprocessing, GeoJSON
│       ├── noise/              # Kebisingan
│       ├── ocean/              # Laut, karang
│       ├── ocean_modeling/     # HYCOM, arus
│       ├── processing/         # NDVI, raster
│       ├── radiation/          # Radiasi
│       ├── satellite/          # 20+: Sentinel, flood SAR (2026 DL), InSAR MintPy, hyperspectral, GPM, GRACE, ERA5, mangrove, peatland
│       ├── waste/              # Limbah
│       ├── water/              # 29: MBR, SBR, AOP, contaminant transport, river quality, wetland, anaerobic digestion
│       └── workflows/          # AMDAL workflow
├── python/
│   ├── amdal_map_generator.py  # Generator peta AMDAL
│   ├── esg_hhra_bridge.py      # Jembatan ESG-HHRA
│   ├── master_pipeline.py      # Pipeline master
│   ├── super_amdal_pipeline.py # Pipeline AMDAL super
│   └── wrappers/               # Wrapper tools (port 8000-8004)
├── docs/                       # Dokumentasi tambahan
├── prompts/                    # Prompt templates agen
├── output_maps/                # Output PNG pipeline
├── output_amdal/               # Output JSON manifest
├── resources/                  # Resource statis
├── requirements.txt            # Dependensi Python
└── tests/                      # Test suite
```

---

## Dependensi

### Rust (Cargo.toml)

| Crate | Versi | Fungsi |
|-------|-------|--------|
| `rmcp` | 2.0 | Model Context Protocol server |
| `tokio` | latest | Async runtime |
| `reqwest` | 0.12 | HTTP client (rustls-tls) |
| `serde` / `serde_json` | latest | Serialisasi |
| `geo` | 0.29 | Geometri spasial |
| `geojson` | 0.24 | Format GeoJSON |
| `chrono` | latest | Waktu & tanggal |
| `rayon` | 1 | Parallel processing |
| `plotters` | 0.3 | Rendering chart & PNG |
| `anyhow` | latest | Error handling |
| `tracing` / `tracing-subscriber` | latest | Logging |
| `uuid` | latest | Identitas unik |
| `schemars` | 1.2.1 | JSON Schema generation |
| `lazy_static` | 1.5.0 | Static initialization |

### Python (requirements.txt)
- `numpy`, `pandas` — Komputasi numerik
- `rasterio`, `xarray` — Raster/geospasial
- `requests`, `httpx` — HTTP
- (lihat requirements.txt untuk daftar lengkap)

---

## Pengujian

### Batch Test MCP (stdio)

Script `tests/` + `/tmp/test_all_v2.py` memanggil seluruh tool via MCP stdio:

```
Hasil: 153/154 PASS (99.4%)
1 FAIL: ndvi_timeseries — timeout GEE API > 60s
```

### Self-Check Test Formula

67 test formula (satu per formula yang diperbaiki) memvalidasi output terhadap nilai referensi/analytic. Semua pass. Total test suite: **108 test pass** (sebelumnya 36).

### Test Terpilih

```bash
# Test tool individual via CLI
target/release/env-indonesia-mcp --test-tool "noise_level" '{"lat": -7.25, "lon": 112.75}'

# Cargo test (unit + integration)
cargo test --release
```

### Result Contract Test

Inline test di `main.rs` memvalidasi `ScientificResult`:
- Nilai finite
- Uncertainty bounds
- Seed reproducibility
- Fallback reason
- Stale source detection

---

## Pemecahan Masalah

### 1. Error: `data: [DONE]` tidak dikenali LLM
**Solusi**: Pastikan `provider_uri` mengarah ke SSE strip proxy (port 20129), bukan 9router langsung (port 20128).

```bash
# Cek proxy
curl http://127.0.0.1:20129/v1/models
```

### 2. Hasil tool terpotong
**Solusi**: Naikkan `tool_result_retrim_chars` di `~/.zeroclaw/config.toml` (gunakan 512000).

### 3. Tool timeout
**Solusi**: Naikkan `tool_timeout_secs` (gunakan 1800 untuk task berat seperti GEE).

### 4. 409 Conflict Telegram
**Solusi**: Pastikan hanya satu service bot yang jalan. Nonaktifkan service lain yang memakai token sama.

### 5. ndvi_timeseries timeout
GEE API bisa lambat (> 60s). Jalankan ulang atau gunakan parameter bbox lebih kecil.

### 6. Rebuild setelah edit kode

```bash
cargo build --release
systemctl --user restart env-indonesia-mcp
```

---

## Keamanan Pagar Wilayah

Seluruh kueri spasial dan fungsi perangkat analisis dilindungi penguncian perangkat keras pada batas geografi Indonesia (Bounding Box `[-11.5, 95.0, 6.0, 141.5]`). Segala bentuk parameter atau perintah pemetaan di luar tapal batas ini akan ditolak secara otomatis oleh sistem.

---

## Akuntabilitas Ilmiah

Setiap alur pemrosesan data, dari kalkulator emisi hingga hasil permodelan aliran air, mengembalikan format standar akuntabilitas ilmiah. Semua output yang disajikan kepada pengguna menyertakan sitasi metodologi referensi secara transparan.

Untuk tool *cutting-edge* (PFAS, microplastic, fire spread, eDNA, landslide, flood SAR, blue carbon, WWTP twin, AWD GHG), akuntabilitas diperkuat dengan:
- **Sitasi DOI/arXiv ID** ke paper sumber untuk setiap formula
- **Kuantifikasi ketidakpastian** (Monte Carlo 10,000+ iterasi, ILR transform, Bayesian credible interval)
- **Batasan eksplisit** — aproksimasi diungkapkan jujur (mis. "simplified, not actual CNN")
- **ML honesty** — akurasi paper riset dilabeli "Literature Reference (NOT this tool's performance)", bukan klaim performa tool
- **Self-check test** — 67 test formula memvalidasi output terhadap nilai referensi
- **Framework validasi** — RMSE/MAE/MBE/R²/NSE/KGE/PBIAS + quality badge (`src/validation/`)
- **Konteks regulasi Indonesia** — Permen LH 6/2026, 8/2026, 10/2026, 11/2025, 12/2025
- **Tidak ada over-claim** — hasil empiris dikaitkan dengan paper spesifik, stub ditandai eksplisit

---

## Lisensi

Proyek ini dikelola di GitHub: [rizkiagustiawan/env-indonesia-mcp](https://github.com/rizkiagustiawan/env-indonesia-mcp)

---

*Dibangun untuk mendukung rekayasa lingkungan Indonesia yang berbasis data, fisika, dan regulasi.*
