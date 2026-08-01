//! River Dispersion Coefficient — Fischer et al. (1979)
//! Longitudinal (K_x), Transverse (K_y), Vertical (K_z)
//! For mixing zone modeling per PP 22/2021
//! Ref: Fischer, List, Koh, Imberger & Brooks (1979), Mixing in Inland and Coastal Waters

pub fn calculate(
    width_m: f64,       // river width (m)
    depth_m: f64,       // average depth (m)
    velocity_ms: f64,   // average velocity (m/s)
    slope: f64,         // energy slope (m/m)
    temperature_c: f64, // water temperature (°C)
) -> String {
    // Shear velocity u* = sqrt(g × R × S), R ≈ depth for wide rivers
    let g = 9.81;
    let u_star = (g * depth_m * slope).sqrt();

    if u_star < 1e-6 {
        return "ERROR: Shear velocity terlalu kecil. Cek slope dan depth.".to_string();
    }

    // Fischer et al. (1979) equation for longitudinal dispersion:
    // K_x = 0.011 × U² × W² / (depth × u*)
    let k_x = 0.011 * velocity_ms.powi(2) * width_m.powi(2) / (depth_m * u_star);

    // Elder (1959) — alternative longitudinal:
    // K_x_elder = 5.93 × depth × u*
    let k_x_elder = 5.93 * depth_m * u_star;

    // Transverse dispersion:
    // K_y = α × depth × u*, α = 0.6 for straight, 1.5 for meandering
    let k_y_straight = 0.6 * depth_m * u_star;
    let k_y_meander = 1.5 * depth_m * u_star;

    // Vertical dispersion:
    // K_z = β × depth × u*, β = 0.067 (Fischer 1979)
    let k_z = 0.067 * depth_m * u_star;

    // Mixing lengths
    // Transverse mixing length: L_y = 0.4 × W² × U / K_y (complete mixing)
    let l_y_straight = 0.4 * width_m.powi(2) * velocity_ms / k_y_straight;
    let l_y_meander = 0.4 * width_m.powi(2) * velocity_ms / k_y_meander;

    // Vertical mixing length: L_z = 0.4 × D² × U / K_z
    let l_z = 0.4 * depth_m.powi(2) * velocity_ms / k_z;

    // Mixing time
    let t_y_straight = l_y_straight / velocity_ms;
    let t_y_meander = l_y_meander / velocity_ms;

    // Molecular diffusion (water at temperature)
    let d_m = 1.0e-9 * (1.0 + 0.02 * (temperature_c - 20.0)); // approx m²/s

    // Peclet number
    let pe = velocity_ms * depth_m / k_x;

    format!(
        "=== KOEFISIEN DISPERSI SUNGAI ===\n\
         Ref: Fischer et al. (1979), Elder (1959)\n\n\
         INPUT:\n  Lebar = {:.1} m\n  Kedalaman = {:.2} m\n  Kecepatan = {:.3} m/s\n  Slope = {:.6}\n  Suhu = {:.1}°C\n\n\
         PARAMETER HIDRAULIK:\n  Shear velocity u* = {:.4} m/s\n  Bilangan Peclet = {:.1}\n\n\
         KOEFISIEN DISPERSI:\n\
         Longitudinal K_x:\n  Fischer (1979) = {:.2} m²/s\n  Elder (1959) = {:.2} m²/s\n\
         Transversal K_y:\n  Lurus (α=0.6) = {:.4} m²/s\n  Meandering (α=1.5) = {:.4} m²/s\n\
         Vertikal K_z:\n  Fischer (β=0.067) = {:.6} m²/s\n\
         Difusi molekuler D_m = {:.2e} m²/s\n\n\
         JARAK PENCAMPURAN SEMPURNA:\n\
         Transversal (lurus): {:.0} m ({:.1} jam)\n\
         Transversal (meandering): {:.0} m ({:.1} jam)\n\
         Vertikal: {:.0} m\n\n\
         CATATAN: K_x Fischer >> K_x Elder untuk sungai lebar.\n\
         Gunakan Fischer untuk sungai alami Indonesia (W/D > 20).\n\
         Ref: PP 22/2021 untuk mixing zone assessment.",
        width_m, depth_m, velocity_ms, slope, temperature_c,
        u_star, pe,
        k_x, k_x_elder,
        k_y_straight, k_y_meander,
        k_z, d_m,
        l_y_straight, t_y_straight / 3600.0,
        l_y_meander, t_y_meander / 3600.0,
        l_z
    )
}
