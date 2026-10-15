//! The agent's view of a course run, computed here so training (Python) and
//! the browser see identical inputs. Every input is scaled to roughly [-1, 1].

use crate::CourseRun;
use std::f64::consts::{PI, TAU};

pub const OBS_LEN: usize = 13;

/// In order: sin and cos of the apparent wind angle (from the bow, positive
/// from starboard), apparent wind speed / 15 m/s, surge / 6 m/s, sway / 2 m/s,
/// heel / 1.2 rad, heel rate / 2 rad/s, yaw rate / 1 rad/s, capsize margin
/// (1 upright, 0 at the capsize heel), sin and cos of the bearing error to the
/// next mark (mark bearing minus heading, negative when the mark is to port),
/// distance to the next mark / 100 m (capped at 3), and the fraction of marks
/// already reached.
pub fn course_observation(run: &CourseRun) -> [f64; OBS_LEN] {
    let obs = run.sim().observation();
    let progress = run.progress();
    let error = (progress.bearing - run.sim().state().heading + PI).rem_euclid(TAU) - PI;
    let marks = run.course().marks.len() as f64;
    let reached = progress.next_mark.map_or(marks, |i| i as f64);
    [
        obs.apparent_wind_angle.sin(),
        obs.apparent_wind_angle.cos(),
        obs.apparent_wind_speed / 15.0,
        obs.surge / 6.0,
        obs.sway / 2.0,
        obs.heel / 1.2,
        obs.heel_rate / 2.0,
        obs.yaw_rate,
        obs.capsize_margin,
        error.sin(),
        error.cos(),
        (progress.distance / 100.0).min(3.0),
        reached / marks,
    ]
}
