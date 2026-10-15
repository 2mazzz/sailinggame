use sailsim::{Controls, Scenario, Sim, SimConfig};

const MEAN: f64 = 6.0;

fn gusty(seed: u64, intensity: f64) -> Sim {
    let mut sim = Sim::new(SimConfig {
        seed,
        gust_intensity: intensity,
        capsize_enabled: false,
        fixed_heading: true,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: MEAN,
        true_wind_dir: 45.0_f64.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    sim
}

fn controls(sheet: f64) -> Controls {
    Controls {
        rudder: 0.0,
        sheet,
        hike: 0.0,
    }
}

#[test]
fn same_seed_and_controls_give_an_identical_trajectory() {
    let run = |seed| {
        let mut sim = gusty(seed, 0.4);
        (0..2000)
            .map(|_| {
                let s = sim.step(controls(0.8)).state;
                (s.x, s.y, s.heel, sim.wind_speed())
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(run(3), run(3));
    assert_ne!(run(3), run(4));
}

#[test]
fn zero_intensity_leaves_the_wind_steady() {
    let mut sim = gusty(9, 0.0);
    for _ in 0..500 {
        sim.step(controls(0.8));
        assert_eq!(sim.wind_speed(), MEAN);
    }
}

#[test]
fn wind_varies_around_the_mean_within_the_stated_range() {
    let intensity = 0.4;
    let mut sim = gusty(1, intensity);
    let (mut lo, mut hi, mut sum) = (f64::MAX, f64::MIN, 0.0);
    let n = 30_000; // 600 s
    for _ in 0..n {
        sim.step(controls(0.8));
        let w = sim.wind_speed();
        lo = lo.min(w);
        hi = hi.max(w);
        sum += w;
    }
    assert!(lo >= MEAN * (1.0 - intensity) - 1e-9, "min {lo}");
    assert!(hi <= MEAN * (1.0 + intensity) + 1e-9, "max {hi}");
    assert!(
        hi - lo > 0.3 * MEAN * intensity,
        "range {lo}..{hi} too flat"
    );
    let mean = sum / n as f64;
    assert!((mean - MEAN).abs() < 0.15 * MEAN, "average {mean}");
}

/// Peak heel magnitude (deg) over 60 s of gusty wind.
fn peak_heel(sheet: f64, intensity: f64) -> f64 {
    let mut sim = gusty(5, intensity);
    let mut peak: f64 = 0.0;
    for _ in 0..3000 {
        peak = peak.max(sim.step(controls(sheet)).state.heel.to_degrees().abs());
    }
    peak
}

#[test]
fn heel_responds_to_gusts_and_an_eased_sheet_reduces_it() {
    let mut sim = gusty(5, 0.4);
    let mut heels = Vec::new();
    for _ in 0..3000 {
        heels.push(sim.step(controls(0.85)).state.heel.to_degrees());
    }
    let tail = &heels[1500..];
    let spread = tail.iter().cloned().fold(f64::MIN, f64::max)
        - tail.iter().cloned().fold(f64::MAX, f64::min);
    // A hiking crew makes the boat stiff (about 700 N m upright, Day 2017), so a
    // 40 % gust moves the heel by about a degree rather than several.
    assert!(spread > 0.5, "heel barely moves in gusts: {spread} deg");

    let trimmed = peak_heel(0.85, 0.4);
    let eased = peak_heel(0.30, 0.4);
    assert!(eased < 0.5 * trimmed, "{trimmed} deg vs eased {eased} deg");
}
