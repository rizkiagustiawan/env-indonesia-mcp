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
- [Katalog Tools MCP (228 Tools)](#katalog-tools-mcp-228-tools)
- [AMDAL Pipeline Engine](#amdal-pipeline-engine)
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
Lebih dari 228 model analitik terkalibrasi untuk kondisi iklim dan hidrologi Asia Tenggara:
- **Dispersi Atmosfer**: Peningkatan pada Model Gaussian Plume yang memperhitungkan sumber garis (*line source* seperti jalan raya tol) dan sumber area (*area source* seperti kolam limbah/TPA), dengan parameter stabilitas Pasquill-Gifford.
- **Kekeringan Iklim Tropis**: Implementasi *Standardized Precipitation Evapotranspiration Index* (SPEI) untuk akurasi prediksi kekeringan yang lebih komprehensif dibandingkan SPI konvensional.
- **Kesehatan Masyarakat**: *2D Monte Carlo Risk Analysis* yang membedakan ketidakpastian episodik dan fundamental pada Human Health Risk Assessment (HHRA).

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
│  │ 228 MCP tools│  │ pipeline.rs  │  │ circuit_breaker.rs  │  │
│  └──────┬──────┘  └──────┬───────┘  └─────────────────────┘  │
│         │                 │                                   │
│  ┌──────▼──────┐  ┌──────▼───────┐                            │
│  │ src/tools/* │  │ python/      │                            │
│  │ 20 kategori │  │ pipeline     │                            │
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

## Katalog Tools MCP (228 Tools)

Server mendaftarkan **228 tool MCP** (dengan 191 struct parameter terdeskripsi otomatis). Tool dibagi ke dalam 20 kategori direktori di `src/tools/`:

### Kategori Tools

| Kategori | Direktori | Contoh Tools |
|----------|-----------|--------------|
| **Fisika Lanjutan** | `advanced_physics/` | Pemodelan dispersi Gaussian plume, SWE flood, oil spill |
| **Kualitas Udara** | `airquality/` | ISPU, PM2.5/PM10, dispersi polutan, indeks kualitas udara |
| **Biodiversitas** | `biodiversity/` | Analisis habitat, keanekaragaman hayati, spesies terancam |
| **Kalkulator** | `calculators/` | Emisi karbon, jejak ekologis, kalkulator AMDAL |
| **Kepatuhan** | `compliance/` | Cek kepatuhan PP 22/2021, baku mutu, perizinan lingkungan |
| **Data** | `data/` | Katalog data, metadata sumber data lingkungan |
| **Datasources** | `datasources/` | Wrapper BMKG, BPS, Satu Data Indonesia |
| **ESG** | `esg/` | OJK ESG, TCFD, SDGs, rating ESG |
| **GIS** | `gis/` | Geoprocessing, buffer, overlay, GeoJSON, transformasi koordinat |
| **Kebisingan** | `noise/` | Tingkat kebisingan, KepMenLH 48/96, pemetaan noise |
| **Kelautan** | `ocean/` | Kualitas air laut, terumbu karang, suhu permukaan laut |
| **Oseanografi** | `ocean_modeling/` | HYCOM, arus laut, oil spill trajectory |
| **Pemrosesan** | `processing/` | NDVI timeseries, reklasifikasi, mosaik raster |
| **Radiasi** | `radiation/` | Monitoring radiasi lingkungan |
| **Satelit** | `satellite/` | Sentinel-2, Sentinel-1 SAR, Landsat, MODIS, FIRMS, TROPOMI |
| **Limbah** | `waste/` | Pengelolaan limbah B3, non-B3, sampah |
| **Air** | `water/` | Kualitas air (COD, BOD, DO), SPI/SPEI, kekeruhan |
| **Workflow** | `workflows/` | Workflow AMDAL, KLHS, dokumen lingkungan |
| **Meteorologi** | — | BMKG, Open-Meteo, NASA POWER, curah hujan, evapotranspirasi |
| **Risiko Bencana** | — | InaRISK, MAGMA Indonesia, subsidence, banjir |

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
# Konfigurasi bot (di zeroclaw config)
TELEGRAM_BOT_TOKEN="8802330994:AAGvzwFZzFCMzMtdxn36Dq2R1mawkCwvtZA"
TELEGRAM_CHAT_ID="775545807"
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
# Di ~/.config/systemd/user/zeroclaw.service
Environment="DEMNAS_EMAIL=katagiriawan@gmail.com"
Environment="DEMNAS_PASSWORD=@Awanfinger123"
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
│   ├── server.rs               # MCP server, 228 tools, 191 param structs (4214 baris)
│   ├── amdal_pipeline.rs       # AMDAL pipeline engine (20 maps)
│   ├── physics_validator.rs    # Validasi berbasis fisika
│   ├── circuit_breaker.rs      # Circuit breaker multi-agent
│   └── tools/
│       ├── advanced_physics/   # Dispersi, SWE, oil spill
│       ├── airquality/         # ISPU, PM, dispersi
│       ├── biodiversity/       # Habitat, keanekaragaman
│       ├── calculators/        # Kalkulator emisi, AMDAL
│       ├── compliance/         # Kepatuhan regulasi
│       ├── data/               # Katalog data
│       ├── datasources/        # BMKG, BPS, Satu Data
│       ├── esg/                # OJK ESG, TCFD, SDGs
│       ├── gis/                # Geoprocessing, GeoJSON
│       ├── noise/              # Kebisingan
│       ├── ocean/              # Laut, karang
│       ├── ocean_modeling/     # HYCOM, arus
│       ├── processing/         # NDVI, raster
│       ├── radiation/          # Radiasi
│       ├── satellite/          # Sentinel, Landsat, MODIS
│       ├── waste/              # Limbah
│       ├── water/              # Kualitas air, SPI/SPEI
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

---

## Lisensi

Proyek ini dikelola di GitHub: [rizkiagustiawan/env-indonesia-mcp](https://github.com/rizkiagustiawan/env-indonesia-mcp)

---

*Dibangun untuk mendukung rekayasa lingkungan Indonesia yang berbasis data, fisika, dan regulasi.*
