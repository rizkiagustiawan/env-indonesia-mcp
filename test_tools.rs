fn main() {
    let aermod = env_indonesia_mcp::tools::airquality::aermod_generator::generate_aermod_inp(
        "PLTU_Suralaya_Unit_9", -5.88, 106.02, 150.0, 6.5, 25.0, 420.0, 250.5, "SO2", true
    );
    println!("{}\n\n", aermod);

    let phreeqc = env_indonesia_mcp::tools::waste::phreeqc_leaching::generate_phreeqc_script(
        "Tailing_Nikel_Limonit", 100.0, 1.0, 4.5, "Ni: 15000.0, Cr: 500.0, Fe: 200000.0"
    );
    println!("{}", phreeqc);
}
