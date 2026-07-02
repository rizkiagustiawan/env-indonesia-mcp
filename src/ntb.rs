// NTB constants
pub const NTB_BBOX: [f64; 4] = [-9.5, 115.46, -7.9, 119.6]; // [south, west, north, east]
pub const NTB_CENTER: [f64; 2] = [-8.65, 117.5];
pub const NTB_PROVINCE_CODE: &str = "52"; // BPS province code

// BMKG adm4 codes for major NTB cities
pub const MATARAM_ADM4: &str = "52.71.01.1001";
pub const LOMBOK_BARAT_ADM4: &str = "52.01.01.2001";
pub const SUMBAWA_ADM4: &str = "52.04.01.2001";
pub const BIMA_ADM4: &str = "52.72.01.1001";
pub const DOMPU_ADM4: &str = "52.05.01.2001";

pub const NTB_KABUPATEN: &[(&str, &str)] = &[
    ("5201", "Lombok Barat"),
    ("5202", "Lombok Tengah"),
    ("5203", "Lombok Timur"),
    ("5204", "Sumbawa"),
    ("5205", "Dompu"),
    ("5206", "Bima"),
    ("5207", "Sumbawa Barat"),
    ("5208", "Lombok Utara"),
    ("5271", "Kota Mataram"),
    ("5272", "Kota Bima"),
];
