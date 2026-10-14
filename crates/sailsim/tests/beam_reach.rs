use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Heading north, 6 m/s wind from due east: a beam reach, from rest.
fn beam_reach() -> Scenario {
    Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: std::f64::consts::FRAC_PI_2,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    }
}

fn surge_after(sim: &mut Sim, seconds: f64, controls: Controls) -> f64 {
    let mut surge = 0.0;
    for _ in 0..(seconds / 0.02).round() as usize {
        surge = sim.step(controls).state.surge;
    }
    surge
}

#[test]
fn beam_reach_accelerates_to_a_plausible_steady_speed() {
    let mut sim = Sim::new(SimConfig::default());
    sim.reset(beam_reach());
    let controls = Controls { rudder: 0.0, sheet: 0.5 };

    let at_10s = surge_after(&mut sim, 10.0, controls);
    let at_50s = surge_after(&mut sim, 40.0, controls);
    let at_60s = surge_after(&mut sim, 10.0, controls);

    assert!(at_10s > 1.0, "boat should be well under way after 10 s: {at_10s}");
    assert!(
        (2.0..4.8).contains(&at_60s),
        "beam-reach speed in 6 m/s wind should be a few m/s, got {at_60s}"
    );
    assert!(
        (at_60s - at_50s).abs() < 0.05 * at_60s,
        "speed should have levelled off: {at_50s} -> {at_60s}"
    );
}
