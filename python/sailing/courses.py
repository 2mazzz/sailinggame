"""Random courses and winds for training sailors that do not just memorise one course.

A scenario is `{"marks": [(x, y), ...], "wind_speed": m/s, "wind_dir": degrees}`, the
`options` that `SailEnv.reset` takes. Courses are drawn in the wind's own frame, every
leg a reach or a broad reach (the sailor is not asked to beat or run dead downwind yet),
then turned to wherever the wind comes from, so wind direction only changes the world
orientation, not the sailing.
"""

import numpy as np

MARKS = 8
LEG_LENGTH = (32.0, 42.0)  # m, like the figure 8's 35 to 40 m legs
LEG_ANGLE = (65.0, 140.0)  # degrees off the wind, either tack
WIND_SPEED = (8.0, 16.0)  # m/s


def random_course(rng, wind_dir=0.0, marks=MARKS):
    """Marks from a start at the origin; each leg's angle to the wind is in `LEG_ANGLE`."""
    x = y = 0.0
    out = []
    for _ in range(marks):
        angle = np.radians(rng.uniform(*LEG_ANGLE) * rng.choice([-1.0, 1.0]) + wind_dir)
        length = rng.uniform(*LEG_LENGTH)
        x += length * np.sin(angle)
        y += length * np.cos(angle)
        out.append((round(float(x), 2), round(float(y), 2)))
    return out


def random_scenario(rng, random_wind=True):
    """A random course in the given wind; with `random_wind=False` the wind is the
    default 13 m/s from the north every time."""
    if random_wind:
        speed, direction = float(rng.uniform(*WIND_SPEED)), float(rng.uniform(0.0, 360.0))
    else:
        speed, direction = 13.0, 0.0
    return {"marks": random_course(rng, direction), "wind_speed": speed, "wind_dir": direction}
