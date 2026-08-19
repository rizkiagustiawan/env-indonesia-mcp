use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Baku Mutu Air Laut — KepMen LH No. 51 Tahun 2004
/// Lampiran I: Wisata Bahari, Lampiran II: Biota Laut, Lampiran III: Pelabuhan
/// Ref: Keputusan Menteri Negara Lingkungan Hidup No. 51 Tahun 2004

/// Parameter limit definition
struct ParamLimit {
    min: Option<f64>,
    max: Option<f64>,
    unit: &'static str,
    _notes: &'static str,
}

impl ParamLimit {
    fn max_only(max: f64, unit: &'static str) -> Self {
        Self {
            min: None,
            max: Some(max),
            unit,
            _notes: "",
        }
    }
    fn min_only(min: f64, unit: &'static str) -> Self {
        Self {
            min: Some(min),
            max: None,
            unit,
            _notes: "",
        }
    }
    fn range(min: f64, max: f64, unit: &'static str) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
            unit,
            _notes: "",
        }
    }
    fn with_notes(mut self, notes: &'static str) -> Self {
        self._notes = notes;
        self
    }
}

/// Lookup baku mutu for a given parameter and peruntukan category
fn get_limit(parameter: &str, peruntukan: &str) -> Option<ParamLimit> {
    let p = parameter.to_lowercase();
    let k = peruntukan.to_lowercase();

    match k.as_str() {
        s if s.contains("wisata") => match p.as_str() {
            "ph" => Some(ParamLimit::range(7.0, 8.5, "")),
            "do" => Some(ParamLimit::min_only(5.0, "mg/L")),
            "bod5" | "bod" => Some(ParamLimit::max_only(10.0, "mg/L")),
            "kekeruhan" | "turbidity" => Some(ParamLimit::max_only(5.0, "NTU")),
            "sampah" => Some(ParamLimit::max_only(0.0, "").with_notes("Nihil")),
            "ammonia" | "amonia" | "nh3" | "nh3-n" => {
                Some(ParamLimit::max_only(0.0, "mg/L").with_notes("Nihil"))
            }
            "fosfat" | "po4" | "phosphate" => Some(ParamLimit::max_only(0.015, "mg/L")),
            "nitrat" | "no3" | "nitrate" => Some(ParamLimit::max_only(0.008, "mg/L")),
            "sulfida" | "h2s" | "sulfide" => Some(ParamLimit::max_only(0.03, "mg/L")),
            "surfaktan" | "deterjen" | "mbas" => Some(ParamLimit::max_only(1.0, "mg/L")),
            "minyak_lemak" | "oil_grease" | "minyak" => Some(ParamLimit::max_only(1.0, "mg/L")),
            "fenol" | "phenol" => Some(ParamLimit::max_only(0.0, "mg/L").with_notes("Nihil")),
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
                Some(ParamLimit::max_only(3.0, "°C"))
            }
            "salinitas" | "salinity" => {
                Some(ParamLimit::range(33.0, 34.0, "‰"))
            }
            "kecerahan" | "transparency" => {
                Some(ParamLimit::min_only(6.0, "m"))
            }
            "bau" | "odor" | "lapisan_minyak" | "oil_layer" => {
                Some(ParamLimit::max_only(0.0, "").with_notes("Nihil"))
            }
            _ => None,
        },
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
                Some(ParamLimit::range(28.0, 30.0, "°C"))
            }
            "salinitas" | "salinity" => {
                Some(ParamLimit::range(33.0, 34.0, "‰"))
            }
            "kecerahan" | "transparency" => {
                Some(ParamLimit::min_only(5.0, "m"))
            }
            "tss" => Some(ParamLimit::max_only(20.0, "mg/L")),
            "pah" | "polycyclic_aromatic" => Some(ParamLimit::max_only(0.003, "mg/L")),
            "pcb" => Some(ParamLimit::max_only(0.01, "μg/L")),
            "tributyltin" | "tbt" => Some(ParamLimit::max_only(0.01, "μg/L")),
            "pestisida" | "organoklorin" | "lindane" => {
                Some(ParamLimit::max_only(0.004, "mg/L"))
            }
            _ => None,
        },
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
                "minyak_lemak" | "oil_grease" | "minyak" => Some(ParamLimit::max_only(5.0, "mg/L")),
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
                    Some(ParamLimit::max_only(3.0, "°C"))
                }
                "tss" => Some(ParamLimit::max_only(80.0, "mg/L")),
                "salinitas" | "salinity" => {
                    Some(ParamLimit::range(0.0, 40.0, "‰"))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn check(parameter: &str, concentration: f64, peruntukan: &str) -> String {
    let p_lower = peruntukan.to_lowercase();
    let _lampiran = if p_lower.contains("wisata") {
        "Lampiran I — Wisata Bahari"
    } else if p_lower.contains("biota") {
        "Lampiran II — Biota Laut"
    } else if p_lower.contains("pelabuhan") || p_lower.contains("harbour") || p_lower.contains("port") {
        "Lampiran III — Pelabuhan"
    } else {
        return json!({"error": "E100", "message": format!("Peruntukan '{}' tidak dikenal", peruntukan)}).to_string();
    };

    match get_limit(parameter, peruntukan) {
        Some(limit) => {
            let (is_pass, _pct_str) = match (limit.min, limit.max) {
                (Some(min), Some(max)) => {
                    if concentration >= min && concentration <= max {
                        let range = max - min;
                        let mid = (min + max) / 2.0;
                        let deviation = ((concentration - mid) / (range / 2.0) * 100.0).abs();
                        (true, format!("{:.1}% dari tengah rentang", deviation))
                    } else {
                        (false, "".to_string())
                    }
                }
                (Some(min), None) => {
                    let pct = (concentration / min) * 100.0;
                    if concentration >= min {
                        (true, format!("{:.1}% dari batas minimum", pct))
                    } else {
                        (false, format!("{:.1}% dari batas minimum", pct))
                    }
                }
                (None, Some(max)) => {
                    if max == 0.0 {
                        if concentration <= 0.0 {
                            (true, "Nihil".to_string())
                        } else {
                            (false, ">nihil".to_string())
                        }
                    } else if concentration <= max {
                        let pct = (concentration / max) * 100.0;
                        (true, format!("{:.1}% dari baku mutu", pct))
                    } else {
                        let pct = (concentration / max) * 100.0;
                        (false, format!("{:.1}% dari baku mutu", pct))
                    }
                }
                (None, None) => (false, "-".to_string()),
            };

            let status = if is_pass { ResultStatus::Valid } else { ResultStatus::ValidationFailed };
            
            let mut claims = vec![Claim::new("peruntukan", peruntukan)];
            if let Some(m) = limit.max { claims.push(Claim::new("regulatory_limit_max", &m.to_string())); }
            if let Some(m) = limit.min { claims.push(Claim::new("regulatory_limit_min", &m.to_string())); }

            let mut res = ScientificResult::new(parameter, concentration, limit.unit)
                .with_status(status)
                .with_provenance(Provenance::new("regulatory_limit", "KepMen_LH_51_2004", "2026-08-19T00:00:00Z"));
                
            for c in claims { res = res.with_claim(c); }

            json!([serde_json::from_str::<serde_json::Value>(&res.emit_validated()).unwrap()]).to_string()
        }
        None => {
             json!({"error": "E100", "message": format!("Parameter '{}' tidak ditemukan", parameter)}).to_string()
        }
    }
}

pub fn check_multi(parameters_json: &str, peruntukan: &str) -> String {
    let p_lower = peruntukan.to_lowercase();
    let _lampiran = if p_lower.contains("wisata") {
        "Lampiran I — Wisata Bahari"
    } else if p_lower.contains("biota") {
        "Lampiran II — Biota Laut"
    } else if p_lower.contains("pelabuhan") || p_lower.contains("harbour") || p_lower.contains("port") {
        "Lampiran III — Pelabuhan"
    } else {
        return json!({"error": "E100", "message": format!("Peruntukan '{}' tidak dikenal", peruntukan)}).to_string();
    };

    let entries = match parse_json_array_simple(parameters_json) {
        Ok(v) => v,
        Err(e) => return json!({"error": "E100", "message": e}).to_string(),
    };

    if entries.is_empty() {
         return json!({"error": "E102", "message": "Tidak ada parameter valid di array JSON"}).to_string();
    }

    let mut results = vec![];
    let mut any_fail = false;

    for (param, value) in &entries {
        match get_limit(param, peruntukan) {
            Some(limit) => {
                let passes = match (limit.min, limit.max) {
                    (Some(min), Some(max)) => *value >= min && *value <= max,
                    (Some(min), None) => *value >= min,
                    (None, Some(max)) => {
                        if max == 0.0 { *value <= 0.0 } else { *value <= max }
                    }
                    _ => false,
                };

                if !passes { any_fail = true; }

                let status = if passes { ResultStatus::Valid } else { ResultStatus::ValidationFailed };
                
                let mut claims = vec![];
                if let Some(m) = limit.max { claims.push(Claim::new("regulatory_limit_max", &m.to_string())); }
                if let Some(m) = limit.min { claims.push(Claim::new("regulatory_limit_min", &m.to_string())); }

                let mut res = ScientificResult::new(&format!("kepmenlh51_compliance_{}", param), *value, limit.unit)
                    .with_status(status)
                    .with_provenance(Provenance::new("regulatory_limit", "KepMen_LH_51_2004", "2026-08-19T00:00:00Z"));
                    
                for c in claims { res = res.with_claim(c); }
                results.push(res);
            }
            None => {
                 let res = ScientificResult::new(&format!("kepmenlh51_compliance_{}", param), *value, "unknown")
                    .with_status(ResultStatus::OutOfDomain)
                    .with_provenance(Provenance::new("regulatory_limit", "KepMen_LH_51_2004", "2026-08-19T00:00:00Z"))
                    .with_claim(Claim::new("warning", "Parameter unknown for peruntukan"));
                 results.push(res);
            }
        }
    }

    let res_overall = ScientificResult::new("overall_compliance", if any_fail { 0.0 } else { 1.0 }, "boolean_pass")
        .with_status(if any_fail { ResultStatus::ValidationFailed } else { ResultStatus::Valid })
        .with_provenance(Provenance::new("regulatory_limit", "KepMen_LH_51_2004", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("peruntukan", peruntukan));

    results.push(res_overall);

    let json_array: Vec<serde_json::Value> = results.iter()
        .map(|r| serde_json::from_str(&r.clone().emit_validated()).unwrap())
        .collect();

    json!(json_array).to_string()
}

fn parse_json_array_simple(json_str: &str) -> Result<Vec<(String, f64)>, String> {
    let mut entries = Vec::new();
    let trimmed = json_str.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err("Input harus berupa JSON array".to_string());
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let mut depth = 0;
    let mut start = 0;
    let chars: Vec<char> = inner.chars().collect();

    for i in 0..chars.len() {
        match chars[i] {
            '{' => {
                if depth == 0 { start = i; }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let obj_str: String = chars[start..=i].iter().collect();
                    if let Some((p, v)) = parse_param_value(&obj_str) {
                        entries.push((p, v));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(entries)
}

fn parse_param_value(obj: &str) -> Option<(String, f64)> {
    let param = extract_string_field(obj, "parameter")?;
    let value = extract_number_field(obj, "value")?;
    Some((param, value))
}

fn extract_string_field(obj: &str, field: &str) -> Option<String> {
    let key = format!("\"{}\"", field);
    let pos = obj.find(&key)?;
    let after_key = &obj[pos + key.len()..];
    let after_colon = after_key.find(':')?;
    let value_part = &after_key[after_colon + 1..];
    let value_trimmed = value_part.trim_start();
    if !value_trimmed.starts_with('"') { return None; }
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
    let num_str: String = value_trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == 'e' || *c == 'E' || *c == '+')
        .collect();
    num_str.parse().ok()
}
