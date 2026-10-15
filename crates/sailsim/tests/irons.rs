use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Head to a 6 m/s wind and drifting astern at 0.3 m/s, with the sheet eased
/// and full rudder held until the bow is past the beam (then sheet trimmed
/// and the boat steered to hold a reach), the boat must be
/// sailing again (more than 2 m/s forward) within about 10 s. It used to take
/// 20 to 25 s.
fn recovery_time(rudder: f64, wind_dir: f64) -> Option<f64> {
    let mut sim = Sim::new(SimConfig::default());
    sim.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: wind_dir,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: -0.3,
        start_heel: 0.0,
    });
    for i in 0..3000 {
        let heading = sim.state().heading;
        let off_wind = heading - wind_dir;
        let controls = if off_wind.abs() > 95.0_f64.to_radians() {
            // Bore away onto a reach: trim in and steer to hold 110 degrees off the wind.
            let target = wind_dir + 110.0_f64.to_radians() * off_wind.signum();
            Controls {
                rudder: (2.0 * (target - heading)).clamp(-1.0, 1.0),
                sheet: 0.5,
                hike: 0.0,
            }
        } else {
            Controls {
                rudder,
                sheet: 0.0,
                hike: 0.0,
            }
        };
        let s = sim.step(controls).state;
        if s.surge > 2.0 {
            return Some((i + 1) as f64 * sim.dt());
        }
    }
    None
}

#[test]
fn boat_in_irons_sails_away_within_about_ten_seconds() {
    for (rudder, wind_dir) in [(1.0, 0.0), (-1.0, 0.0), (1.0, 0.05), (-1.0, -0.05)] {
        let t = recovery_time(rudder, wind_dir);
        assert!(
            t.is_some_and(|t| t <= 10.0),
            "rudder {rudder}, wind {wind_dir}: recovered at {t:?}"
        );
    }
}
