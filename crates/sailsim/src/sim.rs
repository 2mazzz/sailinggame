use crate::aero::{apparent_wind, sail_force};
use crate::gust::Gusts;
use crate::hydro::{board_force, hull_resistance, rudder_force};
use crate::integrator::rk4;
use crate::observation::Observation;
use crate::params::BoatParams;
use crate::stability::righting_moment;
use crate::state::{BoatState, Controls, Status};

#[derive(Debug, Clone, Copy)]
pub struct SimConfig {
    pub dt: f64,
    pub seed: u64,
    pub capsize_enabled: bool,
    /// Lock the heading (no yaw): for tests and for curricula that learn sail trim first.
    pub fixed_heading: bool,
    /// Gust strength as a fraction of the mean wind: the wind stays within
    /// `mean * (1 +/- gust_intensity)`. Zero (the default) gives steady wind.
    /// Gusts are deterministic given `seed`.
    pub gust_intensity: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            dt: 0.02,
            seed: 0,
            capsize_enabled: true,
            fixed_heading: false,
            gust_intensity: 0.0,
        }
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
    /// Initial heel (rad, positive = starboard side down).
    pub start_heel: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct StepOutput {
    pub state: BoatState,
    pub status: Status,
}

pub struct Sim {
    config: SimConfig,
    params: BoatParams,
    /// Mean true wind speed; gusts vary around it.
    wind_speed: f64,
    wind_dir: f64,
    gusts: Gusts,
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
            gusts: Gusts::new(config.seed),
            state: BoatState::default(),
            status: Status::Sailing,
        }
    }

    pub fn reset(&mut self, scenario: Scenario) {
        self.wind_speed = scenario.true_wind_speed;
        self.wind_dir = scenario.true_wind_dir;
        self.gusts = Gusts::new(self.config.seed);
        self.state = BoatState {
            x: scenario.start_x,
            y: scenario.start_y,
            heading: scenario.start_heading,
            surge: scenario.start_speed,
            heel: scenario.start_heel,
            ..BoatState::default()
        };
        self.status = Status::Sailing;
    }

    /// Change the true wind while running. `dir` is where it blows from
    /// (radians clockwise from north).
    pub fn set_wind(&mut self, speed: f64, dir: f64) {
        self.wind_speed = speed;
        self.wind_dir = dir;
    }

    /// Fixed timestep (s) per `step`.
    pub fn dt(&self) -> f64 {
        self.config.dt
    }

    /// Live true wind speed (m/s), including the current gust.
    pub fn wind_speed(&self) -> f64 {
        self.wind_speed * (1.0 + self.config.gust_intensity * self.gusts.value())
    }

    pub fn state(&self) -> BoatState {
        self.state
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn observation(&self) -> Observation {
        let s = &self.state;
        let wind = apparent_wind(self.wind_speed(), self.wind_dir, s.heading, s.surge, s.sway);
        Observation {
            apparent_wind_angle: wind.angle,
            apparent_wind_speed: wind.speed,
            surge: s.surge,
            sway: s.sway,
            heel: s.heel,
            heel_rate: s.heel_rate,
            yaw_rate: s.yaw_rate,
            capsize_margin: (1.0 - s.heel.abs() / self.params.capsize_heel).clamp(0.0, 1.0),
        }
    }

    pub fn step(&mut self, controls: Controls) -> StepOutput {
        if self.status == Status::Capsized {
            return StepOutput {
                state: self.state,
                status: self.status,
            };
        }
        self.gusts.advance(self.config.dt);
        let wind_speed = self.wind_speed();
        let s = &self.state;
        let y = [
            s.x,
            s.y,
            s.heading,
            s.surge,
            s.sway,
            s.heel,
            s.heel_rate,
            s.yaw_rate,
        ];
        let params = self.params;
        let (wind_dir, sheet) = (self.wind_dir, controls.sheet);
        let (rudder, fixed_heading) = (controls.rudder, self.config.fixed_heading);
        let y = rk4(y, self.config.dt, |y| {
            let [_, _, heading, surge, sway, heel, heel_rate, yaw_rate] = *y;
            let wind = apparent_wind(wind_speed, wind_dir, heading, surge, sway);
            // A heeled sail presents less area to the wind.
            let (sail_surge, sail_sway) = sail_force(&params, &wind, sheet);
            let power = heel.cos().max(0.0);
            let (sail_surge, sail_sway) = (sail_surge * power, sail_sway * power);
            let (board_surge, board_sway) = board_force(&params, surge, sway);
            let (rudder_surge, rudder_sway, rudder_yaw) =
                rudder_force(&params, rudder, surge, sway, yaw_rate);
            let yaw_moment = rudder_yaw + params.board_arm * board_sway
                - params.sail_ce_aft * sail_sway
                - params.yaw_damping_linear * yaw_rate
                - params.yaw_damping_quadratic * yaw_rate.abs() * yaw_rate;
            [
                surge * heading.sin() + sway * heading.cos(),
                surge * heading.cos() - sway * heading.sin(),
                if fixed_heading { 0.0 } else { yaw_rate },
                (sail_surge
                    + board_surge
                    + rudder_surge
                    + hull_resistance(&params, surge)
                    + params.sway_mass * sway * yaw_rate)
                    / params.surge_mass,
                (sail_sway + board_sway + rudder_sway - params.surge_mass * surge * yaw_rate)
                    / params.sway_mass,
                heel_rate,
                (sail_sway * params.sail_heel_arm + righting_moment(&params, heel)
                    - params.heel_damping * heel_rate)
                    / params.heel_inertia,
                if fixed_heading {
                    0.0
                } else {
                    yaw_moment / params.yaw_inertia
                },
            ]
        });
        self.state.x = y[0];
        self.state.y = y[1];
        self.state.heading = y[2];
        self.state.surge = y[3];
        self.state.sway = y[4];
        self.state.heel = y[5];
        self.state.heel_rate = y[6];
        self.state.yaw_rate = y[7];
        if self.config.capsize_enabled && self.state.heel.abs() >= self.params.capsize_heel {
            self.status = Status::Capsized;
        }
        StepOutput {
            state: self.state,
            status: self.status,
        }
    }
}
