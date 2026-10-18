//! Numbers recorded before the planing and rudder work (2026-10-18). They are
//! loose guards, not targets: later tasks must keep them inside these bands or
//! say why they moved.
use sailsim::{Controls, Scenario, Sim, SimConfig};

const KN: f64 = 0.514444;

fn reach_speed(wind_kn: f64) -> f64 {
    let mut sim = Sim::new(SimConfig {
        fixed_heading: true,
        capsize_enabled: false,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: wind_kn * KN,
        true_wind_dir: std::f64::consts::FRAC_PI_2,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let mut surge = 0.0;
    for _ in 0..4500 {
        surge = sim
            .step(Controls { rudder: 0.0, sheet: 0.55, hike: 1.0 })
            .state
            .surge;
    }
    surge / KN
}

#[test]
fn print_reach_speeds_for_the_record() {
    for wind in [8.0, 12.0, 16.0, 20.0, 25.0] {
        println!("beam reach {wind} kn wind: {:.2} kn", reach_speed(wind));
    }
}

#[test]
fn strong_wind_reach_is_fast_but_not_absurd() {
    // No primary data above 9 kn. Forum anecdote: a Laser GPS-logged about
    // 15 kn in a gust with tide. This is a sanity band, not a validation.
    let kn = reach_speed(25.0);
    assert!((7.0..16.0).contains(&kn), "{kn} kn");
}
