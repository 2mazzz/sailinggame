# Course layer: design

Crate `sailcourse`, on top of `sailsim` (issue #5). Game logic, not physics.

## Scope

A course is an ordered list of marks (open water, no rounding direction). The layer reports progress, the completion time, and when the episode ends. RL observations are out of scope. The score (reward) was added later and is described under Score.

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

## Score

`CourseRun::score()` is the episode total and `last_reward()` the latest step's reward. One definition serves the viewer (via wasm) and the Gymnasium env (#10). Weights live in `RewardConfig` (defaults below), all tuned, none sourced.

- Progress: `progress_per_m` (1.0) per metre gained towards the next mark, negative when losing ground. Reaching a mark credits the rest of the leg, so a leg adds up to its length.
- `mark_bonus` (50) for each mark reached in order.
- `finish_bonus` (600) at the finish, scaled down linearly to 0 as the time reaches `finish_time_ref` (150 s), so each second saved is worth 4 points. This is where speed pays most.
- `time_penalty_per_s` (0.3) every second in progress, so speed also pays along the way.
- `capsize_penalty` (100) once, when the boat capsizes before the last mark. A capsized or finished episode scores nothing further.

## Baseline agent

`sailcourse::baseline::baseline_controls(&run)` is a scripted agent for comparison and for viewing one example: steer at the next mark, trim the boom 12 degrees inside the apparent wind angle (a fixed angle of attack), hike when heeled past 12 degrees. It does not tack. On the default figure-8 course it finishes in about 94 s (score near 900) in 6 m/s, in 58 s (score near 950) in 11 m/s, and capsizes on a sharp turn from about 13 m/s, the default wind for training and the viewer (25 kn) (test `the_baseline_agent_finishes_the_default_course_with_a_positive_score`). The viewer runs it with the P key or `?auto=1`.
