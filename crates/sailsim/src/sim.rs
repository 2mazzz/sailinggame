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
}

#[derive(Debug, Clone, Copy)]
pub struct StepOutput {
    pub state: BoatState,
    pub status: Status,
}

pub struct Sim {
    #[allow(dead_code)]
    config: SimConfig,
    state: BoatState,
    status: Status,
}

impl Sim {
    pub fn new(config: SimConfig) -> Self {
        Self { config, state: BoatState::default(), status: Status::Sailing }
    }

    pub fn reset(&mut self, scenario: Scenario) {
        self.state = BoatState {
            x: scenario.start_x,
            y: scenario.start_y,
            heading: scenario.start_heading,
            ..BoatState::default()
        };
        self.status = Status::Sailing;
    }

    pub fn step(&mut self, _controls: Controls) -> StepOutput {
        StepOutput { state: self.state, status: self.status }
    }
}
