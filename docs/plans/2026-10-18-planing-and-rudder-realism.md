# Planing and rudder realism: implementation plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Give the dinghy sim a sourced planing-range hull resistance and a rudder that stalls and pays induced drag like a real one, and put a source and confidence on every other number in `BoatParams::ilca_like()`.

**Architecture:** Three independent changes in `crates/sailsim`: (1) `foil.rs` / `params.rs` get a per-foil efficiency and a physically consistent rudder (effective aspect ratio, section stall, reduced inflow); (2) `hydro.rs::hull_resistance` gets a planing tail above the last tank-test point instead of the constant drag area it has now; (3) a research task that audits the remaining unsourced parameters; (4) a Rust example plus a matplotlib script that draw seven engineering graphs of the dynamics from the sim itself. Every task is test-first, and every number that changes gets a source comment in the existing style.

**Tech Stack:** Rust (`sailsim`, tests in `crates/sailsim/tests`), pytest in `python/tests`, `cargo test --workspace`.

**Commit rule:** this repo's CLAUDE.md forbids any Claude attribution. Commit messages carry no `Co-Authored-By` trailer.

---

## What the research found (read this first)

Findings that change the scope you asked for:

1. **Planing onset is already in the sim.** `hull_drag_area` is the Day and Nixon (2014) tank table, 2 to 9 kn, with the major hump at 6.5 kn where "the boat starts to plane" and the drop after it. What is missing is everything above 9 kn: the table is held constant there, so resistance rises as v², which is the opposite of planing. There is also no dynamic trim, no wave surfing and no planing lift on the foils. The README line "There is no planing" is too strong.
2. **There is no Laser data above 9 kn.** The tank test stopped at 9 kn, and I found no primary source above it. Anything I put there is an assumption and is labelled low confidence. I would not build a dynamic planing model (trim state plus hydrodynamic lift) with nothing to fit it to.
3. **The sim's beam reach is already about 20 % too fast** (7.9 kn against about 6.5 kn measured, `docs/research/2026-10-14-dinghy-physics-validation.md` Q2c). Planing only adds speed, and only above 9 kn. Do not expect it to fix that.
4. **The rudder does not really stall.** `FoilParams` for the rudder has `stall_angle` 0.30 rad with a gentle blend, and I replicated `foil::coefficients_sc` numerically: CL rises monotonically to 0.84 at 40 degrees and never drops. A real NACA 0012 type blade loses lift past about 12 to 14 degrees. So full rudder (35 degrees) is currently the strongest turn, which is wrong, and an RL agent has no reason to use small rudder angles.
5. **The rudder's aspect ratio is inconsistent with its area.** Area 0.09 m² (0.45 m × 0.20 m) is a geometric AR of 2.25, but near the free surface the effective AR is doubled by the image (Day 2017, AR_E = 2b/c = 4.5). The sim uses 2.5 with a hardcoded 0.7 efficiency, so induced drag is too high by about 2.5 times.
6. **Expected effect on speed is small.** Day (2017) finds the rudder-induced drag "rather small as a proportion of total resistance". The main effect of these changes is on turning, tacking and what the agent learns about steering, not on VMG.

## Evidence table

Confidence is mine. "Fig." means read off a published figure.

| Number | Value | Source | Confidence |
|---|---|---|---|
| Hull drag area 2 to 9 kn | already in `params.rs` | Day and Nixon (2014) tank test, 160 kg, level trim | High (one hull) |
| Planing onset | about 6.5 kn, Froude number 0.55; sinkage turns negative and drag area starts to fall | Day and Nixon (2014) | High |
| Resistance above 9 kn | none | No source found | None |
| Generic planing threshold | Froude number about 1.0 to 1.2 on the length Froude number | Wikipedia "Hull speed" and planing-hull literature (secondary); the Laser is already past 0.55 at onset because the table is a transition regime | Low |
| Rudder lift slope | `5.7 AR_E / (1.8 + sqrt(AR_E² + 4))`, AR_E = 2b/c; gives 3.8 per rad for AR_E 4.5 | Day (2017) eq. 12, Keuning and Verwerft (2009) | Medium |
| Rudder induced drag | `CDi = CL² / (π AR_E)`, with AR_E = AR·e, e a planform efficiency (Nita and Scholz 2012) | Day (2017) eq. 16, 17 | Medium (e not yet read, assumed 0.9) |
| Rudder inflow speed | 0.9 × boat speed | Day (2017) | Medium |
| Rudder geometry | blade at most 20.3 mm thick, 203 mm wide; submerged span about 0.45 m (my estimate) | ILCA Class Rules 2020, measurement diagrams | High (outline), low (submerged span) |
| NACA 0012 stall | 14 degrees at Re 1.8e6, CLmax 1.33 smooth | NACA TN 3361 (Critzos 1955) | High for the section |
| Stall at dinghy Reynolds numbers | Re 0.36 to 0.7e6: CLmax about 1.0; a forum XFOIL run at Re 179,000 gives 9 to 11 degrees | Sandia SAND80-2114 (Sheldahl and Klimas 1981); forum post (low) | Medium |
| Dinghy rudder Reynolds number | about 3.5 to 5e5 (chord 0.20 m, 2 to 3 m/s) | My arithmetic | Medium |
| Rudder stall with hull | CFD 470 class, stall at 30 degrees rudder angle (35 at 10 m/s); best lift to drag at 10 to 25 degrees | Lin et al. 2018, Proceedings 2(6) 308 | Low: CFD, a 470 not a Laser, hull wake and board downwash included |
| Rudder share of drag | induced drag of the whole underwater body is 14 % of hydro drag upwind, under 1 % downwind | Day (2017) | Medium |

The 470 CFD figure (30 degrees) and the section figure (12 to 14 degrees) disagree. The section value applies to a blade in clean flow, which is what the sim models. The CFD value includes the hull's wake and downwash from the board, which cut the blade's effective angle. The plan keeps the section value and records the disagreement. Task 3 includes a decision point if testing shows it hurts steering.

## Decisions made (change them before executing if you disagree)

- **Planing:** extend the tank table with a power-law tail `R = R9 · (v / v9)^p`, p = 0.5, instead of a trim-and-lift model. p is an assumption. The tail is continuous at 9 kn and gives a resistance that rises slowly, the shape of a planing curve.
- **Rudder:** keep the existing blend model in `foil.rs` and change its numbers, not its structure. Do not add board downwash (Day's Φ term): the effect is second order and needs the board's CL fed into the rudder.
- **Out of scope:** waves and surfing (the sim has none), dynamic trim, board lift-off.

---

## Task 0: Characterisation baseline

Record what the sim does today so later tasks can say what moved.

**Files:**
- Create: `crates/sailsim/tests/baseline_planing_rudder.rs`

**Step 1: Write the test file**

```rust
//! Numbers recorded before the planing and rudder work (2026-10-18). They are
//! loose guards, not targets: later tasks must keep them inside these bands or
//! say why they moved.
use sailsim::{Controls, Scenario, Sim, SimConfig};

const KN: f64 = 0.514444;

fn reach_speed(wind_kn: f64) -> f64 {
    let mut sim = Sim::new(SimConfig {
        fixed_heading: true,
        capsize_enabled: false,
        ..SimConfig::default()
    });
    sim.reset(Scenario {
        true_wind_speed: wind_kn * KN,
        true_wind_dir: std::f64::consts::FRAC_PI_2,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 0.0,
        start_heel: 0.0,
    });
    let mut surge = 0.0;
    for _ in 0..4500 {
        surge = sim
            .step(Controls { rudder: 0.0, sheet: 0.55, hike: 1.0 })
            .state
            .surge;
    }
    surge / KN
}

#[test]
fn print_reach_speeds_for_the_record() {
    for wind in [8.0, 12.0, 16.0, 20.0, 25.0] {
        println!("beam reach {wind} kn wind: {:.2} kn", reach_speed(wind));
    }
}
```

**Step 2: Run and write the numbers down**

Run: `cargo test -p sailsim --test baseline_planing_rudder -- --nocapture`
Expected: PASS, with five lines of output. Paste them into the "Baseline" section at the bottom of this file.

**Step 3: Run the whole suite so you know it is green before you start**

Run: `cargo test --workspace`
Expected: all PASS. If anything fails already, stop and report it.

**Step 4: Commit**

```bash
git add crates/sailsim/tests/baseline_planing_rudder.rs docs/plans/2026-10-18-planing-and-rudder-realism.md
git commit -m "test: record beam-reach speeds before the planing and rudder work"
```

---

## Task 1: Per-foil induced-drag efficiency (no behaviour change)

`foil.rs` hardcodes `0.7` in the induced-drag term. The rudder needs its own value, so lift it into `FoilParams`. Board and sail keep 0.7 so nothing else moves.

**Files:**
- Modify: `crates/sailsim/src/params.rs` (`FoilParams`, and the three foil literals in `ilca_like`)
- Modify: `crates/sailsim/src/foil.rs:30`
- Test: unit test at the bottom of `crates/sailsim/src/foil.rs`

**Step 1: Write the failing test** (append to `foil.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::BoatParams;

    #[test]
    fn higher_efficiency_means_less_induced_drag() {
        let mut foil = BoatParams::ilca_like().rudder;
        foil.efficiency = 0.5;
        let (_, cd_low) = coefficients(&foil, 8.0_f64.to_radians());
        foil.efficiency = 1.0;
        let (_, cd_high) = coefficients(&foil, 8.0_f64.to_radians());
        assert!(cd_high < cd_low, "{cd_high} should be below {cd_low}");
    }
}
```

**Step 2: Run it, expect a compile failure**

Run: `cargo test -p sailsim --lib foil`
Expected: FAIL, "no field `efficiency` on type `FoilParams`".

**Step 3: Implement**

In `params.rs`, add to `FoilParams` after `aspect_ratio`:

```rust
    /// Planform efficiency `e`: induced drag is `CL^2 / (pi e AR)`. 0.7 is the
    /// value the sim used for every foil before the rudder work (no source);
    /// a rudder near the free surface uses the Day (2017) form instead.
    pub efficiency: f64,
```

Add `efficiency: 0.7,` to the `board`, `rudder` and `sail` literals. In `foil.rs` replace `std::f64::consts::PI * 0.7 * foil.aspect_ratio` with `std::f64::consts::PI * foil.efficiency * foil.aspect_ratio`.

**Step 4: Run the full suite**

Run: `cargo test --workspace`
Expected: all PASS, same as Task 0 (nothing should have moved). Also re-run Task 0's test and confirm the printed speeds are unchanged.

**Step 5: Commit**

```bash
git add crates/sailsim/src/params.rs crates/sailsim/src/foil.rs
git commit -m "sailsim: per-foil induced-drag efficiency, unchanged values"
```

---

## Task 2: A rudder that stalls and has a consistent aspect ratio

**Files:**
- Modify: `crates/sailsim/src/params.rs` (rudder literal, new field `rudder_inflow_factor`)
- Modify: `crates/sailsim/src/hydro.rs` (`rudder_force`)
- Test: unit tests in `crates/sailsim/src/foil.rs`

**Step 1: Write the failing tests** (add to the `tests` module in `foil.rs`)

```rust
    fn rudder_cl(deg: f64) -> f64 {
        coefficients(&BoatParams::ilca_like().rudder, deg.to_radians()).0
    }

    #[test]
    fn rudder_lift_peaks_near_the_section_stall_angle_then_drops() {
        let peak_deg = (0..=40)
            .max_by(|a, b| rudder_cl(*a as f64).total_cmp(&rudder_cl(*b as f64)))
            .unwrap();
        let peak = rudder_cl(peak_deg as f64);
        // NACA 0012 stalls at 12 to 14 degrees (NACA TN 3361, Sandia SAND80-2114).
        assert!((10..=15).contains(&peak_deg), "peak at {peak_deg} degrees");
        // A finite-aspect-ratio blade tops out between 0.6 and 0.95.
        assert!((0.6..=0.95).contains(&peak), "peak CL {peak}");
        // Fully stalled by 25 degrees: well below the peak.
        assert!(rudder_cl(25.0) < 0.75 * peak, "CL at 25 deg {}", rudder_cl(25.0));
    }

    #[test]
    fn rudder_lift_slope_matches_day_2017() {
        // dCL/dalpha = 5.7 AR_E / (1.8 + sqrt(AR_E^2 + 4)), AR_E = 2 b / c.
        let ar_e: f64 = 4.5;
        let expected = 5.7 * ar_e / (1.8 + (ar_e * ar_e + 4.0).sqrt()); // 3.81
        let slope = BoatParams::ilca_like().rudder.lift_slope;
        assert!((slope - expected).abs() < 0.1, "{slope} vs {expected}");
    }
```

**Step 2: Run, expect FAIL**

Run: `cargo test -p sailsim --lib foil`
Expected: both FAIL (peak at 40 degrees; slope 3.5 against 3.81).

**Step 3: Implement**

Replace the `rudder: FoilParams { .. }` literal in `params.rs` with the values below. They come from a numeric replica of `coefficients_sc`: peak CL 0.71 at 12 degrees, 0.44 at 20, which satisfies both tests. Keep the source comment in the repo's style.

```rust
            // Day (2017) eq. 12: slope 5.7 AR_E / (1.8 + sqrt(AR_E^2 + 4)) with AR_E = 2b/c = 2 x 0.45 / 0.20 = 4.5 (free-surface image), 3.8 per rad. Stall: NACA 0012 stalls at 12 to 14 degrees (NACA TN 3361; Sandia SAND80-2114 at dinghy Reynolds numbers 3.5 to 5e5), so stall_angle 0.26 rad with a sharp blend (exponent 8) and no soft ceiling. Efficiency 0.9 is an assumption (Day 2017 uses Nita and Scholz 2012, not yet read). Confidence: medium for slope and stall, low for efficiency and the submerged span 0.45 m. Lin et al. (2018, CFD, 470 class) put stall at 30 degrees of rudder angle including hull wake and board downwash; not modelled here.
            rudder: FoilParams {
                lift_slope: 3.8,
                stall_angle: 0.26,
                drag_zero_lift: 0.012,
                aspect_ratio: 4.5,
                efficiency: 0.9,
                plate_normal: 1.1,
                stall_sharpness: 8,
                max_lift: None,
            },
```

Add to `BoatParams`:

```rust
    /// Rudder inflow speed as a fraction of boat speed (the blade sits in the
    /// wake of the hull): 0.9, Day (2017). Confidence: medium.
    pub rudder_inflow_factor: f64,
```

with `rudder_inflow_factor: 0.9,` in `ilca_like`. In `hydro.rs::rudder_force`, change only the dynamic pressure line:

```rust
    let inflow = speed * params.rudder_inflow_factor;
    let q_s = 0.5 * WATER_DENSITY * inflow * inflow * params.rudder_area;
```

**Step 4: Run the new tests**

Run: `cargo test -p sailsim --lib foil`
Expected: PASS.

**Step 5: Run the whole suite and record what broke**

Run: `cargo test --workspace`
Expected: some of `steering.rs`, `irons.rs`, `helm.rs`, `no_go_zone.rs`, `polar.rs` may now fail, because the rudder is weaker at large angles. Write the failing test names and numbers into "Retune log" at the bottom. Do not edit tests yet. Task 3 handles them.

**Step 6: Commit (only the source change, on a branch, suite red is expected here)**

```bash
git add crates/sailsim/src
git commit -m "sailsim: rudder stalls near 12 degrees, consistent aspect ratio, 0.9 inflow"
```

---

## Task 3: Rudder behaviour tests and retune

**Files:**
- Create: `crates/sailsim/tests/rudder_stall.rs`
- Modify: whichever existing tests/params Task 2 broke

**Step 1: Write the behavioural tests**

```rust
use sailsim::{Controls, Scenario, Sim, SimConfig};

/// Steady yaw rate (deg/s) after `seconds` at constant rudder, no wind, 2.5 m/s start.
fn yaw_rate_deg(rudder: f64, seconds: f64) -> f64 {
    let mut sim = Sim::new(SimConfig { capsize_enabled: false, ..SimConfig::default() });
    sim.reset(Scenario {
        true_wind_speed: 0.0,
        true_wind_dir: 0.0,
        start_x: 0.0,
        start_y: 0.0,
        start_heading: 0.0,
        start_speed: 2.5,
        start_heel: 0.0,
    });
    let mut r = 0.0;
    for _ in 0..(seconds / 0.02) as usize {
        r = sim.step(Controls { rudder, sheet: 0.5, hike: 0.0 }).state.yaw_rate;
    }
    r.to_degrees()
}

#[test]
fn full_rudder_turns_the_boat_less_than_a_moderate_angle() {
    // 0.35 of 35 degrees is about 12 degrees, at the stall angle; full rudder is deep in the stall.
    let moderate = yaw_rate_deg(0.35, 2.0);
    let full = yaw_rate_deg(1.0, 2.0);
    assert!(full < moderate, "full {full} deg/s, moderate {moderate} deg/s");
}

#[test]
fn holding_the_rudder_over_costs_speed() {
    let speed = |rudder: f64| {
        let mut sim = Sim::new(SimConfig {
            fixed_heading: true,
            capsize_enabled: false,
            ..SimConfig::default()
        });
        sim.reset(Scenario {
            true_wind_speed: 0.0,
            true_wind_dir: 0.0,
            start_x: 0.0,
            start_y: 0.0,
            start_heading: 0.0,
            start_speed: 3.0,
            start_heel: 0.0,
        });
        let mut s = 0.0;
        for _ in 0..250 {
            s = sim.step(Controls { rudder, sheet: 0.5, hike: 0.0 }).state.surge;
        }
        s
    };
    // With the heading locked the rudder only adds drag.
    assert!(speed(1.0) < speed(0.0) - 0.05, "{} vs {}", speed(1.0), speed(0.0));
}
```

**Step 2: Run**

Run: `cargo test -p sailsim --test rudder_stall`
Expected: PASS. If the first test fails, report the two yaw rates. Do not loosen the assertion. The blend, the sculling term (`rudder_scull_moment`) or the 0.9 inflow may be hiding the stall. That is a finding, not a test to weaken.

**Step 3: Fix the Task 2 breakages**

For each failing test in the Retune log: read the test's comment, work out whether the old expectation came from a source (keep it and retune the parameter) or from the old tuning (update the expectation and say why). Parameters you may touch: `rudder_scull_moment`, `yaw_damping_*`, `board_arm`. Each edit gets a "RETUNED, no source" comment in the existing style with the old value. Add one line per change to the Retune log.

**Decision point:** if tacking or turning cannot be restored without breaking the stall test, stop and ask: the choice is between a lower `rudder_max_angle` (a Laser tiller does not usually go to 35 degrees) and the 470 CFD stall angle (30 degrees) with a hull-wake factor. Do not pick silently.

**Step 4: Run everything**

Run: `cargo test --workspace && (cd python && python -m pytest -q)`
Expected: PASS. If Python tests pin specific scores, the new physics changes them: update the pinned values only after confirming the change is due to this plan, and list them in the Retune log.

**Step 5: Commit**

```bash
git add crates/sailsim python
git commit -m "sailsim: rudder stall tests, retune steering after the rudder change"
```

---

## Task 4: Planing tail on the hull resistance

Above 9 kn drag area is currently constant, so R grows as v². Replace it with `R = R9 · (v / v9)^p`, p = 0.5.

**Files:**
- Modify: `crates/sailsim/src/params.rs` (new field `planing_resistance_exponent`)
- Modify: `crates/sailsim/src/hydro.rs:11-18`
- Create: `crates/sailsim/tests/planing.rs`

**Step 1: Write the failing tests**

```rust
use sailsim::{hull_resistance, BoatParams};

const KN: f64 = 0.514444;

fn r(kn: f64) -> f64 {
    -hull_resistance(&BoatParams::ilca_like(), kn * KN)
}

#[test]
fn resistance_is_continuous_across_the_end_of_the_tank_data() {
    let below = r(8.999);
    let above = r(9.001);
    assert!((above - below).abs() / below < 0.01, "{below} -> {above}");
}

#[test]
fn resistance_keeps_rising_but_slower_than_v_squared_once_planing() {
    // 9 -> 16 kn: v^2 would multiply R by 3.16; a planing hull's R rises much less.
    let ratio = r(16.0) / r(9.0);
    assert!(ratio > 1.0, "resistance fell: {ratio}");
    assert!(ratio < 2.0, "still behaving like displacement drag: {ratio}");
}

#[test]
fn resistance_to_weight_stays_below_a_planing_hulls_limit() {
    // Resistance to weight is the inverse of lift to drag; a planing dinghy hull
    // is well under 0.3 at twice its onset speed. Weight = 160 kg x 9.81.
    let weight = 160.0 * 9.81;
    assert!(r(13.0) / weight < 0.3, "R/W = {}", r(13.0) / weight);
}
```

The 0.3 bound is my judgement, not a source. Say so in the test comment when you commit.

**Step 2: Run, expect FAIL**

Run: `cargo test -p sailsim --test planing`
Expected: `resistance_keeps_rising_but_slower...` FAILS (ratio 3.16).

**Step 3: Implement**

`params.rs`: add

```rust
    /// Above the last tank-test point (9 kn) resistance follows
    /// `R = R9 (v / v9)^p`. p = 2 would be the old constant drag area; a
    /// planing hull is lower. ASSUMPTION, no source: Day and Nixon (2014) tested
    /// only to 9 kn. Confidence: low. Sensitivity checked in
    /// docs/research/2026-10-18-planing-and-rudder-sources.md.
    pub planing_resistance_exponent: f64,
```

with `planing_resistance_exponent: 0.5,`. In `hydro.rs` change the `None` arm:

```rust
        None => {
            let (v_last, a_last) = table[table.len() - 1];
            a_last * (v_last / speed).powf(2.0 - params.planing_resistance_exponent)
        }
```

**Step 4: Run**

Run: `cargo test -p sailsim --test planing && cargo test --workspace`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/sailsim
git commit -m "sailsim: planing tail on hull resistance above the 9 kn tank data"
```

---

## Task 5: Strong-wind validation and sensitivity

**Files:**
- Modify: `crates/sailsim/tests/baseline_planing_rudder.rs`
- Create: `docs/research/2026-10-18-planing-and-rudder-sources.md`

**Step 1:** Re-run the Task 0 test and compare the five speeds with the Baseline section. Beam reach speeds at 8 and 12 kn must not move (below 9 kn the table is unchanged). Only the 20 and 25 kn runs may change.

**Step 2:** Run the same reach at p = 0, 0.5 and 1.0 by temporarily editing `planing_resistance_exponent`, and record the speed at 20 and 25 kn for each. Put the three-row table into the research doc along with the evidence table from this plan. Restore p = 0.5.

**Step 3:** Pin the result with a band, not a target, since there is no source above 9 kn:

```rust
#[test]
fn strong_wind_reach_is_fast_but_not_absurd() {
    // No primary data above 9 kn. A Laser on a gusty broad reach has been
    // GPS-logged at about 15 kn (forum anecdote, with tide): treat 14 kn as
    // the ceiling for steady sailing in 25 kn.
    let kn = reach_speed(25.0);
    assert!((7.0..14.0).contains(&kn), "{kn} kn");
}
```

**Step 4:** `cargo test --workspace`, then commit with `git add crates docs && git commit -m "test: strong-wind reach band; research note with planing sensitivity"`.

---

## Task 6: Source audit of every other number ("realistic models for all numbers")

This is research first, code second. The aim is a table in `docs/research/2026-10-18-parameter-audit.md` with, for each parameter in `ilca_like()`, one of: a source and confidence, "sourced elsewhere in the existing research", or "no source found".

From `params.rs`, the ones marked `RETUNED, no source` or with no comment are:

| Parameter | What to look for |
|---|---|
| `boom_angle_in` 21 degrees | ILCA sail dimensions and mainsheet geometry, Laser rig drawings |
| `sail_heel_arm` 2.0 m, `sail_ce_aft` 0.25 m | Day (2017): sail CE 36.4 % of luff above the boom, 33 % of foot aft of luff; compute from the rig |
| `hike_arm_extra` 0.15 m | Taylor et al. (2016) hiking moments at several poses; Day (2017) crew position |
| `depower_moment` 450, `depower_floor` 0.35 | Day (2017) Fig. 8 saturation of upwind VMG; try to derive from the VPP's flat/twist/spill model instead of fitting |
| `board_arm` 0.76 m | Centre of lateral resistance vs centre of effort lead; Larsson and Eliasson (book) or a dinghy-specific paper |
| `heel_inertia`, `yaw_inertia`, `heel_damping`, `yaw_damping_*` | Radius-of-gyration rules of thumb (ITTC small craft), roll-decay tests for a dinghy hull, added-mass formulas (strip theory) |
| `sway_mass` 250 kg | Added mass in sway for a slender hull: strip theory gives about the displaced mass times a factor of 1 to 2; look for a Laser value |
| `rudder_max_angle` 35 degrees | ILCA rules or manufacturer: actual tiller throw |
| `rudder_scull_moment`, `rudder_scull_speed` | Probably no source. Mark as a modelling device, not a physical constant |
| `low_speed_lateral_*` | Probably no source. Same |
| `rudder_arm` 1.7 m | Class drawing: tiller pivot to CG |
| `board_area` 0.15 m² | Class rules area; the existing research could not find it |

**Step 1:** For each row, search the sources already listed in `docs/research/2026-10-14-dinghy-physics-validation.md` first (Day 2017 and Day and Nixon 2014 are already downloaded in the scratchpad as text), then the web. The two PDFs convert cleanly with `pdftotext -layout`; `WebFetch` cannot read them.

**Step 2:** Fill the table. Where a value is replaced by a sourced one, change it in `params.rs`, run the suite, and log the effect on the Task 0 baseline numbers. Do one parameter per commit so a regression can be bisected.

**Step 3:** Open questions that this task may not resolve (report them, do not guess): hiking limits in light air, roll damping, rudder throw, and the Nita and Scholz (2012) efficiency formula that Day (2017) cites for the rudder `efficiency` (currently an assumed 0.9).

---

## Task 7: Documentation, and the retraining consequence

**Files:**
- Modify: `README.md` ("The boat" section: the planing sentence, "Centreboard and rudder" bullet), `docs/plans/2026-10-14-dinghy-sim-design.md` (Known model limits), `docs/research/2026-10-14-dinghy-physics-validation.md` (status note on Q2e and Q5)

**Step 1:** Update the README: planing is represented by the tank hump and a low-confidence tail above 9 kn; the rudder stalls and has Day's induced drag. Leave the other README numbers alone.

**Step 2:** Note in the README results section that the trained boats (the h16 10,000 generation run, figures and the Training page replays) were evolved on the old physics. Their scores no longer correspond to the sim. Retraining is a separate piece of work: it needs the 10,000 generation run again, and I have not planned it here.

**Step 3:** Commit: `git add README.md docs && git commit -m "docs: planing tail, rudder stall, and that the trained boats predate them"`.

---

## Task 8: Engineering graphs that explain the boat dynamics

Seven figures, drawn from the sim itself (not sketches), each with the published points overlaid where a source exists. They answer: why does the boat go this fast, why does it turn like this, and why does it heel. Do this task last so the graphs show the new physics. Read @dataviz before writing any chart code.

| # | Figure | Axes | Overlay |
|---|---|---|---|
| 1 | Hull resistance and the planing hump | x boat speed (kn, with Froude number on a top axis), y resistance (N) and drag area (m²) | Day and Nixon tank points; the tail for p = 0, 0.5, 1 as a shaded band marked "assumption, no data" |
| 2 | Foil lift and drag | x angle of attack (deg), y CL, CD and CL/CD | Old and new rudder, board and sail on one panel each; stall angle marked; NACA 0012 stall band 12 to 14 deg |
| 3 | What the rudder does | x rudder angle (deg), y steady yaw rate, speed lost, and rudder lift | Marks the stall: yaw rate peaks near 12 deg and falls |
| 4 | Polar diagram | polar plot of boat speed against true wind angle at 8, 12, 16 kn, plus VMG upwind and downwind | Day (2017) and Binns points from the research doc |
| 5 | Speed against wind strength | x true wind (kn), y beam-reach speed and upwind VMG | Day (2017) VMG points (2.3, 3.25, 3.75, 3.8 kn at 5, 8, 12, 16 kn wind) |
| 6 | Heeling against righting moment | x heel (deg), y moment (N m): righting curve with and without hiking, sail heeling moment at 8, 12, 16, 20 kn | Where the curves cross is the equilibrium heel; where heeling exceeds righting is capsize |
| 7 | Time responses | two panels: acceleration from rest on a reach (speed, heel against time), and a full-rudder turn at 2.5 m/s (heading, yaw rate, speed against time), for rudder 0.35 and 1.0 | Shows the speed lost in a hard turn |

**Files:**
- Modify: `crates/sailsim/src/lib.rs` (export the two functions the plots need)
- Create: `crates/sailsim/examples/dynamics_data.rs` (writes CSVs)
- Create: `python/scripts/dynamics_figures.py` (reads CSVs, writes PNGs)
- Create: `python/tests/test_dynamics_figures.py`
- Create: `docs/boat-dynamics.md` (the figures with a paragraph each), `docs/img/dynamics/*.png`
- Modify: `README.md` (one link under "The boat")

**Step 1: Write the failing test** (`python/tests/test_dynamics_figures.py`)

```python
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPECTED = [
    "01_hull_resistance", "02_foil_curves", "03_rudder_effect", "04_polar",
    "05_speed_vs_wind", "06_heel_moments", "07_time_responses",
]


def test_every_figure_is_produced(tmp_path):
    data = tmp_path / "data"
    subprocess.run(
        ["cargo", "run", "--release", "-q", "-p", "sailsim", "--example", "dynamics_data", "--", str(data)],
        cwd=ROOT, check=True,
    )
    out = tmp_path / "img"
    subprocess.run(
        [sys.executable, str(ROOT / "python/scripts/dynamics_figures.py"), str(data), str(out)],
        check=True,
    )
    for name in EXPECTED:
        assert (out / f"{name}.png").stat().st_size > 10_000, name
```

Run: `cd python && python -m pytest tests/test_dynamics_figures.py -q`
Expected: FAIL, the example does not exist.

**Step 2: Export what the example needs**

In `crates/sailsim/src/lib.rs` add (analysis API, documented as such):

```rust
/// Exported for the analysis example and plots, not for the game loop.
pub use foil::coefficients as foil_coefficients;
pub use stability::righting_moment;
```

**Step 3: The data example** (`crates/sailsim/examples/dynamics_data.rs`). Each block writes one CSV; it uses only public items. Keep the helper functions small and the CSV headers exactly as the Python script expects.

```rust
use sailsim::{
    foil_coefficients, hull_resistance, righting_moment, BoatParams, BoatState, Controls,
    FoilParams, Scenario, Sim, SimConfig,
};
use std::{fs, io::Write, path::PathBuf};

const KN: f64 = 0.514444;

fn csv(dir: &PathBuf, name: &str, header: &str, rows: Vec<String>) {
    let mut f = fs::File::create(dir.join(name)).unwrap();
    writeln!(f, "{header}").unwrap();
    for r in rows {
        writeln!(f, "{r}").unwrap();
    }
}

/// The rudder as it was before the 2026-10-18 work, so figure 2 can show the change.
fn legacy_rudder() -> FoilParams {
    FoilParams {
        lift_slope: 3.5, stall_angle: 0.30, drag_zero_lift: 0.012, aspect_ratio: 2.5,
        efficiency: 0.7, plate_normal: 1.1, stall_sharpness: 2, max_lift: None,
    }
}

fn run(wind_kn: f64, twa_deg: f64, sheet: f64, rudder: f64, fixed: bool, secs: f64, start: f64)
    -> Vec<BoatState> {
    let mut sim = Sim::new(SimConfig { fixed_heading: fixed, capsize_enabled: false, ..SimConfig::default() });
    sim.reset(Scenario {
        true_wind_speed: wind_kn * KN, true_wind_dir: twa_deg.to_radians(),
        start_x: 0.0, start_y: 0.0, start_heading: 0.0, start_speed: start, start_heel: 0.0,
    });
    (0..(secs / 0.02) as usize)
        .map(|_| sim.step(Controls { rudder, sheet, hike: 1.0 }).state)
        .collect()
}

/// Mean of the last 30 s of a 90 s run: (speed m/s, VMG m/s toward the wind, heel rad).
fn steady(wind_kn: f64, twa: f64, sheet: f64) -> (f64, f64, f64) {
    let s = run(wind_kn, twa, sheet, 0.0, true, 90.0, 0.0);
    let tail = &s[3000..];
    let n = tail.len() as f64;
    let w = twa.to_radians();
    let speed = tail.iter().map(|b| b.surge).sum::<f64>() / n;
    let vmg = (tail.last().unwrap().x - tail[0].x) * w.sin() / 30.0
        + (tail.last().unwrap().y - tail[0].y) * w.cos() / 30.0;
    (speed, vmg, tail.iter().map(|b| b.heel).sum::<f64>() / n)
}

/// The sheet a sailor would choose: best VMG close-hauled, best speed otherwise.
fn best(wind_kn: f64, twa: f64) -> (f64, f64, f64) {
    (0..=20)
        .map(|i| steady(wind_kn, twa, i as f64 / 20.0))
        .max_by(|a, b| {
            let k = |r: &(f64, f64, f64)| if twa < 60.0 { r.1 } else { r.0 };
            k(a).total_cmp(&k(b))
        })
        .unwrap()
}

fn main() {
    let dir = PathBuf::from(std::env::args().nth(1).expect("usage: dynamics_data <out dir>"));
    fs::create_dir_all(&dir).unwrap();
    let base = BoatParams::ilca_like();

    // 1: hull resistance, tail for three exponents.
    let rows = (20..=240).map(|i| {
        let kn = i as f64 / 10.0;
        let r = |p: f64| {
            let mut b = base;
            b.planing_resistance_exponent = p;
            -hull_resistance(&b, kn * KN)
        };
        format!("{kn},{},{},{}", r(0.0), r(0.5), r(1.0))
    });
    csv(&dir, "hull.csv", "kn,r_p0,r_p05,r_p1", rows.collect());

    // 2: foil coefficients versus angle of attack.
    let foils: [(&str, FoilParams); 4] =
        [("rudder", base.rudder), ("rudder_old", legacy_rudder()), ("board", base.board), ("sail", base.sail)];
    let mut rows = vec![];
    for (name, f) in foils {
        for d in 0..=90 {
            let (cl, cd) = foil_coefficients(&f, (d as f64).to_radians());
            rows.push(format!("{name},{d},{cl},{cd}"));
        }
    }
    csv(&dir, "foils.csv", "foil,deg,cl,cd", rows);

    // 3: rudder angle sweep: steady yaw rate (free heading) and speed lost (locked heading).
    let rows = (0..=35).map(|d| {
        let u = d as f64 / 35.0;
        let yaw = run(0.0, 0.0, 0.5, u, false, 2.0, 2.5).last().unwrap().yaw_rate.to_degrees();
        let lost = 2.5 - run(0.0, 0.0, 0.5, u, true, 5.0, 2.5).last().unwrap().surge;
        format!("{d},{yaw},{lost}")
    });
    csv(&dir, "rudder_sweep.csv", "rudder_deg,yaw_rate_dps,speed_lost_ms", rows.collect());

    // 4 and 5: polar and speed against wind strength.
    let mut rows = vec![];
    for wind in [8.0, 12.0, 16.0] {
        for twa in (30..=180).step_by(10) {
            let (s, v, h) = best(wind, twa as f64);
            rows.push(format!("{wind},{twa},{s},{v},{h}"));
        }
    }
    csv(&dir, "polar.csv", "wind_kn,twa_deg,speed_ms,vmg_ms,heel_rad", rows);
    let mut rows = vec![];
    for wind in (4..=24).step_by(2) {
        let up = (38..=50).step_by(2).map(|t| best(wind as f64, t as f64).1).fold(f64::MIN, f64::max);
        rows.push(format!("{wind},{},{up}", best(wind as f64, 90.0).0));
    }
    csv(&dir, "wind_sweep.csv", "wind_kn,reach_speed_ms,upwind_vmg_ms", rows);

    // 6: righting moment (sitting and hiking) and the sail's heeling moment (from the equilibrium runs).
    let rows = (0..=90).map(|d| {
        let h = (d as f64).to_radians();
        format!("{d},{},{}", righting_moment(&base, h, 0.0), righting_moment(&base, h, 1.0))
    });
    csv(&dir, "righting.csv", "heel_deg,rm_sitting,rm_hiking", rows.collect());
    // Heeling moment is not exported by the sim; take it from equilibrium: at steady heel the
    // sail moment equals the righting moment, so plot (heel, rm_hiking(heel)) points per wind.
    let mut rows = vec![];
    for wind in [8.0, 12.0, 16.0, 20.0] {
        let (_, _, heel) = best(wind, 45.0);
        rows.push(format!("{wind},{},{}", heel.to_degrees(), righting_moment(&base, heel, 1.0)));
    }
    csv(&dir, "equilibrium.csv", "wind_kn,heel_deg,moment_nm", rows);

    // 7: time responses, 20 samples per second kept every 5th step.
    let mut rows = vec![];
    for (case, u, start, wind) in [("reach", 0.0, 0.0, 12.0), ("turn_035", 0.35, 2.5, 0.0), ("turn_100", 1.0, 2.5, 0.0)] {
        for (i, s) in run(wind, 90.0, 0.55, u, case == "reach", 15.0, start).iter().enumerate().step_by(5) {
            rows.push(format!("{case},{},{},{},{},{}", i as f64 * 0.02, s.surge, s.heel, s.heading, s.yaw_rate));
        }
    }
    csv(&dir, "time.csv", "case,t,surge,heel,heading,yaw_rate", rows);
}
```

Notes for the implementer: (a) the equilibrium heel is read from the sim, so figure 6 plots the sail's heeling moment at equilibrium (which equals the righting moment there) and does not claim a separate heeling curve; if you want the full heeling curve, export `sail_force` and `depower` from the sim and say so in the commit. (b) Check `BoatState` field names and that `FoilParams` is `pub use`d (it is, in `lib.rs`).

Run: `cargo run --release -p sailsim --example dynamics_data -- /tmp/dyn` (use the scratchpad, not `/tmp`, if your session has one).
Expected: seven or eight CSVs, runtime under a minute.

**Step 4: The plotting script** (`python/scripts/dynamics_figures.py`). Use the palette and `rcParams` block from `readme_figures.py` (import them from there rather than copying). One function per figure; this is figure 1 in full, and the others follow the table above:

```python
"""Engineering graphs of the sim's dynamics.

    .venv/bin/python python/scripts/dynamics_figures.py <csv dir> docs/img/dynamics
"""
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
from readme_figures import INK, MUTED, BLUE, ORANGE, GREEN  # also sets rcParams

KN = 0.514444
LWL = 3.81  # m, Day and Nixon (2014)
# Day and Nixon (2014) tank points, drag area m^2 at kn (hull_drag_area in params.rs).
TANK = [(2, .0125), (3, .0132), (4, .0157), (5, .0207), (6.5, .0245), (8, .0221), (9, .0206)]


def load(d, name):
    return np.genfromtxt(d / name, delimiter=",", names=True, dtype=None, encoding=None)


def hull(d, out):
    t = load(d, "hull.csv")
    fig, ax = plt.subplots(figsize=(7, 4))
    ax.fill_between(t["kn"], t["r_p0"], t["r_p1"], where=t["kn"] > 9, color=ORANGE, alpha=0.2,
                    label="tail: exponent p = 0 to 1 (assumption)")
    ax.plot(t["kn"], t["r_p05"], color=BLUE, label="sim, p = 0.5")
    pts = np.array(TANK)
    ax.plot(pts[:, 0], 0.5 * 1000 * pts[:, 1] * (pts[:, 0] * KN) ** 2, "o", color=INK,
            label="Day and Nixon (2014) tank test")
    ax.axvline(6.5, color=MUTED, ls=":")
    ax.text(6.6, ax.get_ylim()[1] * 0.05, "planing onset", color=MUTED, fontsize=9)
    top = ax.secondary_xaxis("top", functions=(lambda k: k * KN / np.sqrt(9.81 * LWL),
                                               lambda f: f * np.sqrt(9.81 * LWL) / KN))
    top.set_xlabel("length Froude number")
    ax.set(xlabel="boat speed (kn)", ylabel="hull resistance (N)", title="Hull resistance: the hump, then planing")
    ax.legend(frameon=False)
    fig.savefig(out / "01_hull_resistance.png", dpi=160, bbox_inches="tight")
    plt.close(fig)
```

Figures 2 to 7 follow the table: figure 2 is three panels (rudder old and new, board, sail) of CL and CD against angle with a vertical line at the peak CL; figure 3 is three stacked panels from `rudder_sweep.csv` with the yaw-rate peak annotated; figure 4 is a `projection="polar"` plot of `speed_ms / KN` against `twa_deg` for each wind plus a second panel of VMG; figure 5 is two lines from `wind_sweep.csv` with the Day (2017) VMG points as markers; figure 6 is the two righting curves with the equilibrium points as markers; figure 7 is two columns of time series from `time.csv`. Label every axis with units, mark every published point as a marker with its source in the legend, and title each figure with the claim it supports, not the quantity. `main()` calls all seven and creates `out`.

**Step 5: Run the test**

Run: `cd python && python -m pytest tests/test_dynamics_figures.py -q`
Expected: PASS. Then open every PNG and check it by eye: axes labelled, nothing clipped, published points visible. A passing size check does not prove a figure is readable.

**Step 6: Write `docs/boat-dynamics.md`**

One section per figure: the image, then two or three sentences saying what to look at and what it means for sailing (for example for figure 3, "past about 12 degrees of blade angle the rudder stalls and a harder pull makes the boat turn less and slow down more"). Say which numbers are sourced and which are assumptions (figure 1's tail). Add a link from the README "The boat" section.

**Step 7: Commit**

```bash
git add crates/sailsim python docs README.md
git commit -m "docs: engineering graphs of hull resistance, foils, rudder, polars, heel and time responses"
```

---

## Risks

- **Retuning sprawl.** Task 3 may need several parameter changes to restore tacking. That is the point where an existing "RETUNED" number is most likely to be re-guessed. Keep every change logged.
- **Planing tail is unverifiable.** Nothing in the open literature I found covers a Laser above 9 kn. The sensitivity table in Task 5 bounds how much it matters; it does not validate it.
- **Policies trained on the old rudder will behave differently**, probably worse at full deflection. Expect the first retrained boats to use smaller rudder angles, which is the intended effect.

## Baseline (fill in during Task 0)

```
beam reach  8 kn wind: 4.62 kn
beam reach 12 kn wind: 6.50 kn
beam reach 16 kn wind: 8.86 kn
beam reach 20 kn wind: 10.55 kn
beam reach 25 kn wind: 12.16 kn
```

## Retune log (fill in during Tasks 2 and 3)

```
(test name, old value, new value, reason)

Task 2 (rudder stall, AR 4.5, inflow 0.9), failures in `cargo test --workspace --no-fail-fast`:
test name | measured values | assertion
a_released_rudder_holds_a_beam_reach_course (tests/helm.rs:36) | heading drifted -31.94 deg in 30 s with the rudder released | drift.abs() < 5.0
(all other workspace tests pass; steering/irons/no_go_zone/polar/published_targets did not fail)

Task 3 retune (helm balance after the rudder change):
Diagnosis: the failure is a helm-balance effect. Beam drift (deg/30 s) vs board_arm at area 0.09: 0.70 -12.3, 0.72 -18.2, 0.74 -26.7, 0.76 -31.9, 0.80 -34.7, 0.84 -36.2. Vs rudder_area at board 0.76: 0.09 -31.9, 0.10 -8.4, 0.11 -1.4, 0.12 +2.9.
rudder_area | 0.09 | 0.105 | 0.81 inflow factor and slope 3.8 changed rudder force; 0.11 fixed helm.rs but broke steering.rs a_released_helm_rounds_the_boat_up_into_the_wind (close-hauled heading 0.37 deg, needs > 1)
board_arm | 0.76 | 0.75 | small trim with the area so both helm.rs (beam drift -3.4 deg) and steering.rs (close-hauled +1.7 deg) pass
No test expectation was changed.
Beam reach after Task 3 (baseline 4.62, 6.50, 8.86, 10.55, 12.16): 4.62, 6.51, 8.88, 10.57, 12.19
Rudder stall finding (tests/rudder_stall.rs): steady yaw rate is monotone in rudder (2 s: 0.1 -> 3.8 deg/s ... 1.0 -> 31.0) because the turning stern's sideways velocity (yaw_rate x 1.7 m) cuts the blade angle of attack by about 20 deg; the stall only shows in the first 0.1 s (peak at rudder 0.4-0.5: 7.6 deg/s, dip to 5.6 at 0.7).
Decision (user, after Task 3): keep stall physics; stall asserted on turn entry (0.2 s), steady yaw rate monotone by design.

Beam reach after Task 2:
beam reach 8 kn wind: 4.62 kn
beam reach 12 kn wind: 6.52 kn
beam reach 16 kn wind: 8.89 kn
beam reach 20 kn wind: 10.59 kn
beam reach 25 kn wind: 12.22 kn
```

## Sources

- Day, A.H. and Nixon, E. (2014). Measurement and prediction of the resistance of a Laser sailing dinghy. IJSCT. https://strathprints.strath.ac.uk/46407/
- Day, A.H. (2017). Performance prediction for sailing dinghies. Ocean Engineering. https://strathprints.strath.ac.uk/59980/
- Critzos, Heyson and Boswinkle (1955). NACA TN 3361. https://ntrs.nasa.gov/api/citations/19930084501/downloads/19930084501.pdf
- Sheldahl and Klimas (1981). SAND80-2114. https://digital.library.unt.edu/ark:/67531/metadc1191254/
- Lin, S. et al. (2018). Investigation on Rudder Hydrodynamics for 470 Class Yacht. Proceedings 2(6) 308. https://pdfs.semanticscholar.org/6fc7/c70307907c41e7cf68a455240f05abb2d9e8.pdf
- ILCA Class Rules 2020, World Sailing document 26057.
- Secondary or anecdotal, low confidence: Wikipedia "Hull speed"; boatdesign.net forum posts on NACA 0012 stall at Re 179,000; daysailer and sailing forum GPS reports for Laser speeds.
