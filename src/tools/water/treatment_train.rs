//! Wastewater Treatment Train — Multi-Stage Mass Balance
//! Simulates sequential treatment: Screening → Grit → Primary → Biological → Clarifier → Disinfection
//! Each stage has typical removal efficiencies per parameter.
//! Ref: Metcalf & Eddy (2014), PermenLHK 68/2016

pub fn simulate(
    q_m3d: f64,
    bod_in: f64,
    cod_in: f64,
    tss_in: f64,
    tn_in: f64,
    tp_in: f64,
    coliform_mpn: f64,
    stages_json: &str, // JSON array of stage names, e.g. ["screening","grit","primary","activated_sludge","clarifier","chlorination"]
) -> String {
    let stages: Vec<String> = serde_json::from_str(stages_json).unwrap_or_else(|_| {
        vec![
            "screening",
            "grit",
            "primary",
            "activated_sludge",
            "clarifier",
            "chlorination",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    });

    // Removal efficiencies per stage (fraction removed, Metcalf & Eddy Table 5-7, 8-4)
    struct StageEff {
        bod: f64,
        cod: f64,
        tss: f64,
        tn: f64,
        tp: f64,
        coli_log: f64,
    }

    let get_efficiency = |stage: &str| -> StageEff {
        match stage {
            "screening" => StageEff {
                bod: 0.05,
                cod: 0.05,
                tss: 0.05,
                tn: 0.0,
                tp: 0.0,
                coli_log: 0.0,
            },
            "grit" | "grit_chamber" => StageEff {
                bod: 0.05,
                cod: 0.05,
                tss: 0.10,
                tn: 0.0,
                tp: 0.0,
                coli_log: 0.0,
            },
            "primary" | "primary_clarifier" => StageEff {
                bod: 0.30,
                cod: 0.30,
                tss: 0.55,
                tn: 0.10,
                tp: 0.10,
                coli_log: 0.5,
            },
            "activated_sludge" | "aeration" => StageEff {
                bod: 0.85,
                cod: 0.80,
                tss: 0.85,
                tn: 0.30,
                tp: 0.20,
                coli_log: 1.0,
            },
            "clarifier" | "secondary_clarifier" => StageEff {
                bod: 0.10,
                cod: 0.10,
                tss: 0.60,
                tn: 0.05,
                tp: 0.05,
                coli_log: 0.3,
            },
            "chlorination" | "disinfection" => StageEff {
                bod: 0.0,
                cod: 0.0,
                tss: 0.0,
                tn: 0.0,
                tp: 0.0,
                coli_log: 3.0,
            },
            "trickling_filter" => StageEff {
                bod: 0.70,
                cod: 0.65,
                tss: 0.70,
                tn: 0.20,
                tp: 0.10,
                coli_log: 0.5,
            },
            "uasb" | "anaerobic" => StageEff {
                bod: 0.75,
                cod: 0.70,
                tss: 0.65,
                tn: 0.15,
                tp: 0.10,
                coli_log: 0.5,
            },
            "constructed_wetland" | "wetland" => StageEff {
                bod: 0.80,
                cod: 0.70,
                tss: 0.80,
                tn: 0.40,
                tp: 0.35,
                coli_log: 2.0,
            },
            "membrane" | "mbr" => StageEff {
                bod: 0.95,
                cod: 0.90,
                tss: 0.99,
                tn: 0.50,
                tp: 0.40,
                coli_log: 4.0,
            },
            "uv" | "uv_disinfection" => StageEff {
                bod: 0.0,
                cod: 0.0,
                tss: 0.0,
                tn: 0.0,
                tp: 0.0,
                coli_log: 4.0,
            },
            "ozone" | "ozonation" => StageEff {
                bod: 0.10,
                cod: 0.15,
                tss: 0.0,
                tn: 0.0,
                tp: 0.0,
                coli_log: 4.0,
            },
            _ => StageEff {
                bod: 0.0,
                cod: 0.0,
                tss: 0.0,
                tn: 0.0,
                tp: 0.0,
                coli_log: 0.0,
            },
        }
    };

    let mut bod = bod_in;
    let mut cod = cod_in;
    let mut tss = tss_in;
    let mut tn = tn_in;
    let mut tp = tp_in;
    let mut coli = coliform_mpn;

    let mut result = format!(
        "=== SIMULASI TREATMENT TRAIN ===\n\
         Ref: Metcalf & Eddy (2014), PermenLHK 68/2016\n\
         Q = {:.1} m³/hari\n\n\
         {:>20} {:>8} {:>8} {:>8} {:>8} {:>8} {:>12}\n\
         {:>20} {:>8} {:>8} {:>8} {:>8} {:>8} {:>12}\n",
        q_m3d,
        "Tahap",
        "BOD",
        "COD",
        "TSS",
        "TN",
        "TP",
        "Coliform",
        "INFLUENT",
        format!("{:.1}", bod),
        format!("{:.1}", cod),
        format!("{:.1}", tss),
        format!("{:.1}", tn),
        format!("{:.1}", tp),
        format!("{:.0}", coli)
    );

    let mut sludge_total = 0.0_f64;

    for stage_name in &stages {
        let eff = get_efficiency(stage_name);
        let _sludge_bod = bod * eff.bod * q_m3d / 1000.0; // kg/day
        let sludge_tss = tss * eff.tss * q_m3d / 1000.0;
        sludge_total += sludge_tss;

        bod *= 1.0 - eff.bod;
        cod *= 1.0 - eff.cod;
        tss *= 1.0 - eff.tss;
        tn *= 1.0 - eff.tn;
        tp *= 1.0 - eff.tp;
        coli /= 10.0_f64.powf(eff.coli_log);

        result.push_str(&format!(
            "{:>20} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>12.0}\n",
            stage_name, bod, cod, tss, tn, tp, coli
        ));
    }

    // Compliance check (PermenLHK 68/2016)
    result.push_str(&format!(
        "\n=== CEK BAKU MUTU (PermenLHK 68/2016) ===\n\
         BOD: {:.1} mg/L {} (BM: 30 mg/L)\n\
         COD: {:.1} mg/L {} (BM: 100 mg/L)\n\
         TSS: {:.1} mg/L {} (BM: 30 mg/L)\n\
         Total lumpur: {:.1} kg/hari\n\
         Removal total: BOD {:.1}%, COD {:.1}%, TSS {:.1}%\n",
        bod,
        if bod <= 30.0 {
            "✓ MEMENUHI"
        } else {
            "✗ MELEBIHI"
        },
        cod,
        if cod <= 100.0 {
            "✓ MEMENUHI"
        } else {
            "✗ MELEBIHI"
        },
        tss,
        if tss <= 30.0 {
            "✓ MEMENUHI"
        } else {
            "✗ MELEBIHI"
        },
        sludge_total,
        100.0 * (1.0 - bod / bod_in),
        100.0 * (1.0 - cod / cod_in),
        100.0 * (1.0 - tss / tss_in)
    ));

    result
}
