# Retraining on the new physics: plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or subagent-driven-development) to run this plan task by task. This is an experiment plan: most steps run commands and read results, so "tests" here are pilot comparisons and checks, not unit tests.

**Goal:** Retrain the 16-unit neuroevolution boats on the simulator after PR #17 (rudder stall, planing tail), choose the training setup from short pilots instead of guessing, and replace the README results and figures with ones that match the current physics.

**Architecture:** Land the code first (PR #17 plus your uncommitted evolve, courses and combine work), rebuild the Rust extension, measure how the old boats fare on the new physics, run a small pilot grid of 300-generation runs, then run the chosen setup for 2 x 5,000 generations exactly as before, combine, evaluate against the scripted baseline, and refresh the README and Training page.

**Tech Stack:** `python/sailing/evolve.py`, `combine.py`, `sailpy` via maturin, `web/` (Vite, WASM) for the Training page, `python/scripts/readme_figures.py`.

**Commit rule:** no Claude attribution in commit messages or PR text (repo CLAUDE.md).

---

## What I found while planning

1. **Cost is low.** The old run's logged evaluate time was 1.7 h for the first 5,000 generations and 1.2 h for the second, about 3 h for all 10,000 on 8 cores (`timing.evaluate_s` in `training.json`, plus held-out scoring). That is cheap enough for a proper pilot grid first.
2. **Training sits entirely in the part of the sim with the weakest evidence.** `WIND_SPEED = (8, 16)` m/s is 15.5 to 31 kn, and every training leg is a reach or broad reach (65 to 140 degrees off the wind). The new physics gives about 9 to 16 kn boat speed there, which is above the 9 kn tank data, so the hull resistance comes from the assumed tail (`planing_resistance_exponent` 0.5). Sensitivity from the research note: the 25 kn beam reach moves from 12.2 kn (p = 2) to 16.0 kn (p = 0). The boats will learn whatever the tail says. There are also no waves, and 31 kn is far beyond normal Laser racing (about 25 kn is the limit).
3. **The rudder is not capped; how much to use is part of what the boats learn.** Previous runs used the full 35 degree rudder (`rudder_limit` 1.0) and so does this plan, with no cap as a variable. The rudder now stalls near 12 to 15 degrees, so full rudder is deep in the stall on turn entry and costs speed; a boat that learns to use moderate angles should beat one that does not. That makes rudder use an outcome to measure (Task 6), not a setting to choose. One trap: `--rudder-limit` defaults to 1.0 in your working tree but to 0.4 in the committed `evolve.py`, so after Task 0 check that the default is 1.0 (or pass `--rudder-limit 1.0` explicitly in every command).
4. **The training tooling lives on `parallel-training`, not on the physics branch.** Commit 3ab2642 there has `evolve.py` (width read from the weights, default 16 hidden units, default full rudder), `courses.py`, `crates/sailpy/src/lib.rs` and the optional `combine.py`. The Training page changes, README and figure edits and the `add_baseline_*` scripts are still uncommitted in the main tree. The retraining needs the committed part, so merge it with the physics branch first.
5. **The old runs stay valid as recordings.** The Training page replays stored trajectories, so `h16-5k`, `h16-10k` and `h16-full` still play. They just no longer match the simulator. Keep them under their names and give the new ones new names (`p2-*`).
6. **The scripted baseline may have shifted.** It uses fixed gains (`STEER_GAIN` 2.0, 12 degree angle of attack) tuned on the old rudder. Its score on the new physics must be re-measured before any comparison with the trained boats.

## Decisions to make before Task 3 (pilots decide most of them)

| Decision | Options | How it is decided |
|---|---|---|
| Warm start | from `h16-full/best.json`, or from scratch | Pilot: same budget, compare |
| Wind range | keep 8 to 16 m/s (comparable with the old results), or 4 to 12 m/s (mostly below 9 kn boat speed, so the sourced part of the sim) | **Your call.** My recommendation: run both in the pilot and make 4 to 12 m/s the headline run, keeping 8 to 16 as a stress test. Needs a `--wind-min/--wind-max` option (Task 2) |
| Planing exponent p | 0.5 (current), or 1.0 | Pilot at one setting, then repeat the best arm at p = 1.0. If the learned behaviour (finish rate, tacking and reaching lines) does not change, p does not matter for training; if it does, say so in the README |

---

## Task 0: Land the code

**Files:** none new; git operations only.

**Step 1:** The evolve, courses, sailpy and combine work is already committed on `parallel-training` (3ab2642). Decide what to do with the remaining uncommitted Training page, README and baseline-script changes before the merge.

**Step 2:** Merge `planing-rudder-realism` (the physics) and `parallel-training` into `main` (there is no PR; PR #17 was closed). Merge order does not matter much; if `README.md` conflicts, keep both sides (the physics branch edits The boat and Results).

**Step 3:** On the merged `main`, run `cargo test --workspace`. Expected: green.

**Step 4:** Create the training worktree: `git worktree add ../sailinggame-training -b retrain-p2 main`.

## Task 1: Rebuild and re-baseline on the new physics

**Step 1:** Rebuild the extension in the training worktree. Do not reuse the main repo's `.venv` for a different worktree without rebuilding: `python3 -m venv .venv`, then per `python/README.md`: `maturin develop --release -m crates/sailpy/Cargo.toml`. Run `.venv/bin/python -m pytest python/tests`. Expected: green.

**Step 2:** Score the scripted baseline and the old best boat on the new physics. Write `python/scripts/rescore.py`, which loads a run's `best.json`, sails it on the 10 fixed held-out courses used by `evolve.py` and on 50 fresh random courses (fixed seed), and prints finish rate, mean score, capsize rate, mean finish time. Run it for the baseline (policy "baseline") and for `web/public/runs/h16-full/best.json`.

**Step 3:** Record both rows in `docs/research/2026-10-18-retraining-results.md` as "old physics (README)" against "new physics (measured now)". Old numbers to compare with: held-out finish rate 95 %, score about 950, baseline about 1000.

**Check:** if the old boat's finish rate falls below about 50 %, warm-starting from it is probably worse than starting fresh, and the pilot will show it. If the baseline's score collapses, retune its two gains in `crates/sailcourse/src/baseline.rs` first (with a unit test on a fixed course) so the comparison is fair, and note that in the results doc.

## Task 2: Wind range option

Only if you pick the 4 to 12 m/s arm. Today `WIND_SPEED` is a module constant.

**Files:** `python/sailing/courses.py`, `python/sailing/evolve.py`, `python/tests/test_courses.py`, `python/tests/test_evolve.py`

**Step 1: Failing test** (`test_courses.py`):

```python
def test_wind_range_is_respected():
    rng = np.random.default_rng(0)
    speeds = [draw_wind(rng, wind_speed=(4.0, 12.0))[0] for _ in range(200)]
    assert 4.0 <= min(speeds) and max(speeds) <= 12.0
```

**Step 2:** Run it; expect FAIL (`draw_wind` takes no `wind_speed`).

**Step 3:** Add a `wind_speed=WIND_SPEED` keyword to `draw_wind`, thread it through `random_scenario` and the evaluator in `evolve.py`, and add `--wind-min` and `--wind-max` (m/s, defaults 8 and 16) to the CLI. Record them in the run's `config` in `training.json` so `combine` and the figures show them. The held-out courses must use the same range as training.

**Step 4:** Run the tests; expect PASS. Commit.

## Task 3: Pilot grid (300 generations each)

Three arms, all `--hidden 16 --population 100 --courses-per-gen 8 --hard-fraction 0.3 --generations 300`, two seeds each (`--seed 0` and `--seed 1`), `--workers 2` so four runs share the 8 cores:

The rudder is never capped (full 35 degrees, `--rudder-limit 1.0`), so it is not an arm.

| Arm | start | wind (m/s) |
|---|---|---|
| A | warm (`h16-full`) | 8 to 16 |
| B | scratch | 8 to 16 |
| C | scratch | 4 to 12 |

Example (arm C, seed 0):

```sh
.venv/bin/python -m sailing.evolve --out web/public/runs/pilot-C-s0 --seed 0 \
    --generations 300 --population 100 --sigma 0.1 --sigma-final 0.03 \
    --hidden 16 --courses-per-gen 8 --hard-fraction 0.3 \
    --rudder-limit 1.0 --wind-min 4 --wind-max 12 --workers 2
```

For a warm start add `--init-from web/public/runs/h16-full` and use `--sigma 0.03 --sigma-final 0.01`.

**Expected time:** about 3 to 6 minutes per 300 generations per run at 8 cores, so the six runs take under an hour.

**Step: Compare.** Write `python/scripts/compare_runs.py` that reads each `training.json` and prints, over the last 50 generations: held-out finish rate and score, population finish rate, capsize rate, and (from `best_controls`) the mean absolute rudder and the share of steps with rudder beyond 0.4 (about 14 degrees, the stall). The last two show whether the boats learned to use the rudder moderately, which is the point of leaving it uncapped.

**Decision rule (written before looking at the results):**
- Prefer the arm with the higher held-out score; a difference under one seed-to-seed spread (use the two seeds) is a tie, and ties go to the simpler setup (scratch, wind 8 to 16 for comparability).
- Report rudder use for every arm whether or not the arms tie: how much rudder the boats learn to use is a finding for the README.
- If warm start (A) is within one spread of scratch (B), start from scratch, because a clean run is easier to describe.

Commit the comparison script and the table in the results doc. Do not commit `web/public/runs` (gitignored).

## Task 4: Planing exponent check

Repeat the winning arm once with `planing_resistance_exponent` 1.0 (temporarily edit `params.rs` in the training worktree, 300 generations, one seed, rebuild the extension with `maturin develop --release`, and then revert the edit). Compare finish rate, score and mean boat speed on beam-reach legs against the p = 0.5 pilot. Write the outcome in one sentence in the results doc: either "p does not change what the boats learn" or the specific difference.

## Task 5: The full run

Use the chosen arm for 2 x 5,000 generations, same schedule as before. `combine` is an optional extra, not a required step: it only joins the two stretches into one run for viewing and for the figures (`readme_figures.py` takes the stretches separately too), and nothing in `evolve` calls it. Skip the third command if you would rather keep two separate runs.

```sh
.venv/bin/python -m sailing.evolve --out web/public/runs/p2-h16-5k \
    --generations 5000 --population 100 --sigma 0.1 --sigma-final 0.01 \
    --hidden 16 --courses-per-gen 8 --hard-fraction 0.3 <chosen flags>
.venv/bin/python -m sailing.evolve --out web/public/runs/p2-h16-10k \
    --init-from web/public/runs/p2-h16-5k \
    --generations 5000 --population 100 --sigma 0.03 --sigma-final 0.005 \
    --hidden 16 --courses-per-gen 8 --hard-fraction 0.3 <chosen flags>
.venv/bin/python -m sailing.combine --out web/public/runs/p2-h16-full \
    web/public/runs/p2-h16-5k web/public/runs/p2-h16-10k   # optional
```

Run it in the background with `caffeinate -i` so the laptop does not sleep, and check the generation count at intervals instead of tailing the log. Expected: about 3 hours on 8 cores. If a run dies, it can be continued with `--init-from` from the last `best.json`; the run directory records where it stopped.

**Checks while it runs:** at generation 500 the population finish rate should be well above the pilot's at the same generation. If it is flat or falling, stop and look before spending three hours.

## Task 6: Evaluation

**Step 1:** Run `rescore.py` on `p2-h16-full/best.json` and the baseline: finish rate, score, capsize rate and finish time on the 10 held-out and 50 fresh courses.

**Step 2:** Robustness sweep (not trained on): the best boat sails 20 fresh courses at each of 6, 10, 14, 18, 22 m/s wind. Report finish rate by wind. This is where the planing and wave assumptions show: expect the boats to be weakest outside the training range.

**Step 3:** Behaviour check: from the replays, plot rudder angle against time for one leg, and the best boat's boat speed against true wind angle. Does it use rudder angles near the stall (12 to 15 degrees) or avoid them? Put one figure in `docs/img/` and one sentence in the results doc.

**Step 4:** Compare with the old README table (finish rate 91 % population, 95 % held-out, score about 950 against a baseline of about 1000). State the comparison with both physics labelled; do not present old and new scores as one series.

## Task 7: Refresh README and the Training page

**Files:** `README.md`, `python/README.md` (if flags changed), `docs/img/learning.png`, `tracks.png`, `midrun.png`, `web/src` (only if the page needs the new run name)

**Step 1:** Regenerate the figures: `.venv/bin/python python/scripts/readme_figures.py web/public/runs/p2-h16-5k web/public/runs/p2-h16-10k docs/img`.

**Step 2:** Regenerate `docs/img/midrun.png` with the Chrome command in the README, pointing at `?run=p2-h16-10k`.

**Step 3:** Rewrite the Results section from the new `training.json` (the same table shape: finish rate and score per 2,000 generations), remove the "evolved on the earlier physics" note, and describe the setup actually used (wind range, warm or scratch, the full uncapped rudder). Keep one short paragraph saying the old run used the earlier physics and where its recordings still are, if they stay in the repo.

**Step 4:** Update the "Run it" commands to the ones in Task 5. Check by running the first command for 2 generations.

**Step 5:** `npm run dev` in `web/` and open `/training.html?run=p2-h16-10k&gen=...`: the boats must play and the baseline autopilot (WASM) must use the rebuilt module. If WASM is stale (old physics), rebuild it first.

**Step 6:** Commit and open a PR.

---

## Risks

- **The boats learn the planing assumption.** At 15 to 31 kn wind the speed ceiling comes from a tail with no data. Task 4 measures how much it matters; the README must say it is an assumption either way.
- **A stalling rudder may make evolution harder.** Early generations may spin more. If the scratch arms stay near 0 % finish rate for 100 generations, add a short curriculum (fixed heading first, `fixed_heading=True`, a switch already in `SailEnv`) before the full run.
- **Seed noise.** Two seeds per arm is thin. If two arms are close, add a third seed only for those two instead of reading a difference into the noise.
- **Disk.** Each run directory is about 60 MB; the pilot grid adds about a dozen. Delete losing pilots after the comparison table is committed.
- **Wall time on a laptop.** Use `caffeinate -i`, and do not run the full run at the same time as other heavy work.
