import numpy as np
import pytest
from gymnasium.utils.env_checker import check_env

from sailing import SailEnv


def rollout(env, seed, actions):
    obs, _ = env.reset(seed=seed)
    trace = [obs.copy()]
    rewards = []
    for a in actions:
        obs, r, terminated, truncated, info = env.step(a)
        trace.append(obs.copy())
        rewards.append(r)
        if terminated or truncated:
            break
    return np.array(trace), np.array(rewards), info


def random_actions(n, size, seed=0):
    rng = np.random.default_rng(seed)
    return rng.uniform(-1, 1, size=(n, size)).astype(np.float32)


def test_passes_the_standard_environment_checker():
    check_env(SailEnv(), skip_render_check=True)


def test_a_random_agent_runs_to_termination_or_the_time_limit():
    env = SailEnv(time_limit=60.0)
    env.reset(seed=0)
    rng = np.random.default_rng(1)
    for steps in range(1, 10_000):
        _, _, terminated, truncated, info = env.step(env.action_space.sample())
        if terminated or truncated:
            break
    assert terminated or truncated
    assert info["outcome"] in {"finished", "capsized", "time_limit"}
    # 60 s at 10 decisions per second.
    assert steps <= 601


def test_the_same_seed_gives_the_same_episode():
    # Shifts make the seed matter: the wind direction wanders deterministically.
    kwargs = dict(shift_deg=15.0, gust=0.3, time_limit=40.0)
    actions = random_actions(400, 2)
    a, ra, _ = rollout(SailEnv(**kwargs), 5, actions)
    b, rb, _ = rollout(SailEnv(**kwargs), 5, actions)
    c, _, _ = rollout(SailEnv(**kwargs), 6, actions)
    assert np.array_equal(a, b) and np.array_equal(ra, rb)
    assert not np.array_equal(a, c)


def test_rewards_add_up_to_the_score():
    _, rewards, info = rollout(SailEnv(time_limit=30.0), 0, random_actions(300, 2, seed=3))
    assert rewards.sum() == pytest.approx(info["score"], abs=1e-3)


def test_capsize_can_be_switched_off():
    # 16 m/s on a beam reach (the env starts there), sheeted hard in, heading locked:
    # capsizes when allowed.
    kwargs = dict(wind_speed=16.0, fixed_heading=True, time_limit=60.0)
    hard_in = np.array([0.0, 1.0], dtype=np.float32)
    _, _, on = rollout(SailEnv(capsize=True, **kwargs), 0, [hard_in] * 700)
    _, _, off = rollout(SailEnv(capsize=False, **kwargs), 0, [hard_in] * 700)
    assert on["outcome"] == "capsized"
    assert off["outcome"] != "capsized"


def test_a_fixed_heading_ignores_the_rudder():
    env = SailEnv(fixed_heading=True, time_limit=20.0)
    env.reset(seed=0)
    start = None
    for _ in range(100):
        _, _, _, _, info = env.step(np.array([1.0, 0.0], dtype=np.float32))
        start = info["heading"] if start is None else start
        assert info["heading"] == pytest.approx(start)


def test_hike_is_part_of_the_action_space_only_when_asked():
    assert SailEnv().action_space.shape == (2,)
    assert SailEnv(hike=True).action_space.shape == (3,)


def test_observation_matches_its_declared_space():
    env = SailEnv()
    obs, _ = env.reset(seed=0)
    assert env.observation_space.contains(obs)


def test_the_baseline_action_finishes_the_course_and_the_course_is_exposed():
    env = SailEnv(time_limit=200.0, wind_speed=11.0)  # the baseline does not survive the default 13 m/s
    obs, info = env.reset(seed=0)
    assert len(env.course["marks"]) == 8 and env.course["radius"] == 6.0
    terminated = truncated = False
    while not (terminated or truncated):
        obs, _, terminated, truncated, info = env.step(env.baseline_action())
    assert info["outcome"] == "finished"
    assert info["score"] > 400
