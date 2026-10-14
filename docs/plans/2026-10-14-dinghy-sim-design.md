# Dinghy physics sim: design

Milestone 1 of the sailing game: a deterministic, headless Rust physics core for a single-handed dinghy. It is the single source of truth for the RL training env (via PyO3) and the browser game (via WASM).

## Product context

- Realistic top-down 2D sailing game; an RL agent learns to sail courses.
- Stack: Rust core (`sailsim`), PyO3 bindings + Gymnasium + PPO/SAC for training, wasm-bindgen + PixiJS/TypeScript for the game, ONNX policy in-browser.
- This document covers only the physics core. The course layer (marks, order, rounding, reward) is a separate design: see [Out of scope](#out-of-scope).

## Boat

ILCA/Laser-class dinghy, SI units, all in one `BoatParams` struct (`DinghyParams::ilca_like()`):

- Length 4.2 m, beam 1.4 m, hull 60 kg + sailor 80 kg (fixed crew weight).
- Sail area 7 m²; centerboard and rudder as foils with area and aspect ratio.

## State

Position x, y; heading; surge and sway velocity; yaw rate; heel angle and rate; `Status { Sailing, Capsized }`. 3-DOF plus heel.

## Forces

1. **Apparent wind**: true wind minus boat velocity, in the boat frame.
2. **Sail**: lift and drag from angle of attack (thin-airfoil lift curve with stall). Trim via sheet. Force acts at the center of effort, giving the heeling moment.
3. **Hull**: frictional + wave-making resistance (rises with Froude number); added mass in sway and yaw.
4. **Centerboard and rudder**: lift and drag from angle of attack. Board provides lateral resistance; rudder gives yaw moment.
5. **Heel**: righting moment from a GZ stability curve (peaks ~40-60 deg, zero at the angle of vanishing stability, ~80-90 deg). Heel reduces effective sail area.

## Environment

- **Constant wind** of given speed and direction (from `Scenario`). Gusts, shifts and waves are deferred.

## Steering

Yaw is part of milestone 1 (it was implicit in the original design; RL needs it). The rudder is a foil at the stern; it sees the water at the stern, so rotation and sideslip reduce its angle of attack (rudder damping). The centreboard sits ahead of the centre of gravity and the sail's centre of effort aft of it, giving a mild weather helm: a released helm rounds the boat up. `SimConfig::fixed_heading` locks the heading for tests and for curricula that learn sail trim first.

## Controls

Rudder in [-1, 1], sheet in [0, 1], mapped to physical angles inside the crate. These are the RL action space.

## Capsize

Terminal and switchable. When heel exceeds the capsize threshold and `capsize_enabled` is true, `status` becomes `Capsized`; RL ends the episode with a penalty. When disabled, heel is allowed to continue without terminating (curriculum: learn trimming first). Inverted-hull and recovery physics are out of scope.

## Crate layout

```
crates/sailsim/
  src/
    lib.rs          # public API
    params.rs       # BoatParams, DinghyParams::ilca_like()
    state.rs        # BoatState, Controls, Status
    wind.rs         # constant true wind from Scenario
    aero.rs         # sail lift/drag, apparent wind
    hydro.rs        # hull resistance, foils, added mass
    stability.rs    # GZ righting moment, capsize check
    integrator.rs   # fixed-step RK4
    sim.rs          # Sim: step, reset
  tests/
```

## API

```rust
let mut sim = Sim::new(SimConfig { dt: 0.02, seed: 42, capsize_enabled: true, .. });
sim.reset(Scenario { true_wind_speed: 6.0, true_wind_dir: 0.0, .. });
let out = sim.step(Controls { rudder: 0.1, sheet: 0.5 });
// out.state, out.apparent_wind, out.status
let obs = sim.observation(); // RL features
```

- Pure and deterministic: the same seed and controls give the same trajectory.
- No rendering types, no allocation in `step`.
- `observation()` is computed in Rust so training and the game feed the policy identical inputs: apparent wind angle and speed, speed through water, heel and rate, yaw rate, capsize margin.
- `SimConfig` holds the switches (`capsize_enabled`, later wave and gust settings).

## Testing

Qualitative sailing:
- No-go zone: within ~30 deg of the wind the boat stalls.
- Close-hauled VMG: best upwind angle ~40-45 deg, positive upwind progress on both tacks.
- Speed by point of sail rises from close-hauled to a reach and falls dead downwind.
- Apparent wind comes forward as the boat accelerates on a reach.
- Mirror symmetry of wind and controls mirrors the trajectory.
- Leeway is small but nonzero; centerboard reduces it.
- A tack loses speed and recovers it in plausible time.

Heel and capsize:
- Heel grows with wind and trim and eases when the sheet is released.
- GZ curve is positive up to the vanishing angle, negative beyond.
- Capsize flag triggers with the switch on and not with it off.

Numerical:
- Determinism for equal seed and inputs.
- No energy gain with wind off; the integrator is stable at the configured dt.
- Halving dt changes results within tolerance.

Parameters are exposed for tuning. Tuning decisions are human-in-the-loop work.

## Implementation status (milestone 1)

Built test-first at these seams, all in `crates/sailsim/tests/`: rest and determinism, no-go zone, beam reach, speed polar, upwind and leeway, heel and capsize, mirror symmetry, steering and tacking, observation. 25 tests, `cargo test -p sailsim`.

Deviations from the design above:
- Steering/yaw and `fixed_heading` were added (see Steering). Mirror symmetry runs with free yaw.
- `Observation` is a plain struct: apparent wind angle and speed, surge, sway, heel, heel rate, yaw rate, capsize margin.
- Weather helm is mild (`board_arm` 0.76 m, retuned from 0.42 m when the rudder area rose to 0.09 m^2): on a beam reach a released rudder drifts about 2 deg in 30 s (test `helm`). Source validation of this value is pending (see `docs/research/`).
- Capsize threshold is 70 deg of heel; the stability curve vanishes at 85 deg. Past 90 deg a flat sail makes no force.
- `SimConfig::seed` is reserved: with constant wind there is nothing random yet.

Known model limits, to revisit:
- Close-hauled is still optimistic (about 5.6 kn at 45 deg TWA in 11.7 kn of wind) and the boat can still sail at 30 deg TWA; the no-go zone is soft. Heel depowers the sail but there is no crew hiking or active depowering.
- No planing: resistance is a simple linear + quadratic curve, tuned so beam-reach speed is plausible.
- Particulars are sourced from the research doc (ticket 13): displacement 160 kg and surge mass 170 kg (high confidence; the 10 kg added mass is an estimate), rudder area 0.09 m^2 (LOW confidence: my estimate from the class drawing, not a published figure). Board area 0.15 m^2 and the hull lateral area are still unsourced. `board_arm` 0.76 m has no source: it was retuned only to keep the weather helm mild after the rudder grew, and puts the sail CE about 1 m aft of the board, more than the research thinks plausible. Heel and yaw inertia were not scaled for the extra rig mass.
- Tuning the parameters in `BoatParams::ilca_like()` for feel and realism is human-in-the-loop work.

## Out of scope

- Waves, gusts, wind shifts, planing, hiking dynamics, capsize recovery.
- **Course layer** (own design doc: [course layer design](2026-10-14-course-layer-design.md)): a map with goals the boat must reach in a specific order. Marks, order, rounding detection, progress and reward, episode termination, and its use from both the Gymnasium env and the game. It sits on top of `sailsim` as a separate crate, because it is game/RL logic, not physics.
