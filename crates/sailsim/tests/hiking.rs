use sailsim::{Controls, Scenario, Sim, SimConfig, Status};

/// Sheeted in close-hauled at 45 degrees (wind from starboard), heading
/// locked, `hike` held for 90 s. Returns the final heel (deg, magnitude) and
/// whether the boat capsized.
fn close_hauled(wind_speed: f64, hike: f64) -> (f64, Status) {
    let mut sim = Sim::new(SimConfig {
        fixed_heading: true,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: wind_speed,
        true_wind_dir: 45.0_f64.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let controls = Controls {
        rudder: 0.0,
        sheet: 1.0,
        hike,
    };
    let (mut heel, mut status) = (0.0, Status::Sailing);
    for _ in 0..4500 {
        let out = sim.step(controls);
        heel = out.state.heel.to_degrees().abs();
        status = out.status;
    }
    (heel, status)
}

#[test]
fn hiking_lowers_the_heel_in_the_same_wind() {
    let (sitting, _) = close_hauled(6.0, 0.0);
    let (half, _) = close_hauled(6.0, 0.5);
    let (hiking, _) = close_hauled(6.0, 1.0);
    assert!(
        hiking < half && half < sitting,
        "heel {sitting} -> {half} -> {hiking} deg"
    );
    assert!(hiking < 0.85 * sitting, "{sitting} deg vs hiking {hiking} deg");
}

#[test]
fn hiking_survives_a_wind_that_capsizes_a_sailor_sitting_in() {
    let (_, sitting) = close_hauled(10.5, 0.0);
    let (_, hiking) = close_hauled(10.5, 1.0);
    assert_eq!(sitting, Status::Capsized);
    assert_eq!(hiking, Status::Sailing);
}

#[test]
fn hiking_does_nothing_to_a_boat_that_is_upright_in_no_wind() {
    for hike in [0.0, 1.0] {
        let mut sim = Sim::new(SimConfig::default());
        sim.reset(Scenario {
            true_wind_speed: 0.0,
            true_wind_dir: 0.0,
            start_x: 0.0,
            start_y: 0.0,
            start_heading: 0.0,
            start_speed: 2.0,
            start_heel: 0.0,
        });
        let controls = Controls {
            rudder: 0.0,
            sheet: 0.5,
            hike,
        };
        let state = (0..1000).fold(sim.state(), |_, _| sim.step(controls).state);
        assert_eq!(state.heel, 0.0, "hike {hike}");
    }
}
