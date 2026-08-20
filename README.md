# env-indonesia-mcp

**Server MCP untuk analisis dan pemodelan rekayasa lingkungan Indonesia yang dapat diaudit.**

[🇬🇧 English version](README.en.md)

`env-indonesia-mcp` menghubungkan agen AI dengan kalkulator Rust, akuisisi data spasial, solver hidrologi/geokimia nyata, dan kontrak hasil ilmiah. Fokusnya bukan membuat AI selalu menjawab. Fokusnya adalah membuat AI **tahu kapan harus berhenti, data apa yang kurang, dan asumsi apa yang membatasi hasil**.

> **Status ilmiah:** sebagian besar keluaran adalah `screening_only` atau `valid_with_assumptions`. Status `valid` hanya dapat diperoleh melalui validasi split-sample yang memenuhi ambang metrik dan data pendukung. Sistem bukan pengganti engineer, surveyor, laboratorium, regulator, atau validasi lapangan.

## Daftar Isi

- [Apa yang Dibangun](#apa-yang-dibangun)
- [Mesin Fisika yang Berjalan](#mesin-fisika-yang-berjalan)
- [Lapisan Kejujuran Ilmiah](#lapisan-kejujuran-ilmiah)
- [Arsitektur](#arsitektur)
- [Pemasangan](#pemasangan)
- [Penggunaan](#penggunaan)
- [Status dan Batasan](#status-dan-batasan)
- [Pengujian](#pengujian)
- [Struktur Proyek](#struktur-proyek)
- [Lisensi dan Data](#lisensi-dan-data)

## Apa yang Dibangun

Server ini menyediakan katalog besar tool lingkungan melalui MCP, tetapi tingkat kematangannya tidak seragam. Setiap domain harus diperlakukan sesuai tingkat bukti dan implementasinya:

| Tingkat | Arti |
|---|---|
| `insufficient_data` | Input minimum belum tersedia; analisis ditolak. |
| `screening_only` | Perhitungan berjalan, tetapi belum dikalibrasi/validasi lapangan. |
| `valid_with_assumptions` | Model memiliki data atau kalibrasi terbatas; asumsi tetap membatasi interpretasi. |
| `valid` | Hanya diberikan setelah bukti validasi independen melewati gate. |
| `validation_failed` | Kontrak, solver, input, atau gate fisik gagal. |

Sistem menggabungkan:

- Data satelit/STAC: Sentinel-1/2, Landsat, DEM, GPM/IMERG dan katalog terkait.
- GIS: GeoJSON, bbox, raster, CRS, clipping, manifest, dan provenance.
- Hidrologi: SWE 2D dan coupling sewer 1D SWMM.
- Geokimia: PHREEQC equilibrium, titrasi kapur, kinetika oksidasi pirit, dan reactive transport 1D.
- Hidrogeologi: MODFLOW 6 melalui FloPy.
- Evidence: sumber, artefak, claim, lineage independen, konflik, human review, dan audit chain SHA-256.

## Mesin Fisika yang Berjalan

### Banjir: SWMM 1D + SWE 2D

Tool `swmm_1d2d_coupling` menjalankan EPA SWMM melalui `pyswmm`, mengambil volume surcharge setiap node, memetakan node ke grid DEM, lalu menginjeksikannya ke solver SWE multi-source.

Gate yang diterapkan:

- Volume SWMM dibandingkan dengan volume yang masuk ke domain 2D.
- Toleransi default mass balance: 1%.
- Node sewer yang tidak memiliki mapping ditolak.
- Hasil tetap `screening_only` karena belum divalidasi terhadap extent/depth banjir observasi.

Contoh hasil verifikasi sintetik:

```text
SWMM surcharge: 1231.92 m³
SWE injected:   1231.88 m³
Mass error:     0.0030%
Gate:           passed
Status:         screening_only
```

### PHREEQC: Spesiasi dan Leaching

Tool `phreeqc_speciation` menjalankan PHREEQC nyata melalui `phreeqpython`, bukan hanya menghasilkan skrip. Fitur yang tersedia:

- Spesiasi larutan dan saturation index.
- Titrasi Ca(OH)2 ke target pH dengan bracket + bisection.
- Equilibrium phase dengan mode precipitate-only (`in_phase = 0`).
- Pelaporan logam terlarut sebelum/sesudah treatment.

Database `wateq4f_PWN_repaired.dat` berada di `resources/phreeqc/`. File ini adalah salinan perbaikan dengan header provenance dan SHA-256. Perbaikan diperlukan karena file sumber memiliki tiga entri `log_k` kosong yang membuat PHREEQC gagal memuat database.

### Kinetika Oksidasi Pirit

Tool `pyrite_oxidation_kinetics` memakai blok `RATES` database dan hukum Williamson & Rimstidt (1994). Output mencakup pH, Fe, sulfate, sisa pyrite, dan guard:

- `oxygen_limited`: membedakan sistem tertutup yang kehabisan O2 dari reaksi yang benar-benar stabil.
- `pyrite_depleted`: membedakan pH datar karena sulfida habis.
- `stoichiometry_consistent`: FeS2 seharusnya memberi rasio S:Fe sekitar 2; presipitasi Fe dapat memutus hubungan ini.
- `rate_is_laboratory_derived`: selalu true; laju lapangan belum dikalibrasi.

### Reactive Transport 1D

Tool `reactive_transport` menjalankan PHREEQC `TRANSPORT` pada kolom mineral:

- Adveksi, dispersi, dan reaksi per cell.
- Output berdasarkan pore volume, bukan hanya nilai akhir.
- `grid_peclet`: jika lebih dari 2, numerical dispersion grid dominan.
- `front_traversed_column`: front harus melewati minimal satu pore volume sebelum outlet dapat ditafsirkan.
- `buffer_exhausted`: mendeteksi barrier mineral yang habis dan breakthrough.
- Asumsi full equilibrium di setiap cell dilaporkan secara eksplisit.

Ini adalah model kolom 1D, bukan kopling 3D MODFLOW-GWT/PhreeqcRM.

### MODFLOW 6 Groundwater

Tool `modflow_groundwater` menjalankan MODFLOW 6.7.0 melalui FloPy dengan satuan eksplisit:

- Panjang: meter.
- Waktu: hari.
- Konduktivitas: m/day.
- Recharge: mm/year, dikonversi internal.
- Ekstraksi sumur: m³/day.

Gate yang diterapkan:

- MODFLOW harus konvergen.
- `PERCENT_DISCREPANCY` budget harus berada dalam toleransi.
- Sentinel head ±1e30 tidak boleh masuk statistik.
- Sumur yang dimatikan MODFLOW karena cell kering dideteksi dengan membandingkan extraction diminta vs delivered.
- Dominasi constant-head boundary dilaporkan karena dapat mengontrol hasil drawdown.

Tidak ada fallback otomatis ke Theis. Jika MODFLOW gagal, hasil adalah error, bukan angka pengganti.

## Lapisan Kejujuran Ilmiah

### Data Maturity Ladder

Tool `assess_data_maturity` menentukan level tertinggi yang didukung data:

```text
insufficient_data → screening → conceptual → calibrated → validated
```

Data sintetik wajib ditandai `synthetic: true` dan tidak pernah dapat berstatus `valid`.

### Earned Validation

Tool `calibrate_and_validate` memakai split kontigu, bukan random split, agar tidak membocorkan informasi pada deret waktu autocorrelated.

`validated` hanya earned bila partisi test memenuhi:

- NSE > 0.5
- |PBIAS| < 25%
- minimal 5 titik test

Model yang hanya bagus di data train tetapi gagal di test dikunci di `valid_with_assumptions` sebagai indikasi overfitting. Output membawa `PredictionInterval` dari RMSE test.

### Evidence Assessment

Tool `evidence_assess`:

- Memerlukan minimal dua lineage independen untuk corroboration.
- Menganggap salinan laporan dengan `independence_group` sama sebagai satu sumber.
- Mengirim konflik antar-lineage ke `human_review`.
- Mendukung tier-1 official finding sebagai bukti tunggal sesuai aturan evidence.
- Selalu menghasilkan `screening_only` dan tidak membuat kesimpulan legal/regulatory.

### Provenance dan Audit

`record_computation` mencatat eksekusi software eksternal seperti QGIS, SWMM, PHREEQC, dan MODFLOW. Record berisi tool, versi, argumen, hash input/output, waktu, exit code, dan hash event audit.

Semua hasil solver eksternal diperlakukan sebagai **untrusted execution** sampai kontrak dan gate selesai diperiksa.

## Arsitektur

```text
AI Agent / MCP Client
        │ stdio MCP
        ▼
env-indonesia-mcp (Rust)
  ├─ result_contract + honesty ladder
  ├─ evidence + SHA-256 audit chain
  ├─ satellite/STAC + artifact manifests
  ├─ SWE / SWMM coupling
  ├─ PHREEQC / reactive transport
  ├─ MODFLOW 6 / FloPy runner
  └─ legacy calculators and domain tools
        │ subprocess, guarded and provenance-recorded
        ├─ Python environmental venv
        └─ QGIS Agent MCP (optional, live QGIS session)
```

QGIS Agent MCP dipakai sebagai local stdio MCP launcher. Bridge internal QGIS dapat memakai port dinamis; konfigurasi client harus mengikuti connection file yang dibuat plugin.

## Pemasangan

### Rust

```bash
git clone https://github.com/rizkiagustiawan/env-indonesia-mcp.git
cd env-indonesia-mcp
cargo build --release
```

### Python solver environment

Contoh menggunakan virtual environment terpisah:

```bash
python3 -m venv /path/to/env-indonesia
/path/to/env-indonesia/bin/pip install -r requirements.txt
```

Stack solver yang digunakan runner:

- `pyswmm==2.1.0`
- `swmm-toolkit==0.17.0`
- `wntr==1.5.0`
- `phreeqcrm==0.0.20`
- `phreeqpython==1.6.2`
- `flopy==3.10.0`
- `numpy`, `scipy`, `pandas`

MODFLOW executable diunduh terpisah melalui FloPy:

```bash
get-modflow /path/to/env-indonesia/bin --subset mf6,mf2005,mp7
```

Interpreter runner dapat diganti tanpa mengubah kode:

```bash
export ENV_INDONESIA_SWMM_PYTHON=/path/to/env-indonesia/bin/python
```

### Menjalankan server MCP

```bash
cargo run --release
```

Konfigurasi client MCP umum:

```json
{
  "mcpServers": {
    "env-indonesia": {
      "command": "cargo",
      "args": ["run", "--release", "--manifest-path", "/path/env-indonesia-mcp/Cargo.toml"]
    }
  }
}
```

## Penggunaan CLI

Semua contoh menggunakan JSON langsung dan menghasilkan JSON kontrak.

```bash
# Data maturity
cargo run -- --test-tool assess_data_maturity \
  '{"requested_level":"validated","availability":{"regional_dem":true}}'

# Validasi earned
cargo run -- --test-tool calibrate_and_validate \
  '{"model_name":"example","predicted":[1,2,3,4,5,6,7,8,9,10],"observed":[1,2,3,4,5,6,7,8,9,10],"unit":"m"}'

# PHREEQC equilibrium
cargo run -- --test-tool phreeqc_speciation \
  '{"solution":{"pH":2.8,"Fe(3)":50,"S(6)":200,"Ni":5},"units":"mmol"}'

# Pyrite kinetics
cargo run -- --test-tool pyrite_oxidation_kinetics \
  '{"pyrite_mol_per_kgw":0.05,"initial_ph":6.5,"replenish_o2":true,"steps_days":[1,30,90,365]}'

# Reactive transport column
cargo run -- --test-tool reactive_transport \
  '{"cells":5,"cell_length_m":0.2,"shifts":60,"time_step_s":3600,"dispersivity_m":0.1,"influent":{"pH":2.5,"Fe(3)":30,"S(6)":120},"pore_water":{"pH":7,"Ca":1},"units":"mmol","reactive_phases":[{"phase":"Calcite","moles":0.02}],"tracked_elements":["Fe"]}'

# MODFLOW groundwater
cargo run -- --test-tool modflow_groundwater \
  '{"nlay":2,"nrow":20,"ncol":20,"cell_size_m":100,"top_m":50,"layer_bottoms_m":[30,0],"hk_m_day":10,"vk_m_day":1,"sy":0.15,"ss_per_m":0.00001,"initial_head_m":45,"boundary_head_m":45,"recharge_mm_yr":1800,"wells":[{"layer":1,"row":10,"col":10,"rate_m3_day":2000}],"steady_state":true}'
```

## Status dan Batasan

### Sudah berjalan

- Scientific result contract dengan finite-value, uncertainty, provenance, CRS, stale-source, dan regulatory-claim guards.
- Evidence assessment dan SHA-256 audit chain.
- STAC asset download dengan host allowlist, content validation, manifest, dan hash.
- SWMM 1D + SWE 2D dengan mass-balance gate.
- PHREEQC equilibrium, pyrite kinetics, dan reactive transport 1D.
- MODFLOW 6.7.0 + FloPy dengan budget/dry-cell/well-curtail gates.
- Earned split-sample validation dan prediction interval.

### Belum boleh diklaim

- Belum calibrated/validated lapangan secara otomatis.
- Belum memiliki digital twin real-time.
- Belum memiliki trained PINO/FNO checkpoint yang tervalidasi lintas wilayah.
- Reactive transport masih 1D column; belum 3D MODFLOW-GWT/PhreeqcRM.
- Kinetika pirit memakai rate laboratorium; laju lapangan belum dikalibrasi.
- DEMNAS 8 m dan satellite proxy tidak menggantikan LiDAR, survey, rain gauge, discharge, atau observasi banjir.
- Data drainase, boundary condition, porositas, dispersivitas, roughness, dan parameter kimia harus berasal dari engineer/data lapangan; sistem tidak boleh mengarangnya diam-diam.

### Status kematangan

```text
Kalkulator domain luas       banyak yang screening/conceptual
Physics contract              implemented
SWMM/SWE coupling             screening, mass-conservative
PHREEQC/MODFLOW runners       real execution, uncalibrated
Earned validation             implemented
PINO/digital twin             roadmap, belum production-ready
```

## Pengujian

```bash
cargo test
python3 -m pytest -q
python3 -m py_compile scripts/*.py
git diff --check
```

Baseline verifikasi repository saat dokumentasi ini diperbarui:

- Rust: **247 test pass**
- Python: **11 test pass**
- API gateway: `cargo check` pass
- Semua script solver: `py_compile` pass

Pytest masih mencetak dua warning dependency/deprecation yang tidak menyebabkan test gagal.

## Struktur Proyek

```text
src/
├── main.rs                 # entry point, MCP router, CLI dispatch
├── server.rs               # MCP tool definitions
├── result_contract.rs      # ScientificResult dan contract validation
├── honesty.rs              # maturity ladder dan synthetic lock
├── evidence/               # source/artifact/claim/audit evidence
├── computation.rs          # external computation run manifest
├── calibration.rs          # earned validation dan prediction interval
├── coupling.rs             # SWMM 1D -> SWE 2D mapping dan mass gate
├── swmm_runner.rs          # pyswmm subprocess contract
├── phreeqc_runner.rs       # equilibrium PHREEQC subprocess contract
├── pyrite_kinetics.rs      # PHREEQC KINETICS contract
├── reactive_transport.rs   # PHREEQC TRANSPORT contract
├── modflow_runner.rs       # MODFLOW 6/FloPy subprocess contract
└── tools/                  # kalkulator dan tool domain legacy
scripts/
├── swmm_run.py
├── phreeqc_run.py
├── pyrite_kinetics.py
├── reactive_transport.py
└── modflow_run.py
resources/phreeqc/
└── wateq4f_PWN_repaired.dat
```

## Lisensi dan Data

Kode proyek tersedia di [GitHub](https://github.com/rizkiagustiawan/env-indonesia-mcp). Periksa lisensi masing-masing solver, database termodinamika, data satelit, dan sumber resmi sebelum distribusi komersial atau penggunaan regulator.

Jangan commit credential, token, password DEMNAS, token Telegram, atau endpoint privat ke repository. Gunakan environment variable atau secret manager.

## Prinsip Proyek

> Sistem lingkungan yang baik bukan sistem yang selalu mengeluarkan angka. Sistem yang baik adalah sistem yang dapat menunjukkan sumber angka, asumsi yang dipakai, error budget, batas penggunaan, dan alasan mengapa ia menolak ketika bukti tidak cukup.
