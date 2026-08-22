# Audit Data Resmi Indonesia untuk Pipeline Analisis Lingkungan

Tanggal audit: 22 Agustus 2026. Metode: riset web terhadap portal resmi + endpoint API, diverifikasi langsung per sumber. Tujuan: menentukan mana yang layak jadi GROUND TRUTH dan mana yang hanya INPUT, sebagai dasar benchmark pertama.

## Ringkasan Status per Sumber

| Sumber | Ground Truth | Input | Kendala utama |
|---|---|---|---|
| BMKG Data Online (UPT, RR harian) | ✅ curah hujan/iklim | — | hanya 2 tahun terakhir gratis; ±250 stasiun; kuota |
| BMKG data.bmkg.go.id (realtime) | ✅ gempa (provisional) | ✅ prakiraan cuaca | endpoint pensiun; geo-restriction |
| BMKG GeoPortal gis.bmkg.go.id | — | ✅ peta interpolasi | produk turunan, bukan observasi mentah |
| BIG DEMNAS (8,1 m) | — | ✅ elevasi (wajib conditioning) | per-lembar manual, kualitas tidak seragam, drainase tidak terbentuk |
| BIG RBI (hidrografi) | — | ✅ conditioning DEM | skala 25K/50K, edge-match antar lembar |
| BIG Batas Administrasi | ✅ delimitasi DAS/agregasi | — | pemekaran lambat, desa beda sumber |
| InaRISK BNPB (hazard) | — | ✅ referensi screening | raster 100 m, versi 2015 kasar 1:250K, butuh validasi historis |
| DIBI BNPB (kejadian) | ✅ label kejadian banjir | — | perlu login, kualitas per-entri |
| ISPU KemenLHK | ✅ kualitas udara real-time | — | API tak terdokumentasi; hanya ±24 jam terakhir |
| SIPSN KemenLHK | — | ✅ referensi (indikatif) | self-report, estimasi koefisien, non-response 25–30% |
| Badan Geologi GeoMap ESDM | ✅ litologi/struktur 1:100K | — | hanya 544 lembar vektor; banyak lembar tua |
| Sentinel-2 / Landsat / Sentinel-1 | — | ✅ komposit land-cover | awan tropis; wajib komposit + SAR |
| Copernicus GLO-30 / NASADEM | — | ✅ topografi | statis, bukan truth perubahan |

## Per Sumber (detail)

### 1. BMKG — curah hujan dan iklim

- **Data Online** `https://dataonline.bmkg.go.id`: gratis, wajib registrasi. Format CSV/Excel. Resolusi: titik stasiun UPT (±250 stasiun), harian (RR dihitung 24 jam dibaca 07:00 WIB). Cakupan: data lengkap s.d. 1970, tapi **unduh gratis dibatasi 2 tahun terakhir** (Peraturan BMKG 4/2022), maks 2 stasiun × 1 bulan, kuota 24 bulan.
- **Lisensi**: nonkomersial gratis, komersial perlu izin tertulis. **Scraping dilarang tegas oleh ToU**; otomatisasi harus via API resmi/kerja sama.
- **data.bmkg.go.id**: feed realtime (gempa `autogempa`, prakiraan cuaca per kelurahan) tanpa akun. Prakiraan cuaca = **INPUT**, bukan pengamatan.
- **PTSP** `ptsp.bmkg.go.id`: berbayar untuk data >2 tahun, pos hujan/ARG kerja sama (ribuan titik), format sesuai permintaan. Jalur terbaik untuk data klimatologi jangka panjang dan jaringan rapat.
- **Status**: UPT = **ground truth** untuk kalibrasi/validasi. Untuk forcing spasial gunakan grid produk satelit/reanalisis (GSMaP, ERA5), validasi dengan titik BMKG.

### 2. BIG — geospasial dasar

- **Tanah Air / Ina-Geoportal** `https://tanahair.indonesia.go.id`: registrasi gratis untuk unduh. Aplikasi React (butuh JS), server lambat.
- **DEMNAS** `tanahair.indonesia.go.id/demnas/`: GeoTIFF per lembar, **0,27 arc-second ≈ 8,1 m**, EPSG:4326, vertikal EGM2008. Sumber campuran (IFSAR, TANDEM-X, GDEM, SRTM); akurasi vertikal terbaik ±0,3 m di Jawa–Sumatera–Bali–NTB, lebih kasar di Papua. **Sungai/drainase tidak burned** → wajib sink-fill + stream-burning dengan hidrografi RBI untuk flood modelling. **Status: INPUT elevasi, bukan truth banjir.**
- **RBI** `tanahair.indonesia.go.id/portal-web/unduh` + `geoservices.big.go.id/rbi`: SHP per kabupaten, WMS/WFS/ArcGIS REST tanpa registrasi. Skala 1:5.000–1:250.000. Hidrografi (sungai) + garis pantai untuk conditioning DEM. **Status: INPUT.**
- **Batas administrasi**: MapServer `geoservices.big.go.id/rbi/rest/services/BATASWILAYAH` (KabKota 50K, Kecamatan 10K, KelDesa 10K); alternatif resmi PELITA Kemendagri. Level provinsi (38) s.d. desa. **Status: GROUND TRUTH** untuk delimitasi DAS/agregasi risiko. Catatan: untuk kepentingan hukum gunakan penetapan resmi, bukan peta BIG.
- `idnland.big.go.id` **tidak ada lagi** (DNS tidak resolve). Jangan dipakai.

### 3. InaRISK BNPB — hazard dan kejadian banjir

- **Portal** `inarisk.bnpb.go.id`, unduh `inarisk2.bnpb.go.id/portal/Unduh` (perlu login), API ArcGIS `gis.bnpb.go.id/server/rest/services/inarisk`.
- **Hazard banjir nasional**: raster single-band F32, **100 m pixel, EPSG:3395**, nilai indeks 0–1. Versi lama 2015 skala 1:250.000; **Peta Bahaya Nasional 2024** lebih baru, kerentanan 2025 (dasimetrik WorldPop per desa).
- **DIBI** `dibi.bnpb.go.id` dan `data.bnpb.go.id`: katalog kejadian historis (>50.000 kejadian sejak 1815), CSV/JSON. **Status: GROUND TRUTH untuk label kejadian.**
- **Kesimpulan**: InaRISK adalah peta probabilitas/indeks risiko, **bukan katalog kejadian aktual** → referensi screening, bukan truth absolut. Kombinasi ideal: DIBI sebagai label, InaRISK sebagai cek konsistensi spasial. BNPB sendiri mewajibkan validasi peta bahaya terhadap kejadian historis.

### 4. KemenLHK (KLH/BPLH) — udara, sampah, baku mutu

- **Catatan penting**: KLHK dipecah Okt 2024 menjadi Kemenhut + KLH/BPLH. **Semua `*.menlhk.go.id` sudah mati/DNS gagal**; domain baru `*.kemenlh.go.id`. `aksesdata.menlhk.go.id` tidak pernah ada.
- **ISPU** `https://ispu.kemenlh.go.id`: publik tanpa login. API tak terdokumentasi: `GET /apimobile/v1/getStations` (118 stasiun: 96 KLH/BPLH + 22 integrasi) dan `getDetail/{id}` (24 slot jam). **Hanya ±24 jam terakhir — tidak ada endpoint historis.** Format JSON: `a_*` (konsentrasi μg/m³), `t_*` (ISPU), `c_*` (kategori), `auto_validation`, `is_maintenance`. Dasar hukum Permen LHK 14/2020.
- **Status: GROUND TRUTH real-time** kualitas udara. Pipeline harus cache per jam (cron); data historis lewat PPID atau IKU/IKLH tahunan.
- **SIPSN** `https://sampahnasional.kemenlh.go.id`: `POST /indikatif/public/home/ajax_list` (`jenis=timbulan|komposisi|sumber|capaian|ikps`). 514 kab/kota, tahun 2018–2025, partisipasi parsial (358 kab/kota 2024; 303 indikatif 2025). **Status: INPUT/referensi saja** — self-report DLH, faktor estimasi (kg/org/hari × populasi), nilai sering diulang antar-tahun. Validasi dengan SNI 3242/survei lapang.
- **Baku mutu**: PP 22/2021 teks resmi di `peraturan.bpk.go.id/Details/161852`; Lampiran VII (udara ambien), VI (air), VIII (air laut). Pelaksana: Permen LHK 27/2021 (IKLH), Permen LH 5/2025 (RPPMU/baku mutu udara). JDIH aktif di `jdih.kemenlh.go.id`.

### 5. Badan Geologi ESDM — geologi

- **GeoMap** `https://geologi.esdm.go.id/geomap` (v2.2.1, pengganti geoportal.bgl.esdm.go.id yang sudah mati): registrasi SSO gratis. **Vektor SHP 1:100.000 — 544 lembar**, potong per batas administrasi; raster scan 4.756 peta; skala 1:50K–1:5M (nasional: geologi, patahan aktif, metalogeni, anomali Bouguer 2025). Tidak ada WMS/WFS publik langsung.
- **ESDM One Map** `geoportal.esdm.go.id`: registrasi + kewenangan berbasis peran; migas/mineral strategis dibatasi.
- **Jalur WMS resmi**: Ina-Geoportal BIG (Satu Data/IGT) — Peta Geologi 1:100K "Siap Berbagi Pakai".
- **Status**: SHP 1:100K layak jadi **ground truth litologi/struktur** untuk screening regional, tapi bukan kebenaran piksel-per-piksel — banyak lembar tua (1980-an–2000-an), cakupan vektor parsial, lembar 1:50K berbasis inderaja. Untuk area prospek: ground-truthing lapangan/geokimia wajib.

### 6. Citra satelit global untuk Indonesia

- **Earth Search (AWS)** `https://earth-search.aws.element84.com/v1`: STAC publik **tanpa API key**, COG siap stream. Pilihan utama. **Planetary Computer anonymous sudah berbayar (Pro) sejak 2025–2026 — jangan dibangun di situ.** CDSE butuh akun gratis + OAuth.
- **Sentinel-2 L2A**: 10 m, revisit 5 hari, **1.220.507 scene** untuk Indonesia, cloud median ~10–14% tapi tropis: Sumatera/Kalimantan/Papua + musim hujan 60–80%+ → **wajib komposit median + cloud mask SCL**, hindari single-date optical.
- **Landsat C2 L2**: 30 m, 8 hari gabungan L8+L9, **332.789 scene sejak 1982**, public domain. Untuk baseline historis/trend jangka panjang.
- **Sentinel-1 GRD (SAR)**: 10 m (VV+VH), 6 hari gabungan, **94.884 scene**, penetrasi awan penuh → **kunci saat musim hujan**; perlu kalibrasi γ0 + noise/terrain correction.
- **DEM**: Copernicus GLO-30 (30 m, statis) di Earth Search `cop-dem-glo-30`; NASADEM public domain terbaik untuk "tanah kosong". DEM selalu **INPUT**, tidak pernah truth perubahan.
- **Status**: Sentinel-2/Landsat = **INPUT** + dasar label referensi hanya jika divalidasi sampel acak berstrata dengan VHR + plot lapangan. Klasifikasi tanpa validasi = sirkular.

## Rekomendasi Ground Truth untuk Benchmark Pertama

Benchmark pertama: perubahan tutupan lahan + curah hujan + DEM → flood screening pada satu DAS Indonesia.

- **Label kejadian banjir**: DIBI BNPB (kejadian terdokumentasi per kabupaten/kota + tanggal).
- **Cek konsistensi spasial**: InaRISK hazard (bukan truth absolut).
- **Delimitasi DAS + unit agregasi**: batas administrasi BIG (kecamatan/kabupaten).
- **Elevasi**: DEMNAS 8,1 m sebagai INPUT (wajib conditioning: sink-fill + stream-burning pakai hidrografi RBI), alternatif Copernicus GLO-30 untuk replikasi independen.
- **Curah hujan**: BMKG UPT (2 tahun terakhir gratis) sebagai ground truth titik; grid GSMaP/ERA5 sebagai forcing.
- **Perubahan tutupan lahan**: komposit median tahunan Sentinel-2 (10 m) + Sentinel-1 (pengisi awan); Landsat untuk baseline historis; validasi sampel acak berstrata + VHR.
- **Rekomendasi pipeline**: STAC Earth Search (no-key) → komposit per musim kering → SAR fill → DEMNAS conditioning → flood screening (HAND/threshold baseline) → validasi DIBI + InaRISK.

## Batasan Audit

- Situs resmi berubah cepat (pemecahan KLHK, konsolidasi portal ESDM/BIG, pemindahan ke kemenlh.go.id). URL dan endpoint **wajib diverifikasi ulang saat implementasi**, bukan hanya saat audit.
- API yang "tak terdokumentasi" (ISPU, SIPSN, BMKG) bisa berubah atau diblokir kapan saja; jangan jadikan satu-satunya fondasi produksi tanpa lapisan cache + cadangan.
- Status ground truth di atas = untuk tujuan screening dan validasi ilmiah, bukan untuk klaim hukum/regulasi otomatis.
