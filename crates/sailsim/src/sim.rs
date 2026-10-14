use crate::hydro::hull_resistance;
use crate::integrator::rk4;
use crate::params::BoatParams;
use crate::state::{BoatState, Controls, Status};

#[derive(Debug, Clone, Copy)]
pub struct SimConfig {
    pub dt: f64,
    pub seed: u64,
    pub capsize_enabled: bool,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self { dt: 0.02, seed: 0, capsize_enabled: true }
    }
}

/// Wind blows *from* `true_wind_dir` (radians clockwise from north).
#[derive(Debug, Clone, Copy)]
pub struct Scenario {
    pub true_wind_speed: f64,
    pub true_wind_dir: f64,
    pub start_x: f64,
    pub start_y: f64,
    pub start_heading: f64,
    /// Initial surge speed along the heading (m/s).
    pub start_speed: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct StepOutput {
    pub state: BoatState,
    pub status: Status,
}

pub struct Sim {
    config: SimConfig,
    params: BoatParams,
    state: BoatState,
    status: Status,
}

impl Sim {
    pub fn new(config: SimConfig) -> Self {
        Self {
            config,
            params: BoatParams::ilca_like(),
            state: BoatState::default(),
            status: Status::Sailing,
        }
    }

    pub fn reset(&mut self, scenario: Scenario) {
        self.state = BoatState {
            x: scenario.start_x,
            y: scenario.start_y,
            heading: scenario.start_heading,
            surge: scenario.start_speed,
            ..BoatState::default()
        };
        self.status = Status::Sailing;
    }

    pub fn step(&mut self, _controls: Controls) -> StepOutput {
        let s = &self.state;
        let y = [s.x, s.y, s.heading, s.surge];
        let params = self.params;
        let y = rk4(y, self.config.dt, |y| {
            let [_, _, heading, surge] = *y;
            [
                surge * heading.sin(),
                surge * heading.cos(),
                0.0,
                hull_resistance(&params, surge) / params.surge_mass,
            ]
        });
        self.state.x = y[0];
        self.state.y = y[1];
        self.state.heading = y[2];
        self.state.surge = y[3];
        StepOutput { state: self.state, status: self.status }
    }
}
