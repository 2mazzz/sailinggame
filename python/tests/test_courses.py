import numpy as np
import pytest

from sailing import SailEnv
from sailing.courses import LEG_ANGLE, LEG_LENGTH, MARKS, WIND_SPEED, random_course, random_scenario


def _leg_angles_to_wind(marks, wind_dir):
    pts = np.array([(0.0, 0.0)] + list(marks))
    d = np.diff(pts, axis=0)
    bearing = np.degrees(np.arctan2(d[:, 0], d[:, 1]))
    return np.abs((bearing - wind_dir + 180.0) % 360.0 - 180.0), np.hypot(d[:, 0], d[:, 1])


@pytest.mark.parametrize("wind_dir", [0.0, 77.0, 250.0])
def test_every_leg_is_a_reach_whatever_the_wind_direction(wind_dir):
    marks = random_course(np.random.default_rng(3), wind_dir)
    assert len(marks) == MARKS
    angle, length = _leg_angles_to_wind(marks, wind_dir)
    assert np.all((angle >= LEG_ANGLE[0] - 0.1) & (angle <= LEG_ANGLE[1] + 0.1))
    assert np.all((length >= LEG_LENGTH[0] - 0.1) & (length <= LEG_LENGTH[1] + 0.1))


def test_scenarios_are_reproducible_and_the_wind_varies():
    a = random_scenario(np.random.default_rng([1, 2]))
    assert a == random_scenario(np.random.default_rng([1, 2]))
    speeds = [random_scenario(np.random.default_rng(i))["wind_speed"] for i in range(30)]
    assert WIND_SPEED[0] <= min(speeds) and max(speeds) <= WIND_SPEED[1] and max(speeds) - min(speeds) > 3
    assert random_scenario(np.random.default_rng(0), random_wind=False)["wind_dir"] == 0.0


def test_the_baseline_sails_random_courses_in_any_wind_direction():
    # The same course turned with the wind is the same sailing problem, so it takes about as long.
    times = []
    for turn in (0.0, 120.0):
        scenario = random_scenario(np.random.default_rng(5), random_wind=False)
        angle = np.radians(turn)
        scenario["marks"] = [
            (x * np.cos(angle) + y * np.sin(angle), -x * np.sin(angle) + y * np.cos(angle))
            for x, y in scenario["marks"]
        ]
        scenario["wind_dir"] = turn
        env = SailEnv(time_limit=150.0, hike=True)
        env.reset(seed=0, options=scenario)
        done = False
        while not done:
            _, _, terminated, truncated, info = env.step(env.baseline_action())
            done = terminated or truncated
        times.append((info["outcome"], info["time"]))
    assert times[0][0] == times[1][0]
    assert abs(times[0][1] - times[1][1]) < 3.0, times


def test_hard_courses_have_more_sharp_reversals_and_close_legs_and_stay_legal():
    def turn_and_closest(hard, wind_dir=0.0):
        sharp = close = 0
        for i in range(200):
            marks = random_course(np.random.default_rng(i), wind_dir, hard=hard)
            assert len(marks) == MARKS
            angle, length = _leg_angles_to_wind(marks, wind_dir)
            assert np.all((angle >= LEG_ANGLE[0] - 0.1) & (angle <= LEG_ANGLE[1] + 0.1))
            pts = np.array([(0.0, 0.0)] + list(marks))
            d = np.diff(pts, axis=0)
            bearing = np.degrees(np.arctan2(d[:, 0], d[:, 1]))
            turns = np.abs((np.diff(bearing) + 180.0) % 360.0 - 180.0)
            sharp += int((turns > 165.0).sum())
            close += int((angle < 76.0).sum())
        return sharp, close

    plain, hard = turn_and_closest(0.0), turn_and_closest(0.6)
    assert hard[0] > 2 * plain[0] and hard[1] > 2 * plain[1]


def test_plain_courses_are_unchanged_by_the_hard_option_existing():
    a = random_course(np.random.default_rng(4), 30.0)
    assert a == random_course(np.random.default_rng(4), 30.0, hard=0.0)
