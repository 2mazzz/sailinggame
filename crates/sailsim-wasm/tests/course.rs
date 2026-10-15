use sailsim_wasm::Game;

/// Steer at the next mark with a proportional rudder.
fn sail(game: &mut Game, seconds: f64) {
    use std::f64::consts::{PI, TAU};
    for _ in 0..(seconds * 60.0) as usize {
        let err = game.course_progress()[2] - game.snapshot()[2];
        let err = (err + PI).rem_euclid(TAU) - PI;
        game.advance(1.0 / 60.0, (2.0 * err).clamp(-1.0, 1.0), 0.5, 0.0);
    }
}

#[test]
fn default_course_is_a_reverse_c_starting_at_the_top() {
    let game = Game::new(6.0, 0.0);
    let marks = game.course_marks();
    let pts: Vec<(f64, f64)> = marks.chunks(2).map(|m| (m[0], m[1])).collect();
    assert_eq!(pts.len(), 5);
    // Along the top first: east of the start, at the start's height.
    assert!(pts[0].0 > 0.0 && pts[0].1.abs() < 10.0, "first mark {:?}", pts[0]);
    // Then the back of the C: the easternmost marks are in the middle.
    let max_x = pts.iter().map(|p| p.0).fold(f64::MIN, f64::max);
    assert!(pts[1].0 == max_x || pts[2].0 == max_x);
    assert!(pts[0].0 < max_x && pts[4].0 < max_x);
    // Down to the bottom and back west, ending below and level with the start.
    let min_y = pts.iter().map(|p| p.1).fold(f64::MAX, f64::min);
    assert_eq!(pts[4].1, min_y);
    assert!(pts[4].0.abs() < 10.0, "last mark {:?}", pts[4]);
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

#[test]
fn hiking_through_the_game_lowers_the_heel() {
    let heel_after = |hike: f64| {
        let mut game = Game::new(8.0, 45.0);
        game.set_capsize_enabled(false);
        // Close-hauled to a wind from 45 degrees, heading north.
        for _ in 0..(40.0 * 60.0) as usize {
            game.advance(1.0 / 60.0, 0.0, 1.0, hike);
        }
        game.snapshot()[3].abs()
    };
    let (sitting, hiking) = (heel_after(0.0), heel_after(1.0));
    assert!(hiking < sitting, "heel sitting {sitting}, hiking {hiking}");
}
