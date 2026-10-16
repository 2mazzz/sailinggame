//! Python bindings: one `Core` that owns a course run and steps it, so the
//! Gymnasium environment in `python/sailing` stays thin. The observation and
//! the reward come from `sailcourse`, the same code the browser uses.

use pyo3::prelude::*;
use sailcourse::observation::course_observation;
use sailcourse::{default_course, Course, CourseRun, Mark, Outcome, DEFAULT_MARK_RADIUS};
use sailsim::{Controls, Scenario, Sim, SimConfig};

#[pyclass]
struct Core {
    run: CourseRun,
    scenario: Scenario,
    course: Course,
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
        let mut run = CourseRun::new(sim, self.course.clone());
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
            course: default_course(),
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
    /// `marks` (flat x, y pairs), `wind_speed` and `wind_dir_deg` replace the course and wind
    /// from now on; the boat starts from rest on a beam reach to whatever wind results.
    #[pyo3(signature = (seed, marks=None, wind_speed=None, wind_dir_deg=None))]
    fn reset(
        &mut self,
        seed: u64,
        marks: Option<Vec<f64>>,
        wind_speed: Option<f64>,
        wind_dir_deg: Option<f64>,
    ) -> Vec<f64> {
        if let Some(flat) = marks {
            self.course = Course {
                marks: flat.chunks_exact(2).map(|p| Mark { x: p[0], y: p[1] }).collect(),
                mark_radius: DEFAULT_MARK_RADIUS,
            };
        }
        if let Some(speed) = wind_speed {
            self.scenario.true_wind_speed = speed;
        }
        if let Some(dir) = wind_dir_deg {
            self.scenario.true_wind_dir = dir.to_radians();
            self.scenario.start_heading = (dir + 90.0).to_radians();
        }
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

    /// The scripted baseline agent's controls now: `(rudder, sheet, hike)`.
    fn baseline_controls(&self) -> (f64, f64, f64) {
        let c = sailcourse::baseline::baseline_controls(&self.run);
        (c.rudder, c.sheet, c.hike)
    }

    /// The course: marks as flat x, y pairs (m) and the mark radius (m).
    fn course(&self) -> (Vec<f64>, f64) {
        let course = self.run.course();
        (
            course.marks.iter().flat_map(|m| [m.x, m.y]).collect(),
            course.mark_radius,
        )
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
