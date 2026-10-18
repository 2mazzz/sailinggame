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

Rudder in [-1, 1], sheet in [0, 1] and hike in [0, 1], mapped to physical quantities inside the crate. These are the RL action space. `hike` is how far the sailor leans out beyond the sitting-in baseline: it lengthens the righting arm by up to `hike_arm_extra` (0.15 m, about +33 % righting moment; tuned, no source). `hike` 0 is the baseline curve, so a run that never hikes is unchanged. It is an action, not an observation: the sailor's lean is the agent's own last command, so `Observation` is unchanged. The viewer holds Space or Shift to hike and draws the sailor leaning out on the windward side.

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
let out = sim.step(Controls { rudder: 0.1, sheet: 0.5, hike: 0.0 });
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
- Weather helm is mild (`board_arm` 0.75 m, retuned from 0.42 m when the rudder area rose, and nudged from 0.76 m on 2026-10-18 with `rudder_area` 0.105 m^2): on a beam reach a released rudder drifts about 2 deg in 30 s (test `helm`). Source validation of this value is pending (see `docs/research/`).
- Capsize threshold is 70 deg of heel; the stability curve vanishes at 85 deg. Past 90 deg a flat sail makes no force.
- `SimConfig::seed` is reserved: with constant wind there is nothing random yet.

Known model limits, to revisit:
- The no-go zone is firm (ticket 4): pinned at 30 deg TWA in 6 m/s of wind the best-trimmed boat sustains about 1.4 m/s (was 1.7). This came from raising the minimum boom angle (`boom_angle_in`) from 5 to 21 deg (18 in the first pass, 21 after ticket 14 strengthened the sail), a tuned value with no source (a real boom reaches about 10 deg off the centreline). `sheet` 1.0 now means a fully sheeted 21 deg boom, so the capsize tests moved from sheet 0.85 to 1.0 to keep the same sail trim.
- Ticket 14 (research priorities 1 to 3, landed together): the righting arm is now a hiking-crew curve (`tanh` build-up times a cosine fall-off, 0.45 m peak on 160 kg, about 700 N m upright), the sail depowers once its heeling moment passes 450 N m (easing to 35 % of raw force; `depower_moment`, `depower_floor`, both tuned, no source), sail lift is raised (slope 6.2, soft ceiling 1.15, sharper stall blend; a ceiling of 1.15 rather than the ORC 1.35 keeps the beam reach near 6.7 kn, the 12 kn reach target), and hull resistance is the Day and Nixon tank drag-area table with its hump at 6.5 kn. Results at 12 kn: 5.3 kn and VMG 3.4 kn at 44 deg (targets 5.4 and 3.75), heel about 1.5 deg (target 2.5), VMG about 3.9 kn at 16 kn and 4.1 kn at 20 kn, beam reach 6.8 kn (target 6.5). Light air VMG stays about 20 % low (1.9 kn at 5 kn against 2.3). Sheeted-in close-hauled capsize wind speed moved from 10.5 m/s (20 kn) to 10 m/s (19 kn) at 45 deg TWA, and on a beam reach from 10 m/s to 12.5 m/s. Tests: `published_targets`, `capsize`. Crew hiking is baked into the curve; the hike control (ticket 9) adds up to 0.15 m of arm on top of that (`hike_arm_extra`, tuned, no source); in `hiking` test terms, close-hauled at 45 deg a fully hiking sailor has clearly lower heel and survives 10.5 m/s, which capsizes a sailor sitting in.
- Planing (2026-10-18): the Day and Nixon tank table (2 to 9 kn, hump at 6.5 kn) is unchanged. Above 9 kn the drag area now falls as `(v/v9)^(2-p)`, so resistance is `R9 (v/v9)^p` with `planing_resistance_exponent` p = 0.5. This is an assumption: there is no Laser data above 9 kn. At 25 kn on a beam reach the boat sails 12.2 kn with p = 2 (the old behaviour), 15.0 kn with p = 0.5 and 16.0 kn with p = 0. The test boat hikes fully and cannot capsize, so these are upper bounds. Still missing: dynamic trim and wave surfing. See `docs/research/2026-10-18-planing-and-rudder-sources.md`.
- Rudder (2026-10-18): the blade stalls near 12 deg (lift slope 3.8, aspect ratio 4.5 from the free-surface image) and pays induced drag with `FoilParams.efficiency` 0.97 (Nita and Scholz via Day 2017; it was an assumed 0.9). It sees `rudder_inflow_factor` 0.9 of the boat speed. In a steady turn the yaw rate still rises with rudder angle, because the swinging stern cuts the blade's angle of attack; the stall shows in turn entry (the first 0.2 s) and in drag. To keep the helm balance, `rudder_area` was retuned 0.09 to 0.105 m^2 and `board_arm` 0.76 to 0.75 m; neither retune has a source, they compensate for the weaker rudder.
- Unsourced numbers: most of the dynamics parameters (inertias, damping, sail arms, depower) have no source. `docs/research/2026-10-18-parameter-audit.md` lists which parameters do.
- Particulars are sourced from the research doc (ticket 13): displacement 160 kg and surge mass 170 kg (high confidence; the 10 kg added mass is an estimate), rudder area 0.09 m^2 (LOW confidence: my estimate from the class drawing, not a published figure). Board area 0.15 m^2 and the hull lateral area are still unsourced. `board_arm` 0.76 m has no source: it was retuned only to keep the weather helm mild after the rudder grew, and puts the sail CE about 1 m aft of the board, more than the research thinks plausible. Heel and yaw inertia were not scaled for the extra rig mass.
- Tuning the parameters in `BoatParams::ilca_like()` for feel and realism is human-in-the-loop work.

## Out of scope

- Waves, gusts, wind shifts, dynamic planing trim and wave surfing, hiking dynamics (the lean is instantaneous in the sim; only the viewer smooths it), capsize recovery.
- **Course layer** (own design doc: [course layer design](2026-10-14-course-layer-design.md)): a map with goals the boat must reach in a specific order. Marks, order, rounding detection, progress and reward, episode termination, and its use from both the Gymnasium env and the game. It sits on top of `sailsim` as a separate crate, because it is game/RL logic, not physics.
