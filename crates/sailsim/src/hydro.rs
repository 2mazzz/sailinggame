use crate::params::BoatParams;

/// Hull resistance force along the boat's surge axis (N); opposes motion.
pub fn hull_resistance(params: &BoatParams, surge: f64) -> f64 {
    -surge.signum()
        * (params.resistance_linear * surge.abs() + params.resistance_quadratic * surge * surge)
}
