//! Targets from docs/research/2026-10-14-dinghy-physics-validation.md
//! (Day 2017, Day and Nixon 2014): a sailor trims for speed in a 12 kn
//! breeze, and the boat lands on the published numbers.
use sailsim::{hull_resistance, BoatParams, BoatState, Controls, Scenario, Sim, SimConfig};

const KN: f64 = 0.514444;

struct Run {
    /// Metres per second made good toward the wind.
    vmg: f64,
    state: BoatState,
}

/// 90 s from rest, heading north, wind `twa_deg` off the bow (from starboard).
fn sail(wind_kn: f64, twa_deg: f64, sheet: f64) -> Run {
    let wind_dir = twa_deg.to_radians();
    let mut sim = Sim::new(SimConfig {
        fixed_heading: true,
        capsize_enabled: false,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: wind_kn * KN,
        true_wind_dir: wind_dir,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let progress = |s: &BoatState| s.x * wind_dir.sin() + s.y * wind_dir.cos();
    let (mut at_60, mut state) = (0.0, BoatState::default());
    for i in 0..4500 {
        state = sim
            .step(Controls {
                rudder: 0.0,
                sheet,
                hike: 0.0,
            })
            .state;
        if i == 2999 {
            at_60 = progress(&state);
        }
    }
    Run {
        vmg: (progress(&state) - at_60) / 30.0,
        state,
    }
}

/// The sheet setting a sailor would choose: best VMG upwind, best speed on a reach.
fn best_trim(wind_kn: f64, twa_deg: f64) -> Run {
    let score = |r: &Run| {
        if twa_deg < 60.0 {
            r.vmg
        } else {
            r.state.surge
        }
    };
    (0..=20)
        .map(|i| sail(wind_kn, twa_deg, i as f64 / 20.0))
        .max_by(|a, b| score(a).total_cmp(&score(b)))
        .unwrap()
}

#[test]
fn upwind_at_12_knots_matches_the_published_polar() {
    let run = best_trim(12.0, 46.0);
    let speed_kn = run.state.surge / KN;
    let vmg_kn = run.vmg / KN;
    let heel = run.state.heel.to_degrees().abs();
    assert!((speed_kn - 5.4).abs() < 0.4, "speed {speed_kn} kn");
    assert!((vmg_kn - 3.75).abs() < 0.4, "VMG {vmg_kn} kn");
    assert!((1.0..4.0).contains(&heel), "heel {heel} deg");
}

#[test]
fn upwind_vmg_levels_off_above_12_knots() {
    for wind_kn in [16.0, 18.0] {
        let vmg_kn = best_trim(wind_kn, 46.0).vmg / KN;
        assert!(
            (3.4..4.2).contains(&vmg_kn),
            "{wind_kn} kn wind: VMG {vmg_kn} kn"
        );
    }
}

#[test]
fn beam_reach_at_12_knots_is_about_6_and_a_half_knots() {
    let speed_kn = best_trim(12.0, 90.0).state.surge / KN;
    assert!((speed_kn - 6.5).abs() < 0.5, "speed {speed_kn} kn");
}

#[test]
fn hull_resistance_matches_the_tank_tests_from_2_to_9_knots() {
    // Day and Nixon (2014) bare hull, 160 kg: drag area at speed (kn, m^2).
    let tank = [
        (2.0, 0.0125),
        (3.0, 0.0132),
        (4.0, 0.0157),
        (5.0, 0.0207),
        (6.5, 0.0245),
        (8.0, 0.0221),
        (9.0, 0.0206),
    ];
    let params = BoatParams::ilca_like();
    for (kn, drag_area) in tank {
        let v = kn * KN;
        let expected = 0.5 * 1000.0 * v * v * drag_area;
        let got = -hull_resistance(&params, v);
        assert!(
            (got / expected - 1.0).abs() < 0.1,
            "{kn} kn: {got} N vs tank {expected} N"
        );
    }
}
