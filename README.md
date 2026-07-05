# Env-Indonesia-MCP: God Tier Environmental Engineering AI Agent

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![Tools](https://img.shields.io/badge/tools-219-orange.svg)
![Language](https://img.shields.io/badge/language-Rust_|_Python-red.svg)
![Integration](https://img.shields.io/badge/integration-Google_Earth_Engine-green.svg)

## Deskripsi

Env-Indonesia-MCP menyediakan 219 tools Model Context Protocol (MCP) tingkat lanjut yang mencakup secara komprehensif 20 domain teknik lingkungan. Sistem ini 100% terkunci dan divalidasi khusus untuk domain geografis dan regulasi Indonesia (38 provinsi).

## Daftar 20 Domain Teknik Lingkungan

| Kategori | Domain |
|---|---|
| **Air** | 1. Water Quality & Hydrology<br>2. Wastewater Treatment |
| **Udara & Atmosfer** | 3. Air Quality & Meteorology<br>4. Greenhouse Gases (GHG) & Carbon<br>5. Noise, Odor & Vibration |
| **Tanah & B3** | 6. Solid Waste Management<br>7. Hazardous Waste (B3)<br>8. Soil & Groundwater Remediation<br>9. Land Reclamation & Mining |
| **Ekologi & Spasial** | 10. Ecology & Biodiversity<br>11. GIS & Spatial Analysis (Google Earth Engine) |
| **Dampak & Risiko** | 12. Environmental Impact Assessment (AMDAL/EIA)<br>13. Health & Ecological Risk Assessment (ARKL)<br>14. Climate Change Adaptation |
| **Keberlanjutan** | 15. Life Cycle Assessment (LCA)<br>16. Environmental Economics<br>17. Resource Efficiency & Circular Economy |
| **Industri & K3** | 18. Industrial Ecology & Symbiosis<br>19. Occupational Health, Safety & Environment (HSE)<br>20. Green Building & Infrastructure |

## Kepatuhan Regulasi Indonesia

Alat ini secara ketat merujuk pada standar dan peraturan nasional Indonesia:
- **PP No. 22 Tahun 2021** (Penyelenggaraan Perlindungan dan Pengelolaan Lingkungan Hidup)
- **PP No. 41 Tahun 1999** (Pengendalian Pencemaran Udara)
- **PermenLHK No. 4/5/15/68/73/102** (Termasuk baku mutu emisi, AMDAL, pengelolaan B3)
- **KepMen LH No. 48/49/50/51/115** (Baku tingkat kebisingan, getaran, kebauan, status mutu air)
- **SNI 7645:2014** (Klasifikasi penutup lahan)
- **SNI 8202:2015** (Ketelitian peta tata ruang)

## Parameter Khusus Indonesia

Seluruh algoritma disesuaikan dengan kondisi lokal dan tropis Indonesia:
- **ARKL (Analisis Risiko Kesehatan Lingkungan) Kemenkes 2012**: Menggunakan berat badan rata-rata (BW) = 55 kg.
- **Hidrologi & Kualitas Air Tropis**: Konstanta deoksigenasi (K1) dan reaeresi (K2) disesuaikan untuk karakteristik sungai tropis dengan laju dekomposisi organik tinggi.

## Instalasi & Setup

Bangun executable dari source:
```bash
cargo build --release
```

Konfigurasi untuk ZeroClaw (atau MCP client lain):
```json
{
  "mcp_servers": {
    "env-indonesia": {
      "command": "/path/to/env-indonesia-mcp/target/release/env-indonesia-mcp",
      "args": []
    }
  }
}
```

## Environment Variables

| Variabel | Fungsi |
|---|---|
| `FIRMS_MAP_KEY` | Akses data titik panas (hotspot) kebakaran hutan/lahan |
| `BPS_API_KEY` | Akses data demografi dan statistik BPS (Badan Pusat Statistik) |
| `WAQI_API_KEY` | Akses indeks kualitas udara waktu nyata |

## ⚠️ Disclaimer

Tools ini ditujukan untuk **screening-level analysis** dan evaluasi awal proyek. Hasil dari tools ini **bukan** pengganti dari pemodelan tervalidasi yang memerlukan sertifikasi ahli (misalnya AERMOD, CALPUFF, atau HEC-RAS) yang diwajibkan untuk dokumen final persetujuan lingkungan (AMDAL/UKL-UPL).
