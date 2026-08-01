/// Daftar Flora Fauna Dilindungi Indonesia
/// Ref: PP 7/1999, PermenLHK P.106/2018

struct ProtectedSpecies {
    scientific_name: &'static str,
    common_name_id: &'static str,
    common_name_en: &'static str,
    taxon: &'static str,
    iucn_status: &'static str,
    legal_basis: &'static str,
    habitat: &'static str,
    provinces: &'static [&'static str],
}

const PROTECTED_LIST: &[ProtectedSpecies] = &[
    ProtectedSpecies {
        scientific_name: "Pongo abelii",
        common_name_id: "Orangutan Sumatera",
        common_name_en: "Sumatran Orangutan",
        taxon: "Mammalia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis dataran rendah",
        provinces: &["Aceh", "Sumatera Utara"],
    },
    ProtectedSpecies {
        scientific_name: "Pongo pygmaeus",
        common_name_id: "Orangutan Kalimantan",
        common_name_en: "Bornean Orangutan",
        taxon: "Mammalia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis, hutan gambut",
        provinces: &[
            "Kalimantan Barat",
            "Kalimantan Tengah",
            "Kalimantan Timur",
            "Kalimantan Utara",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Rhinoceros sondaicus",
        common_name_id: "Badak Jawa",
        common_name_en: "Javan Rhinoceros",
        taxon: "Mammalia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan dataran rendah",
        provinces: &["Banten"],
    },
    ProtectedSpecies {
        scientific_name: "Dicerorhinus sumatrensis",
        common_name_id: "Badak Sumatera",
        common_name_en: "Sumatran Rhinoceros",
        taxon: "Mammalia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan pegunungan",
        provinces: &[
            "Lampung",
            "Bengkulu",
            "Sumatera Selatan",
            "Kalimantan Timur",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Panthera tigris sumatrae",
        common_name_id: "Harimau Sumatera",
        common_name_en: "Sumatran Tiger",
        taxon: "Mammalia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis",
        provinces: &[
            "Aceh",
            "Sumatera Utara",
            "Riau",
            "Jambi",
            "Sumatera Selatan",
            "Bengkulu",
            "Lampung",
            "Sumatera Barat",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Elephas maximus sumatranus",
        common_name_id: "Gajah Sumatera",
        common_name_en: "Sumatran Elephant",
        taxon: "Mammalia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis, padang rumput",
        provinces: &[
            "Aceh",
            "Riau",
            "Jambi",
            "Sumatera Selatan",
            "Lampung",
            "Bengkulu",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Nasalis larvatus",
        common_name_id: "Bekantan",
        common_name_en: "Proboscis Monkey",
        taxon: "Mammalia",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan mangrove, hutan rawa",
        provinces: &[
            "Kalimantan Barat",
            "Kalimantan Tengah",
            "Kalimantan Selatan",
            "Kalimantan Timur",
            "Kalimantan Utara",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Varanus komodoensis",
        common_name_id: "Komodo",
        common_name_en: "Komodo Dragon",
        taxon: "Reptilia",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Savana, hutan kering tropis",
        provinces: &["Nusa Tenggara Timur"],
    },
    ProtectedSpecies {
        scientific_name: "Chelonia mydas",
        common_name_id: "Penyu Hijau",
        common_name_en: "Green Sea Turtle",
        taxon: "Reptilia",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Perairan pesisir, padang lamun",
        provinces: &[
            "Bali",
            "NTB",
            "NTT",
            "Papua",
            "Kalimantan Timur",
            "Sulawesi Utara",
            "Maluku",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Eretmochelys imbricata",
        common_name_id: "Penyu Sisik",
        common_name_en: "Hawksbill Turtle",
        taxon: "Reptilia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Terumbu karang",
        provinces: &["Bali", "NTB", "NTT", "Papua", "Sulawesi Utara", "Maluku"],
    },
    ProtectedSpecies {
        scientific_name: "Leucopsar rothschildi",
        common_name_id: "Jalak Bali",
        common_name_en: "Bali Myna",
        taxon: "Aves",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan kering, savana",
        provinces: &["Bali"],
    },
    ProtectedSpecies {
        scientific_name: "Cacatua sulphurea",
        common_name_id: "Kakatua Kecil Jambul Kuning",
        common_name_en: "Yellow-crested Cockatoo",
        taxon: "Aves",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan, perkebunan",
        provinces: &["Sulawesi", "NTB", "NTT", "Bali"],
    },
    ProtectedSpecies {
        scientific_name: "Rafflesia arnoldii",
        common_name_id: "Rafflesia / Bunga Padma Raksasa",
        common_name_en: "Corpse Lily",
        taxon: "Flora",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis primer",
        provinces: &["Bengkulu", "Sumatera Barat", "Jambi", "Kalimantan Barat"],
    },
    ProtectedSpecies {
        scientific_name: "Amorphophallus titanum",
        common_name_id: "Bunga Bangkai Raksasa",
        common_name_en: "Titan Arum",
        taxon: "Flora",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis",
        provinces: &["Sumatera Barat", "Bengkulu", "Lampung", "Jambi"],
    },
    ProtectedSpecies {
        scientific_name: "Manis javanica",
        common_name_id: "Trenggiling Jawa",
        common_name_en: "Sunda Pangolin",
        taxon: "Mammalia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan primer dan sekunder",
        provinces: &[
            "Sumatera Barat",
            "Riau",
            "Jambi",
            "Jawa Barat",
            "Jawa Tengah",
            "Jawa Timur",
            "Kalimantan Barat",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Dugong dugon",
        common_name_id: "Duyung",
        common_name_en: "Dugong",
        taxon: "Mammalia",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Padang lamun, perairan dangkal",
        provinces: &[
            "NTB",
            "NTT",
            "Papua",
            "Sulawesi Tenggara",
            "Maluku",
            "Kalimantan Timur",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Helarctos malayanus",
        common_name_id: "Beruang Madu",
        common_name_en: "Sun Bear",
        taxon: "Mammalia",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis dataran rendah",
        provinces: &[
            "Aceh",
            "Sumatera Utara",
            "Riau",
            "Kalimantan Barat",
            "Kalimantan Tengah",
            "Kalimantan Timur",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Neofelis diardi",
        common_name_id: "Macan Dahan Sunda",
        common_name_en: "Sunda Clouded Leopard",
        taxon: "Mammalia",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis",
        provinces: &[
            "Sumatera Barat",
            "Riau",
            "Jambi",
            "Kalimantan Barat",
            "Kalimantan Tengah",
            "Kalimantan Timur",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Tapirus indicus",
        common_name_id: "Tapir Asia",
        common_name_en: "Malayan Tapir",
        taxon: "Mammalia",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis",
        provinces: &[
            "Sumatera Barat",
            "Riau",
            "Jambi",
            "Bengkulu",
            "Sumatera Selatan",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Hylobates moloch",
        common_name_id: "Owa Jawa",
        common_name_en: "Silvery Gibbon",
        taxon: "Mammalia",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan pegunungan",
        provinces: &["Jawa Barat", "Jawa Tengah", "Banten"],
    },
    ProtectedSpecies {
        scientific_name: "Macaca nigra",
        common_name_id: "Yaki / Monyet Hitam Sulawesi",
        common_name_en: "Celebes Crested Macaque",
        taxon: "Mammalia",
        iucn_status: "CR",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis",
        provinces: &["Sulawesi Utara"],
    },
    ProtectedSpecies {
        scientific_name: "Tarsius tarsier",
        common_name_id: "Tarsius Sulawesi",
        common_name_en: "Sulawesi Tarsier",
        taxon: "Mammalia",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan primer dan sekunder",
        provinces: &[
            "Sulawesi Utara",
            "Sulawesi Tengah",
            "Sulawesi Tenggara",
            "Sulawesi Selatan",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Anoa depressicornis",
        common_name_id: "Anoa Dataran Rendah",
        common_name_en: "Lowland Anoa",
        taxon: "Mammalia",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan dataran rendah",
        provinces: &[
            "Sulawesi Utara",
            "Sulawesi Tengah",
            "Sulawesi Tenggara",
            "Sulawesi Selatan",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Babyrousa babyrussa",
        common_name_id: "Babirusa",
        common_name_en: "Babirusa",
        taxon: "Mammalia",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis",
        provinces: &[
            "Sulawesi Utara",
            "Sulawesi Tengah",
            "Gorontalo",
            "Maluku Utara",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Dermochelys coriacea",
        common_name_id: "Penyu Belimbing",
        common_name_en: "Leatherback Sea Turtle",
        taxon: "Reptilia",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Perairan terbuka, pesisir berpasir",
        provinces: &["Papua", "Kalimantan Timur", "Jawa Barat", "Banten"],
    },
    ProtectedSpecies {
        scientific_name: "Dendrolagus ursinus",
        common_name_id: "Kanguru Pohon",
        common_name_en: "Vogelkop Tree-kangaroo",
        taxon: "Mammalia",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan pegunungan",
        provinces: &["Papua Barat", "Papua"],
    },
    ProtectedSpecies {
        scientific_name: "Casuarius casuarius",
        common_name_id: "Kasuari Gelambir Ganda",
        common_name_en: "Southern Cassowary",
        taxon: "Aves",
        iucn_status: "LC",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis",
        provinces: &["Papua", "Papua Barat", "Maluku"],
    },
    ProtectedSpecies {
        scientific_name: "Buceros rhinoceros",
        common_name_id: "Rangkong Badak",
        common_name_en: "Rhinoceros Hornbill",
        taxon: "Aves",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis dataran rendah",
        provinces: &[
            "Sumatera Barat",
            "Riau",
            "Jambi",
            "Kalimantan Barat",
            "Kalimantan Tengah",
            "Kalimantan Timur",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Rusa timorensis",
        common_name_id: "Rusa Timor",
        common_name_en: "Timor Deer",
        taxon: "Mammalia",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Savana, hutan musim",
        provinces: &[
            "Jawa Timur",
            "Bali",
            "NTB",
            "NTT",
            "Sulawesi Selatan",
            "Maluku",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Numenius madagascariensis",
        common_name_id: "Gajahan Timur",
        common_name_en: "Far Eastern Curlew",
        taxon: "Aves",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Lahan basah pesisir, mudflat",
        provinces: &[
            "Sumatera Utara",
            "Jawa Timur",
            "Kalimantan Selatan",
            "Sulawesi Selatan",
            "Papua",
        ],
    },
    ProtectedSpecies {
        scientific_name: "Presbytis comata",
        common_name_id: "Surili",
        common_name_en: "Grizzled Langur",
        taxon: "Mammalia",
        iucn_status: "EN",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan pegunungan",
        provinces: &["Jawa Barat", "Banten", "Jawa Tengah"],
    },
    ProtectedSpecies {
        scientific_name: "Aceros cassidix",
        common_name_id: "Rangkong Sulawesi",
        common_name_en: "Knobbed Hornbill",
        taxon: "Aves",
        iucn_status: "VU",
        legal_basis: "PP 7/1999, PermenLHK P.106/2018",
        habitat: "Hutan hujan tropis",
        provinces: &[
            "Sulawesi Utara",
            "Sulawesi Tengah",
            "Sulawesi Tenggara",
            "Sulawesi Selatan",
        ],
    },
];

/// Check if a species is protected under Indonesian law
/// Search by scientific name or common Indonesian name (case-insensitive)
pub fn check(species_name: &str) -> String {
    let query = species_name.to_lowercase();

    let mut matches: Vec<&ProtectedSpecies> = Vec::new();

    for sp in PROTECTED_LIST {
        if sp.scientific_name.to_lowercase().contains(&query)
            || sp.common_name_id.to_lowercase().contains(&query)
            || sp.common_name_en.to_lowercase().contains(&query)
        {
            matches.push(sp);
        }
    }

    if matches.is_empty() {
        return format!(
            "Spesies '{}' tidak ditemukan dalam database dilindungi ({} spesies).\n\
             Catatan: Spesies mungkin dilindungi tetapi belum ada dalam database internal.\n\
             Periksa daftar lengkap pada PP 7/1999 dan PermenLHK P.106/2018.",
            species_name,
            PROTECTED_LIST.len()
        );
    }

    let mut result = String::from("══════════════════════════════════════════════\n");
    result.push_str("STATUS PERLINDUNGAN SPESIES\n");
    result.push_str("Ref: PP 7/1999, PermenLHK P.106/2018\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    for sp in &matches {
        let status_desc = match sp.iucn_status {
            "CR" => "Critically Endangered (Kritis)",
            "EN" => "Endangered (Terancam Punah)",
            "VU" => "Vulnerable (Rentan)",
            "NT" => "Near Threatened (Hampir Terancam)",
            "LC" => "Least Concern (Risiko Rendah)",
            _ => sp.iucn_status,
        };
        result.push_str(&format!(
            "Nama Ilmiah    : {}\n\
             Nama Indonesia : {}\n\
             Nama Inggris   : {}\n\
             Takson          : {}\n\
             Status IUCN    : {} - {}\n\
             Dasar Hukum    : {}\n\
             Habitat        : {}\n\
             Distribusi     : {}\n\
             STATUS         : DILINDUNGI\n\n",
            sp.scientific_name,
            sp.common_name_id,
            sp.common_name_en,
            sp.taxon,
            sp.iucn_status,
            status_desc,
            sp.legal_basis,
            sp.habitat,
            sp.provinces.join(", ")
        ));
    }

    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("PERINGATAN: Setiap kegiatan yang berdampak pada spesies dilindungi\n");
    result.push_str("memerlukan izin khusus dari Kementerian LHK (BKSDA).\n");
    result.push_str("Pelanggaran dikenai sanksi UU 5/1990 Pasal 21 & 40.\n");
    result.push_str("══════════════════════════════════════════════\n");
    result
}

/// List all protected species known in a province
pub fn list_by_province(province: &str) -> String {
    let query = province.to_lowercase();

    let mut found: Vec<&ProtectedSpecies> = Vec::new();

    for sp in PROTECTED_LIST {
        for prov in sp.provinces {
            if prov.to_lowercase().contains(&query) {
                found.push(sp);
                break;
            }
        }
    }

    if found.is_empty() {
        return format!(
            "Tidak ditemukan spesies dilindungi untuk provinsi '{}' dalam database.\n\
             Provinsi valid: Aceh, Sumatera Utara, Riau, Jambi, Sumatera Barat, Bengkulu,\n\
             Sumatera Selatan, Lampung, Banten, Jawa Barat, Jawa Tengah, Jawa Timur,\n\
             Bali, NTB, NTT, Kalimantan Barat/Tengah/Timur/Selatan/Utara,\n\
             Sulawesi Utara/Tengah/Selatan/Tenggara, Gorontalo, Maluku, Papua, Papua Barat",
            province
        );
    }

    let mut result = format!(
        "══════════════════════════════════════════════\n\
         SPESIES DILINDUNGI DI PROVINSI {}\n\
         Ref: PP 7/1999, PermenLHK P.106/2018\n\
         ══════════════════════════════════════════════\n\
         Jumlah: {} spesies\n\n",
        province.to_uppercase(),
        found.len()
    );

    // Group by taxon
    let taxa = ["Mammalia", "Aves", "Reptilia", "Flora"];
    for taxon in taxa {
        let group: Vec<&&ProtectedSpecies> = found.iter().filter(|s| s.taxon == taxon).collect();
        if !group.is_empty() {
            result.push_str(&format!("--- {} ---\n", taxon));
            for sp in group {
                result.push_str(&format!(
                    "  • {} ({}) [IUCN: {}]\n",
                    sp.common_name_id, sp.scientific_name, sp.iucn_status
                ));
            }
            result.push('\n');
        }
    }

    result.push_str("══════════════════════════════════════════════\n");
    result
}
