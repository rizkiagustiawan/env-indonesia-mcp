/// Pump Sizing Calculator
/// TDH = static_lift + friction_loss + velocity_head + pressure_head
/// P = (ρ × g × Q × TDH) / (η × 1000) [kW]
/// Ref: Cengel & Cimbala, Fluid Mechanics; Hydraulic Institute Standards

pub fn calculate(
    q_m3s: f64,
    static_lift_m: f64,
    friction_loss_m: f64,
    velocity_head_m: f64,
    pressure_head_m: f64,
    efficiency: f64,
) -> String {
    let mut out = String::from("=== Sizing Pompa ===\n");
    out.push_str("Ref: Cengel & Cimbala, Fluid Mechanics\n\n");

    if q_m3s <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if efficiency <= 0.0 || efficiency > 1.0 {
        return "ERROR: Efisiensi pompa harus antara 0 dan 1.".into();
    }
    if static_lift_m < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }

    let rho = 998.0; // kg/m³ (water at 20°C)
    let g = 9.81; // m/s²

    let tdh = static_lift_m + friction_loss_m + velocity_head_m + pressure_head_m;

    // Motor power
    let power_w = rho * g * q_m3s * tdh / efficiency;
    let power_kw = power_w / 1000.0;
    let power_hp = power_kw / 0.7457;

    // Water power (without losses)
    let water_power_kw = rho * g * q_m3s * tdh / 1000.0;

    out.push_str(&format!("Input:\n  Q = {:.4} m³/s ({:.2} L/s)\n  Static lift = {:.2} m\n  Friction loss = {:.2} m\n  Velocity head = {:.2} m\n  Pressure head = {:.2} m\n  Efisiensi pompa = {:.0}%\n\n",
        q_m3s, q_m3s * 1000.0, static_lift_m, friction_loss_m, velocity_head_m, pressure_head_m, efficiency * 100.0));

    out.push_str("Hasil:\n");
    out.push_str(&format!(
        "  TDH = {:.2} + {:.2} + {:.2} + {:.2} = {:.2} m\n",
        static_lift_m, friction_loss_m, velocity_head_m, pressure_head_m, tdh
    ));
    out.push_str(&format!("  Water power = {:.3} kW\n", water_power_kw));
    out.push_str(&format!(
        "  Motor power = {:.3} kW ({:.2} HP)\n\n",
        power_kw, power_hp
    ));

    // NPSH check
    let h_atm = 10.33; // m (atmospheric pressure at sea level)
    let h_vapor = 0.24; // m (water vapor at 20°C)
    let h_suction = static_lift_m * 0.3; // estimate suction side as 30% of static lift
    let h_friction_suction = friction_loss_m * 0.2; // estimate 20% friction on suction side
    let npsh_a = h_atm - h_suction - h_vapor - h_friction_suction;

    out.push_str("NPSH (estimasi):\n");
    out.push_str(&format!("  Hatm = {:.2} m (permukaan laut)\n", h_atm));
    out.push_str(&format!("  Hsuction ≈ {:.2} m\n", h_suction));
    out.push_str(&format!("  Hvapor = {:.2} m (20°C)\n", h_vapor));
    out.push_str(&format!("  NPSHa ≈ {:.2} m\n", npsh_a));
    if npsh_a < 3.0 {
        out.push_str("  ⚠️ NPSHa rendah — risiko kavitasi! Pertimbangkan pompa submersible.\n\n");
    } else {
        out.push_str("  ✅ NPSHa cukup\n\n");
    }

    // Recommended pump type
    out.push_str("Rekomendasi jenis pompa:\n");
    if tdh < 15.0 && q_m3s > 0.05 {
        out.push_str("  → Pompa sentrifugal horizontal (low head, high flow)\n");
    } else if tdh > 50.0 {
        out.push_str("  → Pompa submersible / deep well (high head)\n");
    } else if q_m3s < 0.005 {
        out.push_str("  → Pompa diafragma / peristaltik (low flow)\n");
    } else {
        out.push_str("  → Pompa sentrifugal end-suction (general purpose)\n");
    }

    out
}
