/// What the sailor can feel and see: the features an RL policy (or a HUD)
/// reads. Computed in Rust so training and the game see identical inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    /// Where the apparent wind comes from, from the bow, positive from
    /// starboard, in (-pi, pi] (rad).
    pub apparent_wind_angle: f64,
    /// Apparent wind speed (m/s).
    pub apparent_wind_speed: f64,
    /// Velocity through the water along and across the boat (m/s).
    pub surge: f64,
    pub sway: f64,
    /// Heel (rad, positive = starboard down) and its rate (rad/s).
    pub heel: f64,
    pub heel_rate: f64,
    pub yaw_rate: f64,
    /// 1 when upright, falling linearly to 0 at the capsize heel.
    pub capsize_margin: f64,
}
