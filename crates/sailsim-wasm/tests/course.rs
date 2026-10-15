use sailsim_wasm::Game;

/// Steer at the next mark with a proportional rudder.
fn sail(game: &mut Game, seconds: f64) {
    use std::f64::consts::{PI, TAU};
    for _ in 0..(seconds * 60.0) as usize {
        let err = game.course_progress()[2] - game.snapshot()[2];
        let err = (err + PI).rem_euclid(TAU) - PI;
        game.advance(1.0 / 60.0, (2.0 * err).clamp(-1.0, 1.0), 0.5);
    }
}

#[test]
fn default_course_starts_upwind_with_progress_to_first_mark() {
    let game = Game::new(6.0, 0.0);
    let marks = game.course_marks();
    assert!((3..=4).contains(&(marks.len() / 2)));
    assert!(
        marks[1] > 0.0,
        "first mark lies north, upwind of a northerly"
    );
    let p = game.course_progress();
    assert_eq!(p[0], 0.0);
    assert!((p[1] - marks[1].hypot(marks[0])).abs() < 1e-9);
    assert_eq!(game.course_status(), 0);
}

#[test]
fn sailing_through_the_marks_finishes_with_a_time_and_reset_restarts() {
    let mut game = Game::new(6.0, 0.0);
    game.set_course(vec![40.0, 0.0, 80.0, 0.0, 120.0, 0.0]);
    sail(&mut game, 120.0);
    assert_eq!(game.course_status(), 1);
    assert_eq!(game.course_progress()[0], -1.0);
    let t = game.finish_time();
    assert!(t > 5.0 && t < 120.0, "time {t}");
    // The clock stops at the finish.
    sail(&mut game, 5.0);
    assert_eq!(game.finish_time(), t);

    game.reset();
    assert_eq!(game.course_status(), 0);
    assert_eq!(game.course_progress()[0], 0.0);
}

#[test]
fn changing_gust_and_shift_keeps_the_run_going() {
    let mut game = Game::new(6.0, 0.0);
    sail(&mut game, 5.0);
    let before = game.snapshot();
    game.set_gust_intensity(0.5);
    game.set_shift_deg(20.0);
    let after = game.snapshot();
    assert!(before[10] > 0.0);
    assert_eq!(after[0], before[0]);
    assert_eq!(after[1], before[1]);
    assert_eq!(after[10], before[10]);
    assert_eq!(game.gust_intensity(), 0.5);
    assert_eq!(game.shift_deg(), 20.0);
}
