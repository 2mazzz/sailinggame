use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Boat at rest, nearly head to a 6 m/s wind, sheet in: the sail is just
/// off luffing and pushes the boat sideways with ~70 N. With no forward
/// speed the centreboard does little, but the hull and board still resist
/// sideways motion, so the boat must not slide sideways faster than 0.2 m/s.
///
/// Note: with the sheet fully eased (the ticket's wording) the sail luffs and
/// the boat already stays put; sheeted in is the case that used to slide
/// (0.48 m/s at 11 degrees off the wind). Well beyond ~11 degrees off the
/// wind the sail drives the boat forward and it is sailing, not stalled.
#[test]
fn stalled_boat_head_to_wind_does_not_slide_sideways() {
    for fixed_heading in [true, false] {
        for wind_dir in [0.2_f64, -0.2, 0.1] {
            let mut sim = Sim::new(SimConfig {
                fixed_heading,
                ..SimConfig::default()
            });
            sim.reset(Scenario {
                true_wind_speed: 6.0,
                true_wind_dir: wind_dir,
                start_x: 0.0,
                start_y: 0.0,
                start_heading: 0.0,
                start_speed: 0.0,
                start_heel: 0.0,
            });
            let mut max_sway = 0.0_f64;
            for _ in 0..3000 {
                let s = sim
                    .step(Controls {
                        rudder: 0.0,
                        sheet: 1.0,
                        hike: 0.0,
                    })
                    .state;
                max_sway = max_sway.max(s.sway.abs());
            }
            assert!(
                max_sway < 0.2,
                "fixed heading {fixed_heading}, wind {wind_dir}: max sway {max_sway} m/s"
            );
        }
    }
}
