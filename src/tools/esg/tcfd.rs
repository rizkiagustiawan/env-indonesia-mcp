pub fn risk_assessment(sector: &str, location: &str) -> String {
    let loc = location.to_lowercase();
    let sec = sector.to_lowercase();

    let physical_risks = match loc.as_str() {
        l if l.contains("jakarta") || l.contains("semarang") || l.contains("demak") => vec![
            (
                "Sea Level Rise & Subsidence",
                "KRITIS",
                "Kenaikan muka air laut + penurunan tanah (subsidence) ekstrim s.d. 10 cm/tahun",
            ),
            (
                "Flood",
                "SANGAT TINGGI",
                "Banjir rob (tidal flood) dan banjir kiriman",
            ),
            (
                "Heat Stress",
                "TINGGI",
                "Urban heat island effect memperburuk gelombang panas",
            ),
        ],
        l if l.contains("kalimantan")
            || l.contains("riau")
            || l.contains("jambi")
            || l.contains("palangkaraya")
            || l.contains("pontianak") =>
        {
            vec![
                (
                    "Wildfire / Karhutla",
                    "SANGAT TINGGI",
                    "Risiko kebakaran lahan gambut sangat tinggi selama kemarau/El Niño",
                ),
                (
                    "Drought",
                    "TINGGI",
                    "Defisit hidrologis di lahan gambut memperparah risiko api",
                ),
                (
                    "Flood",
                    "TINGGI",
                    "Banjir di daerah aliran sungai (DAS) besar",
                ),
            ]
        }
        l if l.contains("sumatera barat") || l.contains("padang") || l.contains("bengkulu") => {
            vec![
                (
                    "Earthquake & Tsunami",
                    "SANGAT TINGGI",
                    "Ancaman megathrust Mentawai",
                ),
                (
                    "Landslide",
                    "TINGGI",
                    "Longsor di kawasan Bukit Barisan akibat curah hujan tinggi",
                ),
            ]
        }
        l if l.contains("papua") || l.contains("maluku") => vec![
            (
                "Flood",
                "TINGGI",
                "Risiko banjir bandang di kawasan lembah / deforestasi",
            ),
            (
                "Biodiversity Loss",
                "TINGGI",
                "Ancaman terhadap ekosistem endemik akibat perubahan suhu",
            ),
        ],
        l if l.contains("lombok") => vec![
            (
                "Earthquake",
                "SANGAT TINGGI",
                "Active seismic zone — 2018 Lombok earthquake",
            ),
            ("Flood", "TINGGI", "Monsoon flooding"),
            ("Drought", "SEDANG", "Kekeringan musiman"),
        ],
        l if l.contains("sumbawa") => vec![
            ("Earthquake", "TINGGI", "Near Flores thrust fault"),
            (
                "Drought",
                "SANGAT TINGGI",
                "Kekeringan ekstrem, defisit air bagi pertanian",
            ),
        ],
        _ => vec![
            (
                "Extreme Weather",
                "TINGGI",
                "Pergeseran musim dan cuaca ekstrem secara umum",
            ),
            (
                "Flood",
                "TINGGI",
                "Banjir akibat intensitas curah hujan tinggi (La Niña)",
            ),
            ("Drought", "SEDANG-TINGGI", "Dampak El Niño"),
        ],
    };

    let transition_risks = match sec.as_str() {
        "agriculture" | "pertanian" => vec![
            (
                "Policy",
                "MODERATE",
                "Carbon tax on agricultural emissions (methane from rice paddies)",
            ),
            (
                "Market",
                "HIGH",
                "Shifting demand toward sustainable/organic products",
            ),
            (
                "Physical-Driven",
                "SANGAT TINGGI",
                "Kegagalan panen mengubah struktur rantai pasok dan asuransi",
            ),
        ],
        "mining" | "tambang" | "pertambangan" => vec![
            (
                "Policy",
                "SANGAT TINGGI",
                "Regulasi reklamasi ketat, kewajiban transisi energi, carbon tax",
            ),
            (
                "Market",
                "TINGGI",
                "Divestasi global dari fossil fuels dan tambang berisiko tinggi",
            ),
            (
                "Reputation",
                "TINGGI",
                "Tekanan sosial/LSM terkait deforestasi dan pencemaran air",
            ),
        ],
        "energy" | "energi" | "migas" => vec![
            (
                "Policy",
                "SANGAT TINGGI",
                "JETP (Just Energy Transition Partnership), coal phase-out",
            ),
            (
                "Technology",
                "TINGGI",
                "Risiko stranded assets untuk PLTU, biaya capex EBT",
            ),
            (
                "Market",
                "TINGGI",
                "Penurunan biaya solar PV & baterai mengancam energi fosil",
            ),
        ],
        "forestry" | "kehutanan" | "pulp" | "paper" => vec![
            (
                "Policy",
                "SANGAT TINGGI",
                "Moratorium sawit/hutan, SVLK, EUDR (EU Deforestation Regulation)",
            ),
            (
                "Market",
                "TINGGI",
                "Eksport sangat bergantung pada sertifikasi zero-deforestation",
            ),
        ],
        "tourism" | "pariwisata" => vec![
            (
                "Physical",
                "TINGGI",
                "Ancaman pada aset di pesisir (sea level rise) dan coral bleaching",
            ),
            (
                "Market",
                "SEDANG",
                "Perubahan preferensi konsumen ke eco-tourism",
            ),
        ],
        "finance" | "perbankan" | "bank" => vec![
            (
                "Policy",
                "TINGGI",
                "Kewajiban reporting POJK 51/2017, stress test iklim OJK",
            ),
            (
                "Market",
                "TINGGI",
                "Non-Performing Loans (NPL) naik di sektor brown (tinggi karbon/risiko fisik)",
            ),
        ],
        _ => vec![
            (
                "Policy",
                "SEDANG",
                "Kewajiban pelaporan emisi & pajak karbon di masa depan",
            ),
            (
                "Market",
                "SEDANG",
                "Tekanan rantai pasok global untuk dekarbonisasi",
            ),
        ],
    };

    let mut out = format!(
        "=== TCFD Climate Risk Assessment ===\nSector: {}\nLocation: {} (Indonesia)\n\n",
        sector, location
    );

    out.push_str("I. PHYSICAL RISKS:\n");
    for (risk, level, desc) in &physical_risks {
        let emoji = match *level {
            "VERY HIGH" => "🔴",
            "HIGH" => "🟠",
            "MODERATE" => "🟡",
            _ => "🟢",
        };
        out.push_str(&format!("  {} {} [{}] — {}\n", emoji, risk, level, desc));
    }

    out.push_str("\nII. TRANSITION RISKS:\n");
    for (risk, level, desc) in &transition_risks {
        let emoji = match *level {
            "VERY HIGH" => "🔴",
            "HIGH" => "🟠",
            "MODERATE" => "🟡",
            _ => "🟢",
        };
        out.push_str(&format!("  {} {} [{}] — {}\n", emoji, risk, level, desc));
    }

    out.push_str("\nIII. TCFD FRAMEWORK:\n");
    out.push_str("  1. Governance — Board oversight of climate risks\n");
    out.push_str("  2. Strategy — Impact on business under climate scenarios\n");
    out.push_str("  3. Risk Management — Processes for identifying & managing climate risks\n");
    out.push_str("  4. Metrics & Targets — GHG emissions, climate targets\n");
    out.push_str("\nReference: https://www.fsb-tcfd.org/\n");
    out.push_str("Indonesia context: POJK 51/2017, Indonesia Climate Risk Atlas (BAPPENAS)\n");
    out
}
