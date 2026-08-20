# Supreme God-Tier 10/10 Environmental Engineering Platform Roadmap (2026)

## Current Status (Per August 20, 2026)
- **Breadth (Tool Coverage): 100%** (347 MCP tools registered)
- **Scientific Contract Maturity: ~5%** (Most tools still return unstructured Strings, missing shared Data Contracts)
- **AI & Physics Coupling: ~20%** (Topography-aware PINO and Mass Balance Gate established, but lacks trained checkpoints)
- **End-to-End Operational Maturity: ~35%** (Orchestrator works for screening, but lacks calibration, UQ propagation, and reactive transport execution)
- **Honesty Gating: implemented** (maturity ladder, synthetic-data lock, external-computation audit chain, earned validation)
- **1D-2D Flood Coupling: implemented at screening level** (real EPA SWMM run, 1% mass-balance gate; NOT calibrated or validated against observed flood extent)
- **Earned Validation: implemented** (`validated` status must be earned from split-sample test metrics; it can no longer be declared by a caller)
- **PHREEQC Geochemistry: real execution** (speciation + lime titration via `phreeqpython`; equilibrium thermodynamics only, no kinetics, no reactive transport)
- **MODFLOW 6 Groundwater: real execution** (MODFLOW 6.7.0 + FloPy, budget/dry-cell/well-curtailment gates; uncalibrated — parameters are supplied, not fitted)
- **Pyrite Oxidation Kinetics: real execution** (PHREEQC KINETICS, Williamson & Rimstidt 1994; laboratory-derived rate, absolute timescale uncalibrated)
- **Reactive Transport 1D: real execution** (PHREEQC TRANSPORT; equilibrium each cell, grid-Peclet/front/buffer guards; not yet coupled to 3D MODFLOW flow)

> **Catatan kejujuran:** "Supreme God-Tier 10/10" adalah nama target, bukan klaim ilmiah.
> Kemampuan puncak (PINO surrogate terlatih, digital twin, asimilasi data) adalah target
> Phase 5–6 dan belum tercapai. Semua keluaran saat ini `screening_only` sampai ada
> kalibrasi dan validasi independen (Phase 4).


---

## Phase 0: The Truth Layer & Inventory (Maturity 0 -> 20%)
*Mengubah 347 tool yang mengembalikan String menjadi sistem berkontrak terstruktur.*
- [ ] Audit dan registrasi 347 tools ke dalam *Maturity Matrix* (screening, conceptual, implemented_tested).
- [ ] Implementasi `result_contract.rs` (JSON) ke **semua** tool.
- [ ] Wajibkan parameter *Uncertainty*, *Provenance*, *Unit*, dan *CRS* di setiap output.

## Phase 1: Shared Scientific Data Contract & Acquisition Engine (Maturity 20 -> 40%)
*Pabrik data tidak boleh menggunakan data manual. Semua harus otomatis dan tersandardisasi.*
- [ ] Buat shared struct untuk: `Grid`, `Mesh`, `TimeAxis`, `Forcing`, `StateVariable`.
- [x] STAC Download Engine: Eksekusi unduhan raster (DEMNAS, Sentinel-1, GPM) bukan sekadar *discovery*.
- [ ] Raster processing otomatis (reprojection ke UTM, clipping AOI, alignment).

## Phase 2: The Physical Backbone - Urban Flood & Leachate (Maturity 40 -> 60%)
*Membangun mesin fisika deterministik sebelum AI diizinkan mengambil alih.*
- [x] **Urban Flood:** Integrasi 1D Sewer Network (EPA SWMM via `pyswmm`) dengan 2D SWE overland flow (`swmm_1d2d_coupling`), dengan *mass-balance gate* 1% dan hasil dikunci di `screening_only`.
- [ ] **Landfill:** Upgrade dari neraca air bulanan ke model *Transient Layered Richards Equation* + *Liner Leakage*.
- [ ] Validasi *wetting-drying* SWE secara numerik dengan resolusi tinggi.

## Phase 2.5: Honesty & Data Gating (baru)
*Sistem harus menolak analisis ketika data tidak memadai, dan tidak boleh memoles data sintetik menjadi fakta.*
- [x] `src/honesty.rs`: *maturity ladder* (`insufficient_data` → `screening` → `conceptual` → `calibrated` → `validated`) + `gate()` yang menolak permintaan di atas ketersediaan data.
- [x] Data sintetik ditandai permanen (`ScientificResult.synthetic`) dan **tidak pernah** bisa berstatus `Valid`.
- [x] Tool `assess_data_maturity` mengembalikan level yang diizinkan + daftar data yang hilang.
- [x] `src/computation.rs` + tool `record_computation`: setiap eksekusi software eksternal (QGIS/SWMM/GDAL) dicatat ke *audit chain* SHA-256.
- [x] `src/evidence/mod.rs` + tool `evidence_assess`: korroborasi hanya bila ada >= 2 *lineage* independen; kontradiksi antar-lineage independen dirutekan ke `human_review`; keluaran selalu `screening_only` dan tidak pernah menyimpulkan status hukum.


## Phase 3: AMD Reactive Transport (Maturity 60 -> 75%)
*Fisika kimia yang nyata, bukan sekadar skrip generator.*
- [x] Ganti `phreeqc_leaching` script generator dengan **eksekutor asli**: tool `phreeqc_speciation` menjalankan PHREEQC lewat `phreeqpython` (`scripts/phreeqc_run.py`), menghitung spesiasi + titrasi kapur Ca(OH)2 ke pH target, dan melaporkan logam terlarut sebelum/sesudah.
- [x] Database termodinamika diperbaiki dan disimpan di repo (`resources/phreeqc/wateq4f_PWN_repaired.dat`) — upstream `wateq4f_PWN.dat` punya CRLF + tiga `log_k` kosong yang membuat PHREEQC gagal total.
- [x] Tiga pengaman kejujuran: elemen tanpa *master species* dilaporkan sebagai `unsupported_elements` (bukan 0 mg/L palsu), konduktivitas `null` bila database tak mampu menghitung, dan fasa lewat-jenuh yang tak diendapkan menandai konsentrasi sebagai **batas atas**.
- [x] **Kinetika oksidasi pirit:** tool `pyrite_oxidation_kinetics` menjalankan PHREEQC KINETICS dengan hukum laju Williamson & Rimstidt (1994) — menjawab *seberapa cepat* asam muncul, yang tak bisa dijawab ABA statis maupun spesiasi kesetimbangan. Empat guard: `oxygen_limited`, `pyrite_depleted`, konsistensi stoikiometri S:Fe, dan catatan permanen bahwa laju berasal dari laboratorium.
- [x] **Transport reaktif 1D:** tool `reactive_transport` menjalankan PHREEQC TRANSPORT (adveksi-dispersi-reaksi) dan melaporkan front breakthrough, pore volumes flushed, Peclet grid, serta kelelahan buffer. Kopling 3D MODFLOW-GWT/PhreeqcRM masih terbuka.
- [x] **MODFLOW 6 nyata:** tool `modflow_groundwater` menjalankan MODFLOW 6.7.0 via FloPy (`scripts/modflow_run.py`) dengan satuan tetap meter/hari, empat gate (konvergensi, `percent_discrepancy`, sel kering, sumur ter-curtail), dan **tanpa fallback Theis** — model gagal = error, bukan angka substitusi.


## Phase 4: The Honesty Engine - Calibration & UQ (Maturity 75 -> 85%)
*Sistem harus tahu kapan dia salah.*
- [x] **Earned validation:** `src/calibration.rs` split-sample kontigu (Klemeš 1986) + metrik Moriasi et al. (2007) terpisah untuk partisi train dan test.
- [x] Wajibkan *Independent Validation*: status `validated` hanya diberikan bila partisi **test** lolos NSE > 0.5, |PBIAS| < 25%, dan n >= 5. Model yang hanya cocok di train dikunci di `calibrated` (kasus overfitting).
- [x] Konversi output absolut menjadi *Prediction Interval* dari residual partisi test (tool `calibrate_and_validate`).
- [x] `assess_level_with_evidence` mengambil **minimum** dari level yang diklaim dan level yang terbukti — bukti hanya bisa menurunkan klaim, tidak pernah menaikkan.
- [ ] Integrasikan GLUE dan DREAM-MCMC ke dalam *Orchestrator* utama (saat ini masih tool terpisah).


## Phase 5: The Speed - Pre-trained PINO & Surrogate AI (Maturity 85 -> 95%)
*Mengakselerasi komputasi 10.000x lipat tanpa melanggar hukum fisika.*
- [ ] Latih *Topography-Aware PINO* untuk 2D SWE menggunakan dataset sintetik Indonesia.
- [ ] Latih *MP-GPT-PINN* untuk transport reaktif geokimia.
- [ ] Aktifkan *Physics-Residual Gate* di semua AI: Jika mass error > 1%, AI ditolak dan sistem *fallback* ke numerik lambat.
- [ ] Deteksi OOD (*Out-of-Distribution*) untuk input curah hujan/topografi ekstrem.

## Phase 6: Production Hardening & Digital Twin (Maturity 95 -> 100%)
*Sistem yang siap di-deploy sebagai "Supreme God-Tier 10/10".*
- [ ] Generative Data Assimilation (HydroGEnDA) untuk asimilasi data IoT / satelit secara *real-time*.
- [ ] Integration test untuk seluruh pipeline (dari GeoJSON AOI hingga PDF Report & GeoTIFF).
- [ ] Circuit breaker, timeout retry, dan *Reproducible Run Manifest* untuk setiap simulasi.
