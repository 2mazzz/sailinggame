use crate::foil;
use crate::params::BoatParams;

const WATER_DENSITY: f64 = 1000.0;

/// Hull resistance force along the boat's surge axis (N); opposes motion.
pub fn hull_resistance(params: &BoatParams, surge: f64) -> f64 {
    let speed = surge.abs();
    let table = &params.hull_drag_area;
    let drag_area = match table.iter().position(|&(v, _)| speed < v) {
        None => table[table.len() - 1].1,
        Some(0) => table[0].1,
        Some(i) => {
            let ((v0, a0), (v1, a1)) = (table[i - 1], table[i]);
            a0 + (a1 - a0) * (speed - v0) / (v1 - v0)
        }
    };
    -surge.signum() * 0.5 * WATER_DENSITY * drag_area * speed * speed
}

/// Force from the centreboard (and lateral hull area), (forward, starboard) in N.
///
/// The water flows past at the boat's velocity reversed. Lift is
/// perpendicular to it and opposes sideslip; drag opposes motion.
pub fn board_force(params: &BoatParams, surge: f64, sway: f64) -> (f64, f64) {
    let speed = (surge * surge + sway * sway).sqrt();
    if speed < 1e-6 {
        return (0.0, 0.0);
    }
    // Viscous hull/board side force that matters only when there is no
    // forward flow over the board (a stalled foil has little lift of its own).
    let flow_ratio = surge / params.low_speed_lateral_speed;
    let low_speed_sway = -params.low_speed_lateral_damping * sway / (1.0 + flow_ratio * flow_ratio);
    let alpha = sway.abs().atan2(surge.abs());
    // sin and cos of the angle of attack are just the flow components over its speed.
    let (cl, cd) = foil::coefficients_sc(
        &params.board,
        alpha,
        sway.abs() / speed,
        surge.abs() / speed,
    );
    let q_s = 0.5 * WATER_DENSITY * speed * speed * params.board_area;

    let lift_dir = (
        surge.signum() * sway.abs() / speed,
        -sway.signum() * surge.abs() / speed,
    );
    let drag_dir = (-surge / speed, -sway / speed);
    (
        q_s * (cl * lift_dir.0 + cd * drag_dir.0),
        q_s * (cl * lift_dir.1 + cd * drag_dir.1) + low_speed_sway,
    )
}

/// Force and moment from the rudder: (forward N, starboard N, yaw N m).
///
/// `rudder` in [-1, 1]; positive turns the bow to starboard. The blade sees
/// the water at the stern, which moves sideways when the boat sways or yaws.
pub fn rudder_force(
    params: &BoatParams,
    rudder: f64,
    surge: f64,
    sway: f64,
    yaw_rate: f64,
) -> (f64, f64, f64) {
    let stern_sway = sway - yaw_rate * params.rudder_arm;
    let speed = (surge * surge + stern_sway * stern_sway).sqrt();
    if speed < 1e-6 {
        return (0.0, 0.0, 0.0);
    }
    let blade_angle = rudder.clamp(-1.0, 1.0) * params.rudder_max_angle * surge.signum();
    let alpha = blade_angle + stern_sway.atan2(surge.abs());
    let (cl, cd) = foil::coefficients(&params.rudder, alpha.abs());
    let inflow = speed * params.rudder_inflow_factor;
    let q_s = 0.5 * WATER_DENSITY * inflow * inflow * params.rudder_area;

    let lateral = -alpha.signum() * cl * q_s;
    let drag = -surge.signum() * cd * q_s;
    // The rudder sits aft of the centre of gravity: moment = x * F_starboard.
    // Sculling: at low speed the blade barely bites, so the sailor works the
    // tiller to turn the boat. Zero at rest, peaking near `rudder_scull_speed`.
    let x = surge.abs() / params.rudder_scull_speed;
    let scull =
        params.rudder_scull_moment * rudder.clamp(-1.0, 1.0) * surge.signum() * x / (1.0 + x * x);
    (drag, lateral, -params.rudder_arm * lateral + scull)
}
