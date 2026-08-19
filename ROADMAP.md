# Supreme God-Tier 10/10 Environmental Engineering Platform Roadmap (2026)

## Current Status (Per August 19, 2026)
- **Breadth (Tool Coverage): 100%** (347 MCP tools registered)
- **Scientific Contract Maturity: ~5%** (Most tools still return unstructured Strings, missing shared Data Contracts)
- **AI & Physics Coupling: ~20%** (Topography-aware PINO and Mass Balance Gate established, but lacks trained checkpoints)
- **End-to-End Operational Maturity: ~35%** (Orchestrator works for screening, but lacks calibration, UQ propagation, and reactive transport execution)

---

## Phase 0: The Truth Layer & Inventory (Maturity 0 -> 20%)
*Mengubah 347 tool yang mengembalikan String menjadi sistem berkontrak terstruktur.*
- [ ] Audit dan registrasi 347 tools ke dalam *Maturity Matrix* (screening, conceptual, implemented_tested).
- [x] Implementasi `result_contract.rs` (JSON) ke **semua** tool.
- [ ] Wajibkan parameter *Uncertainty*, *Provenance*, *Unit*, dan *CRS* di setiap output.

## Phase 1: Shared Scientific Data Contract & Acquisition Engine (Maturity 20 -> 40%)
*Pabrik data tidak boleh menggunakan data manual. Semua harus otomatis dan tersandardisasi.*
- [ ] Buat shared struct untuk: `Grid`, `Mesh`, `TimeAxis`, `Forcing`, `StateVariable`.
- [ ] STAC Download Engine: Eksekusi unduhan raster (DEMNAS, Sentinel-1, GPM) bukan sekadar *discovery*.
- [ ] Raster processing otomatis (reprojection ke UTM, clipping AOI, alignment).

## Phase 2: The Physical Backbone - Urban Flood & Leachate (Maturity 40 -> 60%)
*Membangun mesin fisika deterministik sebelum AI diizinkan mengambil alih.*
- [ ] **Urban Flood:** Integrasi 1D Sewer Network (manhole/grate inlet) dengan 2D SWE overland flow.
- [ ] **Landfill:** Upgrade dari neraca air bulanan ke model *Transient Layered Richards Equation* + *Liner Leakage*.
- [ ] Validasi *wetting-drying* SWE secara numerik dengan resolusi tinggi.

## Phase 3: AMD Reactive Transport (Maturity 60 -> 75%)
*Fisika kimia yang nyata, bukan sekadar skrip generator.*
- [ ] Ganti `phreeqc_leaching` script generator dengan eksekutor asli (PhreeqcRM C-binding atau eksekusi *in-memory*).
- [ ] Kinetika oksidasi pirit dan presipitasi mineral.
- [ ] Transport reaktif (menggabungkan groundwater flow dengan reaksi geokimia).

## Phase 4: The Honesty Engine - Calibration & UQ (Maturity 75 -> 85%)
*Sistem harus tahu kapan dia salah.*
- [ ] Integrasikan GLUE dan DREAM-MCMC ke dalam *Orchestrator* utama.
- [ ] Wajibkan *Independent Validation* (Data Latih vs Data Uji).
- [ ] Konversi output absolut menjadi *Confidence/Prediction Intervals* (misal: banjir 1.5m ± 0.2m).

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
