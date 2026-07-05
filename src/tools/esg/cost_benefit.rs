/// Cost-Benefit Analysis (CBA) for Environmental Projects
/// Ref: ADB Guidelines, World Bank CBA Handbook, PermenLH 15/2012

fn fmt_rp(v: f64) -> String {
    let s = format!("{:.0}", v.abs());
    let bytes: Vec<u8> = s.bytes().collect();
    let mut result = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 { result.push('.'); }
        result.push(*b as char);
    }
    if v < 0.0 { format!("-{}", result) } else { result }
}

pub fn calculate(costs_json: &str, benefits_json: &str, discount_rate: f64, years: u32) -> String {
    if discount_rate < 0.0 || discount_rate > 1.0 {
        return "ERROR: Discount rate harus antara 0.0 dan 1.0 (contoh: 0.10 = 10%).".into();
    }
    if years == 0 { return "ERROR [E102]: Parameter harus > 0 tahun.".into(); }

    // Parse costs
    #[derive(serde::Deserialize)]
    struct CostItem {
        year: u32,
        amount: f64,
        description: String,
        #[serde(default)]
        recurring: bool,
    }

    let costs: Vec<CostItem> = match serde_json::from_str(costs_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parse costs_json: {}.\nFormat: [{{\"year\":0,\"amount\":1000000000,\"description\":\"Konstruksi IPAL\"}}]", e),
    };
    let benefits: Vec<CostItem> = match serde_json::from_str(benefits_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parse benefits_json: {}.\nFormat: [{{\"year\":1,\"amount\":200000000,\"description\":\"Penghematan denda\"}}]", e),
    };

    // Build annual cash flows
    let mut annual_costs = vec![0.0_f64; years as usize + 1];
    let mut annual_benefits = vec![0.0_f64; years as usize + 1];

    for c in &costs {
        if c.year <= years {
            if c.recurring && c.year >= 1 {
                for y in c.year..=years {
                    annual_costs[y as usize] += c.amount;
                }
            } else {
                annual_costs[c.year as usize] += c.amount;
            }
        }
    }
    for b in &benefits {
        if b.year <= years {
            if b.recurring && b.year >= 1 {
                for y in b.year..=years {
                    annual_benefits[y as usize] += b.amount;
                }
            } else {
                annual_benefits[b.year as usize] += b.amount;
            }
        }
    }

    // NPV calculation
    let mut npv_costs = 0.0_f64;
    let mut npv_benefits = 0.0_f64;
    let mut npv = 0.0_f64;
    let mut cumulative_undiscounted = 0.0_f64;
    let mut payback_year: Option<u32> = None;

    for t in 0..=years {
        let df = 1.0 / (1.0 + discount_rate).powi(t as i32);
        let pv_cost = annual_costs[t as usize] * df;
        let pv_benefit = annual_benefits[t as usize] * df;
        npv_costs += pv_cost;
        npv_benefits += pv_benefit;
        npv += pv_benefit - pv_cost;

        cumulative_undiscounted += annual_benefits[t as usize] - annual_costs[t as usize];
        if payback_year.is_none() && cumulative_undiscounted >= 0.0 && t > 0 {
            payback_year = Some(t);
        }
    }

    // BCR
    let bcr = if npv_costs > 0.0 { npv_benefits / npv_costs } else { 0.0 };

    // IRR (bisection method)
    let irr = {
        let mut lo = -0.5_f64;
        let mut hi = 5.0_f64;
        let mut irr_val = 0.0_f64;
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            let mut npv_test = 0.0_f64;
            for t in 0..=years {
                let df = 1.0 / (1.0 + mid).powi(t as i32);
                npv_test += (annual_benefits[t as usize] - annual_costs[t as usize]) * df;
            }
            if npv_test > 0.0 { lo = mid; } else { hi = mid; }
            irr_val = mid;
        }
        irr_val
    };

    // Sensitivity analysis: ±10%, ±20% on discount rate
    let sensitivity_rates = [
        (discount_rate * 0.8, "-20%"),
        (discount_rate * 0.9, "-10%"),
        (discount_rate, "base"),
        (discount_rate * 1.1, "+10%"),
        (discount_rate * 1.2, "+20%"),
    ];

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  ANALISIS BIAYA-MANFAAT (CBA)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: ADB CBA Guidelines, World Bank, PermenLH 15/2012\n\n");

    out.push_str(&format!(
        "PARAMETER:\n  Discount rate = {:.1}%\n  Periode       = {} tahun\n\n",
        discount_rate * 100.0, years
    ));

    // List costs
    out.push_str("BIAYA (Costs):\n");
    for c in &costs {
        out.push_str(&format!("  Tahun {:>3}: Rp {}  {}\n", c.year, fmt_rp(c.amount), c.description));
    }
    let total_cost_nominal: f64 = costs.iter().map(|c| c.amount).sum();
    out.push_str(&format!("  Total (nominal): Rp {}\n\n", fmt_rp(total_cost_nominal)));

    // List benefits
    out.push_str("MANFAAT (Benefits):\n");
    for b in &benefits {
        out.push_str(&format!("  Tahun {:>3}: Rp {}  {}\n", b.year, fmt_rp(b.amount), b.description));
    }
    let total_benefit_nominal: f64 = benefits.iter().map(|b| b.amount).sum();
    out.push_str(&format!("  Total (nominal): Rp {}\n\n", fmt_rp(total_benefit_nominal)));

    // Results
    let npv_status = if npv >= 0.0 { "✓ LAYAK (NPV ≥ 0)" } else { "✗ TIDAK LAYAK (NPV < 0)" };
    let bcr_status = if bcr >= 1.0 { "✓ LAYAK (BCR ≥ 1)" } else { "✗ TIDAK LAYAK (BCR < 1)" };

    out.push_str("HASIL ANALISIS:\n");
    out.push_str(&format!("  NPV (Net Present Value) = Rp {}  {}\n", fmt_rp(npv), npv_status));
    out.push_str(&format!("  BCR (Benefit-Cost Ratio) = {:.3}  {}\n", bcr, bcr_status));
    out.push_str(&format!("  IRR (Internal Rate of Return) = {:.2}%\n", irr * 100.0));
    out.push_str(&format!("  PV Biaya    = Rp {}\n", fmt_rp(npv_costs)));
    out.push_str(&format!("  PV Manfaat  = Rp {}\n\n", fmt_rp(npv_benefits)));

    match payback_year {
        Some(y) => out.push_str(&format!("  Payback Period = {} tahun\n\n", y)),
        None => out.push_str("  Payback Period = Tidak tercapai dalam periode analisis\n\n"),
    }

    // Sensitivity table
    out.push_str("ANALISIS SENSITIVITAS (variasi discount rate):\n");
    out.push_str(&format!("  {:>10} {:>10} {:>20} {:>10}\n", "DR", "Variasi", "NPV (Rp)", "BCR"));
    out.push_str(&format!("  {:>10} {:>10} {:>20} {:>10}\n", "─".repeat(10), "─".repeat(10), "─".repeat(20), "─".repeat(10)));
    for (rate, label) in &sensitivity_rates {
        let mut npv_s = 0.0_f64;
        let mut pvc_s = 0.0_f64;
        let mut pvb_s = 0.0_f64;
        for t in 0..=years {
            let df = 1.0 / (1.0 + rate).powi(t as i32);
            pvc_s += annual_costs[t as usize] * df;
            pvb_s += annual_benefits[t as usize] * df;
            npv_s += (annual_benefits[t as usize] - annual_costs[t as usize]) * df;
        }
        let bcr_s = if pvc_s > 0.0 { pvb_s / pvc_s } else { 0.0 };
        out.push_str(&format!("  {:>9.1}% {:>10} {:>20} {:>10.3}\n", rate * 100.0, label, fmt_rp(npv_s), bcr_s));
    }

    out.push_str("\nKeputusan layak jika: NPV > 0, BCR > 1, IRR > discount rate.\n");
    out
}
