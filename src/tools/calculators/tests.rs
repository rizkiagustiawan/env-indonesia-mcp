use super::*;

#[test]
fn test_rusle() {
    let res = rusle::calculate(1000.0, 0.2, 1.5, 0.1, 1.0);
    assert!(res.contains("30.00")); // 1000 * 0.2 * 1.5 * 0.1 * 1.0 = 30
    assert!(!res.contains("ERROR"));
}

#[test]
fn test_scs_cn() {
    let res = scs_cn::calculate(100.0, 80.0);
    // S = 25400/80 - 254 = 63.5
    // Q = (100 - 0.2*63.5)^2 / (100 + 0.8*63.5) = (87.3)^2 / 150.8 = 50.53
    assert!(res.contains("50.5"));
    assert!(!res.contains("ERROR"));
}

#[test]
fn test_do_saturation() {
    let res = do_saturation::calculate(20.0);
    assert!(res.contains("9.09")); // DO sat at 20C is ~9.09 mg/L
}

#[test]
fn test_first_order_kinetics() {
    let res = first_order_kinetics::calculate(100.0, 0.1, 5.0, "day");
    assert!(res.contains("60.65")); // 100 * exp(-0.1 * 5) = 60.65
}

#[test]
fn test_rational_method() {
    let res = rational_method::calculate(0.0, 50.0, 10.0, "hutan");
    // C for hutan = 0.1
    // Q = 0.1 * 50 * 10 / 360 = 0.1388 m3/s = 138.8 L/s
    assert!(res.contains("138.9")); // Note: 138.88.. rounds to 138.9 in format string
}
