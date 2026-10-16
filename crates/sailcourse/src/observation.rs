//! The agent's view of a course run, computed here so training (Python) and
//! the browser see identical inputs. Every input is scaled to roughly [-1, 1].

use crate::CourseRun;
use std::f64::consts::{PI, TAU};

/// Marks the observation shows: the next one and the ones after it, so with an
/// 8-mark course the boat sees the whole rest of the course. Courses are
/// expected to have this many marks; extra marks beyond the next `MARK_SLOTS`
/// are not shown and missing ones read as zeros.
pub const MARK_SLOTS: usize = 8;
/// Boat and wind inputs, before the marks.
const BOAT_INPUTS: usize = 9;
pub const OBS_LEN: usize = BOAT_INPUTS + 3 * MARK_SLOTS + 1;

/// In order: sin and cos of the apparent wind angle (from the bow, positive
/// from starboard), apparent wind speed / 15 m/s, surge / 6 m/s, sway / 2 m/s,
/// heel / 1.2 rad, heel rate / 2 rad/s, yaw rate / 1 rad/s, capsize margin
/// (1 upright, 0 at the capsize heel). Then, for each of the `MARK_SLOTS`
/// marks still to be rounded, nearest in the order of the course first (slot 0
/// is the next mark): sin and cos of the bearing error to it (mark bearing
/// minus heading, negative when the mark is to port) and its distance / 100 m
/// (capped at 3); all zeros once the course has no mark for the slot. The last
/// input is the share of the slots still to sail (marks left / `MARK_SLOTS`).
/// Marks are seen in the order they are rounded and nothing identifies which
/// of the course's marks one is, so the agent cannot memorise a course as a
/// schedule of legs.
pub fn course_observation(run: &CourseRun) -> [f64; OBS_LEN] {
    let obs = run.sim().observation();
    let state = run.sim().state();
    let mut out = [0.0; OBS_LEN];
    out[..BOAT_INPUTS].copy_from_slice(&[
        obs.apparent_wind_angle.sin(),
        obs.apparent_wind_angle.cos(),
        obs.apparent_wind_speed / 15.0,
        obs.surge / 6.0,
        obs.sway / 2.0,
        obs.heel / 1.2,
        obs.heel_rate / 2.0,
        obs.yaw_rate,
        obs.capsize_margin,
    ]);
    let next = run.progress().next_mark;
    let marks = &run.course().marks;
    let mut left = 0;
    if let Some(first) = next {
        for (slot, mark) in marks.iter().skip(first).take(MARK_SLOTS).enumerate() {
            let (dx, dy) = (mark.x - state.x, mark.y - state.y);
            let error = (dx.atan2(dy) - state.heading + PI).rem_euclid(TAU) - PI;
            let at = BOAT_INPUTS + 3 * slot;
            out[at] = error.sin();
            out[at + 1] = error.cos();
            out[at + 2] = (dx.hypot(dy) / 100.0).min(3.0);
            left += 1;
        }
    }
    out[OBS_LEN - 1] = left as f64 / MARK_SLOTS as f64;
    out
}
