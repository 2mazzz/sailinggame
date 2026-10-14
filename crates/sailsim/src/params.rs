/// Physical parameters of a boat, SI units.
#[derive(Debug, Clone, Copy)]
pub struct BoatParams {
    /// Hull + crew + added mass in surge (kg).
    pub surge_mass: f64,
    /// Hull resistance in surge: R(u) = linear * |u| + quadratic * u^2 (N).
    pub resistance_linear: f64,
    pub resistance_quadratic: f64,
    /// Sail area (m^2).
    pub sail_area: f64,
    /// Boom angle from the centreline when the sheet is fully eased / fully in (rad).
    pub boom_angle_eased: f64,
    pub boom_angle_in: f64,
    /// Attached-flow lift slope (per rad), stall angle (rad), zero-lift drag,
    /// aspect ratio, and flat-plate normal-force coefficient.
    pub sail_lift_slope: f64,
    pub sail_stall_angle: f64,
    pub sail_drag_zero_lift: f64,
    pub sail_aspect_ratio: f64,
    pub sail_plate_normal: f64,
}

impl BoatParams {
    /// A single-handed dinghy in the ILCA/Laser class: ~60 kg hull plus
    /// ~80 kg sailor, plus a little added mass.
    pub fn ilca_like() -> Self {
        Self {
            surge_mass: 150.0,
            resistance_linear: 3.0,
            resistance_quadratic: 6.5,
            sail_area: 7.0,
            boom_angle_eased: 90.0_f64.to_radians(),
            boom_angle_in: 5.0_f64.to_radians(),
            sail_lift_slope: 4.5,
            sail_stall_angle: 0.35,
            sail_drag_zero_lift: 0.04,
            sail_aspect_ratio: 3.0,
            sail_plate_normal: 1.2,
        }
    }
}
