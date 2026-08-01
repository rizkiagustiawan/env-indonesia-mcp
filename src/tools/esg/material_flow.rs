/// Material Flow Analysis (MFA)
/// Ref: Brunner & Rechberger (2004), ISO 14040, Eurostat MFA Guide

pub fn analyze(inputs_json: &str, outputs_json: &str, stock_change: f64) -> String {
    #[derive(serde::Deserialize)]
    struct FlowItem {
        material: String,
        amount_ton: f64,
    }

    let inputs: Vec<FlowItem> = match serde_json::from_str(inputs_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parse inputs_json: {}.\nFormat: [{{\"material\":\"besi\",\"amount_ton\":1000}}]", e),
    };
    let outputs: Vec<FlowItem> = match serde_json::from_str(outputs_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parse outputs_json: {}.\nFormat: [{{\"material\":\"produk\",\"amount_ton\":800}}]", e),
    };

    if inputs.is_empty() {
        return "ERROR: Minimal 1 input material.".into();
    }
    if outputs.is_empty() {
        return "ERROR: Minimal 1 output material.".into();
    }

    let total_input: f64 = inputs.iter().map(|i| i.amount_ton).sum();
    let total_output: f64 = outputs.iter().map(|o| o.amount_ton).sum();

    // Mass balance: Input = Output + Stock_change
    let balance = total_input - total_output - stock_change;
    let balance_pct = if total_input > 0.0 {
        (balance / total_input).abs() * 100.0
    } else {
        0.0
    };

    // Identify useful product vs waste
    let mut useful_output = 0.0_f64;
    let mut waste_output = 0.0_f64;
    let mut waste_items: Vec<&FlowItem> = Vec::new();
    let mut product_items: Vec<&FlowItem> = Vec::new();

    for o in &outputs {
        let lower = o.material.to_lowercase();
        if lower.contains("limbah")
            || lower.contains("waste")
            || lower.contains("emisi")
            || lower.contains("emission")
            || lower.contains("reject")
            || lower.contains("scrap")
            || lower.contains("slag")
            || lower.contains("abu")
            || lower.contains("sludge")
            || lower.contains("residu")
        {
            waste_output += o.amount_ton;
            waste_items.push(o);
        } else {
            useful_output += o.amount_ton;
            product_items.push(o);
        }
    }

    let efficiency = if total_input > 0.0 {
        useful_output / total_input * 100.0
    } else {
        0.0
    };
    let waste_ratio = if total_input > 0.0 {
        waste_output / total_input * 100.0
    } else {
        0.0
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  MATERIAL FLOW ANALYSIS (MFA)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Brunner & Rechberger (2004), ISO 14040\n\n");

    // Inputs
    out.push_str("INPUT MATERIAL:\n");
    for i in &inputs {
        let pct = if total_input > 0.0 {
            i.amount_ton / total_input * 100.0
        } else {
            0.0
        };
        out.push_str(&format!(
            "  {:25} {:>10.2} ton ({:.1}%)\n",
            i.material, i.amount_ton, pct
        ));
    }
    out.push_str(&format!(
        "  {:25} {:>10.2} ton\n\n",
        "TOTAL INPUT", total_input
    ));

    // Outputs
    out.push_str("OUTPUT MATERIAL:\n");
    if !product_items.is_empty() {
        out.push_str("  Produk:\n");
        for o in &product_items {
            let pct = if total_input > 0.0 {
                o.amount_ton / total_input * 100.0
            } else {
                0.0
            };
            out.push_str(&format!(
                "    {:23} {:>10.2} ton ({:.1}%)\n",
                o.material, o.amount_ton, pct
            ));
        }
    }
    if !waste_items.is_empty() {
        out.push_str("  Limbah/Emisi:\n");
        for o in &waste_items {
            let pct = if total_input > 0.0 {
                o.amount_ton / total_input * 100.0
            } else {
                0.0
            };
            out.push_str(&format!(
                "    {:23} {:>10.2} ton ({:.1}%)\n",
                o.material, o.amount_ton, pct
            ));
        }
    }
    out.push_str(&format!(
        "  {:25} {:>10.2} ton\n\n",
        "TOTAL OUTPUT", total_output
    ));

    // Stock change
    out.push_str(&format!("PERUBAHAN STOK: {:+.2} ton\n\n", stock_change));

    // Mass balance
    let balance_status = if balance_pct < 1.0 {
        "✓ Seimbang (deviasi < 1%)"
    } else if balance_pct < 5.0 {
        "⚠️ Deviasi kecil (1-5%) — periksa data"
    } else {
        "✗ Tidak seimbang (deviasi > 5%) — ada aliran tak teridentifikasi"
    };
    out.push_str(&format!(
        "NERACA MASSA:\n  Input - Output - ΔStock = {:+.2} ton ({:.2}%)\n  Status: {}\n\n",
        balance, balance_pct, balance_status
    ));

    // Efficiency
    out.push_str(&format!(
        "EFISIENSI:\n  Efisiensi material = {:.1}% (produk berguna / total input)\n  Rasio limbah       = {:.1}% (limbah / total input)\n\n",
        efficiency, waste_ratio
    ));

    // Sankey data (simplified text representation)
    out.push_str("DATA SANKEY DIAGRAM:\n");
    for i in &inputs {
        out.push_str(&format!(
            "  [INPUT] {} ({:.1}t) → PROSES\n",
            i.material, i.amount_ton
        ));
    }
    for o in &outputs {
        let flow_type = if waste_items.iter().any(|w| w.material == o.material) {
            "LIMBAH"
        } else {
            "PRODUK"
        };
        out.push_str(&format!(
            "  PROSES → [{}] {} ({:.1}t)\n",
            flow_type, o.material, o.amount_ton
        ));
    }
    if stock_change.abs() > 0.01 {
        out.push_str(&format!("  PROSES → [STOK] ({:+.1}t)\n", stock_change));
    }

    out.push_str("\nREKOMENDASI:\n");
    if waste_ratio > 20.0 {
        out.push_str("  • Rasio limbah tinggi (>20%) — pertimbangkan optimasi proses\n");
    }
    if waste_ratio > 10.0 {
        out.push_str("  • Identifikasi peluang daur ulang / reuse limbah\n");
        out.push_str("  • Analisis simbiosis industri dengan industri sekitar\n");
    }
    if balance_pct > 5.0 {
        out.push_str("  • Lakukan audit material untuk identifikasi aliran hilang\n");
        out.push_str("  • Periksa emisi gas / penguapan yang tidak terukur\n");
    }
    out.push_str("  • Terapkan cleaner production (PP 27/2012)\n");
    out
}
