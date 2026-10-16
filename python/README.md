# sailing (Python)

Gymnasium environment on the Rust simulation and course (issue #10).

```sh
python3 -m venv .venv
.venv/bin/pip install maturin gymnasium numpy pytest
# If conda is active, unset CONDA_PREFIX first (maturin refuses both).
env -u CONDA_PREFIX VIRTUAL_ENV=$PWD/.venv PATH=$PWD/.venv/bin:$PATH \
  maturin develop --release -m crates/sailpy/Cargo.toml
.venv/bin/python -m pytest python/tests
```

`SailEnv(rudder_limit=1.0, capsize=True, fixed_heading=False, hike=False, wind_speed=13.0, wind_dir=0.0, gust=0.0, shift_deg=0.0, frame_skip=5, time_limit=150.0)`

- Actions in [-1, 1]: rudder, sheet, and hike when `hike=True`. `rudder_limit` caps the rudder at that fraction of full deflection (35 degrees); evolution defaults to 0.4 (about 14 degrees, near where a rudder stalls).
- Observation: 14 inputs from `sailcourse::observation` (see that file), the same ones the browser will use.
- Reward: `sailcourse` score per step (progress, mark bonuses, a finish bonus for speed, a time cost, a capsize penalty).
- `reset(seed=...)` seeds gusts and shifts; the same seed gives the same episode. `reset(options={"marks": [(x, y), ...], "wind_speed": 12.0, "wind_dir": 90.0})` swaps in a course and wind (they stay until changed).

## Evolution

```sh
.venv/bin/python -m sailing.evolve --out web/public/runs/maps-1000x10 --generations 10000 --population 100 --gens-per-map 10
```

Each generation 100 small networks all sail the same freshly drawn random course (8 reach legs, `sailing/courses.py`) in a random wind (8 to 16 m/s, any direction, no gusts); the best boat is also sailed every generation on 10 fixed held-out courses and on the old figure 8 (`held_out` in `training.json`). `--gens-per-map N` keeps each course for N generations in a row (10 gives 1000 courses over 10000 generations). `--fixed` restores the old single course and wind. Each generation (and so each replay) has a different course; the score of the scripted baseline on it is logged as `baseline_score`. Elite boats are re-scored on every new course, so the best score can drop.

The best 10 % carry over unchanged and the rest are mutated copies of the best 30 %. About 25 s for 100 generations on 8 cores. It writes `training.json` (per-generation best, mean, median, finish and capsize rates, mean finish time, the course, and the scripted baseline for reference) and a `gen_NNNN.json` replay of all boats for checkpoint generations. `web/public/runs/` is not committed; the viewer's Training page reads it.

## Viewing a run

`npm run dev` in `web/`, then open `/training.html` (or `/training.html?run=NAME&gen=99&t=40` to freeze the replay at generation 99, second 40). The page shows the score, finish and capsize rate and finish time per generation, and replays every boat of a checkpoint generation at once as ghosts with wakes. Click or drag on a graph, drag the slider, or use the left and right arrow keys to move between checkpoint generations; Space pauses.
