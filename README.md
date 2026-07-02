# 🌍 ZeroClaw Environmental AI — God Mode

> **The first Physics-Informed Environmental AI Agent in Indonesia.**
> 84 MCP Tools | Domain-Locked Indonesia | Pure Rust + Python

## Architecture

```
User (Telegram)
    │
    ├── ZeroClaw Ultra Orchestrator (Rust)
    │   ├── 60 Model Flagship (Claude, GPT, DeepSeek, Gemini, Kimi, QwQ)
    │   ├── U-Mem Knowledge Base (self-learning from failures)
    │   ├── Consensus Voting (3-model parallel)
    │   ├── Budget Forcing (self-verify reasoning)
    │   └── Physics-Informed System Prompt
    │
    └── env-ntb-mcp (Rust MCP Server)
        ├── 84 Tools across 10 categories
        ├── Physics Validator Engine (30+ constraints)
        ├── Cartography Engine (SNI layout + Sentinel-2 basemap)
        └── PDF Report Generator (AMDAL template PP 22/2021)
```

## 84 MCP Tools

| Category | Count | Examples |
|----------|-------|---------|
| **Data Indonesia** | 10 | BMKG, MAGMA, BNPB InaRISK, BPS, NASA FIRMS, GFW |
| **Satellite** | 10 | Copernicus, Landsat, MODIS, VIIRS, SRTM, CHIRPS, GRACE, ERA5 |
| **GIS & Cartography** | 9 | NDVI, Water Quality, DEM Slope, Map Generator (SNI), Coordinate Transform |
| **ESG & Carbon** | 6 | Carbon Calculator, TCFD, Climate TRACE, SDG Mapper, OJK POJK 51/2017 |
| **Ocean & Marine** | 2 | Coral Reef Health, Marine Protected Areas |
| **Calculators** | 33 | RUSLE, SCS-CN, Penman-Monteith, Streeter-Phelps, Gaussian Plume, AMDAL Leopold, Heat Index, Tsunami, Peatland, Mangrove, etc. |
| **Compliance** | 3 | PROPER Scoring, IKLH Calculator, B3 Waste Classifier |
| **Physics Validator** | 1 | 30+ parameters, 8 domains, Indonesian standards |
| **Processing** | 5 | PDF Report, GeoTIFF (GDAL), Watershed (pysheds), IDW Interpolation |
| **Wrappers** | 5 | GeoESG, Flood AI, Gas Methane, Groundwater, Air Quality |

## Physics-Informed Validator

All calculations are validated against physical laws before output:

- **Radiometry**: NDVI ∈ [-1, 1], Reflectance ∈ [0, 1], Cloud Fraction ∈ [0, 1]
- **Atmosphere**: Concentration ≥ 0, Wind ≥ 0.28 m/s (Gaussian plume singularity)
- **Hydrology**: Runoff ≤ Rainfall (conservation of mass), flow follows gravity (DEM)
- **Water Quality**: COD ≥ BOD (chemistry), DO ∈ [0, 14.6] mg/L, pH ∈ [0, 14]
- **Erosion**: RUSLE factors K, C, P ∈ [0, 1]
- **Indonesian Standards**: PP 22/2021, PermenLHK 68/2016, KepMenLH 48/1996

## Telegram Bot Capabilities

- ✅ Send text (smart paragraph-boundary chunking)
- ✅ Send photos/maps (PNG, auto-detect from tool output)
- ✅ Send documents (PDF, CSV, JSON, GeoTIFF)
- ✅ Send location pins
- ✅ Receive photos → Vision LLM analysis
- ✅ Receive documents → Environmental analysis

## Tech Stack

- **Rust** — ZeroClaw Agent (orchestrator) + MCP Server (tools)
- **Python** — Cartography (matplotlib, geopandas), PDF (fpdf2), Watershed (pysheds)
- **GDAL 3.13** — GeoTIFF processing
- **rmcp 2.0** — Model Context Protocol SDK
- **reqwest** — HTTP client for 37 data sources

## Indonesian Regulations Encoded

- PP 22/2021 (Baku Mutu Air, Udara, Lingkungan)
- PermenLHK 68/2016 (Limbah Domestik)
- PermenLHK P.1/2021 (PROPER)
- PermenLHK P.27/2021 (IKLH)
- KepMenLH 48/1996 (Kebisingan)
- PP 101/2014 (Limbah B3)
- POJK 51/2017 (ESG Keuangan)
- UU 32/2009 (PPLH)
- Perpres 98/2021 (Nilai Ekonomi Karbon)

## Academic References

Metcalf & Eddy (2003), FAO-56 (Allen 1998), USDA TR-55, RUSLE (Renard 1997), Carlson TSI (1977), Streeter-Phelps (1925), Monod kinetics, Hooijer et al. (2012), Komiyama et al. (2005), Synolakis (1987), Rothfusz/NWS, Terzaghi (1943), Gumbel (Chow 1951), Leopold et al. (1971), IPCC AR6.

## License

Private — All rights reserved.

## Author

Environmental AI Engineer | GIS, Remote Sensing & ESG Analytics
Fresh Graduate Environmental Engineering 2025
Domain: Indonesia (West Nusa Tenggara focused)
