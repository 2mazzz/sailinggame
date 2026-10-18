use sailsim::{hull_resistance, BoatParams};

const KN: f64 = 0.514444;

fn r(kn: f64) -> f64 {
    -hull_resistance(&BoatParams::ilca_like(), kn * KN)
}

#[test]
fn resistance_is_continuous_across_the_end_of_the_tank_data() {
    let below = r(8.999);
    let above = r(9.001);
    assert!((above - below).abs() / below < 0.01, "{below} -> {above}");
}

#[test]
fn resistance_keeps_rising_but_slower_than_v_squared_once_planing() {
    // 9 -> 16 kn: v^2 would multiply R by 3.16; a planing hull's R rises much less.
    let ratio = r(16.0) / r(9.0);
    assert!(ratio > 1.0, "resistance fell: {ratio}");
    assert!(ratio < 2.0, "still behaving like displacement drag: {ratio}");
}

#[test]
fn resistance_to_weight_stays_below_a_planing_hulls_limit() {
    // Resistance to weight is the inverse of lift to drag. The 0.3 bound is a judgement,
    // not a sourced figure. Weight = 160 kg x 9.81.
    let weight = 160.0 * 9.81;
    assert!(r(13.0) / weight < 0.3, "R/W = {}", r(13.0) / weight);
}
