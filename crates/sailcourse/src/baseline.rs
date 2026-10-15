//! A scripted agent to compare learned policies against and to show the score
//! on one example run: steer at the next mark, trim the sheet to the apparent
//! wind, hike when heeled. It does not tack, so it only suits legs it can
//! sail directly.

use crate::CourseRun;
use sailsim::Controls;
use std::f64::consts::{PI, TAU};

/// Boom angle (rad) the sheet maps to at 0 (eased) and 1 (hard in); see `BoatParams`.
const BOOM_EASED: f64 = 90.0 * PI / 180.0;
const BOOM_IN: f64 = 21.0 * PI / 180.0;
/// Heel (rad) above which the sailor leans out.
const HIKE_HEEL: f64 = 12.0 * PI / 180.0;
const STEER_GAIN: f64 = 2.0;

pub fn baseline_controls(run: &CourseRun) -> Controls {
    let sim = run.sim();
    let obs = sim.observation();
    let progress = run.progress();
    let error = (progress.bearing - sim.state().heading + PI).rem_euclid(TAU) - PI;
    // Boom at about half the apparent wind angle, within what the sheet can reach.
    let boom = (obs.apparent_wind_angle.abs() / 2.0).clamp(BOOM_IN, BOOM_EASED);
    Controls {
        rudder: (STEER_GAIN * error).clamp(-1.0, 1.0),
        sheet: (BOOM_EASED - boom) / (BOOM_EASED - BOOM_IN),
        hike: if obs.heel.abs() > HIKE_HEEL { 1.0 } else { 0.0 },
    }
}
