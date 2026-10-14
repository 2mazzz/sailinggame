use sailsim::{Controls, Scenario, Sim, SimConfig, Status};
use wasm_bindgen::prelude::*;

/// Fields of the snapshot returned by `Game::snapshot`, in order.
/// x, y (m), heading, heel (rad), surge, sway (m/s), yaw_rate (rad/s),
/// apparent wind angle (rad), apparent wind speed (m/s), capsized (0/1), time (s),
/// live true wind speed incl. gusts (m/s).
const SNAPSHOT_LEN: usize = 12;

/// Gust strength (fraction of the mean wind) the viewer starts with.
const DEFAULT_GUST_INTENSITY: f64 = 0.3;

/// The browser's handle on the simulation: owns a `Sim` and runs it at its
/// fixed step, however long the frame was.
#[wasm_bindgen]
pub struct Game {
    sim: Sim,
    scenario: Scenario,
    dt: f64,
    time: f64,
    pending: f64,
    capsize_enabled: bool,
    gust_intensity: f64,
}

impl Game {
    fn rebuild(&mut self) {
        self.sim = Sim::new(SimConfig {
            capsize_enabled: self.capsize_enabled,
            gust_intensity: self.gust_intensity,
            seed: 1,
            ..SimConfig::default()
        });
        self.sim.reset(self.scenario);
        self.sim
            .set_wind(self.scenario.true_wind_speed, self.scenario.true_wind_dir);
        self.time = 0.0;
        self.pending = 0.0;
    }
}

#[wasm_bindgen]
impl Game {
    /// Wind direction is in degrees clockwise from north, where it blows from.
    #[wasm_bindgen(constructor)]
    pub fn new(wind_speed: f64, wind_dir_deg: f64) -> Game {
        let config = SimConfig {
            gust_intensity: DEFAULT_GUST_INTENSITY,
            seed: 1,
            ..SimConfig::default()
        };
        let scenario = Scenario {
            true_wind_speed: wind_speed,
            true_wind_dir: wind_dir_deg.to_radians(),
            start_x: 0.0,
            start_y: 0.0,
            // Start from rest on a beam reach, where the boat accelerates by
            // itself; close-hauled needs steering and trim to get going.
            start_heading: (wind_dir_deg + 90.0).to_radians(),
            start_speed: 0.0,
            start_heel: 0.0,
        };
        let mut sim = Sim::new(config);
        sim.reset(scenario);
        Game {
            sim,
            scenario,
            dt: config.dt,
            time: 0.0,
            pending: 0.0,
            capsize_enabled: true,
            gust_intensity: DEFAULT_GUST_INTENSITY,
        }
    }

    pub fn reset(&mut self) {
        self.sim.reset(self.scenario);
        self.time = 0.0;
        self.pending = 0.0;
    }

    /// Capsize ends the run when on; off lets you learn trim without flipping.
    pub fn set_capsize_enabled(&mut self, enabled: bool) {
        self.capsize_enabled = enabled;
        self.rebuild();
    }

    /// Gust strength as a fraction of the mean wind (0 = steady); restarts the run.
    pub fn set_gust_intensity(&mut self, intensity: f64) {
        self.gust_intensity = intensity.clamp(0.0, 0.9);
        self.rebuild();
    }

    pub fn gust_intensity(&self) -> f64 {
        self.gust_intensity
    }

    pub fn capsize_enabled(&self) -> bool {
        self.capsize_enabled
    }

    /// Advance by `elapsed` real seconds (clamped so a stalled tab cannot
    /// trigger a spiral of catch-up steps) with the controls held constant.
    pub fn advance(&mut self, elapsed: f64, rudder: f64, sheet: f64) {
        self.pending += elapsed.clamp(0.0, 0.1);
        let controls = Controls {
            rudder: rudder.clamp(-1.0, 1.0),
            sheet: sheet.clamp(0.0, 1.0),
        };
        while self.pending >= self.dt {
            self.pending -= self.dt;
            let out = self.sim.step(controls);
            if out.status == Status::Capsized {
                self.pending = 0.0;
                break;
            }
            self.time += self.dt;
        }
    }

    pub fn snapshot_len() -> usize {
        SNAPSHOT_LEN
    }

    pub fn snapshot(&self) -> Vec<f64> {
        let (state, status) = (self.sim.state(), self.sim.status());
        let obs = self.sim.observation();
        vec![
            state.x,
            state.y,
            state.heading,
            state.heel,
            state.surge,
            state.sway,
            state.yaw_rate,
            obs.apparent_wind_angle,
            obs.apparent_wind_speed,
            if status == Status::Capsized { 1.0 } else { 0.0 },
            self.time,
            self.sim.wind_speed(),
        ]
    }

    /// Change the true wind live: speed in m/s, direction in degrees
    /// clockwise from north (where it blows from).
    pub fn set_wind(&mut self, speed: f64, dir_deg: f64) {
        self.scenario.true_wind_speed = speed.max(0.0);
        self.scenario.true_wind_dir = dir_deg.to_radians();
        self.sim
            .set_wind(self.scenario.true_wind_speed, self.scenario.true_wind_dir);
    }

    pub fn wind_speed(&self) -> f64 {
        self.scenario.true_wind_speed
    }

    /// Degrees clockwise from north, where the wind blows from.
    pub fn wind_dir_deg(&self) -> f64 {
        self.scenario.true_wind_dir.to_degrees()
    }
}
