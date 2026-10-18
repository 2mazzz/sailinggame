use crate::params::FoilParams;

/// Lift and drag coefficients at angle of attack `alpha` (rad, >= 0):
/// attached-flow lift and induced drag blend into flat-plate behaviour as
/// the foil stalls.
pub fn coefficients(foil: &FoilParams, alpha: f64) -> (f64, f64) {
    let (sin, cos) = alpha.sin_cos();
    coefficients_sc(foil, alpha, sin, cos)
}

/// `coefficients` for a caller that already has `sin(alpha)` and `cos(alpha)`.
pub fn coefficients_sc(foil: &FoilParams, alpha: f64, sin: f64, cos: f64) -> (f64, f64) {
    let x = alpha / foil.stall_angle;
    let xn = x.powi(foil.stall_sharpness);
    let stalled = xn / (1.0 + xn);

    let linear = foil.lift_slope * alpha;
    let cl_attached = match foil.max_lift {
        Some(limit) => linear / (1.0 + (linear / limit).powi(8)).sqrt().sqrt().sqrt(),
        None => linear,
    };
    let cl_plate = foil.plate_normal * sin * cos;
    let cl = (1.0 - stalled) * cl_attached + stalled * cl_plate;

    let induced = cl_attached * cl_attached / (std::f64::consts::PI * foil.efficiency * foil.aspect_ratio);
    let cd_plate = foil.plate_normal * sin * sin;
    let cd = foil.drag_zero_lift + (1.0 - stalled) * induced + stalled * cd_plate;
    (cl, cd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::BoatParams;

    #[test]
    fn higher_efficiency_means_less_induced_drag() {
        let mut foil = BoatParams::ilca_like().rudder;
        foil.efficiency = 0.5;
        let (_, cd_low) = coefficients(&foil, 8.0_f64.to_radians());
        foil.efficiency = 1.0;
        let (_, cd_high) = coefficients(&foil, 8.0_f64.to_radians());
        assert!(cd_high < cd_low, "{cd_high} should be below {cd_low}");
    }
}
