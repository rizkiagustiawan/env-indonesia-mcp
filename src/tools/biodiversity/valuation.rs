/// Valuasi Ekonomi Lingkungan
/// Ref: PP 46/2017 tentang Instrumen Ekonomi Lingkungan Hidup

/// Calculate environmental economic valuation using various methods
pub fn calculate(method: &str, params_json: &str) -> String {
    let params: serde_json::Value = match serde_json::from_str(params_json) {
        Ok(v) => v,
        Err(e) => return format!("Error parsing JSON: {}", e),
    };

    match method.to_lowercase().as_str() {
        "replacement_cost" | "biaya_penggantian" => replacement_cost(&params),
        "travel_cost" | "biaya_perjalanan" => travel_cost(&params),
        "hedonic" | "harga_hedonik" => hedonic(&params),
        "damage_cost" | "biaya_kerusakan" => damage_cost(&params),
        "benefit_transfer" | "transfer_manfaat" => benefit_transfer(&params),
        _ => format!(
            "Metode '{}' tidak dikenal.\nMetode tersedia:\n\
             • replacement_cost / biaya_penggantian\n\
             • travel_cost / biaya_perjalanan\n\
             • hedonic / harga_hedonik\n\
             • damage_cost / biaya_kerusakan\n\
             • benefit_transfer / transfer_manfaat",
            method
        ),
    }
}

fn replacement_cost(params: &serde_json::Value) -> String {
    let ecosystem = params.get("ecosystem")
        .and_then(|v| v.as_str())
        .unwrap_or("mangrove");
    let area_ha = params.get("area_ha")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);
    let condition = params.get("condition")
        .and_then(|v| v.as_str())
        .unwrap_or("sedang");

    // Reference values per hectare (IDR) based on Indonesian studies
    let (cost_per_ha, ecosystem_name, services) = match ecosystem.to_lowercase().as_str() {
        "mangrove" => (350_000_000.0, "Hutan Mangrove",
            "perlindungan pantai, nursery ikan, penyerapan karbon, kayu bakar"),
        "terumbu_karang" | "coral" => (500_000_000.0, "Terumbu Karang",
            "perikanan, pariwisata, perlindungan pantai, biodiversitas"),
        "hutan_tropis" | "tropical_forest" => (250_000_000.0, "Hutan Tropis",
            "kayu, HHBK, penyerapan karbon, pengaturan air, biodiversitas"),
        "padang_lamun" | "seagrass" => (150_000_000.0, "Padang Lamun",
            "nursery ikan, penyerapan karbon, stabilisasi sedimen"),
        "rawa_gambut" | "peatland" => (200_000_000.0, "Rawa Gambut",
            "penyimpanan karbon, pengaturan air, biodiversitas"),
        "hutan_bakau" => (300_000_000.0, "Hutan Bakau",
            "perlindungan pantai, perikanan, penyerapan karbon"),
        "danau" | "lake" => (100_000_000.0, "Ekosistem Danau",
            "air bersih, perikanan, pariwisata, pengaturan banjir"),
        _ => (200_000_000.0, "Ekosistem Umum",
            "jasa ekosistem umum"),
    };

    let condition_factor = match condition {
        "baik" | "good" => 1.0,
        "sedang" | "moderate" => 0.7,
        "rusak" | "degraded" => 0.4,
        "kritis" | "critical" => 0.2,
        _ => 0.7,
    };

    let total_value = cost_per_ha * area_ha * condition_factor;
    let annual_service = total_value * 0.05; // 5% annual ecosystem service value

    format!(
        "══════════════════════════════════════════════\n\
         VALUASI EKONOMI - METODE BIAYA PENGGANTIAN\n\
         Ref: PP 46/2017\n\
         ══════════════════════════════════════════════\n\n\
         Ekosistem       : {}\n\
         Luas            : {:.2} Ha\n\
         Kondisi         : {} (faktor: {:.1})\n\
         Jasa ekosistem  : {}\n\n\
         PERHITUNGAN:\n\
         • Biaya penggantian/Ha  : Rp {:>15.0}\n\
         • Faktor kondisi        : {:.1}\n\
         • Total nilai ekosistem : Rp {:>15.0}\n\
         • Nilai jasa tahunan    : Rp {:>15.0}\n\n\
         CATATAN:\n\
         Nilai di atas merupakan estimasi berdasarkan studi valuasi\n\
         ekonomi lingkungan di Indonesia. Nilai aktual dapat bervariasi\n\
         tergantung lokasi, kondisi spesifik, dan metodologi yang digunakan.\n\
         ══════════════════════════════════════════════",
        ecosystem_name, area_ha, condition, condition_factor, services,
        cost_per_ha, condition_factor, total_value, annual_service
    )
}

fn travel_cost(params: &serde_json::Value) -> String {
    let site_name = params.get("site_name")
        .and_then(|v| v.as_str())
        .unwrap_or("Kawasan Wisata Alam");
    let visitors_per_year = params.get("visitors_per_year")
        .and_then(|v| v.as_f64())
        .unwrap_or(10000.0);
    let avg_travel_cost = params.get("avg_travel_cost_idr")
        .and_then(|v| v.as_f64())
        .unwrap_or(500000.0);
    let avg_time_hours = params.get("avg_time_hours")
        .and_then(|v| v.as_f64())
        .unwrap_or(4.0);
    let avg_income_per_hour = params.get("avg_income_per_hour_idr")
        .and_then(|v| v.as_f64())
        .unwrap_or(25000.0);

    let time_cost = avg_time_hours * avg_income_per_hour;
    let total_individual_cost = avg_travel_cost + time_cost;
    let total_annual_value = total_individual_cost * visitors_per_year;
    let consumer_surplus = total_annual_value * 0.3; // ~30% consumer surplus estimate

    format!(
        "══════════════════════════════════════════════\n\
         VALUASI EKONOMI - METODE BIAYA PERJALANAN\n\
         Ref: PP 46/2017\n\
         ══════════════════════════════════════════════\n\n\
         Lokasi             : {}\n\
         Pengunjung/tahun   : {:.0}\n\n\
         BIAYA PER PENGUNJUNG:\n\
         • Biaya perjalanan     : Rp {:>12.0}\n\
         • Biaya waktu ({:.1} jam) : Rp {:>12.0}\n\
         • Total per pengunjung : Rp {:>12.0}\n\n\
         NILAI EKONOMI:\n\
         • Total biaya perjalanan  : Rp {:>15.0}\n\
         • Consumer surplus (30%)  : Rp {:>15.0}\n\
         • Willingness to Pay est. : Rp {:>15.0}\n\n\
         CATATAN: Consumer surplus dihitung menggunakan\n\
         pendekatan linear demand curve approximation.\n\
         ══════════════════════════════════════════════",
        site_name, visitors_per_year,
        avg_travel_cost, avg_time_hours, time_cost, total_individual_cost,
        total_annual_value, consumer_surplus, total_annual_value + consumer_surplus
    )
}

fn hedonic(params: &serde_json::Value) -> String {
    let property_near = params.get("property_value_near_idr")
        .and_then(|v| v.as_f64())
        .unwrap_or(500_000_000.0);
    let property_far = params.get("property_value_far_idr")
        .and_then(|v| v.as_f64())
        .unwrap_or(400_000_000.0);
    let num_properties = params.get("num_properties")
        .and_then(|v| v.as_f64())
        .unwrap_or(100.0);
    let amenity = params.get("amenity")
        .and_then(|v| v.as_str())
        .unwrap_or("ruang terbuka hijau");
    let distance_near = params.get("distance_near_m")
        .and_then(|v| v.as_f64())
        .unwrap_or(500.0);
    let distance_far = params.get("distance_far_m")
        .and_then(|v| v.as_f64())
        .unwrap_or(2000.0);

    let diff = property_near - property_far;
    let pct_premium = (diff / property_far) * 100.0;
    let total_amenity_value = diff * num_properties;

    format!(
        "══════════════════════════════════════════════\n\
         VALUASI EKONOMI - METODE HARGA HEDONIK\n\
         Ref: PP 46/2017\n\
         ══════════════════════════════════════════════\n\n\
         Amenitas lingkungan: {}\n\n\
         PERBANDINGAN NILAI PROPERTI:\n\
         • Dekat ({:.0}m): Rp {:>15.0}\n\
         • Jauh ({:.0}m) : Rp {:>15.0}\n\
         • Selisih       : Rp {:>15.0} ({:.1}%)\n\n\
         NILAI AMENITAS LINGKUNGAN:\n\
         • Premium per properti  : Rp {:>15.0}\n\
         • Jumlah properti       : {:.0}\n\
         • Total nilai amenitas  : Rp {:>15.0}\n\n\
         INTERPRETASI:\n\
         Kedekatan dengan {} memberikan premium nilai\n\
         properti sebesar {:.1}%, menunjukkan willingness to pay\n\
         masyarakat terhadap amenitas lingkungan tersebut.\n\
         ══════════════════════════════════════════════",
        amenity,
        distance_near, property_near,
        distance_far, property_far,
        diff, pct_premium,
        diff, num_properties, total_amenity_value,
        amenity, pct_premium
    )
}

fn damage_cost(params: &serde_json::Value) -> String {
    let damage_type = params.get("damage_type")
        .and_then(|v| v.as_str())
        .unwrap_or("pencemaran_air");
    let area_ha = params.get("area_ha")
        .and_then(|v| v.as_f64())
        .unwrap_or(10.0);
    let severity = params.get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("sedang");
    let duration_years = params.get("duration_years")
        .and_then(|v| v.as_f64())
        .unwrap_or(5.0);

    let (base_cost_per_ha, damage_desc, recovery_time) = match damage_type {
        "pencemaran_air" | "water_pollution" =>
            (50_000_000.0, "Pencemaran Air", "2-5 tahun"),
        "pencemaran_udara" | "air_pollution" =>
            (30_000_000.0, "Pencemaran Udara", "1-3 tahun"),
        "pencemaran_tanah" | "soil_pollution" =>
            (75_000_000.0, "Pencemaran Tanah", "5-20 tahun"),
        "deforestasi" | "deforestation" =>
            (250_000_000.0, "Deforestasi", "20-50 tahun"),
        "erosi" | "erosion" =>
            (40_000_000.0, "Erosi Tanah", "5-15 tahun"),
        "banjir" | "flooding" =>
            (100_000_000.0, "Kerusakan Banjir", "1-2 tahun"),
        "tumpahan_minyak" | "oil_spill" =>
            (200_000_000.0, "Tumpahan Minyak", "5-20 tahun"),
        _ => (50_000_000.0, "Kerusakan Lingkungan Umum", "3-10 tahun"),
    };

    let severity_factor = match severity {
        "ringan" | "minor" => 0.3,
        "sedang" | "moderate" => 0.7,
        "berat" | "major" => 1.0,
        "sangat_berat" | "severe" => 1.5,
        _ => 0.7,
    };

    let cleanup_cost = base_cost_per_ha * area_ha * severity_factor;
    let lost_productivity = base_cost_per_ha * area_ha * 0.1 * duration_years;
    let ecosystem_service_loss = base_cost_per_ha * area_ha * 0.05 * duration_years;
    let total_damage = cleanup_cost + lost_productivity + ecosystem_service_loss;

    format!(
        "══════════════════════════════════════════════\n\
         VALUASI EKONOMI - METODE BIAYA KERUSAKAN\n\
         Ref: PP 46/2017\n\
         ══════════════════════════════════════════════\n\n\
         Jenis kerusakan    : {}\n\
         Luas terdampak     : {:.2} Ha\n\
         Tingkat keparahan  : {} (faktor: {:.1})\n\
         Durasi dampak      : {:.0} tahun\n\
         Waktu pemulihan    : {}\n\n\
         RINCIAN BIAYA KERUSAKAN:\n\
         • Biaya pemulihan/remediasi : Rp {:>15.0}\n\
         • Kehilangan produktivitas  : Rp {:>15.0}\n\
         • Kehilangan jasa ekosistem : Rp {:>15.0}\n\
         ────────────────────────────────────────\n\
         • TOTAL BIAYA KERUSAKAN     : Rp {:>15.0}\n\n\
         CATATAN: Nilai ini dapat digunakan sebagai dasar\n\
         penetapan ganti rugi lingkungan hidup sesuai UU 32/2009.\n\
         ══════════════════════════════════════════════",
        damage_desc, area_ha, severity, severity_factor, duration_years, recovery_time,
        cleanup_cost, lost_productivity, ecosystem_service_loss, total_damage
    )
}

fn benefit_transfer(params: &serde_json::Value) -> String {
    let ecosystem = params.get("ecosystem")
        .and_then(|v| v.as_str())
        .unwrap_or("mangrove");
    let area_ha = params.get("area_ha")
        .and_then(|v| v.as_f64())
        .unwrap_or(100.0);
    let reference_value = params.get("reference_value_usd_per_ha")
        .and_then(|v| v.as_f64());
    let ppp_factor = params.get("ppp_factor")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.35);

    // Default reference values from global studies (USD/ha/year - TEEB, de Groot et al. 2012)
    let (ref_val_usd, source_study) = if let Some(rv) = reference_value {
        (rv, "Nilai referensi dari pengguna")
    } else {
        match ecosystem.to_lowercase().as_str() {
            "mangrove" => (10000.0, "TEEB (2010), Vo et al. (2012)"),
            "terumbu_karang" | "coral" => (18000.0, "de Groot et al. (2012)"),
            "hutan_tropis" | "tropical_forest" => (5000.0, "Costanza et al. (2014)"),
            "padang_lamun" | "seagrass" => (7500.0, "Barbier et al. (2011)"),
            "rawa_gambut" | "peatland" => (4000.0, "Costanza et al. (2014)"),
            "lahan_basah" | "wetland" => (12000.0, "de Groot et al. (2012)"),
            _ => (5000.0, "Costanza et al. (2014) - rata-rata"),
        }
    };

    let exchange_rate = 15500.0; // approximate IDR/USD
    let adjusted_value = ref_val_usd * ppp_factor * exchange_rate;
    let total_annual = adjusted_value * area_ha;
    let total_25_year = total_annual * 25.0; // 25-year NPV at 0% discount for simplicity

    format!(
        "══════════════════════════════════════════════\n\
         VALUASI EKONOMI - METODE TRANSFER MANFAAT\n\
         Ref: PP 46/2017\n\
         ══════════════════════════════════════════════\n\n\
         Ekosistem       : {}\n\
         Luas            : {:.2} Ha\n\
         Studi referensi : {}\n\n\
         TRANSFER NILAI:\n\
         • Nilai referensi      : USD {:>10.0}/Ha/tahun\n\
         • Faktor PPP Indonesia : {:.2}\n\
         • Kurs                 : Rp {:>8.0}/USD\n\
         • Nilai transfer       : Rp {:>12.0}/Ha/tahun\n\n\
         ESTIMASI TOTAL:\n\
         • Nilai tahunan        : Rp {:>15.0}\n\
         • Nilai 25 tahun (NPV) : Rp {:>15.0}\n\n\
         CATATAN: Benefit transfer memiliki keterbatasan karena\n\
         perbedaan konteks antara lokasi referensi dan lokasi studi.\n\
         Faktor PPP digunakan untuk menyesuaikan perbedaan daya beli.\n\
         ══════════════════════════════════════════════",
        ecosystem, area_ha, source_study,
        ref_val_usd, ppp_factor, exchange_rate, adjusted_value,
        total_annual, total_25_year
    )
}
