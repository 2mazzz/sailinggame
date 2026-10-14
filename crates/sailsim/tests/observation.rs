use sailsim::{Scenario, Sim, SimConfig};

fn observe(wind_dir_deg: f64, heel_deg: f64) -> sailsim::Observation {
    let mut sim = Sim::new(SimConfig::default());
    sim.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: wind_dir_deg.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: heel_deg.to_radians(),
    });
    sim.observation()
}

#[test]
fn apparent_wind_is_reported_from_the_boats_point_of_view() {
    let starboard = observe(90.0, 0.0);
    assert!((starboard.apparent_wind_angle - 90.0_f64.to_radians()).abs() < 1e-9);
    assert!((starboard.apparent_wind_speed - 6.0).abs() < 1e-9);

    let port = observe(-90.0, 0.0);
    assert!((port.apparent_wind_angle + 90.0_f64.to_radians()).abs() < 1e-9);
}

#[test]
fn capsize_margin_falls_from_one_to_zero_as_the_boat_heels() {
    assert!((observe(90.0, 0.0).capsize_margin - 1.0).abs() < 1e-9);
    let half = observe(90.0, 35.0).capsize_margin;
    assert!(
        (half - 0.5).abs() < 1e-6,
        "35 deg should be halfway to a 70 deg capsize: {half}"
    );
    assert_eq!(observe(90.0, 80.0).capsize_margin, 0.0);
    assert_eq!(
        observe(90.0, -35.0).capsize_margin,
        half,
        "margin is the same on both sides"
    );
}

#[test]
fn heel_surge_sway_and_yaw_rate_are_passed_through() {
    let obs = observe(0.0, 12.0);
    assert!((obs.heel - 12.0_f64.to_radians()).abs() < 1e-9);
    assert_eq!(obs.surge, 0.0);
    assert_eq!(obs.sway, 0.0);
    assert_eq!(obs.yaw_rate, 0.0);
    assert_eq!(obs.heel_rate, 0.0);
}
