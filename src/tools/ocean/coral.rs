pub fn reef_health() -> String {
    let mut out = String::from("=== Coral Reef Health — NTB ===\n");
    out.push_str("Source: Allen Coral Atlas + Literature\n\n");
    
    out.push_str("KEY REEF SITES IN NTB:\n\n");
    
    out.push_str("1. GILI ISLANDS (Gili Trawangan, Gili Meno, Gili Air)\n");
    out.push_str("   Status: TWP Gili Matra (Marine Protected Area since 2009)\n");
    out.push_str("   Area: ~2,954 ha marine zone\n");
    out.push_str("   Coral cover: 15-45% (varies by site, declining trend)\n");
    out.push_str("   Threats: Tourism pressure, anchor damage, bleaching, waste\n");
    out.push_str("   Monitoring: Sentinel-2 band ratio (B2/B3) for bathymetry + reef mapping\n\n");
    
    out.push_str("2. SOUTH LOMBOK COAST\n");
    out.push_str("   Status: Varied — some protected, many unprotected\n");
    out.push_str("   Coral cover: 20-50%\n");
    out.push_str("   Threats: Sedimentation from land clearing, fishing practices\n\n");
    
    out.push_str("3. SUMBAWA NORTH COAST\n");
    out.push_str("   Status: Generally unprotected\n");
    out.push_str("   Coral cover: 30-60% (less tourist pressure)\n");
    out.push_str("   Threats: Mining runoff, blast fishing\n\n");
    
    out.push_str("4. MOYO ISLAND\n");
    out.push_str("   Status: Taman Buru (Hunting Park) — some marine protection\n");
    out.push_str("   Coral cover: 40-70% (relatively pristine)\n\n");
    
    out.push_str("SATELLITE MONITORING APPROACH:\n");
    out.push_str("  - Sentinel-2 (10m): Reef extent mapping, turbidity\n");
    out.push_str("  - Allen Coral Atlas: Global 5m benthic habitat map\n");
    out.push_str("  - Sentinel-3 OLCI: Chlorophyll-a, water quality\n");
    out.push_str("  - Landsat archive: Long-term reef change (1985-2026)\n\n");
    out.push_str("Data: https://allencoralatlas.org/ (via GEE: ACA/reef_habitat/v2_0)\n");
    out
}
