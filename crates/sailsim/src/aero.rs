use crate::foil;
use crate::params::BoatParams;

const AIR_DENSITY: f64 = 1.225;

/// Wind as the boat feels it. `angle` is where the wind comes from,
/// measured from the bow, positive from starboard, in (-pi, pi].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ApparentWind {
    pub speed: f64,
    pub angle: f64,
    /// Velocity of the air relative to the boat: (forward, starboard) in m/s.
    flow: (f64, f64),
}

/// True wind blows *from* `wind_dir` (rad clockwise from north) at `wind_speed`.
pub fn apparent_wind(
    wind_speed: f64,
    wind_dir: f64,
    heading: f64,
    surge: f64,
    sway: f64,
) -> ApparentWind {
    let true_wind_angle = wind_dir - heading;
    let flow_forward = -wind_speed * true_wind_angle.cos() - surge;
    let flow_starboard = -wind_speed * true_wind_angle.sin() - sway;
    ApparentWind {
        speed: flow_forward.hypot(flow_starboard),
        angle: (-flow_starboard).atan2(-flow_forward),
        flow: (flow_forward, flow_starboard),
    }
}

/// Sail force on the boat, (forward, starboard) in N.
///
/// The boom sits at `boom_angle` from the centreline on the leeward side;
/// the angle of attack is how far the wind is off the sail's chord. A
/// negative angle of attack means the sail is luffing and makes no lift.
pub fn sail_force(params: &BoatParams, wind: &ApparentWind, sheet: f64) -> (f64, f64) {
    if wind.speed < 1e-6 {
        return (0.0, 0.0);
    }
    let sheet = sheet.clamp(0.0, 1.0);
    let boom_angle =
        params.boom_angle_eased + (params.boom_angle_in - params.boom_angle_eased) * sheet;
    let alpha = wind.angle.abs() - boom_angle;

    let (cl, cd) = if alpha <= 0.0 {
        (0.0, params.sail.drag_zero_lift)
    } else {
        foil::coefficients(&params.sail, alpha)
    };
    let q_s = 0.5 * AIR_DENSITY * wind.speed * wind.speed * params.sail_area;

    // Unit flow direction, and the unit lift direction: perpendicular to
    // the flow, leaning forward, on the leeward side of the boat.
    let (flow_f, flow_s) = (wind.flow.0 / wind.speed, wind.flow.1 / wind.speed);
    let side = wind.angle.signum();
    let lift_dir = (side * -flow_s, side * flow_f);

    (
        q_s * (cl * lift_dir.0 + cd * flow_f),
        q_s * (cl * lift_dir.1 + cd * flow_s),
    )
}
