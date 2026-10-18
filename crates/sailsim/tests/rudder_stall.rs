use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Steady yaw rate (deg/s) after `seconds` at constant rudder, no wind, 2.5 m/s start.
fn yaw_rate_deg(rudder: f64, seconds: f64) -> f64 {
    let mut sim = Sim::new(SimConfig { capsize_enabled: false, ..SimConfig::default() });
    sim.reset(Scenario {
        true_wind_speed: 0.0,
        true_wind_dir: 0.0,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 2.5,
        start_heel: 0.0,
    });
    let mut r = 0.0;
    for _ in 0..(seconds / 0.02) as usize {
        r = sim.step(Controls { rudder, sheet: 0.5, hike: 0.0 }).state.yaw_rate;
    }
    r.to_degrees()
}

#[test]
fn holding_the_rudder_over_costs_speed() {
    let speed = |rudder: f64| {
        let mut sim = Sim::new(SimConfig {
            fixed_heading: true,
            capsize_enabled: false,
            ..SimConfig::default()
        });
        sim.reset(Scenario {
            true_wind_speed: 0.0,
            true_wind_dir: 0.0,
            start_x: 0.0,
            start_y: 0.0,
            start_heading: 0.0,
            start_speed: 3.0,
            start_heel: 0.0,
        });
        let mut s = 0.0;
        for _ in 0..250 {
            s = sim.step(Controls { rudder, sheet: 0.5, hike: 0.0 }).state.surge;
        }
        s
    };
    // With the heading locked the rudder only adds drag.
    assert!(speed(1.0) < speed(0.0) - 0.05, "{} vs {}", speed(1.0), speed(0.0));
}
