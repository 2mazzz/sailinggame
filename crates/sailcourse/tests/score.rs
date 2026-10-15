use sailcourse::{Course, CourseRun, Mark, Outcome, RewardConfig};
use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Boat at the origin heading `heading`, 6 m/s wind from due east.
fn scenario(heading: f64) -> Scenario {
    Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: std::f64::consts::FRAC_PI_2,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: heading,
        start_speed: 0.0,
        start_heel: 0.0,
    }
}

fn run_on(marks: &[(f64, f64)], heading: f64) -> CourseRun {
    let sim = Sim::new(SimConfig {
        fixed_heading: true,
        ..SimConfig::default()
    });
    let course = Course {
        marks: marks.iter().map(|&(x, y)| Mark { x, y }).collect(),
        mark_radius: 10.0,
    };
    let mut run = CourseRun::new(sim, course);
    run.reset(scenario(heading));
    run
}

fn controls(sheet: f64) -> Controls {
    Controls {
        rudder: 0.0,
        sheet,
        hike: 0.0,
    }
}

fn sail(run: &mut CourseRun, sheet: f64, seconds: f64) {
    for _ in 0..(seconds / 0.02) as usize {
        if run.step(controls(sheet)) != Outcome::InProgress {
            break;
        }
    }
}

#[test]
fn closing_on_the_next_mark_scores_the_metres_gained_minus_the_clock() {
    let cfg = RewardConfig::default();
    let mut run = run_on(&[(0.0, 300.0)], 0.0);
    sail(&mut run, 0.5, 10.0);
    let gained = 300.0 - run.progress().distance;
    assert!(gained > 5.0, "boat should have moved: {gained}");
    let expected = cfg.progress_per_m * gained - cfg.time_penalty_per_s * 10.0;
    assert!(
        (run.score() - expected).abs() < 1e-6,
        "score {} vs expected {expected}",
        run.score()
    );
}

#[test]
fn sailing_away_from_the_mark_scores_negative() {
    // Heading south with the mark due north.
    let mut run = run_on(&[(0.0, 300.0)], std::f64::consts::PI);
    sail(&mut run, 0.5, 10.0);
    assert!(run.score() < 0.0, "score {}", run.score());
}

#[test]
fn each_mark_and_the_finish_add_a_bonus() {
    let cfg = RewardConfig::default();
    let mut run = run_on(&[(0.0, 30.0), (0.0, 60.0)], 0.0);
    sail(&mut run, 0.5, 120.0);
    assert!(matches!(run.outcome(), Outcome::Finished { .. }));
    assert!(
        run.score() > 2.0 * cfg.mark_bonus + 40.0,
        "score {} should hold two mark bonuses, the metres and a finish bonus",
        run.score()
    );
}

#[test]
fn finishing_faster_scores_higher() {
    let finish = |sheet: f64| {
        let mut run = run_on(&[(0.0, 60.0), (0.0, 120.0)], 0.0);
        sail(&mut run, sheet, 200.0);
        let Outcome::Finished { time } = run.outcome() else {
            panic!("sheet {sheet} did not finish: {:?}", run.outcome());
        };
        (time, run.score())
    };
    let (fast_t, fast) = finish(0.5);
    let (slow_t, slow) = finish(0.1);
    assert!(fast_t < slow_t, "{fast_t} vs {slow_t}");
    assert!(fast > slow, "fast {fast} should beat slow {slow}");
}

#[test]
fn capsizing_costs_the_penalty_and_the_score_then_freezes() {
    let cfg = RewardConfig::default();
    let sim = Sim::new(SimConfig {
        fixed_heading: true,
        ..SimConfig::default()
    });
    let course = Course {
        marks: vec![Mark { x: 0.0, y: 5000.0 }],
        mark_radius: 10.0,
    };
    let mut run = CourseRun::new(sim, course);
    run.reset(Scenario {
        true_wind_speed: 12.0,
        true_wind_dir: 45.0_f64.to_radians(),
        ..scenario(0.0)
    });
    let hard_in = Controls {
        rudder: 0.0,
        sheet: 1.0,
        hike: 0.0,
    };
    let mut last = 0.0;
    for _ in 0..3000 {
        run.step(hard_in);
        if matches!(run.outcome(), Outcome::Capsized { .. }) {
            last = run.last_reward();
            break;
        }
    }
    assert!(matches!(run.outcome(), Outcome::Capsized { .. }));
    assert!(last <= -cfg.capsize_penalty + 1.0, "reward at capsize {last}");
    let frozen = run.score();
    run.step(hard_in);
    assert_eq!(run.score(), frozen);
    assert_eq!(run.last_reward(), 0.0);
}

#[test]
fn reset_clears_the_score() {
    let mut run = run_on(&[(0.0, 300.0)], 0.0);
    sail(&mut run, 0.5, 5.0);
    assert!(run.score() != 0.0);
    run.reset(scenario(0.0));
    assert_eq!(run.score(), 0.0);
    assert_eq!(run.last_reward(), 0.0);
}
