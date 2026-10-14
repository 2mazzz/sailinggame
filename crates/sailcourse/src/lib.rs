//! Course layer on top of `sailsim`: ordered marks, progress, completion time
//! and episode end. See docs/plans/2026-10-14-course-layer-design.md.

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

pub struct CourseRun {
    sim: Sim,
    course: Course,
    dt: f64,
    steps: u64,
    next: usize,
    outcome: Outcome,
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
        }
    }

    pub fn reset(&mut self, scenario: Scenario) {
        self.sim.reset(scenario);
        self.steps = 0;
        self.next = 0;
        self.outcome = Outcome::InProgress;
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    pub fn step(&mut self, controls: Controls) -> Outcome {
        if self.outcome != Outcome::InProgress {
            return self.outcome;
        }
        let out = self.sim.step(controls);
        let state = out.state;
        self.steps += 1;
        if out.status == Status::Capsized {
            self.outcome = Outcome::Capsized {
                time: self.steps as f64 * self.dt,
            };
            return self.outcome;
        }
        let mark = self.course.marks[self.next];
        if (mark.x - state.x).hypot(mark.y - state.y) <= self.course.mark_radius {
            self.next += 1;
            if self.next == self.course.marks.len() {
                self.outcome = Outcome::Finished {
                    time: self.steps as f64 * self.dt,
                };
            }
        }
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
