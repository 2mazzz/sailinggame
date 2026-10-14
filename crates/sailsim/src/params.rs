/// Physical parameters of a boat, SI units.
#[derive(Debug, Clone, Copy)]
pub struct BoatParams {
    /// Hull + crew + added mass in surge (kg).
    pub surge_mass: f64,
    /// Hull resistance in surge: R(u) = linear * |u| + quadratic * u^2 (N).
    pub resistance_linear: f64,
    pub resistance_quadratic: f64,
}

impl BoatParams {
    /// A single-handed dinghy in the ILCA/Laser class: ~60 kg hull plus
    /// ~80 kg sailor, plus a little added mass.
    pub fn ilca_like() -> Self {
        Self {
            surge_mass: 150.0,
            resistance_linear: 3.0,
            resistance_quadratic: 5.0,
        }
    }
}
