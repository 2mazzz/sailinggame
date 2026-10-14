use crate::aero::{apparent_wind, sail_force};
use crate::hydro::{board_force, hull_resistance};
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
    wind_speed: f64,
    wind_dir: f64,
    state: BoatState,
    status: Status,
}

impl Sim {
    pub fn new(config: SimConfig) -> Self {
        Self {
            config,
            params: BoatParams::ilca_like(),
            wind_speed: 0.0,
            wind_dir: 0.0,
            state: BoatState::default(),
            status: Status::Sailing,
        }
    }

    pub fn reset(&mut self, scenario: Scenario) {
        self.wind_speed = scenario.true_wind_speed;
        self.wind_dir = scenario.true_wind_dir;
        self.state = BoatState {
            x: scenario.start_x,
            y: scenario.start_y,
            heading: scenario.start_heading,
            surge: scenario.start_speed,
            ..BoatState::default()
        };
        self.status = Status::Sailing;
    }

    pub fn step(&mut self, controls: Controls) -> StepOutput {
        let s = &self.state;
        let y = [s.x, s.y, s.heading, s.surge, s.sway];
        let params = self.params;
        let (wind_speed, wind_dir, sheet) = (self.wind_speed, self.wind_dir, controls.sheet);
        let y = rk4(y, self.config.dt, |y| {
            let [_, _, heading, surge, sway] = *y;
            let wind = apparent_wind(wind_speed, wind_dir, heading, surge, sway);
            let (sail_surge, sail_sway) = sail_force(&params, &wind, sheet);
            let (board_surge, board_sway) = board_force(&params, surge, sway);
            [
                surge * heading.sin() + sway * heading.cos(),
                surge * heading.cos() - sway * heading.sin(),
                0.0,
                (sail_surge + board_surge + hull_resistance(&params, surge)) / params.surge_mass,
                (sail_sway + board_sway) / params.sway_mass,
            ]
        });
        self.state.x = y[0];
        self.state.y = y[1];
        self.state.heading = y[2];
        self.state.surge = y[3];
        self.state.sway = y[4];
        StepOutput { state: self.state, status: self.status }
    }
}
