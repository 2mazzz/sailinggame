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

`SailEnv(capsize=True, fixed_heading=False, hike=False, wind_speed=6.2, wind_dir=0.0, gust=0.0, shift_deg=0.0, frame_skip=5, time_limit=150.0)`

- Actions in [-1, 1]: rudder, sheet, and hike when `hike=True`.
- Observation: 13 inputs from `sailcourse::observation` (see that file), the same ones the browser will use.
- Reward: `sailcourse` score per step (progress, mark bonuses, a finish bonus for speed, a time cost, a capsize penalty).
- `reset(seed=...)` seeds gusts and shifts; the same seed gives the same episode.
