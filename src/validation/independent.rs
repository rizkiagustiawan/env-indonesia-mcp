use serde_json::Value;

/// Apply hard checks to a structured scientific result envelope.
///
/// Passing these checks is evidence of contract integrity, not independent
/// scientific truth. The producer's result status is never promoted.
pub fn validate_result(payload: &Value) -> Value {
    let mut errors = Vec::new();

    let Some(object) = payload.as_object() else {
        return serde_json::json!({
            "validation_status": "reject",
            "result_status": null,
            "errors": ["Result must be a JSON object"]
        });
    };

    let result_status = object.get("status").cloned().unwrap_or(Value::Null);
    match object.get("provenance").and_then(Value::as_object) {
        Some(provenance) => {
            for field in ["source_ids", "input_hash", "execution_id"] {
                if provenance.get(field).is_none_or(Value::is_null)
                    || provenance.get(field).is_some_and(|value| value == "")
                {
                    errors.push(format!("Provenance field is missing: {field}"));
                }
            }
        }
        None => errors.push("Provenance is required".into()),
    }

    match object.get("uncertainty").and_then(Value::as_object) {
        Some(uncertainty) => {
            let lower = finite_number(uncertainty.get("lower"));
            let upper = finite_number(uncertainty.get("upper"));
            match (lower, upper) {
                (Some(lower), Some(upper)) if lower <= upper => {}
                (Some(_), Some(_)) => errors.push("Uncertainty lower bound exceeds upper bound".into()),
                _ => errors.push("Uncertainty bounds must be finite numbers".into()),
            }
        }
        None => errors.push("Uncertainty bounds are required".into()),
    }

    match object.get("geospatial").and_then(Value::as_object) {
        Some(geospatial) => {
            if geospatial.get("crs").is_none_or(Value::is_null)
                || geospatial.get("crs").is_some_and(|value| value == "")
            {
                errors.push("CRS is required".into());
            }
            if !valid_bbox(geospatial.get("bbox").and_then(Value::as_array)) {
                errors.push("BBox must be [west, south, east, north] with east/north greater than west/south".into());
            }
            if finite_number(geospatial.get("resolution_m")).is_none_or(|value| value <= 0.0) {
                errors.push("Resolution must be a positive finite number".into());
            }
        }
        None => errors.push("Geospatial metadata is required".into()),
    }

    match object.get("mass_balance").and_then(Value::as_object) {
        Some(balance) => {
            let input = finite_number(balance.get("input_volume_m3"));
            let output = finite_number(balance.get("output_volume_m3"));
            let tolerance = finite_number(balance.get("tolerance_fraction"));
            match (input, output, tolerance) {
                (Some(input), Some(output), Some(tolerance))
                    if input >= 0.0 && output >= 0.0 && tolerance >= 0.0 =>
                {
                    if (input - output).abs() > input.abs().max(1.0) * tolerance {
                        errors.push("Mass balance exceeds tolerance".into());
                    }
                }
                _ => errors.push("Mass balance values must be finite and non-negative".into()),
            }
        }
        None => errors.push("Mass balance is required".into()),
    }

    let reported_values = object
        .get("execution_receipt")
        .and_then(Value::as_object)
        .and_then(|receipt| receipt.get("reported_values"))
        .and_then(Value::as_array);
    match reported_values {
        Some(values) => {
            if let Some(claims) = object.get("claims").and_then(Value::as_array) {
                for claim in claims {
                    let Some(value) = claim.get("value") else {
                        errors.push("Claim must contain a value".into());
                        continue;
                    };
                    if !values.iter().any(|reported| reported == value) {
                        errors.push("Claim value is not present in execution receipt".into());
                    }
                }
            }
        }
        None => errors.push("Execution receipt with reported values is required".into()),
    }

    serde_json::json!({
        "validation_status": if errors.is_empty() { "pass" } else { "reject" },
        "result_status": result_status,
        "errors": errors,
    })
}

fn finite_number(value: Option<&Value>) -> Option<f64> {
    let number = value?.as_f64()?;
    number.is_finite().then_some(number)
}

fn valid_bbox(bbox: Option<&Vec<Value>>) -> bool {
    let Some(bbox) = bbox else { return false };
    if bbox.len() != 4 { return false }
    let Some(values) = bbox.iter().map(Value::as_f64).collect::<Option<Vec<_>>>() else {
        return false;
    };
    let [west, south, east, north] = values.as_slice() else { return false };
    -180.0 <= *west && *west < *east && *east <= 180.0
        && -90.0 <= *south && *south < *north && *north <= 90.0
}

#[cfg(test)]
mod tests {
    use super::validate_result;

    fn valid_payload() -> serde_json::Value {
        serde_json::json!({
            "status": "screening_only",
            "provenance": {"source_ids": ["dibi:event-1"], "input_hash": "abc", "execution_id": "run-1"},
            "uncertainty": {"lower": 10.0, "upper": 20.0},
            "geospatial": {"crs": "EPSG:4326", "bbox": [106.0, -7.0, 107.0, -6.0], "resolution_m": 10.0},
            "mass_balance": {"input_volume_m3": 100.0, "output_volume_m3": 99.0, "tolerance_fraction": 0.02},
            "claims": [{"value": 123.0}],
            "execution_receipt": {"reported_values": [123.0]}
        })
    }

    #[test]
    fn complete_screening_result_passes_without_status_promotion() {
        let result = validate_result(&valid_payload());
        assert_eq!(result["validation_status"], "pass");
        assert_eq!(result["result_status"], "screening_only");
    }

    #[test]
    fn failed_mass_balance_is_rejected() {
        let mut payload = valid_payload();
        payload["mass_balance"]["output_volume_m3"] = serde_json::json!(80.0);
        let result = validate_result(&payload);
        assert_eq!(result["validation_status"], "reject");
        assert!(result["errors"].to_string().contains("Mass balance"));
    }
}
