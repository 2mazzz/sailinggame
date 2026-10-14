use sailsim::{Controls, Scenario, Sim, SimConfig, Status};

fn calm_scenario() -> Scenario {
    Scenario {
        true_wind_speed: 0.0,
        true_wind_dir: 0.0,
        start_x: 10.0,
        start_y: -5.0,
        start_heading: 1.0,
        start_speed: 0.0,
        start_heel: 0.0,
    }
}

#[test]
fn boat_at_rest_in_no_wind_stays_put() {
    let mut sim = Sim::new(SimConfig { fixed_heading: true, ..SimConfig::default() });
    sim.reset(calm_scenario());

    let mut out = sim.step(Controls { rudder: 0.0, sheet: 0.5 });
    for _ in 0..1000 {
        out = sim.step(Controls { rudder: 0.0, sheet: 0.5 });
    }

    assert_eq!(out.status, Status::Sailing);
    assert!((out.state.x - 10.0).abs() < 1e-9);
    assert!((out.state.y - -5.0).abs() < 1e-9);
    assert!((out.state.heading - 1.0).abs() < 1e-9);
    assert!(out.state.surge.abs() < 1e-9);
    assert!(out.state.sway.abs() < 1e-9);
}

#[test]
fn same_seed_and_controls_give_identical_trajectories() {
    let scenario = Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: 0.3,
        ..calm_scenario()
    };
    let run = |seed: u64| {
        let mut sim = Sim::new(SimConfig { seed, fixed_heading: true, ..SimConfig::default() });
        sim.reset(scenario);
        (0..500)
            .map(|i| {
                let c = Controls {
                    rudder: ((i as f64) * 0.05).sin() * 0.3,
                    sheet: 0.4,
                };
                sim.step(c).state
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(run(42), run(42));
}
