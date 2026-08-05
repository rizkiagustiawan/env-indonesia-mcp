use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ERROR_COUNTS: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

pub struct CircuitBreaker;

impl CircuitBreaker {
    pub fn check(tool_name: &str, max_retries: u32) -> Result<(), String> {
        let mut counts = ERROR_COUNTS.lock().unwrap();
        let count = counts.entry(tool_name.to_string()).or_insert(0);
        
        if *count >= max_retries {
            Err(format!(
                "CIRCUIT BREAKER OPEN: Alat '{}' gagal {} kali beruntun. Eksekusi dihentikan paksa untuk mencegah infinite loop. Cek parameter input Anda.",
                tool_name, count
            ))
        } else {
            Ok(())
        }
    }

    pub fn record_error(tool_name: &str) {
        let mut counts = ERROR_COUNTS.lock().unwrap();
        let count = counts.entry(tool_name.to_string()).or_insert(0);
        *count += 1;
    }

    pub fn record_success(tool_name: &str) {
        let mut counts = ERROR_COUNTS.lock().unwrap();
        counts.insert(tool_name.to_string(), 0); // Reset
    }
}
