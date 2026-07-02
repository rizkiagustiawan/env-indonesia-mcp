pub fn protected_areas() -> String {
    let mut out = String::from("=== Marine Protected Areas — NTB ===\n");
    out.push_str("Source: WDPA (World Database on Protected Areas) + KLHK\n\n");
    
    out.push_str("MARINE PROTECTED AREAS:\n\n");
    
    out.push_str("1. TWP GILI MATRA (Taman Wisata Perairan Gili Matra)\n");
    out.push_str("   WDPA ID: 555629432\n");
    out.push_str("   Area: 2,954 ha\n");
    out.push_str("   Established: 2009 (Kepmenhut No.96/Menhut-II/2009)\n");
    out.push_str("   IUCN Category: VI (Managed Resource Protected Area)\n");
    out.push_str("   Covers: Gili Trawangan, Gili Meno, Gili Air + surrounding waters\n");
    out.push_str("   Management: BKKPN Kupang\n\n");
    
    out.push_str("2. TAMAN NASIONAL GUNUNG RINJANI (TNGR)\n");
    out.push_str("   WDPA ID: 4178\n");
    out.push_str("   Area: 41,330 ha (terrestrial, includes Segara Anak lake)\n");
    out.push_str("   Established: 1997\n");
    out.push_str("   IUCN Category: II (National Park)\n");
    out.push_str("   UNESCO Global Geopark: 2018\n\n");
    
    out.push_str("3. TAMAN NASIONAL TAMBORA\n");
    out.push_str("   Area: 71,645 ha\n");
    out.push_str("   Established: 2015\n");
    out.push_str("   Includes Mount Tambora (2,850m) + savanna + forest\n\n");
    
    out.push_str("4. CAGAR ALAM PULAU MOYO\n");
    out.push_str("   Area: 6,000 ha (terrestrial + marine)\n");
    out.push_str("   Established: 1986\n\n");
    
    out.push_str("5. SUAKA MARGASATWA PULAU SANGIANG\n");
    out.push_str("   Area: 766 ha\n\n");
    
    out.push_str("TOTAL PROTECTED AREA NTB: ~122,695 ha\n\n");
    out.push_str("Data: https://www.protectedplanet.net/country/IDN\n");
    out.push_str("API: https://api.protectedplanet.net/v3/protected_areas?country=IDN\n");
    out
}
