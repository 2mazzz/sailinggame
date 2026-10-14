use crate::foil;
use crate::params::BoatParams;

const WATER_DENSITY: f64 = 1000.0;

/// Hull resistance force along the boat's surge axis (N); opposes motion.
pub fn hull_resistance(params: &BoatParams, surge: f64) -> f64 {
    -surge.signum()
        * (params.resistance_linear * surge.abs() + params.resistance_quadratic * surge * surge)
}

/// Force from the centreboard (and lateral hull area), (forward, starboard) in N.
///
/// The water flows past at the boat's velocity reversed. Lift is
/// perpendicular to it and opposes sideslip; drag opposes motion.
pub fn board_force(params: &BoatParams, surge: f64, sway: f64) -> (f64, f64) {
    let speed = surge.hypot(sway);
    if speed < 1e-6 {
        return (0.0, 0.0);
    }
    let alpha = sway.abs().atan2(surge.abs());
    let (cl, cd) = foil::coefficients(&params.board, alpha);
    let q_s = 0.5 * WATER_DENSITY * speed * speed * params.board_area;

    let lift_dir = (
        surge.signum() * sway.abs() / speed,
        -sway.signum() * surge.abs() / speed,
    );
    let drag_dir = (-surge / speed, -sway / speed);
    (
        q_s * (cl * lift_dir.0 + cd * drag_dir.0),
        q_s * (cl * lift_dir.1 + cd * drag_dir.1),
    )
}
