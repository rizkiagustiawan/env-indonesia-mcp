/// Baku Mutu Air Laut — KepMen LH No. 51 Tahun 2004
/// Lampiran I: Wisata Bahari, Lampiran II: Biota Laut, Lampiran III: Pelabuhan
/// Ref: Keputusan Menteri Negara Lingkungan Hidup No. 51 Tahun 2004

/// Parameter limit definition
struct ParamLimit {
    min: Option<f64>,
    max: Option<f64>,
    unit: &'static str,
    notes: &'static str,
}

impl ParamLimit {
    fn max_only(max: f64, unit: &'static str) -> Self {
        Self { min: None, max: Some(max), unit, notes: "" }
    }
    fn min_only(min: f64, unit: &'static str) -> Self {
        Self { min: Some(min), max: None, unit, notes: "" }
    }
    fn range(min: f64, max: f64, unit: &'static str) -> Self {
        Self { min: Some(min), max: Some(max), unit, notes: "" }
    }
    fn with_notes(mut self, notes: &'static str) -> Self {
        self.notes = notes;
        self
    }
}

/// Lookup baku mutu for a given parameter and peruntukan category
fn get_limit(parameter: &str, peruntukan: &str) -> Option<ParamLimit> {
    let p = parameter.to_lowercase();
    let k = peruntukan.to_lowercase();

    match k.as_str() {
        // ════════════════════════════════════════════
        // LAMPIRAN I — WISATA BAHARI
        // ════════════════════════════════════════════
        s if s.contains("wisata") => match p.as_str() {
            "ph" => Some(ParamLimit::range(7.0, 8.5, "")),
            "do" => Some(ParamLimit::min_only(5.0, "mg/L")),
            "bod5" | "bod" => Some(ParamLimit::max_only(10.0, "mg/L")),
            "kekeruhan" | "turbidity" => Some(ParamLimit::max_only(5.0, "NTU")),
            "sampah" => Some(ParamLimit::max_only(0.0, "").with_notes("Nihil — tidak boleh ada")),
            "ammonia" | "amonia" | "nh3" | "nh3-n" => {
                Some(ParamLimit::max_only(0.0, "mg/L").with_notes("Nihil"))
            }
            "fosfat" | "po4" | "phosphate" => Some(ParamLimit::max_only(0.015, "mg/L")),
            "nitrat" | "no3" | "nitrate" => Some(ParamLimit::max_only(0.008, "mg/L")),
            "sulfida" | "h2s" | "sulfide" => Some(ParamLimit::max_only(0.03, "mg/L")),
            "surfaktan" | "deterjen" | "mbas" => Some(ParamLimit::max_only(1.0, "mg/L")),
            "minyak_lemak" | "oil_grease" | "minyak" => Some(ParamLimit::max_only(1.0, "mg/L")),
            "fenol" | "phenol" => {
                Some(ParamLimit::max_only(0.0, "mg/L").with_notes("Nihil"))
            }
            "sianida" | "cn" | "cyanide" => Some(ParamLimit::max_only(0.5, "mg/L")),
            "merkuri" | "hg" | "mercury" => Some(ParamLimit::max_only(0.001, "mg/L")),
            "kromium_vi" | "cr6" | "cr(vi)" | "chromium_vi" => {
                Some(ParamLimit::max_only(0.005, "mg/L"))
            }
            "arsenik" | "as" | "arsenic" => Some(ParamLimit::max_only(0.025, "mg/L")),
            "kadmium" | "cd" | "cadmium" => Some(ParamLimit::max_only(0.001, "mg/L")),
            "tembaga" | "cu" | "copper" => Some(ParamLimit::max_only(0.05, "mg/L")),
            "timbal" | "pb" | "lead" => Some(ParamLimit::max_only(0.005, "mg/L")),
            "seng" | "zn" | "zinc" => Some(ParamLimit::max_only(0.05, "mg/L")),
            "nikel" | "ni" | "nickel" => Some(ParamLimit::max_only(0.05, "mg/L")),
            "coliform" | "fecal_coliform" | "total_coliform" => {
                Some(ParamLimit::max_only(1000.0, "MPN/100mL"))
            }
            "suhu" | "temperature" | "suhu_delta" => {
                Some(ParamLimit::max_only(3.0, "°C").with_notes("Deviasi maks dari suhu alami"))
            }
            "salinitas" | "salinity" => {
                Some(ParamLimit::range(33.0, 34.0, "‰").with_notes("Alami"))
            }
            "kecerahan" | "transparency" => {
                Some(ParamLimit::min_only(6.0, "m").with_notes("> 6 m"))
            }
            "bau" | "odor" => {
                Some(ParamLimit::max_only(0.0, "").with_notes("Tidak berbau"))
            }
            "lapisan_minyak" | "oil_layer" => {
                Some(ParamLimit::max_only(0.0, "").with_notes("Nihil"))
            }
            _ => None,
        },

        // ════════════════════════════════════════════
        // LAMPIRAN II — BIOTA LAUT
        // ════════════════════════════════════════════
        s if s.contains("biota") => match p.as_str() {
            "ph" => Some(ParamLimit::range(7.0, 8.5, "")),
            "do" => Some(ParamLimit::min_only(5.0, "mg/L")),
            "bod5" | "bod" => Some(ParamLimit::max_only(20.0, "mg/L")),
            "ammonia" | "amonia" | "nh3" | "nh3-n" => Some(ParamLimit::max_only(0.3, "mg/L")),
            "fosfat" | "po4" | "phosphate" => Some(ParamLimit::max_only(0.015, "mg/L")),
            "nitrat" | "no3" | "nitrate" => Some(ParamLimit::max_only(0.008, "mg/L")),
            "sulfida" | "h2s" | "sulfide" => Some(ParamLimit::max_only(0.01, "mg/L")),
            "surfaktan" | "deterjen" | "mbas" => Some(ParamLimit::max_only(1.0, "mg/L")),
            "minyak_lemak" | "oil_grease" | "minyak" => Some(ParamLimit::max_only(1.0, "mg/L")),
            "fenol" | "phenol" => Some(ParamLimit::max_only(0.002, "mg/L")),
            "sianida" | "cn" | "cyanide" => Some(ParamLimit::max_only(0.5, "mg/L")),
            "merkuri" | "hg" | "mercury" => Some(ParamLimit::max_only(0.001, "mg/L")),
            "kromium_vi" | "cr6" | "cr(vi)" | "chromium_vi" => {
                Some(ParamLimit::max_only(0.005, "mg/L"))
            }
            "arsenik" | "as" | "arsenic" => Some(ParamLimit::max_only(0.025, "mg/L")),
            "kadmium" | "cd" | "cadmium" => Some(ParamLimit::max_only(0.001, "mg/L")),
            "tembaga" | "cu" | "copper" => Some(ParamLimit::max_only(0.008, "mg/L")),
            "timbal" | "pb" | "lead" => Some(ParamLimit::max_only(0.008, "mg/L")),
            "seng" | "zn" | "zinc" => Some(ParamLimit::max_only(0.05, "mg/L")),
            "nikel" | "ni" | "nickel" => Some(ParamLimit::max_only(0.05, "mg/L")),
            "coliform" | "fecal_coliform" | "total_coliform" => {
                Some(ParamLimit::max_only(1000.0, "MPN/100mL"))
            }
            "suhu" | "temperature" => {
                Some(ParamLimit::range(28.0, 30.0, "°C").with_notes("Suhu optimal terumbu karang"))
            }
            "salinitas" | "salinity" => {
                Some(ParamLimit::range(33.0, 34.0, "‰").with_notes("Alami"))
            }
            "kecerahan" | "transparency" => {
                Some(ParamLimit::min_only(5.0, "m").with_notes("> 5 m untuk karang"))
            }
            "tss" => Some(ParamLimit::max_only(20.0, "mg/L")),
            "pah" | "polycyclic_aromatic" => Some(ParamLimit::max_only(0.003, "mg/L")),
            "pcb" => Some(ParamLimit::max_only(0.01, "μg/L")),
            "tributyltin" | "tbt" => Some(ParamLimit::max_only(0.01, "μg/L")),
            "pestisida" | "organoklorin" | "lindane" => {
                Some(ParamLimit::max_only(0.004, "mg/L").with_notes("Total pestisida"))
            }
            "radioaktif" => {
                Some(ParamLimit::max_only(0.0, "").with_notes("Sesuai peraturan BAPETEN"))
            }
            _ => None,
        },

        // ════════════════════════════════════════════
        // LAMPIRAN III — PELABUHAN
        // ════════════════════════════════════════════
        s if s.contains("pelabuhan") || s.contains("harbour") || s.contains("port") => {
            match p.as_str() {
                "ph" => Some(ParamLimit::range(6.5, 8.5, "")),
                "do" => Some(ParamLimit::min_only(4.0, "mg/L")),
                "bod5" | "bod" => Some(ParamLimit::max_only(20.0, "mg/L")),
                "ammonia" | "amonia" | "nh3" | "nh3-n" => Some(ParamLimit::max_only(0.3, "mg/L")),
                "fosfat" | "po4" | "phosphate" => Some(ParamLimit::max_only(0.045, "mg/L")),
                "nitrat" | "no3" | "nitrate" => Some(ParamLimit::max_only(0.02, "mg/L")),
                "sulfida" | "h2s" | "sulfide" => Some(ParamLimit::max_only(0.03, "mg/L")),
                "surfaktan" | "deterjen" | "mbas" => Some(ParamLimit::max_only(1.0, "mg/L")),
                "minyak_lemak" | "oil_grease" | "minyak" => {
                    Some(ParamLimit::max_only(5.0, "mg/L"))
                }
                "fenol" | "phenol" => Some(ParamLimit::max_only(0.002, "mg/L")),
                "sianida" | "cn" | "cyanide" => Some(ParamLimit::max_only(0.5, "mg/L")),
                "merkuri" | "hg" | "mercury" => Some(ParamLimit::max_only(0.003, "mg/L")),
                "kromium_vi" | "cr6" | "cr(vi)" | "chromium_vi" => {
                    Some(ParamLimit::max_only(0.01, "mg/L"))
                }
                "arsenik" | "as" | "arsenic" => Some(ParamLimit::max_only(0.05, "mg/L")),
                "kadmium" | "cd" | "cadmium" => Some(ParamLimit::max_only(0.01, "mg/L")),
                "tembaga" | "cu" | "copper" => Some(ParamLimit::max_only(0.05, "mg/L")),
                "timbal" | "pb" | "lead" => Some(ParamLimit::max_only(0.05, "mg/L")),
                "seng" | "zn" | "zinc" => Some(ParamLimit::max_only(0.1, "mg/L")),
                "nikel" | "ni" | "nickel" => Some(ParamLimit::max_only(0.05, "mg/L")),
                "coliform" | "fecal_coliform" | "total_coliform" => {
                    Some(ParamLimit::max_only(10000.0, "MPN/100mL"))
                }
                "suhu" | "temperature" | "suhu_delta" => {
                    Some(ParamLimit::max_only(3.0, "°C").with_notes("Deviasi maks dari suhu alami"))
                }
                "tss" => Some(ParamLimit::max_only(80.0, "mg/L")),
                "salinitas" | "salinity" => {
                    Some(ParamLimit::range(0.0, 40.0, "‰").with_notes("Alami"))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Check a single parameter against baku mutu air laut
pub fn check(parameter: &str, concentration: f64, peruntukan: &str) -> String {
    let mut out = String::new();
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("  BAKU MUTU AIR LAUT — KepMen LH No. 51/2004\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    let p_lower = peruntukan.to_lowercase();
    let lampiran = if p_lower.contains("wisata") {
        "Lampiran I — Wisata Bahari"
    } else if p_lower.contains("biota") {
        "Lampiran II — Biota Laut"
    } else if p_lower.contains("pelabuhan") || p_lower.contains("harbour") || p_lower.contains("port") {
        "Lampiran III — Pelabuhan"
    } else {
        return format!(
            "ERROR: Peruntukan '{}' tidak dikenal.\n\
             Pilihan: wisata, biota, pelabuhan",
            peruntukan
        );
    };

    out.push_str(&format!("Peruntukan : {}\n", lampiran));
    out.push_str(&format!("Parameter  : {}\n", parameter));
    out.push_str(&format!("Terukur    : {}\n\n", concentration));

    match get_limit(parameter, peruntukan) {
        Some(limit) => {
            // Determine compliance
            let (status, pct_str) = match (limit.min, limit.max) {
                (Some(min), Some(max)) => {
                    // Range check (e.g., pH 7–8.5)
                    if concentration >= min && concentration <= max {
                        let range = max - min;
                        let mid = (min + max) / 2.0;
                        let deviation = ((concentration - mid) / (range / 2.0) * 100.0).abs();
                        ("✅ Memenuhi".to_string(), format!("{:.1}% dari tengah rentang", deviation))
                    } else {
                        let violation = if concentration < min {
                            format!("{:.1}% di bawah batas minimum", ((min - concentration) / min) * 100.0)
                        } else {
                            format!("{:.1}% di atas batas maksimum", ((concentration - max) / max) * 100.0)
                        };
                        ("❌ Melebihi".to_string(), violation)
                    }
                }
                (Some(min), None) => {
                    // Minimum only (e.g., DO > 5)
                    if concentration >= min {
                        let pct = (concentration / min) * 100.0;
                        ("✅ Memenuhi".to_string(), format!("{:.1}% dari batas minimum", pct))
                    } else {
                        let pct = (concentration / min) * 100.0;
                        ("❌ Melebihi".to_string(), format!("{:.1}% dari batas minimum (kurang)", pct))
                    }
                }
                (None, Some(max)) => {
                    // Maximum only (e.g., BOD5 ≤ 10)
                    if max == 0.0 {
                        // Nihil parameter
                        if concentration <= 0.0 {
                            ("✅ Memenuhi".to_string(), "Nihil (tidak terdeteksi)".to_string())
                        } else {
                            ("❌ Melebihi".to_string(), "Harus nihil (0), tetapi terdeteksi".to_string())
                        }
                    } else if concentration <= max {
                        let pct = (concentration / max) * 100.0;
                        ("✅ Memenuhi".to_string(), format!("{:.1}% dari baku mutu", pct))
                    } else {
                        let pct = (concentration / max) * 100.0;
                        ("❌ Melebihi".to_string(), format!("{:.1}% dari baku mutu", pct))
                    }
                }
                (None, None) => ("⚠️ Tidak dapat dievaluasi".to_string(), "-".to_string()),
            };

            // Build limit display string
            let limit_str = match (limit.min, limit.max) {
                (Some(min), Some(max)) => format!("{} – {} {}", min, max, limit.unit),
                (Some(min), None) => format!("≥ {} {}", min, limit.unit),
                (None, Some(max)) => {
                    if max == 0.0 {
                        format!("Nihil {}", limit.unit)
                    } else {
                        format!("≤ {} {}", max, limit.unit)
                    }
                }
                (None, None) => "-".to_string(),
            };

            out.push_str(&format!("Baku Mutu  : {}\n", limit_str));
            out.push_str(&format!("Status     : {}\n", status));
            out.push_str(&format!("% BM       : {}\n", pct_str));

            if !limit.notes.is_empty() {
                out.push_str(&format!("Catatan    : {}\n", limit.notes));
            }
        }
        None => {
            out.push_str(&format!(
                "⚠️ Parameter '{}' tidak ditemukan untuk peruntukan '{}'.\n\n",
                parameter, peruntukan
            ));
            out.push_str("Parameter tersedia:\n");
            out.push_str("  Fisika  : ph, do, bod5, tss, kekeruhan, suhu, salinitas, kecerahan\n");
            out.push_str("  Kimia   : ammonia, fosfat, nitrat, sulfida, surfaktan, minyak_lemak\n");
            out.push_str("            fenol, sianida\n");
            out.push_str("  Logam   : merkuri/hg, kromium_vi/cr6, arsenik/as, kadmium/cd\n");
            out.push_str("            tembaga/cu, timbal/pb, seng/zn, nikel/ni\n");
            out.push_str("  Biologi : coliform\n");
            out.push_str("  Organik : pah, pcb, tributyltin, pestisida (biota only)\n");
        }
    }

    out.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out
}

/// Check multiple parameters at once. Accepts JSON array: [{"parameter":"ph","value":7.5}, ...]
pub fn check_multi(parameters_json: &str, peruntukan: &str) -> String {
    // Parse JSON manually (no serde dependency assumed)
    let mut out = String::new();
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("  BAKU MUTU AIR LAUT — EVALUASI MULTI-PARAMETER\n");
    out.push_str("  KepMen LH No. 51 Tahun 2004\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    let p_lower = peruntukan.to_lowercase();
    let lampiran = if p_lower.contains("wisata") {
        "Lampiran I — Wisata Bahari"
    } else if p_lower.contains("biota") {
        "Lampiran II — Biota Laut"
    } else if p_lower.contains("pelabuhan") || p_lower.contains("harbour") || p_lower.contains("port") {
        "Lampiran III — Pelabuhan"
    } else {
        return format!(
            "ERROR: Peruntukan '{}' tidak dikenal.\nPilihan: wisata, biota, pelabuhan",
            peruntukan
        );
    };

    out.push_str(&format!("Peruntukan: {}\n\n", lampiran));

    // Simple JSON parser for array of {parameter, value}
    // Expected: [{"parameter":"ph","value":7.5},{"parameter":"do","value":6.0}]
    let trimmed = parameters_json.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return "ERROR: Input harus berupa JSON array.\nFormat: [{\"parameter\":\"ph\",\"value\":7.5},{\"parameter\":\"do\",\"value\":6.0}]".into();
    }

    let inner = &trimmed[1..trimmed.len() - 1];

    // Split by objects — find matching braces
    let mut entries: Vec<(String, f64)> = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let chars: Vec<char> = inner.chars().collect();

    for i in 0..chars.len() {
        match chars[i] {
            '{' => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let obj_str: String = chars[start..=i].iter().collect();
                    // Extract parameter and value from object string
                    if let Some(entry) = parse_param_value(&obj_str) {
                        entries.push(entry);
                    }
                }
            }
            _ => {}
        }
    }

    if entries.is_empty() {
        return "ERROR: Tidak ada parameter yang valid dalam JSON.\nFormat: [{\"parameter\":\"ph\",\"value\":7.5}]".into();
    }

    // Table header
    out.push_str(&format!(
        "{:<20} {:>12} {:>15} {:>10} {}\n",
        "PARAMETER", "TERUKUR", "BAKU MUTU", "% BM", "STATUS"
    ));
    out.push_str(&format!("{}\n", "─".repeat(75)));

    let mut pass_count = 0;
    let mut fail_count = 0;
    let mut unknown_count = 0;

    for (param, value) in &entries {
        match get_limit(param, peruntukan) {
            Some(limit) => {
                let (status_icon, pct_display, passes) = match (limit.min, limit.max) {
                    (Some(min), Some(max)) => {
                        if *value >= min && *value <= max {
                            ("✅", format!("-"), true)
                        } else {
                            ("❌", format!("-"), false)
                        }
                    }
                    (Some(min), None) => {
                        let pct = (*value / min) * 100.0;
                        if *value >= min {
                            ("✅", format!("{:.0}%", pct), true)
                        } else {
                            ("❌", format!("{:.0}%", pct), false)
                        }
                    }
                    (None, Some(max)) => {
                        if max == 0.0 {
                            if *value <= 0.0 {
                                ("✅", "nihil".to_string(), true)
                            } else {
                                ("❌", ">nihil".to_string(), false)
                            }
                        } else {
                            let pct = (*value / max) * 100.0;
                            if *value <= max {
                                ("✅", format!("{:.0}%", pct), true)
                            } else {
                                ("❌", format!("{:.0}%", pct), false)
                            }
                        }
                    }
                    _ => ("⚠️", "-".to_string(), false),
                };

                let limit_str = match (limit.min, limit.max) {
                    (Some(min), Some(max)) => format!("{}-{} {}", min, max, limit.unit),
                    (Some(min), None) => format!("≥{} {}", min, limit.unit),
                    (None, Some(max)) => {
                        if max == 0.0 {
                            "nihil".to_string()
                        } else {
                            format!("≤{} {}", max, limit.unit)
                        }
                    }
                    _ => "-".to_string(),
                };

                if passes {
                    pass_count += 1;
                } else {
                    fail_count += 1;
                }

                out.push_str(&format!(
                    "{:<20} {:>12.4} {:>15} {:>10} {}\n",
                    param, value, limit_str, pct_display, status_icon
                ));
            }
            None => {
                unknown_count += 1;
                out.push_str(&format!(
                    "{:<20} {:>12.4} {:>15} {:>10} ⚠️ tidak dikenal\n",
                    param, value, "-", "-"
                ));
            }
        }
    }

    let total = pass_count + fail_count;
    out.push_str(&format!("\n{}\n", "─".repeat(75)));
    out.push_str(&format!(
        "RINGKASAN: {} parameter dievaluasi\n",
        total + unknown_count
    ));
    out.push_str(&format!("  ✅ Memenuhi  : {}\n", pass_count));
    out.push_str(&format!("  ❌ Melebihi  : {}\n", fail_count));
    if unknown_count > 0 {
        out.push_str(&format!("  ⚠️ Tidak dikenal: {}\n", unknown_count));
    }

    let compliance_pct = if total > 0 {
        (pass_count as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    out.push_str(&format!("  Tingkat kepatuhan: {:.1}%\n\n", compliance_pct));

    if fail_count == 0 && unknown_count == 0 {
        out.push_str("STATUS KESELURUHAN: ✅ MEMENUHI BAKU MUTU\n");
    } else if fail_count > 0 {
        out.push_str("STATUS KESELURUHAN: ❌ TIDAK MEMENUHI BAKU MUTU\n");
        out.push_str("Tindak lanjut: Identifikasi sumber pencemar dan lakukan pengendalian.\n");
    }

    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    out
}

/// Parse a JSON object string like {"parameter":"ph","value":7.5}
fn parse_param_value(obj: &str) -> Option<(String, f64)> {
    // Extract "parameter" field
    let param = extract_string_field(obj, "parameter")?;
    // Extract "value" field
    let value = extract_number_field(obj, "value")?;
    Some((param, value))
}

fn extract_string_field(obj: &str, field: &str) -> Option<String> {
    let key = format!("\"{}\"", field);
    let pos = obj.find(&key)?;
    let after_key = &obj[pos + key.len()..];
    // Skip whitespace and colon
    let after_colon = after_key.find(':')?;
    let value_part = &after_key[after_colon + 1..];
    let value_trimmed = value_part.trim_start();
    // Find opening quote
    if !value_trimmed.starts_with('"') {
        return None;
    }
    let inner = &value_trimmed[1..];
    let end_quote = inner.find('"')?;
    Some(inner[..end_quote].to_string())
}

fn extract_number_field(obj: &str, field: &str) -> Option<f64> {
    let key = format!("\"{}\"", field);
    let pos = obj.find(&key)?;
    let after_key = &obj[pos + key.len()..];
    let after_colon = after_key.find(':')?;
    let value_part = &after_key[after_colon + 1..];
    let value_trimmed = value_part.trim_start();
    // Read number characters
    let num_str: String = value_trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == 'e' || *c == 'E' || *c == '+')
        .collect();
    num_str.parse().ok()
}
