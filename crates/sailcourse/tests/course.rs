use sailcourse::{Course, CourseRun, Mark, Outcome};
use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Boat at the origin heading north, 6 m/s wind from due east (beam reach).
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

fn course(marks: &[(f64, f64)]) -> Course {
    Course {
        marks: marks.iter().map(|&(x, y)| Mark { x, y }).collect(),
        // Generous: the boat drifts to leeward on a beam reach.
        mark_radius: 10.0,
    }
}

fn run_on(marks: &[(f64, f64)]) -> CourseRun {
    let sim = Sim::new(SimConfig {
        fixed_heading: true,
        ..SimConfig::default()
    });
    let mut run = CourseRun::new(sim, course(marks));
    run.reset(beam_reach());
    run
}

const BEAM_REACH_CONTROLS: Controls = Controls {
    rudder: 0.0,
    sheet: 0.5,
};

/// Step until the episode ends or `max_seconds` pass.
fn sail(run: &mut CourseRun, controls: Controls, max_seconds: f64) -> Outcome {
    for _ in 0..(max_seconds / 0.02) as usize {
        if run.step(controls) != Outcome::InProgress {
            break;
        }
    }
    run.outcome()
}

#[test]
fn sailing_through_the_marks_in_order_finishes_and_reports_the_time() {
    // Boat holds north; marks lie dead ahead, 60 m in total (minus the radius).
    let mut run = run_on(&[(0.0, 30.0), (0.0, 60.0)]);
    let Outcome::Finished { time } = sail(&mut run, BEAM_REACH_CONTROLS, 120.0) else {
        panic!("expected to finish, got {:?}", run.outcome());
    };
    // At least 50 m to cover, from rest, at a beam-reach speed of a few m/s.
    assert!(
        (50.0 / 5.0..60.0).contains(&time),
        "implausible finish time {time}"
    );
    assert_eq!(run.progress().next_mark, None);
}

#[test]
fn progress_points_at_the_first_mark() {
    let run = run_on(&[(0.0, 100.0), (0.0, 200.0)]);
    let p = run.progress();
    assert_eq!(p.next_mark, Some(0));
    assert!((p.distance - 100.0).abs() < 1e-9);
    assert!(p.bearing.abs() < 1e-9, "due north is bearing 0");
}

#[test]
fn marks_reached_out_of_order_do_not_count() {
    // The boat sails north through the second mark (30 m) on its way to the
    // first (120 m). Having passed it early, it is not credited afterwards.
    let mut run = run_on(&[(0.0, 120.0), (0.0, 30.0)]);
    for _ in 0..(30.0 / 0.02) as usize {
        run.step(BEAM_REACH_CONTROLS);
    }
    // More than 30 m north (distance to the first mark under 90 m, so the
    // second mark has been passed) yet the course has not advanced.
    let p = run.progress();
    assert!(p.distance < 90.0, "boat should be past 30 m: {p:?}");
    assert_eq!(p.next_mark, Some(0));

    // Carry on well beyond the first mark: it counts, the second does not.
    let outcome = sail(&mut run, BEAM_REACH_CONTROLS, 90.0);
    assert_eq!(outcome, Outcome::InProgress);
    assert_eq!(run.progress().next_mark, Some(1), "{:?}", run.progress());
}

#[test]
fn capsizing_ends_the_episode_with_its_own_outcome() {
    // 12 m/s from 45 degrees, sheeted in hard, heading locked north: capsizes.
    let sim = Sim::new(SimConfig {
        fixed_heading: true,
        ..SimConfig::default()
    });
    let mut run = CourseRun::new(sim, course(&[(0.0, 5000.0)]));
    run.reset(Scenario {
        true_wind_speed: 12.0,
        true_wind_dir: 45.0_f64.to_radians(),
        ..beam_reach()
    });
    let controls = Controls {
        rudder: 0.0,
        sheet: 0.85,
    };
    let Outcome::Capsized { time } = sail(&mut run, controls, 60.0) else {
        panic!("expected capsize, got {:?}", run.outcome());
    };
    assert!(time > 0.0 && time < 60.0);
    // The episode is over: stepping on changes nothing.
    assert_eq!(run.step(controls), run.outcome());
}

#[test]
fn the_same_seed_and_controls_give_the_same_result() {
    let play = || {
        let sim = Sim::new(SimConfig {
            seed: 7,
            ..SimConfig::default()
        });
        let mut run = CourseRun::new(sim, course(&[(0.0, 40.0), (30.0, 80.0)]));
        run.reset(beam_reach());
        for i in 0..3000 {
            let rudder = if (i / 100) % 2 == 0 { 0.1 } else { -0.1 };
            run.step(Controls { rudder, sheet: 0.5 });
        }
        (run.outcome(), run.progress())
    };
    assert_eq!(play(), play());
}

#[test]
fn bearing_is_clockwise_from_north() {
    let east = run_on(&[(100.0, 0.0)]).progress();
    assert!((east.bearing - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    let south = run_on(&[(0.0, -50.0)]).progress();
    assert!((south.bearing.abs() - std::f64::consts::PI).abs() < 1e-9);
    assert!((south.distance - 50.0).abs() < 1e-9);
}
