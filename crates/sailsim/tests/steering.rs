use sailsim::{BoatState, Controls, Scenario, Sim, SimConfig, Status};

fn sim_with(wind_speed: f64, wind_dir_deg: f64, heading_deg: f64, speed: f64) -> Sim {
    let mut sim = Sim::new(SimConfig::default());
    sim.reset(Scenario {
        true_wind_speed: wind_speed,
        true_wind_dir: wind_dir_deg.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: heading_deg.to_radians(),
        start_speed: speed,
        start_heel: 0.0,
    });
    sim
}

fn run(sim: &mut Sim, seconds: f64, controls: Controls) -> BoatState {
    let mut state = BoatState::default();
    for _ in 0..(seconds / 0.02).round() as usize {
        state = sim.step(controls).state;
    }
    state
}

#[test]
fn rudder_turns_the_boat_the_way_it_points_at_a_plausible_rate() {
    for (rudder, direction) in [(1.0, 1.0), (-1.0, -1.0)] {
        let mut sim = sim_with(0.0, 0.0, 0.0, 2.0);
        let controls = Controls { rudder, sheet: 0.5 };
        let mut peak_rate_deg = 0.0_f64;
        let mut state = BoatState::default();
        for _ in 0..(3.0 / 0.02) as usize {
            state = sim.step(controls).state;
            peak_rate_deg = peak_rate_deg.max(state.yaw_rate.to_degrees() * direction);
        }
        assert!((15.0..60.0).contains(&peak_rate_deg), "rudder {rudder}: peak yaw rate {peak_rate_deg} deg/s");
        assert!(state.heading * direction > 10.0_f64.to_radians(), "heading {}", state.heading);
    }
}

#[test]
fn a_stationary_boat_cannot_steer() {
    let mut sim = sim_with(0.0, 0.0, 0.0, 0.0);
    let state = run(&mut sim, 10.0, Controls { rudder: 1.0, sheet: 0.5 });
    assert!(state.heading.to_degrees().abs() < 3.0, "heading {} deg", state.heading.to_degrees());
}

#[test]
fn a_released_helm_rounds_the_boat_up_into_the_wind() {
    // Close-hauled with the wind on the starboard bow: rounding up means
    // the bow swings to starboard, toward the wind (positive heading).
    let mut sim = sim_with(6.0, 45.0, 0.0, 2.0);
    let state = run(&mut sim, 15.0, Controls { rudder: 0.0, sheet: 0.85 });
    assert!(state.heading.to_degrees() > 5.0, "heading {} deg", state.heading.to_degrees());
}

/// Wrap an angle to (-pi, pi].
fn wrap(a: f64) -> f64 {
    a.sin().atan2(a.cos())
}

#[test]
fn a_tack_loses_speed_and_recovers_it() {
    // Wind from north. Port tack close-hauled heading 45 deg, then tack to
    // -45 deg (starboard tack) at t = 25 s with a simple helmsman.
    let mut sim = sim_with(6.0, 0.0, 45.0, 2.5);
    let steer = |state: &BoatState, target_deg: f64| Controls {
        rudder: (2.0 * wrap(target_deg.to_radians() - state.heading) - 1.0 * state.yaw_rate).clamp(-1.0, 1.0),
        sheet: 0.85,
    };

    let mut state = BoatState::default();
    let mut pre_tack_speeds = vec![];
    for i in 0..(25.0 / 0.02) as usize {
        let out = sim.step(steer(&state, 45.0));
        state = out.state;
        assert_eq!(out.status, Status::Sailing);
        if i as f64 * 0.02 > 20.0 {
            pre_tack_speeds.push(state.surge);
        }
    }
    let pre_tack = pre_tack_speeds.iter().sum::<f64>() / pre_tack_speeds.len() as f64;
    assert!(pre_tack > 1.5, "should be sailing upwind before the tack: {pre_tack}");

    let (mut min_speed, mut done_at, mut recovered_at) = (f64::MAX, None, None);
    for i in 0..(60.0 / 0.02) as usize {
        let t = i as f64 * 0.02;
        let out = sim.step(steer(&state, -45.0));
        state = out.state;
        assert_eq!(out.status, Status::Sailing, "capsized during the tack at t={t}");
        min_speed = min_speed.min(state.surge);
        if done_at.is_none() && wrap(state.heading + 45.0_f64.to_radians()).abs() < 5.0_f64.to_radians() {
            done_at = Some(t);
        }
        if done_at.is_some() && recovered_at.is_none() && state.surge > 0.9 * pre_tack {
            recovered_at = Some(t);
        }
    }

    let done_at = done_at.expect("tack never completed");
    assert!(done_at < 15.0, "tack took {done_at} s");
    assert!(min_speed < 0.7 * pre_tack, "tack should cost speed: {pre_tack} -> {min_speed}");
    let recovered_at = recovered_at.expect("speed never recovered after the tack");
    assert!(recovered_at - done_at < 30.0, "recovery took {} s", recovered_at - done_at);
}
