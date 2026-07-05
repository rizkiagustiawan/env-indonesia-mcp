use std::process::Command;

fn run_python(args: &[&str]) -> String {
    let script = "/home/awan/Documents/env-indonesia-mcp/src/tools/amdal/amdal_engine.py";
    match Command::new("python3").arg(script).args(args).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if o.status.success() { out } else { format!("ERROR [E502]: Python Engine Failed: {}\nStderr: {}", out, &err[..err.len().min(500)]) }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn generate_ka_andal(project_name: &str, location: &str, project_type: &str, rona_json: &str, output_path: &str) -> String {
    run_python(&["ka_andal", project_name, location, project_type, rona_json, output_path])
}

pub fn generate_andal(project_name: &str, location: &str, impacts_json: &str, output_path: &str) -> String {
    run_python(&["andal", project_name, location, impacts_json, output_path])
}

pub fn generate_rkl_rpl(project_name: &str, location: &str, management_json: &str, output_path: &str) -> String {
    run_python(&["rkl_rpl", project_name, location, management_json, output_path])
}

pub fn generate_ukl_upl(project_name: &str, location: &str, impacts_json: &str, output_path: &str) -> String {
    run_python(&["ukl_upl", project_name, location, impacts_json, output_path])
}

pub fn klhs_assessment(policy_name: &str, daya_dukung_json: &str, output_path: &str) -> String {
    run_python(&["klhs", policy_name, daya_dukung_json, output_path])
}
