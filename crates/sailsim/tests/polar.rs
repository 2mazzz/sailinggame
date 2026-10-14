use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Steady speed after 90 s on a heading `twa` degrees off a 6 m/s wind
/// (wind from the starboard side), with the sheet at `sheet`.
fn steady_speed(twa_deg: f64, sheet: f64) -> f64 {
    let mut sim = Sim::new(SimConfig {
        fixed_heading: true,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: twa_deg.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let mut surge = 0.0;
    for _ in 0..(90.0 / 0.02) as usize {
        surge = sim.step(Controls { rudder: 0.0, sheet }).state.surge;
    }
    surge
}

/// A sailor trims for speed: the best steady speed over all sheet settings.
fn best_speed(twa_deg: f64) -> f64 {
    (0..=20)
        .map(|i| steady_speed(twa_deg, i as f64 / 20.0))
        .fold(f64::MIN, f64::max)
}

#[test]
fn speed_peaks_on_a_reach_and_falls_close_hauled_and_dead_downwind() {
    let close_hauled = best_speed(45.0);
    let beam_reach = best_speed(90.0);
    let broad_reach = best_speed(135.0);
    let dead_downwind = best_speed(180.0);

    assert!(
        close_hauled > 0.5,
        "should make way close-hauled: {close_hauled}"
    );
    assert!(close_hauled < beam_reach, "{close_hauled} !< {beam_reach}");
    assert!(
        dead_downwind < broad_reach,
        "{dead_downwind} !< {broad_reach}"
    );
    assert!(
        dead_downwind < beam_reach,
        "{dead_downwind} !< {beam_reach}"
    );
    for v in [close_hauled, beam_reach, broad_reach, dead_downwind] {
        assert!(v < 4.8, "speed should stay below ~4.8 m/s, got {v}");
    }
}
