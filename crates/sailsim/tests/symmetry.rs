use sailsim::{BoatState, Controls, Scenario, Sim, SimConfig};

fn trajectory(wind_dir_deg: f64, start_heading_deg: f64, rudder_sign: f64) -> Vec<BoatState> {
    let mut sim = Sim::new(SimConfig {
        capsize_enabled: false,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: 7.0,
        true_wind_dir: wind_dir_deg.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: start_heading_deg.to_radians(),
        start_speed: 1.0,
        start_heel: 0.0,
    });
    (0..2000)
        .map(|i| {
            let t = i as f64 * 0.02;
            let controls = Controls {
                rudder: rudder_sign * 0.2 * (t * 0.5).sin(),
                sheet: 0.4 + 0.3 * (t * 0.2).sin().abs(),
            };
            sim.step(controls).state
        })
        .collect()
}

#[test]
fn mirroring_wind_heading_and_rudder_mirrors_the_trajectory() {
    for (wind, heading) in [(45.0, 0.0), (100.0, 10.0), (170.0, -20.0)] {
        let starboard = trajectory(wind, heading, 1.0);
        let port = trajectory(-wind, -heading, -1.0);
        for (a, b) in starboard.iter().zip(&port) {
            let mirrored = BoatState {
                x: -b.x,
                heading: -b.heading,
                sway: -b.sway,
                yaw_rate: -b.yaw_rate,
                heel: -b.heel,
                heel_rate: -b.heel_rate,
                ..*b
            };
            for (name, l, r) in [
                ("x", a.x, mirrored.x),
                ("y", a.y, mirrored.y),
                ("heading", a.heading, mirrored.heading),
                ("surge", a.surge, mirrored.surge),
                ("sway", a.sway, mirrored.sway),
                ("yaw_rate", a.yaw_rate, mirrored.yaw_rate),
                ("heel", a.heel, mirrored.heel),
            ] {
                assert!(
                    (l - r).abs() < 1e-9,
                    "wind {wind}: {name} {l} vs mirrored {r}"
                );
            }
        }
    }
}
