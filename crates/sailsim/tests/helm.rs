use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Seam: beam reach with the rudder released. A dinghy has a little weather
/// helm, but left alone it holds its course gently: the sailor is not fighting
/// the tiller to stop it rounding up.
#[test]
fn a_released_rudder_holds_a_beam_reach_course() {
    let mut sim = Sim::new(SimConfig::default());
    sim.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: 0.0,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 90.0_f64.to_radians(),
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let controls = Controls {
        rudder: 0.0,
        sheet: 0.5,
    };
    let steps = |n: f64| (n / 0.02).round() as usize;

    // Let the boat accelerate to its steady speed first.
    let mut settled = sim.step(controls).state;
    for _ in 0..steps(5.0) {
        settled = sim.step(controls).state;
    }
    let mut later = settled;
    for _ in 0..steps(30.0) {
        later = sim.step(controls).state;
    }

    let drift = (later.heading - settled.heading).to_degrees();
    assert!(
        drift.abs() < 5.0,
        "heading drifted {drift} deg in 30 s with the rudder released"
    );
}
