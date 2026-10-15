use sailcourse::{Course, CourseRun, Mark, Outcome};
use sailsim::{Controls, Scenario, Sim, SimConfig, Status};
use wasm_bindgen::prelude::*;

/// Fields of the snapshot returned by `Game::snapshot`, in order.
/// x, y (m), heading, heel (rad), surge, sway (m/s), yaw_rate (rad/s),
/// apparent wind angle (rad), apparent wind speed (m/s), capsized (0/1), time (s),
/// live true wind speed incl. gusts (m/s), live true wind direction incl. shifts (deg).
const SNAPSHOT_LEN: usize = 13;

/// Gust strength (fraction of the mean wind) the viewer starts with.
const DEFAULT_GUST_INTENSITY: f64 = 0.0;
/// Wind shift range (degrees either side of the mean) the viewer starts with.
const DEFAULT_SHIFT_DEG: f64 = 10.0;

/// A mark is reached within this many metres.
const MARK_RADIUS: f64 = 10.0;

/// Default course (x, y pairs, metres; x east, y north): upwind first for the
/// default northerly wind, then a reach and a run back past the start.
const DEFAULT_MARKS: [(f64, f64); 4] = [(0.0, 120.0), (90.0, 70.0), (90.0, -40.0), (0.0, -40.0)];

fn default_course() -> Course {
    Course {
        marks: DEFAULT_MARKS.iter().map(|&(x, y)| Mark { x, y }).collect(),
        mark_radius: MARK_RADIUS,
    }
}

/// The browser's handle on the simulation: owns a `Sim` and runs it at its
/// fixed step, however long the frame was.
#[wasm_bindgen]
pub struct Game {
    run: CourseRun,
    scenario: Scenario,
    dt: f64,
    time: f64,
    pending: f64,
    capsize_enabled: bool,
    gust_intensity: f64,
    shift_deg: f64,
}

impl Game {
    fn rebuild(&mut self) {
        let course = self.run.course().clone();
        self.rebuild_with(course);
    }

    fn apply_variability(&mut self) {
        self.run
            .sim_mut()
            .set_wind_variability(self.gust_intensity, self.shift_deg.to_radians());
    }

    fn rebuild_with(&mut self, course: Course) {
        self.run = CourseRun::new(
            Sim::new(SimConfig {
                capsize_enabled: self.capsize_enabled,
                gust_intensity: self.gust_intensity,
                shift_intensity: self.shift_deg.to_radians(),
                seed: 1,
                ..SimConfig::default()
            }),
            course,
        );
        self.run.reset(self.scenario);
        self.run
            .sim_mut()
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
            shift_intensity: DEFAULT_SHIFT_DEG.to_radians(),
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
        let mut run = CourseRun::new(Sim::new(config), default_course());
        run.reset(scenario);
        Game {
            run,
            scenario,
            dt: config.dt,
            time: 0.0,
            pending: 0.0,
            capsize_enabled: true,
            gust_intensity: DEFAULT_GUST_INTENSITY,
            shift_deg: DEFAULT_SHIFT_DEG,
        }
    }

    pub fn reset(&mut self) {
        self.run.reset(self.scenario);
        self.time = 0.0;
        self.pending = 0.0;
    }

    /// Capsize ends the run when on; off lets you learn trim without flipping.
    pub fn set_capsize_enabled(&mut self, enabled: bool) {
        self.capsize_enabled = enabled;
        self.rebuild();
    }

    /// Gust strength as a fraction of the mean wind (0 = steady); the run carries on.
    pub fn set_gust_intensity(&mut self, intensity: f64) {
        self.gust_intensity = intensity.clamp(0.0, 0.9);
        self.apply_variability();
    }

    pub fn gust_intensity(&self) -> f64 {
        self.gust_intensity
    }

    /// Largest wind shift in degrees either side of the mean direction (0 = steady); the run carries on.
    pub fn set_shift_deg(&mut self, deg: f64) {
        self.shift_deg = deg.clamp(0.0, 45.0);
        self.apply_variability();
    }

    pub fn shift_deg(&self) -> f64 {
        self.shift_deg
    }

    pub fn capsize_enabled(&self) -> bool {
        self.capsize_enabled
    }

    /// Advance by `elapsed` real seconds (clamped so a stalled tab cannot
    /// trigger a spiral of catch-up steps) with the controls held constant.
    /// `hike` is how far the sailor leans out, 0 (sitting in) to 1.
    pub fn advance(&mut self, elapsed: f64, rudder: f64, sheet: f64, hike: f64) {
        self.pending += elapsed.clamp(0.0, 0.1);
        let controls = Controls {
            rudder: rudder.clamp(-1.0, 1.0),
            sheet: sheet.clamp(0.0, 1.0),
            hike: hike.clamp(0.0, 1.0),
        };
        while self.pending >= self.dt {
            self.pending -= self.dt;
            let outcome = self.run.step(controls);
            if matches!(outcome, Outcome::Capsized { .. }) {
                self.pending = 0.0;
                break;
            }
            self.time += self.dt;
            if matches!(outcome, Outcome::Finished { .. }) {
                self.pending = 0.0;
                break;
            }
        }
    }

    pub fn snapshot_len() -> usize {
        SNAPSHOT_LEN
    }

    pub fn snapshot(&self) -> Vec<f64> {
        let sim = self.run.sim();
        let (state, status) = (sim.state(), sim.status());
        let obs = sim.observation();
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
            sim.wind_speed(),
            sim.wind_dir().to_degrees(),
        ]
    }

    /// Change the true wind live: speed in m/s, direction in degrees
    /// clockwise from north (where it blows from).
    pub fn set_wind(&mut self, speed: f64, dir_deg: f64) {
        self.scenario.true_wind_speed = speed.max(0.0);
        self.scenario.true_wind_dir = dir_deg.to_radians();
        self.run
            .sim_mut()
            .set_wind(self.scenario.true_wind_speed, self.scenario.true_wind_dir);
    }

    pub fn wind_speed(&self) -> f64 {
        self.scenario.true_wind_speed
    }

    /// Degrees clockwise from north, where the wind blows from.
    pub fn wind_dir_deg(&self) -> f64 {
        self.scenario.true_wind_dir.to_degrees()
    }

    /// Replace the course with marks given as flat x, y pairs (metres) and
    /// restart the run.
    pub fn set_course(&mut self, marks: Vec<f64>) {
        let course = Course {
            marks: marks
                .iter()
                .step_by(2)
                .zip(marks.iter().skip(1).step_by(2))
                .map(|(&x, &y)| Mark { x, y })
                .collect(),
            mark_radius: MARK_RADIUS,
        };
        self.rebuild_with(course);
    }

    /// Marks as flat x, y pairs (metres).
    pub fn course_marks(&self) -> Vec<f64> {
        let marks = &self.run.course().marks;
        marks.iter().flat_map(|m| [m.x, m.y]).collect()
    }

    pub fn mark_radius(&self) -> f64 {
        self.run.course().mark_radius
    }

    /// `[next mark index (-1 when finished), distance (m), bearing (rad
    /// clockwise from north)]` to the next mark.
    pub fn course_progress(&self) -> Vec<f64> {
        let p = self.run.progress();
        vec![
            p.next_mark.map_or(-1.0, |i| i as f64),
            p.distance,
            p.bearing,
        ]
    }

    /// 0 = in progress, 1 = finished, 2 = capsized.
    pub fn course_status(&self) -> u8 {
        match self.run.outcome() {
            Outcome::InProgress => 0,
            Outcome::Finished { .. } => 1,
            Outcome::Capsized { .. } => 2,
        }
    }

    /// Final time (s) once finished, otherwise 0.
    pub fn finish_time(&self) -> f64 {
        match self.run.outcome() {
            Outcome::Finished { time } => time,
            _ => 0.0,
        }
    }
}
