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

    fn rudder_cl(deg: f64) -> f64 {
        coefficients(&BoatParams::ilca_like().rudder, deg.to_radians()).0
    }

    #[test]
    fn rudder_lift_peaks_near_the_section_stall_angle_then_drops() {
        let peak_deg = (0..=40)
            .max_by(|a, b| rudder_cl(*a as f64).total_cmp(&rudder_cl(*b as f64)))
            .unwrap();
        let peak = rudder_cl(peak_deg as f64);
        // NACA 0012 stalls at 12 to 14 degrees (NACA TN 3361, Sandia SAND80-2114).
        assert!((10..=15).contains(&peak_deg), "peak at {peak_deg} degrees");
        // A finite-aspect-ratio blade tops out between 0.6 and 0.95.
        assert!((0.6..=0.95).contains(&peak), "peak CL {peak}");
        // Fully stalled by 25 degrees: well below the peak.
        assert!(rudder_cl(25.0) < 0.75 * peak, "CL at 25 deg {}", rudder_cl(25.0));
    }

    #[test]
    fn rudder_lift_slope_matches_day_2017() {
        // dCL/dalpha = 5.7 AR_E / (1.8 + sqrt(AR_E^2 + 4)), AR_E = 2 b / c.
        let ar_e: f64 = 4.5;
        let expected = 5.7 * ar_e / (1.8 + (ar_e * ar_e + 4.0).sqrt()); // 3.81
        let slope = BoatParams::ilca_like().rudder.lift_slope;
        assert!((slope - expected).abs() < 0.1, "{slope} vs {expected}");
    }

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
