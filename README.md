# env-indonesia-mcp

**Sistem Model Context Protocol (MCP) Tingkat Lanjut untuk Rekayasa Lingkungan di Indonesia**

Sistem `env-indonesia-mcp` adalah server komputasi spasial dan rekayasa lingkungan (Environmental Engineering) berbasis fisika yang dirancang khusus untuk memandu dan membatasi model kecerdasan buatan (LLM) seperti arsitektur agen ZeroClaw. Server ini menjembatani kemampuan dialog kecerdasan buatan dengan kepatuhan sains lingkungan yang deterministik dan berbasis regulasi nasional.

Sistem ini memastikan setiap analisis spasial, perhitungan hidrologi, hingga pemodelan atmosfer yang dilakukan oleh AI tidak menyalahi hukum alam (seperti termodinamika atau dinamika fluida) serta mematuhi Standar Nasional Indonesia (SNI) dan peraturan kementerian terkait.

---

## Arsitektur Utama dan Fitur Unggulan

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

---

## Integrasi Ekosistem

Server ini berfungsi sebagai instrumen observasi dan tindakan bagi orkestrator seperti sistem **ZeroClaw**.
- Rantai pekerjaan (*workflow*) dapat diarahkan secara otomatis untuk mensintesis laporan kelayakan lingkungan, hingga mengisi otomatis matriks dampak Leopold (AMDAL).
- Ekspor hasil analisis spasial (SNI 6502:2010 Kartografi) dan laporan PDF yang diantarkan langsung secara seketika melalui sistem notifikasi pengiriman ke Telegram klien.

---

## Konfigurasi dan Pemasangan

Tambahkan konfigurasi berikut pada klien MCP Anda (contoh pada Claude Desktop, Cursor, atau sistem kustom ZeroClaw):

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

## Keamanan Pagar Wilayah
Seluruh kueri spasial dan fungsi perangkat analisis dilindungi penguncian perangkat keras pada batas geografi Indonesia (Bounding Box `[-11.5, 95.0, 6.0, 141.5]`). Segala bentuk parameter atau perintah pemetaan di luar tapal batas ini akan ditolak secara otomatis oleh sistem.

## Akuntabilitas Ilmiah
Setiap alur pemrosesan data, dari kalkulator emisi hingga hasil permodelan aliran air, mengembalikan format standar akuntabilitas ilmiah. Semua output yang disajikan kepada pengguna menyertakan sitasi metodologi referensi secara transparan.
