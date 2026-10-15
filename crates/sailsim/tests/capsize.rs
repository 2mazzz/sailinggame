use sailsim::{BoatState, Controls, Scenario, Sim, SimConfig, Status};

fn gale_close_hauled(config: SimConfig, start_heel_deg: f64) -> Sim {
    let mut sim = Sim::new(config);
    sim.reset(Scenario {
        true_wind_speed: 12.0,
        true_wind_dir: 45.0_f64.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: start_heel_deg.to_radians(),
    });
    sim
}

const SHEETED_IN: Controls = Controls {
    rudder: 0.0,
    sheet: 1.0,
};

fn run(
    sim: &mut Sim,
    seconds: f64,
    controls: impl Fn(&BoatState) -> Controls,
) -> (BoatState, Status) {
    let mut last = sim.step(controls(&BoatState::default()));
    for _ in 0..(seconds / 0.02) as usize {
        last = sim.step(controls(&last.state));
    }
    (last.state, last.status)
}

#[test]
fn sheeted_in_close_hauled_in_a_gale_capsizes() {
    let mut sim = gale_close_hauled(
        SimConfig {
            fixed_heading: true,
            ..SimConfig::default()
        },
        0.0,
    );
    let (_, status) = run(&mut sim, 60.0, |_| SHEETED_IN);
    assert_eq!(status, Status::Capsized);
}

#[test]
fn the_same_gale_does_not_capsize_when_capsize_is_disabled() {
    let mut sim = gale_close_hauled(
        SimConfig {
            capsize_enabled: false,
            fixed_heading: true,
            ..SimConfig::default()
        },
        0.0,
    );
    for _ in 0..3000 {
        assert_eq!(sim.step(SHEETED_IN).status, Status::Sailing);
    }
}

#[test]
fn easing_the_sheet_when_heeled_over_avoids_the_capsize() {
    let mut sim = gale_close_hauled(
        SimConfig {
            fixed_heading: true,
            ..SimConfig::default()
        },
        0.0,
    );
    let (state, status) = run(&mut sim, 60.0, |s| {
        let sheet = if s.heel.abs() > 30.0_f64.to_radians() {
            0.1
        } else {
            0.85
        };
        Controls { rudder: 0.0, sheet }
    });
    assert_eq!(
        status,
        Status::Sailing,
        "heel {} deg",
        state.heel.to_degrees()
    );
}

#[test]
fn capsize_is_terminal_and_freezes_the_boat() {
    let mut sim = gale_close_hauled(
        SimConfig {
            fixed_heading: true,
            ..SimConfig::default()
        },
        0.0,
    );
    let (frozen, status) = run(&mut sim, 60.0, |_| SHEETED_IN);
    assert_eq!(status, Status::Capsized);

    let after = sim.step(Controls {
        rudder: 0.5,
        sheet: 0.0,
    });
    assert_eq!(after.status, Status::Capsized);
    assert_eq!(after.state, frozen);
}

#[test]
fn a_boat_flat_on_the_water_does_not_sail() {
    let mut sim = gale_close_hauled(
        SimConfig {
            capsize_enabled: false,
            fixed_heading: true,
            ..SimConfig::default()
        },
        120.0,
    );
    let (state, _) = run(&mut sim, 30.0, |_| SHEETED_IN);
    assert!(state.surge.abs() < 1.0, "surge {} m/s", state.surge);
}

/// Capsize wind speed, re-measured after the hiking righting curve and
/// depowering (ticket 14): sheeted in close-hauled at 45 degrees, the boat
/// survives a fresh 16 kn (8 m/s) breeze but capsizes by 12 m/s; the threshold
/// is about 10 m/s (19 kn), against 10.5 m/s (20 kn) before.
#[test]
fn sheeted_in_close_hauled_survives_a_fresh_breeze_but_not_a_gale() {
    for (wind, expected) in [(8.0, Status::Sailing), (12.0, Status::Capsized)] {
        let mut sim = Sim::new(SimConfig {
            fixed_heading: true,
            ..SimConfig::default()
        });
        sim.reset(Scenario {
            true_wind_speed: wind,
            true_wind_dir: 45.0_f64.to_radians(),
            start_x: 0.0,
            start_y: 0.0,
            start_heading: 0.0,
            start_speed: 0.0,
            start_heel: 0.0,
        });
        let (_, status) = run(&mut sim, 90.0, |_| SHEETED_IN);
        assert_eq!(status, expected, "{wind} m/s");
    }
}
