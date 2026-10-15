use sailsim::{BoatState, Controls, Scenario, Sim, SimConfig};

const WIND: f64 = 6.0;

struct Run {
    /// Metres made good toward the wind per second, over the last 30 s.
    vmg: f64,
    final_state: BoatState,
}

/// Heading north with the wind `twa_deg` off the bow (positive = from the
/// starboard side), sheet fixed, 90 s from rest.
fn sail(twa_deg: f64, sheet: f64) -> Run {
    let wind_dir = twa_deg.to_radians();
    let mut sim = Sim::new(SimConfig {
        fixed_heading: true,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: WIND,
        true_wind_dir: wind_dir,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let toward_wind = (wind_dir.sin(), wind_dir.cos());
    let progress = |s: &BoatState| s.x * toward_wind.0 + s.y * toward_wind.1;

    let mut at_60 = 0.0;
    let mut state = BoatState::default();
    for i in 0..4500 {
        state = sim.step(Controls { rudder: 0.0, sheet }).state;
        if i == 2999 {
            at_60 = progress(&state);
        }
    }
    Run {
        vmg: (progress(&state) - at_60) / 30.0,
        final_state: state,
    }
}

/// A sailor trims for the best upwind progress.
fn best_trim(twa_deg: f64, sheets: usize) -> Run {
    (0..=sheets)
        .map(|i| sail(twa_deg, i as f64 / sheets as f64))
        .max_by(|a, b| a.vmg.total_cmp(&b.vmg))
        .unwrap()
}

#[test]
fn makes_way_upwind_on_both_tacks() {
    for twa in [45.0, -45.0] {
        let run = best_trim(twa, 20);
        assert!(run.vmg > 0.5, "tack {twa}: VMG {} m/s", run.vmg);
    }
}

#[test]
fn leeway_is_small_but_nonzero_and_points_downwind() {
    for twa in [45.0, -45.0] {
        let s = best_trim(twa, 20).final_state;
        let leeway_deg = (s.sway / s.surge).atan().to_degrees();
        assert!(
            leeway_deg.abs() > 1.0,
            "tack {twa}: leeway {leeway_deg} deg too small"
        );
        assert!(
            leeway_deg.abs() < 10.0,
            "tack {twa}: leeway {leeway_deg} deg too big"
        );
        // Wind from starboard pushes the boat to port (negative sway).
        assert_eq!(
            leeway_deg.signum(),
            -twa.signum(),
            "tack {twa}: leeway {leeway_deg} deg"
        );
    }
}

#[test]
fn best_upwind_angle_is_between_35_and_55_degrees() {
    let angles: Vec<f64> = (30..=70).step_by(5).map(f64::from).collect();
    let vmgs: Vec<f64> = angles.iter().map(|&a| best_trim(a, 10).vmg).collect();
    let (best_idx, _) = vmgs
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .unwrap();
    let best_angle = angles[best_idx];
    assert!(
        (35.0..=55.0).contains(&best_angle),
        "best VMG at {best_angle} deg; VMGs by angle: {:?}",
        angles.iter().zip(&vmgs).collect::<Vec<_>>()
    );
}

/// The no-go zone is firm: pinned at 30 degrees to the wind, even trimmed as
/// well as a sailor can, the boat cannot keep driving, whichever tack.
#[test]
fn held_at_30_degrees_to_the_wind_the_boat_cannot_keep_driving() {
    for twa in [30.0, -30.0] {
        let surge = best_trim(twa, 20).final_state.surge;
        assert!(surge <= 1.5, "tack {twa}: sustains {surge} m/s at 30 deg");
    }
}
