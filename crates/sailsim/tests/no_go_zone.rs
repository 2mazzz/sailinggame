use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Boat pointing straight into a 6 m/s wind, carrying 2 m/s of way, with
/// the sheet eased as a sailor would in irons.
fn head_to_wind() -> Scenario {
    Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: 0.0,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 2.0,
        start_heel: 0.0,
    }
}

#[test]
fn boat_head_to_wind_loses_way_and_stops() {
    let mut sim = Sim::new(SimConfig {
        fixed_heading: true,
        ..SimConfig::default()
    });
    sim.reset(head_to_wind());

    let mut out = sim.step(Controls {
        rudder: 0.0,
        sheet: 0.3,
    });
    let mut max_speed = out.state.surge;
    for _ in 0..(60.0 / 0.02) as usize {
        out = sim.step(Controls {
            rudder: 0.0,
            sheet: 0.3,
        });
        max_speed = max_speed.max(out.state.surge);
    }

    assert!(
        max_speed <= 2.0 + 1e-9,
        "boat accelerated in irons: {max_speed}"
    );
    assert!(
        out.state.surge < 0.5,
        "boat should have almost stopped after 60 s, surge = {}",
        out.state.surge
    );
    // In irons a dinghy drifts astern slowly (roughly 1-2 knots at most).
    assert!(
        out.state.surge > -1.0,
        "boat should not sail backwards fast"
    );
}
