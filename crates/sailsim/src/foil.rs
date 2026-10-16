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

    let induced = cl_attached * cl_attached / (std::f64::consts::PI * 0.7 * foil.aspect_ratio);
    let cd_plate = foil.plate_normal * sin * sin;
    let cd = foil.drag_zero_lift + (1.0 - stalled) * induced + stalled * cd_plate;
    (cl, cd)
}
