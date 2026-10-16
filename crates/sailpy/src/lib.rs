//! Python bindings: one `Core` that owns a course run and steps it, so the
//! Gymnasium environment in `python/sailing` stays thin. The observation and
//! the reward come from `sailcourse`, the same code the browser uses.

use pyo3::prelude::*;
use rayon::prelude::*;
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
                marks: flat
                    .chunks_exact(2)
                    .map(|p| Mark { x: p[0], y: p[1] })
                    .collect(),
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
        (
            course_observation(&self.run).to_vec(),
            reward,
            terminated,
            truncated,
        )
    }

    /// `[x, y, heading, heel, time, score, next mark (-1 when finished)]`.
    fn state(&self) -> Vec<f64> {
        let s = self.run.sim().state();
        let next = self.run.progress().next_mark.map_or(-1.0, |i| i as f64);
        vec![
            s.x,
            s.y,
            s.heading,
            s.heel,
            self.time,
            self.run.score(),
            next,
        ]
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

/// Hidden units of the evolved policy: obs -> HIDDEN tanh -> ACTIONS tanh.
const HIDDEN: usize = 16;
const ACTIONS: usize = 3;

fn param_count() -> usize {
    let obs = sailcourse::observation::OBS_LEN;
    obs * HIDDEN + HIDDEN + HIDDEN * ACTIONS + ACTIONS
}

/// The policy network on one observation. Same flat layout as the Python `act`:
/// `w1` (obs x hidden), `b1`, `w2` (hidden x actions), `b2`.
fn act(params: &[f64], obs: &[f64]) -> [f64; ACTIONS] {
    let n = obs.len();
    let b1 = n * HIDDEN;
    let w2 = b1 + HIDDEN;
    let b2 = w2 + HIDDEN * ACTIONS;
    let mut hidden = [0.0f64; HIDDEN];
    for (j, h) in hidden.iter_mut().enumerate() {
        let mut sum = params[b1 + j];
        for (i, o) in obs.iter().enumerate() {
            // The Python env hands the network float32 observations.
            sum += f64::from(*o as f32) * params[i * HIDDEN + j];
        }
        *h = sum.tanh();
    }
    let mut out = [0.0f64; ACTIONS];
    for (a, o) in out.iter_mut().enumerate() {
        let mut sum = params[b2 + a];
        for (j, h) in hidden.iter().enumerate() {
            sum += h * params[w2 + j * ACTIONS + a];
        }
        *o = sum.tanh();
    }
    out
}

type Episode = (f64, &'static str, f64, f64, f64, f64, Vec<i64>);

/// Sail one episode with a network and return
/// `(score, outcome, time, rudder use, sheet range, hike use, path)`; the path is
/// `[x*10, y*10, heading*100, heel*100]` per recorded point (every `record_every`
/// decisions, and the last), empty when `record_every` is 0.
fn run_episode(
    params: &[f64],
    scenario: &(Vec<f64>, f64, f64),
    rudder_limit: f64,
    time_limit: f64,
    record_every: usize,
) -> Episode {
    let (marks, wind_speed, wind_dir) = scenario;
    let mut core = Core::new(*wind_speed, *wind_dir, true, false, 0.0, 0.0, 5, time_limit);
    let mut obs = core.reset(0, Some(marks.clone()), Some(*wind_speed), Some(*wind_dir));
    let mut path = Vec::new();
    let pack = |core: &Core, path: &mut Vec<i64>| {
        let s = core.state();
        path.push((s[0] * 10.0).round_ties_even() as i64);
        path.push((s[1] * 10.0).round_ties_even() as i64);
        path.push((s[2] * 100.0).round_ties_even() as i64);
        path.push((s[3] * 100.0).round_ties_even() as i64);
    };
    if record_every > 0 {
        pack(&core, &mut path);
    }
    let (mut rudder_sum, mut hike_sum, mut steps) = (0.0, 0.0, 0usize);
    let (mut sheet_min, mut sheet_max) = (f64::INFINITY, f64::NEG_INFINITY);
    loop {
        let a = act(params, &obs);
        let sheet = (a[1] + 1.0) / 2.0;
        let hike = (a[2] + 1.0) / 2.0;
        rudder_sum += a[0].abs();
        hike_sum += hike;
        sheet_min = sheet_min.min(sheet);
        sheet_max = sheet_max.max(sheet);
        let (next, _, terminated, truncated) = core.step(a[0] * rudder_limit, sheet, hike);
        obs = next;
        steps += 1;
        let done = terminated || truncated;
        if record_every > 0 && (steps % record_every == 0 || done) {
            pack(&core, &mut path);
        }
        if done {
            break;
        }
    }
    let s = core.state();
    let n = steps as f64;
    (
        s[5],
        core.outcome(),
        s[4],
        rudder_sum / n,
        sheet_max - sheet_min,
        hike_sum / n,
        path,
    )
}

/// Sail many episodes at once, in parallel and without Python in the step loop:
/// episode `i` is network `params[i]` on `scenarios[i]` (marks as flat x, y pairs,
/// wind speed in m/s, wind direction in degrees). `threads` of 0 means all cores.
#[pyfunction]
#[pyo3(signature = (params, scenarios, rudder_limit, time_limit, record_every=0, threads=0))]
fn evaluate_episodes(
    py: Python<'_>,
    params: Vec<Vec<f64>>,
    scenarios: Vec<(Vec<f64>, f64, f64)>,
    rudder_limit: f64,
    time_limit: f64,
    record_every: usize,
    threads: usize,
) -> PyResult<Vec<Episode>> {
    use pyo3::exceptions::PyValueError;
    if params.len() != scenarios.len() {
        return Err(PyValueError::new_err("one scenario per network is needed"));
    }
    if let Some(bad) = params.iter().find(|p| p.len() != param_count()) {
        return Err(PyValueError::new_err(format!(
            "a network needs {} weights, got {}",
            param_count(),
            bad.len()
        )));
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(py.allow_threads(|| {
        pool.install(|| {
            params
                .par_iter()
                .zip(scenarios.par_iter())
                .map(|(p, sc)| run_episode(p, sc, rudder_limit, time_limit, record_every))
                .collect()
        })
    }))
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Core>()?;
    m.add_function(wrap_pyfunction!(evaluate_episodes, m)?)?;
    Ok(())
}
