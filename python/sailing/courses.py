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


CLOSE_ANGLE = (65.0, 75.0)  # degrees off the wind: the closest legs the boat is asked to sail
REVERSAL_SPREAD = 15.0  # degrees either side of straight back along the previous leg


def _hard_angle(rng, previous):
    """Signed leg angle to the wind (degrees) for a hard leg: a reversal (almost straight
    back along the previous leg) when that is a legal leg, else a leg close to the wind."""
    if previous is not None:
        back = previous + 180.0 + rng.uniform(-REVERSAL_SPREAD, REVERSAL_SPREAD)
        back = (back + 180.0) % 360.0 - 180.0
        if LEG_ANGLE[0] <= abs(back) <= LEG_ANGLE[1]:
            return back
    return rng.uniform(*CLOSE_ANGLE) * rng.choice([-1.0, 1.0])


def random_course(rng, wind_dir=0.0, marks=MARKS, hard=0.0):
    """Marks from a start at the origin; each leg's angle to the wind is in `LEG_ANGLE`.

    With `hard` above 0 that share of legs (on average) is a hard one: a sharp reversal
    of the previous leg or a leg close to the wind, the cases a boat trained on plain
    random courses sails worst."""
    x = y = 0.0
    out = []
    previous = None
    for _ in range(marks):
        if hard > 0.0 and rng.random() < hard:
            angle_to_wind = _hard_angle(rng, previous)
        else:
            angle_to_wind = rng.uniform(*LEG_ANGLE) * rng.choice([-1.0, 1.0])
        previous = angle_to_wind
        angle = np.radians(angle_to_wind + wind_dir)
        length = rng.uniform(*LEG_LENGTH)
        x += length * np.sin(angle)
        y += length * np.cos(angle)
        out.append((round(float(x), 2), round(float(y), 2)))
    return out


def random_scenario(rng, random_wind=True, hard=0.0):
    """A random course in the given wind; with `random_wind=False` the wind is the
    default 13 m/s from the north every time. `hard` is as in `random_course`. Every
    course has exactly `MARKS` marks, which is what the sailor's observation shows."""
    if random_wind:
        speed, direction = float(rng.uniform(*WIND_SPEED)), float(rng.uniform(0.0, 360.0))
    else:
        speed, direction = 13.0, 0.0
    return {"marks": random_course(rng, direction, hard=hard), "wind_speed": speed, "wind_dir": direction}
