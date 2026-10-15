use crate::params::BoatParams;

const GRAVITY: f64 = 9.81;

/// Heel (rad) over which a hiking crew's moment builds up from zero: the
/// sailor is already out on the gunwale as soon as the boat leans.
const HIKING_BUILD_UP: f64 = 0.035;

/// Righting moment (N m) at `heel` (rad, positive = starboard down).
///
/// A hiking crew is strongest near upright (about 800 N m, Day 2017; Taylor
/// et al. 2016), not zero there as a pure sine arc would be. The righting arm
/// builds up quickly through a few degrees (the `tanh`), then falls off with
/// the cosine of the heel, reaching zero at the vanishing heel; beyond it the
/// moment pushes the boat further over.
pub fn righting_moment(params: &BoatParams, heel: f64) -> f64 {
    let fall_off = (std::f64::consts::FRAC_PI_2 * heel.abs() / params.vanishing_heel).cos();
    let gz = params.righting_arm_max * (heel / HIKING_BUILD_UP).tanh() * fall_off;
    -params.displacement_mass * GRAVITY * gz
}
