pub fn map_activity(activity: &str) -> String {
    let lower = activity.to_lowercase();
    let mut matches = Vec::new();

    let sdg_keywords: &[(&str, &str, &[&str])] = &[
        (
            "SDG 1",
            "No Poverty",
            &["poverty", "kemiskinan", "income", "pendapatan", "welfare"],
        ),
        (
            "SDG 2",
            "Zero Hunger",
            &[
                "hunger",
                "food",
                "agriculture",
                "pertanian",
                "pangan",
                "nutrisi",
            ],
        ),
        (
            "SDG 3",
            "Good Health",
            &["health", "kesehatan", "disease", "sanitation", "pollution"],
        ),
        (
            "SDG 4",
            "Quality Education",
            &["education", "pendidikan", "training", "pelatihan", "school"],
        ),
        (
            "SDG 5",
            "Gender Equality",
            &["gender", "women", "perempuan", "equality"],
        ),
        (
            "SDG 6",
            "Clean Water",
            &[
                "water",
                "air",
                "sanitation",
                "sanitasi",
                "clean water",
                "PDAM",
            ],
        ),
        (
            "SDG 7",
            "Affordable Energy",
            &[
                "energy",
                "energi",
                "solar",
                "renewable",
                "EBT",
                "listrik",
                "PLTS",
            ],
        ),
        (
            "SDG 8",
            "Decent Work",
            &[
                "employment",
                "kerja",
                "economic",
                "ekonomi",
                "tourism",
                "pariwisata",
            ],
        ),
        (
            "SDG 9",
            "Industry & Innovation",
            &[
                "industry",
                "industri",
                "infrastructure",
                "infrastruktur",
                "innovation",
            ],
        ),
        (
            "SDG 10",
            "Reduced Inequalities",
            &["inequality", "ketimpangan", "inclusion", "inklusi"],
        ),
        (
            "SDG 11",
            "Sustainable Cities",
            &[
                "urban",
                "city",
                "kota",
                "transport",
                "housing",
                "waste management",
            ],
        ),
        (
            "SDG 12",
            "Responsible Consumption",
            &[
                "consumption",
                "production",
                "waste",
                "limbah",
                "circular",
                "recycling",
            ],
        ),
        (
            "SDG 13",
            "Climate Action",
            &[
                "climate",
                "iklim",
                "carbon",
                "emission",
                "emisi",
                "adaptation",
                "mitigation",
                "GRK",
            ],
        ),
        (
            "SDG 14",
            "Life Below Water",
            &[
                "ocean",
                "marine",
                "laut",
                "coral",
                "fisheries",
                "perikanan",
                "mangrove",
            ],
        ),
        (
            "SDG 15",
            "Life on Land",
            &[
                "forest",
                "hutan",
                "biodiversity",
                "land",
                "deforestation",
                "ecosystem",
                "rinjani",
            ],
        ),
        (
            "SDG 16",
            "Peace & Justice",
            &[
                "peace",
                "justice",
                "governance",
                "institution",
                "corruption",
            ],
        ),
        (
            "SDG 17",
            "Partnerships",
            &[
                "partnership",
                "cooperation",
                "kerjasama",
                "international",
                "financing",
            ],
        ),
    ];

    for (sdg, name, keywords) in sdg_keywords {
        for kw in *keywords {
            if lower.contains(kw) {
                matches.push((*sdg, *name, *kw));
                break;
            }
        }
    }

    let mut out = format!("=== SDG Mapping ===\nActivity: {}\n\n", activity);
    if matches.is_empty() {
        // Default match based on environmental context
        out.push_str("No direct keyword match. Based on Environmental Engineering context:\n");
        out.push_str("  → SDG 6 (Clean Water & Sanitation)\n");
        out.push_str("  → SDG 13 (Climate Action)\n");
        out.push_str("  → SDG 15 (Life on Land)\n");
    } else {
        out.push_str(&format!("Matched {} SDGs:\n\n", matches.len()));
        for (sdg, name, keyword) in &matches {
            out.push_str(&format!("  {} — {} (matched: '{}')\n", sdg, name, keyword));
        }
    }
    out.push_str("\nReference: UN SDGs → https://sdgs.un.org/goals\n");
    out.push_str("Indonesia SDG Dashboard: https://sdgs.bappenas.go.id/\n");
    out
}
