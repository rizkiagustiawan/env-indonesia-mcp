use reqwest::Client;

/// Embedded Indonesian protected species data with IUCN status and distribution
struct SpeciesRecord {
    scientific_name: &'static str,
    common_name: &'static str,
    iucn_status: &'static str,
    habitat: &'static str,
    islands: &'static [&'static str],
    lat_range: (f64, f64),
    lon_range: (f64, f64),
}

const SPECIES_DB: &[SpeciesRecord] = &[
    SpeciesRecord {
        scientific_name: "Pongo abelii",
        common_name: "Orangutan Sumatera",
        iucn_status: "CR",
        habitat: "Hutan hujan tropis dataran rendah",
        islands: &["Sumatera"],
        lat_range: (1.0, 4.5),
        lon_range: (95.0, 99.0),
    },
    SpeciesRecord {
        scientific_name: "Pongo pygmaeus",
        common_name: "Orangutan Kalimantan",
        iucn_status: "CR",
        habitat: "Hutan hujan tropis, hutan gambut",
        islands: &["Kalimantan"],
        lat_range: (-3.5, 2.5),
        lon_range: (108.0, 117.0),
    },
    SpeciesRecord {
        scientific_name: "Rhinoceros sondaicus",
        common_name: "Badak Jawa",
        iucn_status: "CR",
        habitat: "Hutan hujan tropis dataran rendah",
        islands: &["Jawa"],
        lat_range: (-7.0, -6.0),
        lon_range: (105.0, 106.5),
    },
    SpeciesRecord {
        scientific_name: "Dicerorhinus sumatrensis",
        common_name: "Badak Sumatera",
        iucn_status: "CR",
        habitat: "Hutan hujan pegunungan dan dataran rendah",
        islands: &["Sumatera", "Kalimantan"],
        lat_range: (-4.0, 5.0),
        lon_range: (95.0, 117.0),
    },
    SpeciesRecord {
        scientific_name: "Panthera tigris sumatrae",
        common_name: "Harimau Sumatera",
        iucn_status: "CR",
        habitat: "Hutan hujan tropis",
        islands: &["Sumatera"],
        lat_range: (-5.5, 5.0),
        lon_range: (95.0, 106.0),
    },
    SpeciesRecord {
        scientific_name: "Elephas maximus sumatranus",
        common_name: "Gajah Sumatera",
        iucn_status: "CR",
        habitat: "Hutan hujan tropis, padang rumput",
        islands: &["Sumatera"],
        lat_range: (-5.0, 4.0),
        lon_range: (95.0, 106.0),
    },
    SpeciesRecord {
        scientific_name: "Nasalis larvatus",
        common_name: "Bekantan",
        iucn_status: "EN",
        habitat: "Hutan mangrove, hutan rawa",
        islands: &["Kalimantan"],
        lat_range: (-4.0, 2.5),
        lon_range: (108.0, 117.5),
    },
    SpeciesRecord {
        scientific_name: "Hylobates moloch",
        common_name: "Owa Jawa",
        iucn_status: "EN",
        habitat: "Hutan hujan pegunungan",
        islands: &["Jawa"],
        lat_range: (-8.0, -6.0),
        lon_range: (105.0, 114.0),
    },
    SpeciesRecord {
        scientific_name: "Presbytis comata",
        common_name: "Surili",
        iucn_status: "EN",
        habitat: "Hutan hujan tropis pegunungan",
        islands: &["Jawa"],
        lat_range: (-7.5, -6.0),
        lon_range: (105.0, 112.0),
    },
    SpeciesRecord {
        scientific_name: "Chelonia mydas",
        common_name: "Penyu Hijau",
        iucn_status: "EN",
        habitat: "Perairan pesisir, padang lamun",
        islands: &[
            "Sumatera",
            "Jawa",
            "Kalimantan",
            "Sulawesi",
            "Papua",
            "Bali",
            "NTB",
            "NTT",
            "Maluku",
        ],
        lat_range: (-11.5, 6.0),
        lon_range: (95.0, 141.0),
    },
    SpeciesRecord {
        scientific_name: "Eretmochelys imbricata",
        common_name: "Penyu Sisik",
        iucn_status: "CR",
        habitat: "Terumbu karang, perairan dangkal",
        islands: &[
            "Sumatera",
            "Jawa",
            "Kalimantan",
            "Sulawesi",
            "Papua",
            "Bali",
            "NTB",
            "NTT",
            "Maluku",
        ],
        lat_range: (-11.5, 6.0),
        lon_range: (95.0, 141.0),
    },
    SpeciesRecord {
        scientific_name: "Dermochelys coriacea",
        common_name: "Penyu Belimbing",
        iucn_status: "VU",
        habitat: "Perairan terbuka, pesisir berpasir",
        islands: &["Sumatera", "Jawa", "Papua", "Kalimantan"],
        lat_range: (-10.0, 5.0),
        lon_range: (95.0, 141.0),
    },
    SpeciesRecord {
        scientific_name: "Varanus komodoensis",
        common_name: "Komodo",
        iucn_status: "EN",
        habitat: "Savana, hutan kering",
        islands: &["NTT", "NTB"],
        lat_range: (-9.0, -8.0),
        lon_range: (119.0, 121.0),
    },
    SpeciesRecord {
        scientific_name: "Babyrousa babyrussa",
        common_name: "Babirusa",
        iucn_status: "VU",
        habitat: "Hutan hujan tropis",
        islands: &["Sulawesi", "Maluku"],
        lat_range: (-5.5, 1.5),
        lon_range: (119.0, 128.0),
    },
    SpeciesRecord {
        scientific_name: "Anoa depressicornis",
        common_name: "Anoa Dataran Rendah",
        iucn_status: "EN",
        habitat: "Hutan hujan dataran rendah",
        islands: &["Sulawesi"],
        lat_range: (-5.5, 1.5),
        lon_range: (119.0, 125.0),
    },
    SpeciesRecord {
        scientific_name: "Bubalus quarlesi",
        common_name: "Anoa Pegunungan",
        iucn_status: "EN",
        habitat: "Hutan pegunungan",
        islands: &["Sulawesi"],
        lat_range: (-5.5, 1.5),
        lon_range: (119.0, 125.0),
    },
    SpeciesRecord {
        scientific_name: "Leucopsar rothschildi",
        common_name: "Jalak Bali",
        iucn_status: "CR",
        habitat: "Hutan kering, savana",
        islands: &["Bali"],
        lat_range: (-8.5, -8.0),
        lon_range: (114.0, 115.5),
    },
    SpeciesRecord {
        scientific_name: "Rafflesia arnoldii",
        common_name: "Rafflesia",
        iucn_status: "CR",
        habitat: "Hutan hujan tropis primer",
        islands: &["Sumatera", "Kalimantan"],
        lat_range: (-4.0, 3.0),
        lon_range: (100.0, 116.0),
    },
    SpeciesRecord {
        scientific_name: "Amorphophallus titanum",
        common_name: "Bunga Bangkai Raksasa",
        iucn_status: "EN",
        habitat: "Hutan hujan tropis",
        islands: &["Sumatera"],
        lat_range: (-4.0, 2.0),
        lon_range: (100.0, 105.0),
    },
    SpeciesRecord {
        scientific_name: "Dugong dugon",
        common_name: "Duyung",
        iucn_status: "VU",
        habitat: "Padang lamun, perairan dangkal",
        islands: &[
            "Sumatera",
            "Kalimantan",
            "Sulawesi",
            "Papua",
            "NTB",
            "NTT",
            "Maluku",
        ],
        lat_range: (-10.0, 4.0),
        lon_range: (95.0, 141.0),
    },
    SpeciesRecord {
        scientific_name: "Macaca nigra",
        common_name: "Yaki / Monyet Hitam Sulawesi",
        iucn_status: "CR",
        habitat: "Hutan hujan tropis",
        islands: &["Sulawesi"],
        lat_range: (-2.0, 2.0),
        lon_range: (123.0, 126.0),
    },
    SpeciesRecord {
        scientific_name: "Tarsius tarsier",
        common_name: "Tarsius Sulawesi",
        iucn_status: "VU",
        habitat: "Hutan primer dan sekunder",
        islands: &["Sulawesi"],
        lat_range: (-5.5, 1.5),
        lon_range: (119.0, 125.0),
    },
    SpeciesRecord {
        scientific_name: "Helarctos malayanus",
        common_name: "Beruang Madu",
        iucn_status: "VU",
        habitat: "Hutan hujan tropis dataran rendah",
        islands: &["Sumatera", "Kalimantan"],
        lat_range: (-4.0, 5.0),
        lon_range: (95.0, 117.0),
    },
    SpeciesRecord {
        scientific_name: "Neofelis diardi",
        common_name: "Macan Dahan",
        iucn_status: "VU",
        habitat: "Hutan hujan tropis",
        islands: &["Sumatera", "Kalimantan"],
        lat_range: (-4.0, 5.0),
        lon_range: (95.0, 117.0),
    },
    SpeciesRecord {
        scientific_name: "Tapirus indicus",
        common_name: "Tapir Asia",
        iucn_status: "EN",
        habitat: "Hutan hujan tropis",
        islands: &["Sumatera"],
        lat_range: (-4.0, 4.0),
        lon_range: (97.0, 105.0),
    },
    SpeciesRecord {
        scientific_name: "Cacatua sulphurea",
        common_name: "Kakatua Kecil Jambul Kuning",
        iucn_status: "CR",
        habitat: "Hutan, perkebunan",
        islands: &["Sulawesi", "NTB", "NTT", "Bali"],
        lat_range: (-10.0, 1.0),
        lon_range: (114.0, 128.0),
    },
    SpeciesRecord {
        scientific_name: "Dendrolagus ursinus",
        common_name: "Kanguru Pohon",
        iucn_status: "VU",
        habitat: "Hutan hujan pegunungan",
        islands: &["Papua"],
        lat_range: (-8.0, -2.0),
        lon_range: (130.0, 141.0),
    },
    SpeciesRecord {
        scientific_name: "Casuarius casuarius",
        common_name: "Kasuari Gelambir Ganda",
        iucn_status: "LC",
        habitat: "Hutan hujan tropis",
        islands: &["Papua", "Maluku"],
        lat_range: (-9.0, -1.0),
        lon_range: (130.0, 141.0),
    },
    SpeciesRecord {
        scientific_name: "Manis javanica",
        common_name: "Trenggiling Jawa",
        iucn_status: "CR",
        habitat: "Hutan primer dan sekunder",
        islands: &["Sumatera", "Jawa", "Kalimantan", "Bali"],
        lat_range: (-8.5, 5.0),
        lon_range: (95.0, 117.0),
    },
    SpeciesRecord {
        scientific_name: "Aceros cassidix",
        common_name: "Rangkong Sulawesi",
        iucn_status: "VU",
        habitat: "Hutan hujan tropis",
        islands: &["Sulawesi"],
        lat_range: (-5.5, 1.5),
        lon_range: (119.0, 125.0),
    },
    SpeciesRecord {
        scientific_name: "Buceros rhinoceros",
        common_name: "Rangkong Badak",
        iucn_status: "VU",
        habitat: "Hutan hujan tropis dataran rendah",
        islands: &["Sumatera", "Kalimantan", "Jawa"],
        lat_range: (-8.0, 5.0),
        lon_range: (95.0, 117.0),
    },
    SpeciesRecord {
        scientific_name: "Numenius madagascariensis",
        common_name: "Gajahan Timur",
        iucn_status: "EN",
        habitat: "Lahan basah pesisir, mudflat",
        islands: &[
            "Sumatera",
            "Jawa",
            "Kalimantan",
            "Sulawesi",
            "NTB",
            "NTT",
            "Papua",
        ],
        lat_range: (-10.0, 5.0),
        lon_range: (95.0, 141.0),
    },
    SpeciesRecord {
        scientific_name: "Rusa timorensis",
        common_name: "Rusa Timor",
        iucn_status: "VU",
        habitat: "Savana, hutan musim",
        islands: &["Jawa", "Bali", "NTB", "NTT", "Sulawesi", "Maluku"],
        lat_range: (-10.0, -1.0),
        lon_range: (105.0, 130.0),
    },
];

/// Determine which island a coordinate falls on (approximate)
fn get_island(lat: f64, lon: f64) -> Vec<&'static str> {
    let mut islands = Vec::new();
    // Approximate bounding boxes for Indonesian islands
    if lat >= -6.0 && lat <= 6.0 && lon >= 95.0 && lon <= 106.0 {
        islands.push("Sumatera");
    }
    if lat >= -8.8 && lat <= -5.5 && lon >= 105.0 && lon <= 114.5 {
        islands.push("Jawa");
    }
    if lat >= -4.5 && lat <= 4.5 && lon >= 108.0 && lon <= 117.5 {
        islands.push("Kalimantan");
    }
    if lat >= -5.8 && lat <= 2.0 && lon >= 119.0 && lon <= 126.0 {
        islands.push("Sulawesi");
    }
    if lat >= -9.0 && lat <= -8.0 && lon >= 114.0 && lon <= 115.8 {
        islands.push("Bali");
    }
    if lat >= -9.5 && lat <= -8.0 && lon >= 115.5 && lon <= 119.5 {
        islands.push("NTB");
    }
    if lat >= -10.5 && lat <= -8.0 && lon >= 119.5 && lon <= 125.5 {
        islands.push("NTT");
    }
    if lat >= -8.5 && lat <= -1.0 && lon >= 124.0 && lon <= 135.0 {
        islands.push("Maluku");
    }
    if lat >= -9.5 && lat <= -0.5 && lon >= 130.0 && lon <= 141.5 {
        islands.push("Papua");
    }
    islands
}

/// Check distance in km between two lat/lon points (Haversine)
fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}

/// Query species near a coordinate using embedded data
/// Since IUCN API requires registration/key, this uses embedded Indonesian species data
pub async fn check_species(_client: &Client, lat: f64, lon: f64, radius_km: f64) -> String {
    let islands = get_island(lat, lon);

    let mut found: Vec<String> = Vec::new();

    for sp in SPECIES_DB {
        // Check if species distribution overlaps with query area
        let mut matches = false;

        // Check island match
        for island in &islands {
            if sp.islands.contains(island) {
                matches = true;
                break;
            }
        }

        // Also check coordinate range overlap with radius
        if !matches {
            let center_lat = (sp.lat_range.0 + sp.lat_range.1) / 2.0;
            let center_lon = (sp.lon_range.0 + sp.lon_range.1) / 2.0;
            let dist = haversine_km(lat, lon, center_lat, center_lon);
            if dist <= radius_km + 200.0 {
                matches = true;
            }
        }

        if matches {
            let status_desc = match sp.iucn_status {
                "CR" => "Critically Endangered (Kritis)",
                "EN" => "Endangered (Terancam Punah)",
                "VU" => "Vulnerable (Rentan)",
                "NT" => "Near Threatened (Hampir Terancam)",
                "LC" => "Least Concern (Risiko Rendah)",
                _ => sp.iucn_status,
            };
            found.push(format!(
                "• {} ({}) - IUCN: {} [{}]\n  Habitat: {}",
                sp.common_name, sp.scientific_name, sp.iucn_status, status_desc, sp.habitat
            ));
        }
    }

    if found.is_empty() {
        return format!(
            "Tidak ditemukan spesies dilindungi dalam database untuk koordinat ({}, {}) radius {} km.\n\
             Catatan: Data ini menggunakan database internal terbatas ({} spesies).\n\
             Untuk data lengkap, gunakan IUCN Red List API (memerlukan API key dari apiv3.iucnredlist.org).",
            lat, lon, radius_km, SPECIES_DB.len()
        );
    }

    let island_str = if islands.is_empty() {
        "Tidak teridentifikasi".to_string()
    } else {
        islands.join(", ")
    };

    format!(
        "══════════════════════════════════════════════\n\
         SPESIES DILINDUNGI DI SEKITAR KOORDINAT\n\
         ══════════════════════════════════════════════\n\
         Koordinat: {}, {}\n\
         Radius: {} km\n\
         Pulau/Wilayah: {}\n\
         Jumlah spesies ditemukan: {}\n\n\
         {}\n\n\
         ══════════════════════════════════════════════\n\
         DISCLAIMER: Data menggunakan database internal ({} spesies Indonesia).\n\
         Untuk survei biodiversitas resmi AMDAL, diperlukan survei lapangan\n\
         dan konsultasi dengan IUCN Red List API (apiv3.iucnredlist.org).\n\
         Ref: PP 7/1999, PermenLHK P.106/2018\n\
         ══════════════════════════════════════════════",
        lat,
        lon,
        radius_km,
        island_str,
        found.len(),
        found.join("\n"),
        SPECIES_DB.len()
    )
}
