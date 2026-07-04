# Env-Indonesia-MCP: God Tier Environmental Engineering AI Agent

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![Tools](https://img.shields.io/badge/tools-224-orange.svg)
![Language](https://img.shields.io/badge/language-Rust-red.svg)

## Deskripsi

Env-Indonesia-MCP menyediakan 224 tools Model Context Protocol (MCP) yang mencakup 20 domain teknik lingkungan. Sistem terkunci khusus domain Indonesia.

## 20 Domain Teknik Lingkungan

| Kategori | Domain |
|---|---|
| Kualitas Air | 1. Water Quality & Hydrology<br>2. Wastewater Treatment |
| Udara & Atmosfer | 3. Air Quality & Meteorology<br>4. Greenhouse Gases (GHG) & Carbon<br>5. Noise, Odor & Vibration |
| Tanah & B3 | 6. Solid Waste Management<br>7. Hazardous Waste (B3)<br>8. Soil & Groundwater Remediation<br>9. Land Reclamation & Mining |
| Ekologi & Spasial | 10. Ecology & Biodiversity<br>11. GIS & Spatial Analysis |
| Dampak & Risiko | 12. Environmental Impact Assessment (AMDAL/EIA)<br>13. Health & Ecological Risk Assessment (ARKL)<br>14. Climate Change Adaptation |
| Keberlanjutan | 15. Life Cycle Assessment (LCA)<br>16. Environmental Economics<br>17. Resource Efficiency & Circular Economy |
| Industri & K3 | 18. Industrial Ecology & Symbiosis<br>19. Occupational Health, Safety & Environment (HSE)<br>20. Green Building & Infrastructure |

## Kepatuhan Regulasi Indonesia

Tunduk standar nasional:
- PP 22/2021
- PP 41/1999
- PermenLHK 4/5/15/68/73/102
- KepMen LH 48/49/50/51/115
- SNI 7645:2014
- SNI 8202:2015
- Dll.

## Parameter Khusus Indonesia

Asumsi tropis lokal:
- ARKL Kemenkes: BW = 55kg
- Hidrologi Tropis: K1/K2 sungai tropis.

## Instalasi & Setup

Build release:
```bash
cargo build --release
```

Konfigurasi ZeroClaw:
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
| `FIRMS_MAP_KEY` | Akses hotspot |
| `BPS_API_KEY` | Akses data demografi BPS |

## Disclaimer

Tools ini adalah screening-level, bukan pengganti model tervalidasi (AERMOD/HEC-RAS) untuk dokumen final AMDAL.
