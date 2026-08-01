use super::haversine;

pub struct ReefSite {
    pub name: &'static str,
    pub province: &'static str,
    pub lat: f64,
    pub lon: f64,
    pub condition: &'static str,
    pub description: &'static str,
}

pub const REEF_SITES: &[ReefSite] = &[
    ReefSite {
        name: "Raja Ampat",
        province: "Papua Barat Daya",
        lat: -0.23,
        lon: 130.52,
        condition: "Excellent",
        description: "600+ coral, 1700+ fish — global epicenter biodiversity laut",
    },
    ReefSite {
        name: "Bunaken",
        province: "Sulawesi Utara",
        lat: 1.62,
        lon: 124.75,
        condition: "Good",
        description: "390 coral, 2000+ fish, coral wall 25-50m",
    },
    ReefSite {
        name: "Wakatobi",
        province: "Sulawesi Tenggara",
        lat: -5.70,
        lon: 124.00,
        condition: "Good",
        description: "750 coral species, 942 fish, barrier reef terbesar Indonesia",
    },
    ReefSite {
        name: "Komodo",
        province: "NTT",
        lat: -8.55,
        lon: 119.48,
        condition: "Good",
        description: "Arus kuat, whale shark, manta ray, dugong",
    },
    ReefSite {
        name: "Derawan/Berau",
        province: "Kalimantan Timur",
        lat: 2.25,
        lon: 118.42,
        condition: "Good",
        description: "507 coral, 872 fish, nesting penyu hijau terbesar",
    },
    ReefSite {
        name: "Karimunjawa",
        province: "Jawa Tengah",
        lat: -5.82,
        lon: 110.40,
        condition: "Moderate",
        description: "90+ coral, 242 fish, black coral",
    },
    ReefSite {
        name: "Banda Sea",
        province: "Maluku",
        lat: -4.52,
        lon: 129.90,
        condition: "Good",
        description: "310+ coral, 600+ fish, deep-sea coral 3800m+",
    },
    ReefSite {
        name: "Togean",
        province: "Sulawesi Tengah",
        lat: -0.30,
        lon: 121.90,
        condition: "Good",
        description: "262 coral, 596 fish, satu-satunya 4 tipe reef",
    },
    ReefSite {
        name: "Gili Matra",
        province: "NTB",
        lat: -8.35,
        lon: 116.05,
        condition: "Moderate",
        description: "200+ coral, recovering pasca gempa 2018",
    },
    ReefSite {
        name: "Nusa Penida",
        province: "Bali",
        lat: -8.73,
        lon: 115.53,
        condition: "Good",
        description: "298 coral, 576 fish, mola mola musiman",
    },
    ReefSite {
        name: "Takabonerate",
        province: "Sulawesi Selatan",
        lat: -6.68,
        lon: 121.15,
        condition: "Good",
        description: "261 coral, atoll terbesar ke-3 dunia",
    },
    ReefSite {
        name: "Cenderawasih Bay",
        province: "Papua",
        lat: -2.50,
        lon: 134.63,
        condition: "Good",
        description: "150 coral, whale shark resident, TN laut terbesar",
    },
    ReefSite {
        name: "Sawu Sea",
        province: "NTT",
        lat: -10.00,
        lon: 122.00,
        condition: "Good",
        description: "17 cetacea, jalur migrasi Pasifik-Hindia",
    },
    ReefSite {
        name: "Padaido",
        province: "Papua",
        lat: -1.25,
        lon: 136.25,
        condition: "Good",
        description: "250+ coral, relatively pristine",
    },
    ReefSite {
        name: "Anambas",
        province: "Kepulauan Riau",
        lat: 3.50,
        lon: 106.00,
        condition: "Moderate",
        description: "Dugong, Napoleon wrasse, penyu",
    },
];

/// Show all reefs OR nearest reefs if lat/lon provided
pub fn reef_health(lat: Option<f64>, lon: Option<f64>, n: Option<usize>) -> String {
    let mut out = String::from("=== Coral Reef Health — Indonesia ===\n");
    out.push_str("Source: LIPI/BRIN, KKP, Coral Triangle Initiative\n");
    out.push_str("Indonesia: 51,000 km² reef | 590 coral species | 3,000+ reef fish species\n\n");

    match (lat, lon) {
        (Some(la), Some(lo)) => {
            // Nearest sites mode
            let max_n = n.unwrap_or(5).min(REEF_SITES.len());
            let mut distances: Vec<(f64, &ReefSite)> = REEF_SITES
                .iter()
                .map(|s| (haversine(la, lo, s.lat, s.lon), s))
                .collect();
            distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            out.push_str(&format!(
                "TERUMBU KARANG TERDEKAT dari ({:.4}, {:.4}):\n\n",
                la, lo
            ));
            for (i, (dist, site)) in distances.iter().take(max_n).enumerate() {
                out.push_str(&format!(
                    "  {}. {} ({}) — {:.0} km\n",
                    i + 1,
                    site.name,
                    site.province,
                    dist
                ));
                out.push_str(&format!(
                    "     [{:.2}, {:.2}] | {} | {}\n\n",
                    site.lat, site.lon, site.condition, site.description
                ));
            }
        }
        _ => {
            // Show all sites (original behavior)
            out.push_str("MAJOR REEF SITES:\n\n");
            for site in REEF_SITES {
                out.push_str(&format!(
                    "  {} ({}) [{:.2}, {:.2}]\n",
                    site.name, site.province, site.lat, site.lon
                ));
                out.push_str(&format!(
                    "    Kondisi: {} | {}\n\n",
                    site.condition, site.description
                ));
            }
        }
    }

    out.push_str("CORAL TRIANGLE FACTS:\n");
    out.push_str("  Indonesia = 18% terumbu karang dunia (terbesar single-country)\n");
    out.push_str("  Bird's Head Seascape: 574 coral species = 95% Coral Triangle\n");
    out.push_str("  Wallace Line: batas biogeografi melewati Bali-Lombok\n\n");
    out.push_str("Ref: LIPI/BRIN, KKP, Veron et al., Allen & Erdmann 2009, CTI-CFF\n");
    out
}
