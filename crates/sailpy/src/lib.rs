//! Python bindings: one `Core` that owns a course run and steps it, so the
//! Gymnasium environment in `python/sailing` stays thin. The observation and
//! the reward come from `sailcourse`, the same code the browser uses.

use pyo3::prelude::*;
use sailcourse::observation::course_observation;
use sailcourse::{default_course, CourseRun, Outcome};
use sailsim::{Controls, Scenario, Sim, SimConfig};

#[pyclass]
struct Core {
    run: CourseRun,
    scenario: Scenario,
    capsize_enabled: bool,
    fixed_heading: bool,
    gust: f64,
    shift: f64,
    frame_skip: usize,
    time_limit: f64,
    dt: f64,
    time: f64,
}

impl Core {
    fn build(&self, seed: u64) -> CourseRun {
        let sim = Sim::new(SimConfig {
            seed,
            capsize_enabled: self.capsize_enabled,
            fixed_heading: self.fixed_heading,
            gust_intensity: self.gust,
            shift_intensity: self.shift,
            ..SimConfig::default()
        });
        let mut run = CourseRun::new(sim, default_course());
        run.reset(self.scenario);
        run
    }

    fn timed_out(&self) -> bool {
        self.run.outcome() == Outcome::InProgress && self.time >= self.time_limit
    }
}

#[pymethods]
impl Core {
    #[new]
    #[pyo3(signature = (wind_speed, wind_dir_deg, capsize_enabled, fixed_heading, gust, shift_deg, frame_skip, time_limit))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        wind_speed: f64,
        wind_dir_deg: f64,
        capsize_enabled: bool,
        fixed_heading: bool,
        gust: f64,
        shift_deg: f64,
        frame_skip: usize,
        time_limit: f64,
    ) -> Self {
        let scenario = Scenario {
            true_wind_speed: wind_speed,
            true_wind_dir: wind_dir_deg.to_radians(),
            start_x: 0.0,
            start_y: 0.0,
            // From rest on a beam reach, as in the viewer.
            start_heading: (wind_dir_deg + 90.0).to_radians(),
            start_speed: 0.0,
            start_heel: 0.0,
        };
        let mut core = Core {
            run: CourseRun::new(Sim::new(SimConfig::default()), default_course()),
            scenario,
            capsize_enabled,
            fixed_heading,
            gust,
            shift: shift_deg.to_radians(),
            frame_skip: frame_skip.max(1),
            time_limit,
            dt: SimConfig::default().dt,
            time: 0.0,
        };
        core.run = core.build(0);
        core
    }

    /// Start a new episode with this seed (it drives gusts and shifts); returns the observation.
    fn reset(&mut self, seed: u64) -> Vec<f64> {
        self.run = self.build(seed);
        self.time = 0.0;
        course_observation(&self.run).to_vec()
    }

    /// Apply the controls for `frame_skip` sim steps. Returns
    /// `(observation, reward, terminated, truncated)`.
    fn step(&mut self, rudder: f64, sheet: f64, hike: f64) -> (Vec<f64>, f64, bool, bool) {
        let controls = Controls {
            rudder: rudder.clamp(-1.0, 1.0),
            sheet: sheet.clamp(0.0, 1.0),
            hike: hike.clamp(0.0, 1.0),
        };
        let mut reward = 0.0;
        for _ in 0..self.frame_skip {
            if self.run.outcome() != Outcome::InProgress {
                break;
            }
            self.run.step(controls);
            self.time += self.dt;
            reward += self.run.last_reward();
        }
        let terminated = self.run.outcome() != Outcome::InProgress;
        let truncated = !terminated && self.timed_out();
        (course_observation(&self.run).to_vec(), reward, terminated, truncated)
    }

    /// `[x, y, heading, heel, time, score, next mark (-1 when finished)]`.
    fn state(&self) -> Vec<f64> {
        let s = self.run.sim().state();
        let next = self.run.progress().next_mark.map_or(-1.0, |i| i as f64);
        vec![s.x, s.y, s.heading, s.heel, self.time, self.run.score(), next]
    }

    fn outcome(&self) -> &'static str {
        match self.run.outcome() {
            Outcome::Finished { .. } => "finished",
            Outcome::Capsized { .. } => "capsized",
            Outcome::InProgress if self.timed_out() => "time_limit",
            Outcome::InProgress => "running",
        }
    }

    #[staticmethod]
    fn obs_len() -> usize {
        sailcourse::observation::OBS_LEN
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Core>()?;
    Ok(())
}
