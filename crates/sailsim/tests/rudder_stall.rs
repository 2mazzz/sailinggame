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

#[test]
fn turn_entry_is_strongest_at_a_moderate_rudder_angle_not_at_full_rudder() {
    // The blade stalls at about 12 degrees of angle of attack. In the first 0.2 s of a turn
    // the stern has not started to swing yet, so the stall is visible: full rudder
    // gives less yaw than a moderate one. (In a steady turn the stern's sideways speed
    // cuts the blade's angle of attack, so steady yaw rate rises with rudder; see the
    // Retune log in docs/plans/2026-10-18-planing-and-rudder-realism.md.)
    let moderate = [0.4, 0.5, 0.6]
        .iter()
        .map(|&u| yaw_rate_deg(u, 0.2))
        .fold(f64::MIN, f64::max);
    let full = yaw_rate_deg(1.0, 0.2);
    assert!(full < 0.95 * moderate, "full {full} deg/s, moderate {moderate} deg/s at 0.2 s");
}

#[test]
fn steady_turn_rate_still_rises_with_rudder() {
    // The flip side of the stern-swing effect, kept as a guard so nobody "fixes" it by accident.
    let rates: Vec<f64> = [0.1, 0.3, 0.5, 0.7, 1.0].iter().map(|&u| yaw_rate_deg(u, 2.0)).collect();
    assert!(rates.windows(2).all(|w| w[1] > w[0]), "{rates:?}");
}
