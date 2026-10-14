# Course layer: design

Crate `sailcourse`, on top of `sailsim` (issue #5). Game logic, not physics.

## Scope

A course is an ordered list of marks (open water, no rounding direction). The layer reports progress, the completion time, and when the episode ends. Reward, shaping and RL observations are explicitly out of scope; completion time is reported and nothing more.

## Model

- `Mark { x, y }` in the sim world frame (metres, x east, y north). `Course { marks, mark_radius }`: a mark is reached when the boat is within `mark_radius` of it after a step.
- Only the *next* mark counts. Passing through a later mark early is ignored, and is not credited retroactively once the earlier marks are done.
- `CourseRun` owns a `Sim` (so the seed in `SimConfig` makes runs deterministic) plus the course, and counts steps; time = steps x dt.
- `Progress { next_mark: Option<usize>, distance, bearing }`: distance in metres to the next mark, bearing in radians clockwise from north (same convention as heading). `next_mark` is `None` when finished.
- `Outcome { InProgress, Finished { time }, Capsized { time } }`. Finished and Capsized end the episode; further steps are no-ops that leave the outcome unchanged.

## Public interface

```rust
CourseRun::new(sim: Sim, course: Course) -> CourseRun
run.reset(scenario: Scenario)
run.step(controls: Controls) -> Outcome
run.progress() -> Progress
run.outcome() -> Outcome
```

## Test seams (all through the interface above)

1. Progress: distance and bearing to the first mark after `reset` (catches geometry/convention errors; not dynamics).
2. Sailing a course in order finishes and reports the time (catches mark detection and time accounting; time is bounded by plausible boat speed, not an exact value).
3. Out-of-order marks do not count (catches order logic).
4. Capsize ends with `Capsized`, distinct from `Finished` (catches outcome precedence).
5. Determinism: same seed and controls give an identical outcome (catches hidden state).

Not tested: mark radius edge exactly on the boundary, anything inside `sailsim`.
