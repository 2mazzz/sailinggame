"""Gymnasium environment: sail the default course, scored by `sailcourse`.

Observation and reward are computed in Rust (the same code the browser runs);
this class only maps actions and reports episode state.
"""

import gymnasium as gym
import numpy as np
from gymnasium import spaces

from ._core import Core

OBS_LEN = Core.obs_len()


class SailEnv(gym.Env):
    """Actions are in [-1, 1]: rudder, sheet (mapped to 0..1 trim) and, when
    `hike=True`, hike (mapped to 0..1 lean). One step is `frame_skip` sim steps
    (0.02 s each), so the default is a decision every 0.1 s.

    Curriculum switches: `capsize=False` lets heel grow without ending the run,
    `fixed_heading=True` locks the heading so only sail trim matters.
    """

    metadata = {"render_modes": []}

    def __init__(
        self,
        capsize=True,
        fixed_heading=False,
        hike=False,
        wind_speed=6.2,
        wind_dir=0.0,
        gust=0.0,
        shift_deg=0.0,
        frame_skip=5,
        time_limit=150.0,
    ):
        self._hike = hike
        self._core = Core(
            wind_speed, wind_dir, capsize, fixed_heading, gust, shift_deg, frame_skip, time_limit
        )
        self.action_space = spaces.Box(-1.0, 1.0, shape=(3 if hike else 2,), dtype=np.float32)
        self.observation_space = spaces.Box(-3.0, 3.0, shape=(OBS_LEN,), dtype=np.float32)

    @property
    def course(self):
        flat, radius = self._core.course()
        return {"marks": [(flat[i], flat[i + 1]) for i in range(0, len(flat), 2)], "radius": radius}

    def baseline_action(self):
        """The scripted baseline agent's action now, in this env's action space."""
        rudder, sheet, hike = self._core.baseline_controls()
        action = [rudder, 2.0 * sheet - 1.0] + ([2.0 * hike - 1.0] if self._hike else [])
        return np.asarray(action, dtype=np.float32)

    def _info(self):
        x, y, heading, heel, t, score, next_mark = self._core.state()
        return {
            "x": x,
            "y": y,
            "heading": heading,
            "heel": heel,
            "time": t,
            "score": score,
            "next_mark": int(next_mark),
            "outcome": self._core.outcome(),
        }

    def reset(self, *, seed=None, options=None):
        super().reset(seed=seed)
        # An unseeded reset still draws from the env's own generator, so runs stay reproducible per env.
        run_seed = int(self.np_random.integers(0, 2**32))
        obs = self._core.reset(run_seed)
        return np.asarray(obs, dtype=np.float32), self._info()

    def step(self, action):
        a = np.asarray(action, dtype=np.float64)
        rudder = a[0]
        sheet = (a[1] + 1.0) / 2.0
        hike = (a[2] + 1.0) / 2.0 if self._hike else 0.0
        obs, reward, terminated, truncated = self._core.step(rudder, sheet, hike)
        return np.asarray(obs, dtype=np.float32), reward, terminated, truncated, self._info()
