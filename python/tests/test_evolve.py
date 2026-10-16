import json

import numpy as np

from sailing import evolve


def test_a_policy_maps_observations_to_actions_in_range_deterministically():
    rng = np.random.default_rng(0)
    params = rng.normal(size=evolve.PARAM_COUNT)
    obs = rng.uniform(-1, 1, size=evolve.OBS_LEN)
    a = evolve.act(params, obs)
    assert a.shape == (3,) and np.all(np.abs(a) <= 1.0)  # rudder, sheet, hike
    assert np.array_equal(a, evolve.act(params, obs))


def test_a_run_logs_each_generation_and_records_replays(tmp_path):
    out = tmp_path / "run"
    evolve.run(out, population=12, generations=4, workers=1, seed=1, time_limit=100.0)
    log = json.loads((out / "training.json").read_text())
    assert len(log["generations"]) == 4
    g = log["generations"][0]
    for key in ("gen", "best", "mean", "median", "finish_rate", "capsize_rate", "mean_finish_time", "fastest_finish_time"):
        assert key in g
    # How the best boat used each control: rudder activity, sheet range and hike lean, all 0 to 1.
    for key in ("rudder", "sheet", "hike"):
        assert 0.0 <= g["best_controls"][key] <= 1.0
    assert log["config"]["rudder_limit"] == 0.4
    assert log["course"]["radius"] == 6.0 and len(log["course"]["marks"]) == 8
    assert log["baseline"]["outcome"] in {"finished", "capsized", "time_limit"}
    # Each generation sails its own course and wind, and the best boat is also tried on held-out ones.
    assert len({(g["wind_speed"], g["wind_dir"]) for g in log["generations"]}) == 4
    assert 8.0 <= g["wind_speed"] <= 16.0 and 0.0 <= g["wind_dir"] < 360.0
    assert 0.0 <= g["held_out"]["finish_rate"] <= 1.0 and "figure8_score" in g["held_out"]
    # The first and last generations are checkpoints with one recorded run per boat.
    assert log["checkpoints"][0] == 0 and log["checkpoints"][-1] == 3
    replay = json.loads((out / "gen_0003.json").read_text())
    assert len(replay["boats"]) == 12
    boat = replay["boats"][0]
    assert len(boat["p"]) % 4 == 0 and len(boat["p"]) >= 8
    # Boats are sorted best first, by their mean score over the generation's courses.
    fitness = [b["fitness"] for b in replay["boats"]]
    assert fitness == sorted(fitness, reverse=True)


def test_the_same_seed_gives_the_same_run(tmp_path):
    for name in ("a", "b"):
        evolve.run(tmp_path / name, population=10, generations=3, workers=1, seed=7, time_limit=20.0)
    a, b = (_comparable(json.loads((tmp_path / n / "training.json").read_text()))[0] for n in ("a", "b"))
    assert a == b


def _run_logged(tmp_path, name, **kwargs):
    out = tmp_path / name
    evolve.run(out, population=10, generations=3, seed=3, time_limit=40.0, **kwargs)
    log = json.loads((out / "training.json").read_text())
    replay = json.loads((out / "gen_0002.json").read_text())
    return log, replay


def _comparable(log):
    """The log without wall-clock timings, which differ from run to run."""
    gens = [{k: v for k, v in g.items() if k != "timing"} for g in log["generations"]]
    return gens, log["baseline"], log["checkpoints"]


def test_the_same_seed_gives_the_same_run_on_every_backend(tmp_path):
    reference, ref_replay = _run_logged(tmp_path, "py1", backend="python", workers=1)
    for name, kwargs in (("py2", dict(backend="python", workers=2)), ("rust", dict(backend="rust", workers=2))):
        log, replay = _run_logged(tmp_path, name, **kwargs)
        ref_gens, got_gens = _comparable(reference)[0], _comparable(log)[0]
        assert len(got_gens) == len(ref_gens)
        for want, got in zip(ref_gens, got_gens):
            assert got["finish_rate"] == want["finish_rate"] and got["capsize_rate"] == want["capsize_rate"]
            assert np.isclose(got["best"], want["best"], atol=1.0), name
            assert np.isclose(got["held_out"]["score"], want["held_out"]["score"], atol=1.0), name
        assert [b["outcome"] for b in replay["boats"]] == [b["outcome"] for b in ref_replay["boats"]], name


def test_every_generation_logs_where_its_time_went(tmp_path):
    out = tmp_path / "run"
    evolve.run(out, population=10, generations=3, seed=2, time_limit=20.0, workers=1)
    for g in json.loads((out / "training.json").read_text())["generations"]:
        assert g["timing"]["evaluate_s"] > 0 and g["timing"]["baseline_s"] >= 0


def test_every_boat_is_scored_on_several_courses_and_the_score_is_the_mean(tmp_path):
    out = tmp_path / "multi"
    evolve.run(out, population=8, generations=2, workers=1, seed=2, time_limit=30.0, courses_per_gen=3)
    log = json.loads((out / "training.json").read_text())
    assert log["config"]["courses_per_gen"] == 3
    best = json.loads((out / "best.json").read_text())
    # Re-sail the best boat of the last generation on that generation's three courses.
    from sailing.courses import random_scenario

    scenarios = [random_scenario(np.random.default_rng([2, 2, 3 + j]), hard=0.3) for j in range(3)]
    assert len(scenarios) == 3
    rows = evolve._core.evaluate_episodes(
        [best["params"]] * 3, [evolve._scenario_args(sc) for sc in scenarios], 0.4, 30.0, 0, 1
    )
    assert abs(np.mean([r[0] for r in rows]) - best["score"]) < 1e-6
    # The finish rate counts every episode (8 boats x 3 courses).
    g = log["generations"][0]
    assert abs(g["finish_rate"] * 24 - round(g["finish_rate"] * 24)) < 1e-9


def test_a_course_with_the_wrong_number_of_marks_is_refused():
    import pytest

    with pytest.raises(ValueError):
        evolve._scenario_args({"marks": [(0.0, 10.0)] * 3, "wind_speed": 10.0, "wind_dir": 0.0})
