use sailcourse::observation::{course_observation, MARK_SLOTS, OBS_LEN};
use sailcourse::{default_course, Course, CourseRun, Mark};
use sailsim::{Controls, Scenario, Sim, SimConfig};

fn run_heading(heading: f64) -> CourseRun {
    let course = Course {
        marks: vec![Mark { x: 0.0, y: 100.0 }],
        mark_radius: 6.0,
    };
    let mut run = CourseRun::new(Sim::new(SimConfig::default()), course);
    run.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: 0.0,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: heading,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    run
}

// Indices into the observation, see `observation.rs`.
const SIN_ERR: usize = 9;
const COS_ERR: usize = 10;
const DIST: usize = 11;

#[test]
fn the_mark_dead_ahead_has_no_bearing_error() {
    let obs = course_observation(&run_heading(0.0));
    assert_eq!(obs.len(), OBS_LEN);
    assert!(obs[SIN_ERR].abs() < 1e-9);
    assert!((obs[COS_ERR] - 1.0).abs() < 1e-9);
    assert!((obs[DIST] - 1.0).abs() < 1e-9, "100 m is 1.0: {}", obs[DIST]);
}

#[test]
fn a_mark_to_port_gives_a_negative_error_sine() {
    // Heading east, mark due north: it lies 90 degrees to port.
    let obs = course_observation(&run_heading(std::f64::consts::FRAC_PI_2));
    assert!((obs[SIN_ERR] + 1.0).abs() < 1e-9, "{}", obs[SIN_ERR]);
    assert!(obs[COS_ERR].abs() < 1e-9);
}

#[test]
fn every_input_stays_finite_and_bounded_while_sailing() {
    let mut run = run_heading(1.0);
    for i in 0..3000 {
        let rudder = if (i / 150) % 2 == 0 { 0.4 } else { -0.4 };
        run.step(Controls {
            rudder,
            sheet: 0.5,
            hike: 0.0,
        });
        for (k, v) in course_observation(&run).iter().enumerate() {
            assert!(v.is_finite() && v.abs() <= 3.0, "obs[{k}] = {v} at step {i}");
        }
    }
}

#[test]
fn the_default_course_is_the_tight_figure_eight() {
    let c = default_course();
    assert_eq!(c.marks.len(), 8);
    assert_eq!(c.mark_radius, 6.0);
}

#[test]
fn the_mark_after_next_is_seen_relative_to_the_heading() {
    // Next mark 100 m north, the one after due east of the boat: 90 degrees to starboard.
    let course = Course {
        marks: vec![Mark { x: 0.0, y: 100.0 }, Mark { x: 100.0, y: 0.0 }],
        mark_radius: 6.0,
    };
    let mut run = CourseRun::new(Sim::new(SimConfig::default()), course);
    run.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: 0.0,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let obs = course_observation(&run);
    assert!((obs[12] - 1.0).abs() < 1e-9, "sin {}", obs[12]);
    assert!(obs[13].abs() < 1e-9, "cos {}", obs[13]);
}

#[test]
fn on_the_last_leg_there_is_no_mark_after_next() {
    let obs = course_observation(&run_heading(0.0));
    assert_eq!(&obs[12..15], &[0.0, 0.0, 0.0]);
    assert!((obs[OBS_LEN - 1] - 1.0 / MARK_SLOTS as f64).abs() < 1e-9);
}

#[test]
fn every_remaining_mark_is_seen_in_the_order_it_is_rounded() {
    // Marks 100 m north, then north-east, then due east of the start, heading north.
    let course = Course {
        marks: vec![
            Mark { x: 0.0, y: 100.0 },
            Mark { x: 100.0, y: 100.0 },
            Mark { x: 100.0, y: 0.0 },
        ],
        mark_radius: 6.0,
    };
    let mut run = CourseRun::new(Sim::new(SimConfig::default()), course);
    run.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: 0.0,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let obs = course_observation(&run);
    // Slot 2 is the third mark: due east, 90 degrees to starboard, 100 m away.
    let (sin, cos, dist) = (obs[15], obs[16], obs[17]);
    assert!((sin - 1.0).abs() < 1e-9 && cos.abs() < 1e-9 && (dist - 1.0).abs() < 1e-9);
    // Slots 3 and up are empty.
    assert!(obs[18..OBS_LEN - 1].iter().all(|v| *v == 0.0));
    assert!((obs[OBS_LEN - 1] - 3.0 / MARK_SLOTS as f64).abs() < 1e-9);
}
