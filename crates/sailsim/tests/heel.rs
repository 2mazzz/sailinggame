use sailsim::{Controls, Scenario, Sim, SimConfig};

fn sim_with(wind_speed: f64, wind_dir_deg: f64, start_heel_deg: f64, config: SimConfig) -> Sim {
    let mut sim = Sim::new(config);
    sim.reset(Scenario {
        true_wind_speed: wind_speed,
        true_wind_dir: wind_dir_deg.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: start_heel_deg.to_radians(),
    });
    sim
}

fn no_capsize() -> SimConfig {
    SimConfig {
        capsize_enabled: false,
        fixed_heading: true,
        ..SimConfig::default()
    }
}

/// Heel in degrees after `seconds` close-hauled with wind from starboard.
fn close_hauled_heel(wind_speed: f64, sheet: f64, seconds: f64) -> f64 {
    let mut sim = sim_with(wind_speed, 45.0, 0.0, no_capsize());
    let mut heel = 0.0;
    for _ in 0..(seconds / 0.02) as usize {
        heel = sim
            .step(Controls {
                rudder: 0.0,
                sheet,
                hike: 0.0,
            })
            .state
            .heel;
    }
    heel.to_degrees()
}

#[test]
fn heel_grows_with_wind_and_leans_away_from_it() {
    let light = close_hauled_heel(4.0, 0.85, 40.0);
    let fresh = close_hauled_heel(7.0, 0.85, 40.0);

    // Wind from starboard presses the sail to port: port side down (negative).
    // A hiking crew keeps the boat nearly flat in light air (Day 2017: "sailed
    // flat upwind up to about 9 knots"), so the heel is small but never zero.
    assert!(light < -0.1, "light-air heel {light} deg");
    assert!(
        fresh < light,
        "heel should grow with wind: {light} -> {fresh} deg"
    );
    assert!(
        fresh > -70.0,
        "should not be rolling over in a fresh breeze: {fresh} deg"
    );
}

#[test]
fn easing_the_sheet_reduces_heel() {
    let sheeted_in = close_hauled_heel(7.0, 0.85, 40.0);
    let eased = close_hauled_heel(7.0, 0.30, 40.0);
    assert!(
        eased.abs() < 0.5 * sheeted_in.abs(),
        "{sheeted_in} deg -> {eased} deg"
    );
}

#[test]
fn a_heeled_boat_rights_itself_below_the_vanishing_angle() {
    let mut sim = sim_with(0.0, 0.0, 60.0, no_capsize());
    let mut heel = 0.0;
    for _ in 0..(30.0 / 0.02) as usize {
        heel = sim
            .step(Controls {
                rudder: 0.0,
                sheet: 0.5,
                hike: 0.0,
            })
            .state
            .heel;
    }
    assert!(
        heel.to_degrees().abs() < 2.0,
        "heel after 30 s: {} deg",
        heel.to_degrees()
    );
}

#[test]
fn beyond_the_vanishing_angle_the_boat_keeps_rolling() {
    let mut sim = sim_with(0.0, 0.0, 100.0, no_capsize());
    let mut heel = 0.0;
    for _ in 0..(30.0 / 0.02) as usize {
        heel = sim
            .step(Controls {
                rudder: 0.0,
                sheet: 0.5,
                hike: 0.0,
            })
            .state
            .heel;
    }
    assert!(
        heel.to_degrees() > 120.0,
        "should roll further over, heel {} deg",
        heel.to_degrees()
    );
}
