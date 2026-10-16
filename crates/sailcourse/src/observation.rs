//! The agent's view of a course run, computed here so training (Python) and
//! the browser see identical inputs. Every input is scaled to roughly [-1, 1].

use crate::CourseRun;
use std::f64::consts::{PI, TAU};

pub const OBS_LEN: usize = 14;

/// In order: sin and cos of the apparent wind angle (from the bow, positive
/// from starboard), apparent wind speed / 15 m/s, surge / 6 m/s, sway / 2 m/s,
/// heel / 1.2 rad, heel rate / 2 rad/s, yaw rate / 1 rad/s, capsize margin
/// (1 upright, 0 at the capsize heel), sin and cos of the bearing error to the
/// next mark (mark bearing minus heading, negative when the mark is to port),
/// distance to the next mark / 100 m (capped at 3), and sin and cos of the
/// bearing error to the mark after that, so a rounding can be set up for the
/// next leg (the next mark again on the last leg). Nothing identifies which
/// mark it is, so the agent cannot memorise a course as a schedule of legs.
pub fn course_observation(run: &CourseRun) -> [f64; OBS_LEN] {
    let obs = run.sim().observation();
    let progress = run.progress();
    let error = (progress.bearing - run.sim().state().heading + PI).rem_euclid(TAU) - PI;
    let state = run.sim().state();
    let after = progress.next_mark.map_or(0.0, |i| {
        let marks = &run.course().marks;
        let mark = marks.get(i + 1).unwrap_or(&marks[i]);
        let bearing = (mark.x - state.x).atan2(mark.y - state.y);
        (bearing - state.heading + PI).rem_euclid(TAU) - PI
    });
    let has_after = progress.next_mark.is_some();
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
        if has_after { after.sin() } else { 0.0 },
        if has_after { after.cos() } else { 0.0 },
    ]
}
