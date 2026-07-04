// Indonesia constants — domain lock and reference data
// 38 Provinces (post-2022 Papua split)

/// Indonesia bounding box [south, west, north, east]
pub const INDONESIA_BBOX: [f64; 4] = [-11.5, 95.0, 6.0, 141.5];
pub const INDONESIA_CENTER: [f64; 2] = [-2.5, 118.0];

/// Validate coordinates are within Indonesia
pub fn validate_coords(lat: f64, lon: f64) -> Result<(), String> {
    if lat < -11.5 || lat > 6.0 {
        return Err(format!("Latitude {:.4} di luar Indonesia. Range: -11.5 s/d 6.0", lat));
    }
    if lon < 95.0 || lon > 141.5 {
        return Err(format!("Longitude {:.4} di luar Indonesia. Range: 95.0 s/d 141.5", lon));
    }
    Ok(())
}

/// 38 Indonesian provinces with center coordinates and BPS code
pub struct Province {
    pub name: &'static str,
    pub capital: &'static str,
    pub bps_code: &'static str,
    pub lat: f64,
    pub lon: f64,
}

pub const PROVINCES: &[Province] = &[
    Province { name: "Aceh", capital: "Banda Aceh", bps_code: "11", lat: 4.7, lon: 96.7 },
    Province { name: "Sumatera Utara", capital: "Medan", bps_code: "12", lat: 2.5, lon: 99.0 },
    Province { name: "Sumatera Barat", capital: "Padang", bps_code: "13", lat: -0.9, lon: 100.4 },
    Province { name: "Riau", capital: "Pekanbaru", bps_code: "14", lat: 1.5, lon: 102.1 },
    Province { name: "Jambi", capital: "Jambi", bps_code: "15", lat: -1.6, lon: 103.6 },
    Province { name: "Sumatera Selatan", capital: "Palembang", bps_code: "16", lat: -3.3, lon: 104.8 },
    Province { name: "Bengkulu", capital: "Bengkulu", bps_code: "17", lat: -3.8, lon: 102.3 },
    Province { name: "Lampung", capital: "Bandar Lampung", bps_code: "18", lat: -5.0, lon: 105.3 },
    Province { name: "Kepulauan Bangka Belitung", capital: "Pangkal Pinang", bps_code: "19", lat: -2.1, lon: 106.1 },
    Province { name: "Kepulauan Riau", capital: "Tanjung Pinang", bps_code: "21", lat: 1.0, lon: 104.5 },
    Province { name: "DKI Jakarta", capital: "Jakarta", bps_code: "31", lat: -6.2, lon: 106.85 },
    Province { name: "Jawa Barat", capital: "Bandung", bps_code: "32", lat: -6.9, lon: 107.6 },
    Province { name: "Jawa Tengah", capital: "Semarang", bps_code: "33", lat: -7.0, lon: 110.4 },
    Province { name: "DI Yogyakarta", capital: "Yogyakarta", bps_code: "34", lat: -7.8, lon: 110.4 },
    Province { name: "Jawa Timur", capital: "Surabaya", bps_code: "35", lat: -7.5, lon: 112.8 },
    Province { name: "Banten", capital: "Serang", bps_code: "36", lat: -6.4, lon: 106.2 },
    Province { name: "Bali", capital: "Denpasar", bps_code: "51", lat: -8.4, lon: 115.2 },
    Province { name: "Nusa Tenggara Barat", capital: "Mataram", bps_code: "52", lat: -8.6, lon: 117.0 },
    Province { name: "Nusa Tenggara Timur", capital: "Kupang", bps_code: "53", lat: -9.6, lon: 123.6 },
    Province { name: "Kalimantan Barat", capital: "Pontianak", bps_code: "61", lat: 0.0, lon: 109.3 },
    Province { name: "Kalimantan Tengah", capital: "Palangka Raya", bps_code: "62", lat: -1.7, lon: 114.0 },
    Province { name: "Kalimantan Selatan", capital: "Banjarbaru", bps_code: "63", lat: -3.4, lon: 115.0 },
    Province { name: "Kalimantan Timur", capital: "Samarinda", bps_code: "64", lat: 0.5, lon: 117.0 },
    Province { name: "Kalimantan Utara", capital: "Tanjung Selor", bps_code: "65", lat: 3.0, lon: 116.5 },
    Province { name: "Sulawesi Utara", capital: "Manado", bps_code: "71", lat: 1.5, lon: 124.8 },
    Province { name: "Sulawesi Tengah", capital: "Palu", bps_code: "72", lat: -1.0, lon: 121.5 },
    Province { name: "Sulawesi Selatan", capital: "Makassar", bps_code: "73", lat: -3.7, lon: 120.0 },
    Province { name: "Sulawesi Tenggara", capital: "Kendari", bps_code: "74", lat: -4.0, lon: 122.5 },
    Province { name: "Gorontalo", capital: "Gorontalo", bps_code: "75", lat: 0.5, lon: 122.5 },
    Province { name: "Sulawesi Barat", capital: "Mamuju", bps_code: "76", lat: -2.7, lon: 119.0 },
    Province { name: "Maluku", capital: "Ambon", bps_code: "81", lat: -3.7, lon: 128.2 },
    Province { name: "Maluku Utara", capital: "Sofifi", bps_code: "82", lat: 1.7, lon: 127.8 },
    Province { name: "Papua", capital: "Jayapura", bps_code: "91", lat: -2.5, lon: 140.7 },
    Province { name: "Papua Barat", capital: "Manokwari", bps_code: "92", lat: -0.9, lon: 134.1 },
    Province { name: "Papua Selatan", capital: "Merauke", bps_code: "93", lat: -7.0, lon: 139.0 },
    Province { name: "Papua Tengah", capital: "Nabire", bps_code: "94", lat: -4.0, lon: 137.0 },
    Province { name: "Papua Pegunungan", capital: "Jayawijaya", bps_code: "95", lat: -4.1, lon: 138.5 },
    Province { name: "Papua Barat Daya", capital: "Sorong", bps_code: "96", lat: -1.0, lon: 131.5 },
];

/// Lookup province by name (case-insensitive, partial match)
pub fn find_province(query: &str) -> Option<&'static Province> {
    let q = query.to_lowercase();
    PROVINCES.iter().find(|p| {
        p.name.to_lowercase().contains(&q) || 
        p.capital.to_lowercase().contains(&q) ||
        p.bps_code == query
    })
}

/// BMKG city → adm4 code mapping (Indonesian cities)
/// Format: PP.KK.CC.DDDD (provinsi.kab_kota.kecamatan.kelurahan)
/// Ref: Keputusan Mendagri No. 100.1.1-6117 Tahun 2022
pub fn bmkg_adm4<'a>(city: &'a str) -> &'a str {
    match city.to_lowercase().as_str() {
        // === JAWA ===
        "jakarta" | "dki" | "dki jakarta" => "31.71.01.1001",           // Jakarta Pusat, Gambir
        "bogor" => "32.71.01.1001",
        "depok" => "32.76.01.1001",
        "tangerang" => "36.71.01.1001",
        "tangerang selatan" | "tangsel" => "36.74.01.1001",
        "bekasi" => "32.75.01.1001",
        "bandung" => "32.73.01.1001",
        "cirebon" => "32.74.01.1001",
        "tasikmalaya" => "32.78.01.1001",
        "sukabumi" => "32.72.01.1001",
        "semarang" => "33.74.01.1001",
        "solo" | "surakarta" => "33.72.01.1001",
        "pekalongan" => "33.75.01.1001",
        "tegal" => "33.76.01.1001",
        "yogyakarta" | "jogja" | "yogya" => "34.71.01.1001",
        "surabaya" => "35.78.01.1001",
        "malang" => "35.73.01.1001",
        "kediri" => "35.71.01.1001",
        "madiun" => "35.77.01.1001",
        "serang" => "36.73.01.1001",
        "cilegon" => "36.72.01.1001",
        // === SUMATERA ===
        "banda aceh" | "aceh" => "11.71.01.1001",
        "medan" => "12.71.01.1001",
        "padang" => "13.71.01.1001",
        "pekanbaru" => "14.71.01.1001",
        "jambi" => "15.71.01.1001",
        "palembang" => "16.71.01.1001",
        "bengkulu" => "17.71.01.1001",
        "lampung" | "bandar lampung" => "18.71.01.1001",
        "pangkal pinang" | "pangkalpinang" => "19.71.01.1001",
        "tanjung pinang" | "tanjungpinang" => "21.71.01.1001",
        "batam" => "21.72.01.1001",
        "bukittinggi" => "13.72.01.1001",
        "dumai" => "14.72.01.1001",
        // === BALI & NUSA TENGGARA ===
        "denpasar" => "51.71.01.1001",
        "mataram" => "52.71.01.1004",                                   // Verified: 1004
        "kupang" => "53.71.01.1001",
        "bima" => "52.72.01.1001",
        "sumbawa" | "sumbawa besar" => "52.04.01.2001",
        // === KALIMANTAN ===
        "pontianak" => "61.71.01.1001",
        "palangka raya" | "palangkaraya" => "62.71.01.1001",
        "banjarmasin" => "63.72.01.1001",
        "banjarbaru" => "63.71.01.1001",
        "samarinda" => "64.72.01.1001",
        "balikpapan" => "64.71.01.1001",
        "tarakan" => "65.71.01.1001",
        "tanjung selor" => "65.01.01.2001",
        // === SULAWESI ===
        "manado" => "71.71.01.1001",
        "palu" => "72.71.01.1001",
        "makassar" | "ujung pandang" => "73.71.01.1001",
        "kendari" => "74.71.01.1001",
        "gorontalo" => "75.71.01.1001",
        "mamuju" => "76.01.01.2001",
        "bitung" => "71.72.01.1001",
        // === MALUKU ===
        "ambon" => "81.71.01.1001",
        "ternate" => "82.71.01.1001",
        "sofifi" => "82.08.01.2001",
        "tual" => "81.72.01.1001",
        // === PAPUA ===
        "jayapura" => "91.71.01.1001",
        "manokwari" => "92.01.01.2001",
        "sorong" => "96.71.01.1001",
        "merauke" => "91.01.01.2001",
        "nabire" => "94.01.01.2001",
        "timika" => "94.03.01.2001",
        _ => city, // pass through as raw adm4 code
    }
}

// Keep old NTB constants for backward compatibility
pub const NTB_BBOX: [f64; 4] = [-9.5, 115.46, -7.9, 119.6];
pub const NTB_CENTER: [f64; 2] = [-8.65, 117.5];
pub const NTB_PROVINCE_CODE: &str = "52";
pub const MATARAM_ADM4: &str = "52.71.01.1004";
pub const LOMBOK_BARAT_ADM4: &str = "52.01.01.2001";
pub const SUMBAWA_ADM4: &str = "52.04.01.2001";
pub const BIMA_ADM4: &str = "52.06.01.2001";
pub const DOMPU_ADM4: &str = "52.05.01.2001";

/// Indonesian-specific environmental parameters (from peer-reviewed papers)
pub mod env_params {
    /// K1 deoxygenation rate for Indonesian tropical urban rivers
    /// Ref: Yustiani 2019 (Citarum), 37 citations
    pub const K1_TROPICAL_URBAN_MIN: f64 = 0.10; // per day at 20°C
    pub const K1_TROPICAL_URBAN_MAX: f64 = 0.17;
    
    /// Indonesian domestic wastewater BOD range
    /// Ref: Widyarani et al. 2022, 207 citations
    pub const DOMESTIC_WW_BOD_MIN: f64 = 135.0; // mg/L
    pub const DOMESTIC_WW_BOD_MAX: f64 = 480.0;
    
    /// Peat fire risk: water table depth threshold
    /// Ref: Kartiwa et al. 2025, Choy & Onuma 2025
    pub const PEAT_FIRE_WT_THRESHOLD_CM: f64 = 40.0; // cm below surface
    
    /// Peat subsidence rate (drained peatlands)
    /// Ref: Hoyt et al. 2020 (Nature Geoscience, 167 citations)
    pub const PEAT_SUBSIDENCE_MIN_CM_YR: f64 = 2.0;
    pub const PEAT_SUBSIDENCE_MAX_CM_YR: f64 = 5.0;
    
    /// Coastal subsidence near cities
    /// Ref: van Bijsterveldt et al. 2023 (Nature Sustainability, 37 citations)
    pub const COASTAL_SUBSIDENCE_MIN_CM_YR: f64 = 8.0;
    pub const COASTAL_SUBSIDENCE_MAX_CM_YR: f64 = 20.0;
    
    /// Jakarta PM2.5 annual mean
    /// Ref: Santoso et al. 2020, 97 citations
    pub const JAKARTA_PM25_ANNUAL: f64 = 25.76; // µg/m³
    
    /// Indonesian ARKL body weight
    /// Ref: Pedoman ARKL Kemenkes 2012
    pub const ARKL_BW_ADULT_KG: f64 = 55.0;
    pub const ARKL_BW_CHILD_KG: f64 = 15.0;
    
    /// Indonesia peatland total area
    pub const PEATLAND_AREA_HA: f64 = 15_000_000.0; // ~15 million ha
    /// Indonesia peatland carbon stock
    pub const PEATLAND_CARBON_GT: f64 = 57.0; // Gt C
    
    /// Indonesia mangrove total area
    pub const MANGROVE_AREA_HA: f64 = 3_300_000.0; // ~3.3 million ha
    /// Mangrove value range (USD/ha/year)
    pub const MANGROVE_VALUE_MIN_USD: f64 = 2_950.0;
    pub const MANGROVE_VALUE_MAX_USD: f64 = 189_027.0;
    
    /// Indonesia coral reef total area
    pub const CORAL_REEF_AREA_KM2: f64 = 51_000.0;
    /// Coral species count
    pub const CORAL_SPECIES: u32 = 590;
    /// Reef fish species
    pub const REEF_FISH_SPECIES: u32 = 3000;
}
