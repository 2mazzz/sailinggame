//! Writes the CSVs behind the engineering graphs in docs/boat-dynamics.md.
//!
//!     cargo run --release -p sailsim --example dynamics_data -- <out dir>
//!
//! Then `python/scripts/dynamics_figures.py <out dir> docs/img/dynamics` draws them.

use sailsim::{
    foil_coefficients, hull_resistance, righting_moment, BoatParams, BoatState, Controls,
    FoilParams, Scenario, Sim, SimConfig,
};
use std::{fs, io::Write, path::Path, path::PathBuf};

const KN: f64 = 0.514_444;

fn csv(dir: &Path, name: &str, header: &str, rows: Vec<String>) {
    let mut f = fs::File::create(dir.join(name)).unwrap();
    writeln!(f, "{header}").unwrap();
    for r in rows {
        writeln!(f, "{r}").unwrap();
    }
}

/// The rudder as it was before the 2026-10-18 work, so figure 2 can show the change.
fn legacy_rudder() -> FoilParams {
    FoilParams {
        lift_slope: 3.5,
        stall_angle: 0.30,
        drag_zero_lift: 0.012,
        aspect_ratio: 2.5,
        efficiency: 0.7,
        plate_normal: 1.1,
        stall_sharpness: 2,
        max_lift: None,
    }
}

struct Run {
    wind_kn: f64,
    twa_deg: f64,
    sheet: f64,
    rudder: f64,
    hike: f64,
    fixed: bool,
    secs: f64,
    start: f64,
}

fn run(r: Run) -> Vec<BoatState> {
    let mut sim = Sim::new(SimConfig {
        fixed_heading: r.fixed,
        capsize_enabled: false,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: r.wind_kn * KN,
        true_wind_dir: r.twa_deg.to_radians(),
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: r.start,
        start_heel: 0.0,
    });
    (0..(r.secs / 0.02).round() as usize)
        .map(|_| {
            sim.step(Controls { rudder: r.rudder, sheet: r.sheet, hike: r.hike }).state
        })
        .collect()
}

/// Mean over the last 30 s of a 90 s locked-heading run: (speed m/s, VMG m/s toward the wind, heel rad).
fn steady(wind_kn: f64, twa: f64, sheet: f64) -> (f64, f64, f64) {
    let s = run(Run { wind_kn, twa_deg: twa, sheet, rudder: 0.0, hike: 1.0, fixed: true, secs: 90.0, start: 0.0 });
    let tail = &s[3000..];
    let n = tail.len() as f64;
    let w = twa.to_radians();
    let speed = tail.iter().map(|b| b.surge).sum::<f64>() / n;
    let (first, last) = (&tail[0], tail.last().unwrap());
    let vmg = ((last.x - first.x) * w.sin() + (last.y - first.y) * w.cos()) / 30.0;
    (speed, vmg, tail.iter().map(|b| b.heel.abs()).sum::<f64>() / n)
}

/// The sheet a sailor would choose: best VMG close-hauled, best speed otherwise.
fn best(wind_kn: f64, twa: f64) -> (f64, f64, f64) {
    let key = |r: &(f64, f64, f64)| if twa < 60.0 { r.1 } else { r.0 };
    (0..=20)
        .map(|i| steady(wind_kn, twa, i as f64 / 20.0))
        .max_by(|a, b| key(a).total_cmp(&key(b)))
        .unwrap()
}

fn main() {
    let dir = PathBuf::from(std::env::args().nth(1).expect("usage: dynamics_data <out dir>"));
    fs::create_dir_all(&dir).unwrap();
    let base = BoatParams::ilca_like();

    // Parameters the plots annotate, so the script never hard-codes a sim value.
    let rows = vec![
        format!("depower_moment_nm,{}", base.depower_moment),
        format!("depower_floor,{}", base.depower_floor),
        format!("capsize_heel_deg,{}", base.capsize_heel.to_degrees()),
        format!("vanishing_heel_deg,{}", base.vanishing_heel.to_degrees()),
        format!("rudder_stall_deg,{}", base.rudder.stall_angle.to_degrees()),
        format!("rudder_old_stall_deg,{}", legacy_rudder().stall_angle.to_degrees()),
        format!("board_stall_deg,{}", base.board.stall_angle.to_degrees()),
        format!("sail_stall_deg,{}", base.sail.stall_angle.to_degrees()),
        format!("rudder_max_deg,{}", base.rudder_max_angle.to_degrees()),
        format!("planing_exponent,{}", base.planing_resistance_exponent),
    ];
    csv(&dir, "params.csv", "name,value", rows);

    // 1: hull resistance, with the tail above 9 kn for three exponents.
    let rows = (20..=240).map(|i| {
        let kn = i as f64 / 10.0;
        let r = |p: f64| {
            let mut b = base;
            b.planing_resistance_exponent = p;
            hull_resistance(&b, kn * KN).abs()
        };
        format!("{kn},{},{},{}", r(0.0), r(0.5), r(1.0))
    });
    csv(&dir, "hull.csv", "kn,r_p0,r_p05,r_p1", rows.collect());

    // 2: foil coefficients against angle of attack (0.5 degree steps).
    let foils: [(&str, FoilParams); 4] = [
        ("rudder", base.rudder),
        ("rudder_old", legacy_rudder()),
        ("board", base.board),
        ("sail", base.sail),
    ];
    let mut rows = vec![];
    for (name, f) in foils {
        for d in 0..=180 {
            let deg = d as f64 / 2.0;
            let (cl, cd) = foil_coefficients(&f, deg.to_radians());
            rows.push(format!("{name},{deg},{cl},{cd}"));
        }
    }
    csv(&dir, "foils.csv", "foil,deg,cl,cd", rows);

    // 3: rudder sweep, no wind, 2.5 m/s start. Yaw rate on turn entry (0.2 s) and in a
    // steady turn (2 s), both heading free; speed lost with the heading locked for 5 s.
    let yaw_at = |u: f64, secs: f64| {
        run(Run { wind_kn: 0.0, twa_deg: 0.0, sheet: 0.5, rudder: u, hike: 0.0, fixed: false, secs, start: 2.5 })
            .last().unwrap().yaw_rate.to_degrees()
    };
    let rows = (0..=35).map(|d| {
        let u = d as f64 / 35.0;
        let lost = 2.5
            - run(Run { wind_kn: 0.0, twa_deg: 0.0, sheet: 0.5, rudder: u, hike: 0.0, fixed: true, secs: 5.0, start: 2.5 })
                .last().unwrap().surge;
        format!("{d},{},{},{lost}", yaw_at(u, 0.2), yaw_at(u, 2.0))
    });
    csv(&dir, "rudder_sweep.csv", "rudder_deg,yaw_entry_dps,yaw_steady_dps,speed_lost_ms", rows.collect());

    // 4: polar. 5: speed against wind strength.
    let mut rows = vec![];
    for wind in [8.0, 12.0, 16.0] {
        for twa in (30..=180).step_by(10) {
            let (s, v, h) = best(wind as f64, twa as f64);
            rows.push(format!("{wind},{twa},{s},{v},{h}"));
        }
    }
    csv(&dir, "polar.csv", "wind_kn,twa_deg,speed_ms,vmg_ms,heel_rad", rows);

    let mut rows = vec![];
    for wind in (4..=24).step_by(2) {
        let w = wind as f64;
        let (up_vmg, up_twa, up_speed) = (34..=54)
            .step_by(2)
            .map(|t| {
                let r = best(w, t as f64);
                (r.1, t, r.0)
            })
            .fold((f64::MIN, 0, 0.0), |a, b| if b.0 > a.0 { b } else { a });
        rows.push(format!("{wind},{},{up_vmg},{up_twa},{up_speed}", best(w, 90.0).0));
    }
    csv(&dir, "wind_sweep.csv", "wind_kn,reach_speed_ms,upwind_vmg_ms,upwind_twa_deg,upwind_speed_ms", rows);

    // 6: righting moment (sitting, hiking) and the equilibrium points read from close-hauled runs.
    // The sim does not export the sail's heeling moment, so the heeling side is the steady heel at
    // which it balanced the (hiking) righting moment.
    let rows = (0..=90).map(|d| {
        let h = (d as f64).to_radians();
        format!("{d},{},{}", -righting_moment(&base, h, 0.0), -righting_moment(&base, h, 1.0))
    });
    csv(&dir, "righting.csv", "heel_deg,rm_sitting,rm_hiking", rows.collect());
    let mut rows = vec![];
    for wind in [8.0, 12.0, 16.0, 20.0] {
        let (_, _, heel) = best(wind, 45.0);
        rows.push(format!("{wind},{},{}", heel.to_degrees(), -righting_moment(&base, heel, 1.0)));
    }
    csv(&dir, "equilibrium.csv", "wind_kn,heel_deg,moment_nm", rows);

    // 7: time responses, every 5th step (10 samples per second).
    let mut rows = vec![];
    let cases = [
        ("reach", 12.0, 90.0, 0.55, 0.0, true, 0.0),
        ("turn_035", 0.0, 0.0, 0.5, 0.35, false, 2.5),
        ("turn_100", 0.0, 0.0, 0.5, 1.0, false, 2.5),
    ];
    for (case, wind, twa, sheet, u, fixed, start) in cases {
        let s = run(Run { wind_kn: wind, twa_deg: twa, sheet, rudder: u, hike: 1.0, fixed, secs: 15.0, start });
        for (i, b) in s.iter().enumerate().step_by(5) {
            rows.push(format!("{case},{},{},{},{},{}", (i + 1) as f64 * 0.02, b.surge, b.heel, b.heading, b.yaw_rate));
        }
    }
    csv(&dir, "time.csv", "case,t,surge,heel,heading,yaw_rate", rows);
}
