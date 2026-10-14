use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Seam: the true wind can be changed while the boat is sailing (the viewer's
/// sliders). With no true wind the only wind the boat feels is its own motion,
/// dead ahead at the boat's speed.
#[test]
fn changing_the_wind_mid_run_changes_what_the_boat_feels() {
    let mut sim = Sim::new(SimConfig::default());
    sim.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: 0.0,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 90.0_f64.to_radians(),
        start_speed: 3.0,
        start_heel: 0.0,
    });
    let controls = Controls {
        rudder: 0.0,
        sheet: 0.5,
    };
    for _ in 0..100 {
        sim.step(controls);
    }
    assert!(sim.observation().apparent_wind_speed > 4.0);

    sim.set_wind(0.0, 0.0);
    let obs = sim.observation();
    let boat_speed = obs.surge.hypot(obs.sway);
    assert!(
        (obs.apparent_wind_speed - boat_speed).abs() < 0.01,
        "apparent wind {} vs boat speed {boat_speed}",
        obs.apparent_wind_speed
    );
    assert!(
        obs.apparent_wind_angle.abs() < 0.2,
        "{}",
        obs.apparent_wind_angle
    );
}
