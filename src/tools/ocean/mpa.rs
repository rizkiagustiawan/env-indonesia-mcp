use super::haversine;

pub struct MpaSite {
    pub name: &'static str,
    pub mpa_type: &'static str,
    pub province: &'static str,
    pub area_ha: u64,
    pub year: u32,
    pub lat: f64,
    pub lon: f64,
    pub description: &'static str,
}

pub const MPA_SITES: &[MpaSite] = &[
    MpaSite { name: "TNP Laut Sawu", mpa_type: "Taman Nasional Perairan", province: "NTT", area_ha: 3_521_130, year: 2014, lat: -10.0, lon: 122.0, description: "MPA terbesar Indonesia. 17 cetacea, 5 penyu" },
    MpaSite { name: "TN Teluk Cenderawasih", mpa_type: "Taman Nasional Laut", province: "Papua Barat", area_ha: 1_453_500, year: 2002, lat: -2.5, lon: 134.6, description: "TN laut terbesar. Whale shark resident" },
    MpaSite { name: "TN Wakatobi", mpa_type: "Taman Nasional Laut", province: "Sulawesi Tenggara", area_ha: 1_390_000, year: 2002, lat: -5.7, lon: 124.0, description: "750 coral spp. UNESCO Biosphere Reserve" },
    MpaSite { name: "KKP Anambas", mpa_type: "KKPD", province: "Kepulauan Riau", area_ha: 1_262_686, year: 2014, lat: 3.5, lon: 106.0, description: "Dugong, Napoleon wrasse, penyu" },
    MpaSite { name: "TWP Raja Ampat", mpa_type: "Taman Wisata Perairan", province: "Papua Barat Daya", area_ha: 600_000, year: 2009, lat: -0.5, lon: 130.5, description: "600+ coral spp. UNESCO Global Geopark" },
    MpaSite { name: "TN Takabonerate", mpa_type: "Taman Nasional Laut", province: "Sulawesi Selatan", area_ha: 530_765, year: 1992, lat: -6.7, lon: 121.2, description: "Atoll terbesar ke-3 dunia" },
    MpaSite { name: "TN Kepulauan Togean", mpa_type: "Taman Nasional", province: "Sulawesi Tengah", area_ha: 362_000, year: 2004, lat: -0.3, lon: 121.9, description: "4 tipe reef. Coconut crab" },
    MpaSite { name: "KKP Berau/Derawan", mpa_type: "KKPD", province: "Kalimantan Timur", area_ha: 280_000, year: 2005, lat: 2.1, lon: 118.1, description: "507 coral, nesting penyu terbesar" },
    MpaSite { name: "TWP Padaido", mpa_type: "Taman Wisata Perairan", province: "Papua", area_ha: 183_000, year: 2009, lat: -1.3, lon: 136.3, description: "250+ coral, pristine" },
    MpaSite { name: "TN Komodo", mpa_type: "Taman Nasional", province: "NTT", area_ha: 173_300, year: 1980, lat: -8.6, lon: 119.5, description: "UNESCO WHS. Komodo dragon, manta" },
    MpaSite { name: "TN Karimunjawa", mpa_type: "Taman Nasional Laut", province: "Jawa Tengah", area_ha: 111_625, year: 2001, lat: -5.8, lon: 110.4, description: "90+ coral, black coral, penyu" },
    MpaSite { name: "TN Kepulauan Seribu", mpa_type: "Taman Nasional Laut", province: "DKI Jakarta", area_ha: 107_489, year: 1982, lat: -5.6, lon: 106.6, description: "Terdekat ke ibukota" },
    MpaSite { name: "TN Bunaken", mpa_type: "Taman Nasional Laut", province: "Sulawesi Utara", area_ha: 89_065, year: 1991, lat: 1.6, lon: 124.8, description: "390 coral, 2000+ fish" },
    MpaSite { name: "TWP Kapoposang", mpa_type: "Taman Wisata Perairan", province: "Sulawesi Selatan", area_ha: 50_000, year: 1999, lat: -4.7, lon: 118.9, description: "Napoleon wrasse, penyu" },
    MpaSite { name: "KKP Nusa Penida", mpa_type: "KKPD", province: "Bali", area_ha: 20_057, year: 2010, lat: -8.7, lon: 115.5, description: "Mola mola, manta ray, 298 coral spp" },
    MpaSite { name: "TWP Gili Matra", mpa_type: "Taman Wisata Perairan", province: "NTB", area_ha: 2_954, year: 2001, lat: -8.4, lon: 116.1, description: "Penyu hijau, reef shark" },
];

/// Show all MPAs OR nearest MPAs if lat/lon provided
pub fn protected_areas(lat: Option<f64>, lon: Option<f64>, n: Option<usize>) -> String {
    let mut out = String::from("=== Kawasan Konservasi Perairan — Indonesia ===\n");
    out.push_str("Source: KKP, KLHK, UNEP-WCMC Protected Planet\n");
    out.push_str("Total MPA: ~28.4 juta ha (target 30 juta ha 2030)\n\n");
    
    match (lat, lon) {
        (Some(la), Some(lo)) => {
            let max_n = n.unwrap_or(5).min(MPA_SITES.len());
            let mut distances: Vec<(f64, &MpaSite)> = MPA_SITES.iter()
                .map(|s| (haversine(la, lo, s.lat, s.lon), s))
                .collect();
            distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            
            out.push_str(&format!("KKP TERDEKAT dari ({:.4}, {:.4}):\n\n", la, lo));
            for (i, (dist, site)) in distances.iter().take(max_n).enumerate() {
                out.push_str(&format!("  {}. {} — {:.0} km\n", i+1, site.name, dist));
                out.push_str(&format!("     {} | {} | {} ha | Est. {}\n", site.mpa_type, site.province, site.area_ha, site.year));
                out.push_str(&format!("     [{:.1}, {:.1}] | {}\n\n", site.lat, site.lon, site.description));
            }
        }
        _ => {
            // Show all (original behavior) — table format
            out.push_str(&format!("{:<30} {:>10} {:<20} {:<15} {:<5}\n", "Nama", "Area (ha)", "Tipe", "Provinsi", "Thn"));
            out.push_str(&"-".repeat(85));
            out.push_str("\n");
            let mut total = 0u64;
            for s in MPA_SITES {
                total += s.area_ha;
                out.push_str(&format!("{:<30} {:>10} {:<20} {:<15} {:<5}\n",
                    &s.name[..s.name.len().min(29)], s.area_ha,
                    &s.mpa_type[..s.mpa_type.len().min(19)],
                    &s.province[..s.province.len().min(14)], s.year));
            }
            out.push_str(&format!("\nTOTAL: {} ha ({:.1} juta ha)\n", total, total as f64 / 1e6));
        }
    }
    
    out.push_str("\nTipe KKP: TN (Taman Nasional) | TWP (Taman Wisata Perairan) | SAP (Suaka Alam) | KKPD (Daerah)\n");
    out.push_str("Ref: KKP, KLHK, UNEP-WCMC, CTI-CFF\n");
    out
}
