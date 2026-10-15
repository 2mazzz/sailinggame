use sailsim::{Controls, Scenario, Sim, SimConfig};

const MEAN_DIR_DEG: f64 = 45.0;

fn shifty(seed: u64, intensity_deg: f64) -> Sim {
    let mut sim = Sim::new(SimConfig {
        seed,
        shift_intensity: intensity_deg.to_radians(),
        capsize_enabled: false,
        fixed_heading: true,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: 6.0,
        true_wind_dir: MEAN_DIR_DEG.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    sim
}

const HOLD: Controls = Controls {
    rudder: 0.0,
    sheet: 0.8,
    hike: 0.0,
};

#[test]
fn same_seed_gives_the_same_shifts() {
    let run = |seed| {
        let mut sim = shifty(seed, 15.0);
        (0..3000)
            .map(|_| {
                sim.step(HOLD);
                sim.wind_dir()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(run(2), run(2));
    assert_ne!(run(2), run(3));
}

#[test]
fn zero_intensity_keeps_the_direction_fixed() {
    let mut sim = shifty(9, 0.0);
    for _ in 0..500 {
        sim.step(HOLD);
        assert_eq!(sim.wind_dir(), MEAN_DIR_DEG.to_radians());
    }
}

#[test]
fn shifts_stay_within_intensity_and_actually_swing_the_wind() {
    let intensity = 15.0_f64;
    let mut sim = shifty(1, intensity);
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for _ in 0..60_000 {
        sim.step(HOLD);
        let d = (sim.wind_dir().to_degrees() - MEAN_DIR_DEG).abs();
        assert!(d <= intensity + 1e-9, "shift {d}");
        lo = lo.min(sim.wind_dir());
        hi = hi.max(sim.wind_dir());
    }
    assert!((hi - lo).to_degrees() > 0.5 * intensity, "too flat");
}

#[test]
fn a_shift_changes_the_true_wind_angle_for_a_boat_holding_heading() {
    let mut sim = shifty(1, 20.0);
    let twa = |sim: &Sim| sim.wind_dir() - sim.state().heading;
    let first = twa(&sim);
    let mut max_change: f64 = 0.0;
    for _ in 0..30_000 {
        sim.step(HOLD);
        assert_eq!(sim.state().heading, 0.0);
        max_change = max_change.max((twa(&sim) - first).abs());
    }
    assert!(max_change.to_degrees() > 5.0, "{max_change}");
}

#[test]
fn set_wind_sets_the_mean_direction_the_shifts_swing_around() {
    let mut sim = shifty(1, 10.0);
    sim.set_wind(6.0, 2.0);
    sim.step(HOLD);
    assert!((sim.wind_dir() - 2.0).abs() <= 10.0_f64.to_radians() + 1e-9);
}
