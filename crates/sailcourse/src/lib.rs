//! Course layer on top of `sailsim`: ordered marks, progress, completion time
//! and episode end. See docs/plans/2026-10-14-course-layer-design.md.

pub mod baseline;
pub mod observation;

use sailsim::{Controls, Scenario, Sim, Status};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Course {
    pub marks: Vec<Mark>,
    /// A mark is reached when the boat is within this distance (m) of it.
    pub mark_radius: f64,
}

/// A mark is reached within this many metres on the default course.
pub const DEFAULT_MARK_RADIUS: f64 = 6.0;

/// Default course (x, y pairs, metres; x east, y north): a reverse C that
/// starts at the top. East along the top, south down the back, west along the
/// bottom, so for a northerly wind it is a beam reach, a run, then a beam reach.
/// Tight on purpose: marks 35 to 50 m apart with a 6 m radius, so the angle
/// you come in at decides how well you round each one.
const DEFAULT_MARKS: [(f64, f64); 5] = [
    (50.0, 0.0),
    (75.0, -25.0),
    (75.0, -65.0),
    (50.0, -90.0),
    (0.0, -90.0),
];

pub fn default_course() -> Course {
    Course {
        marks: DEFAULT_MARKS.iter().map(|&(x, y)| Mark { x, y }).collect(),
        mark_radius: DEFAULT_MARK_RADIUS,
    }
}

/// Distance (m) and bearing (rad clockwise from north) to the next mark.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Progress {
    /// `None` once every mark has been reached.
    pub next_mark: Option<usize>,
    pub distance: f64,
    pub bearing: f64,
}

/// How the episode stands. `Finished` ends it; `time` is seconds of sim time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    InProgress,
    Finished {
        time: f64,
    },
    /// The boat capsized before the last mark; `time` is when it happened.
    Capsized {
        time: f64,
    },
}

/// How a run is scored: progress along the course and how fast it is made.
/// These weights are the shared reward for the viewer and the RL environment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RewardConfig {
    /// Per metre gained towards the next mark (negative when losing ground).
    pub progress_per_m: f64,
    /// For each mark reached, in order.
    pub mark_bonus: f64,
    /// At the finish, scaled down linearly to zero as the time reaches `finish_time_ref`.
    pub finish_bonus: f64,
    /// Finishing in this many seconds or more earns no finish bonus.
    pub finish_time_ref: f64,
    /// Lost every second the run is in progress, so speed pays all along.
    pub time_penalty_per_s: f64,
    /// Lost once when the boat capsizes before the last mark.
    pub capsize_penalty: f64,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            progress_per_m: 1.0,
            mark_bonus: 50.0,
            finish_bonus: 300.0,
            finish_time_ref: 300.0,
            time_penalty_per_s: 0.2,
            capsize_penalty: 100.0,
        }
    }
}

pub struct CourseRun {
    sim: Sim,
    course: Course,
    dt: f64,
    steps: u64,
    next: usize,
    outcome: Outcome,
    reward: RewardConfig,
    score: f64,
    last_reward: f64,
}

impl CourseRun {
    pub fn new(sim: Sim, course: Course) -> Self {
        let dt = sim.dt();
        Self {
            sim,
            course,
            dt,
            steps: 0,
            next: 0,
            outcome: Outcome::InProgress,
            reward: RewardConfig::default(),
            score: 0.0,
            last_reward: 0.0,
        }
    }

    pub fn with_reward(mut self, reward: RewardConfig) -> Self {
        self.reward = reward;
        self
    }

    /// Total reward so far this episode.
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Reward from the latest `step` (0 once the episode has ended).
    pub fn last_reward(&self) -> f64 {
        self.last_reward
    }

    pub fn reset(&mut self, scenario: Scenario) {
        self.sim.reset(scenario);
        self.steps = 0;
        self.next = 0;
        self.outcome = Outcome::InProgress;
        self.score = 0.0;
        self.last_reward = 0.0;
    }

    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    pub fn sim_mut(&mut self) -> &mut Sim {
        &mut self.sim
    }

    pub fn course(&self) -> &Course {
        &self.course
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    pub fn step(&mut self, controls: Controls) -> Outcome {
        if self.outcome != Outcome::InProgress {
            self.last_reward = 0.0;
            return self.outcome;
        }
        let before = self.sim.state();
        let out = self.sim.step(controls);
        let state = out.state;
        self.steps += 1;
        let time = self.steps as f64 * self.dt;
        let mut reward = -self.reward.time_penalty_per_s * self.dt;
        if out.status == Status::Capsized {
            self.outcome = Outcome::Capsized { time };
            reward -= self.reward.capsize_penalty;
            return self.finish_step(reward);
        }
        let mark = self.course.marks[self.next];
        let dist_before = (mark.x - before.x).hypot(mark.y - before.y);
        let dist_after = (mark.x - state.x).hypot(mark.y - state.y);
        if dist_after <= self.course.mark_radius {
            // Credit the rest of the leg, so progress adds up to the leg length.
            reward += self.reward.progress_per_m * dist_before + self.reward.mark_bonus;
            self.next += 1;
            if self.next == self.course.marks.len() {
                self.outcome = Outcome::Finished { time };
                let speed = (1.0 - time / self.reward.finish_time_ref).max(0.0);
                reward += self.reward.finish_bonus * speed;
            }
        } else {
            reward += self.reward.progress_per_m * (dist_before - dist_after);
        }
        self.finish_step(reward)
    }

    fn finish_step(&mut self, reward: f64) -> Outcome {
        self.last_reward = reward;
        self.score += reward;
        self.outcome
    }

    pub fn progress(&self) -> Progress {
        let state = self.sim.state();
        let Some(&mark) = self.course.marks.get(self.next) else {
            return Progress {
                next_mark: None,
                distance: 0.0,
                bearing: 0.0,
            };
        };
        let (dx, dy) = (mark.x - state.x, mark.y - state.y);
        Progress {
            next_mark: Some(self.next),
            distance: dx.hypot(dy),
            bearing: dx.atan2(dy),
        }
    }
}
