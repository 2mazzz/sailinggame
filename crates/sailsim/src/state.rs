/// Boat state in the world frame: x east, y north (metres); heading in
/// radians clockwise from north; surge/sway in the boat frame (m/s).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoatState {
    pub x: f64,
    pub y: f64,
    pub heading: f64,
    pub surge: f64,
    pub sway: f64,
    pub yaw_rate: f64,
    pub heel: f64,
    pub heel_rate: f64,
}

/// Normalised controls: rudder in [-1, 1], sheet in [0, 1].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Controls {
    pub rudder: f64,
    pub sheet: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Sailing,
    Capsized,
}
