mod aero;
mod foil;
mod gust;
mod hydro;
mod integrator;
mod observation;
mod params;
mod sim;
mod stability;
mod state;

/// Exported for the analysis example and plots, not for the game loop.
pub use foil::coefficients as foil_coefficients;
pub use hydro::hull_resistance;
pub use observation::Observation;
pub use params::{BoatParams, FoilParams};
pub use stability::righting_moment;
pub use sim::{Scenario, Sim, SimConfig, StepOutput};
pub use state::{BoatState, Controls, Status};
