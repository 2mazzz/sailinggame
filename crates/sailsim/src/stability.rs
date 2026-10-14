use crate::params::BoatParams;

const GRAVITY: f64 = 9.81;

/// Righting moment (N m) at `heel` (rad, positive = starboard down).
///
/// The righting arm GZ follows a sine arc: it peaks at half the vanishing
/// heel and is zero at the vanishing heel; beyond it the moment pushes the
/// boat further over.
pub fn righting_moment(params: &BoatParams, heel: f64) -> f64 {
    let gz = params.righting_arm_max * (std::f64::consts::PI * heel / params.vanishing_heel).sin();
    -params.displacement_mass * GRAVITY * gz
}
