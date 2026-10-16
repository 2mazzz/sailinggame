"""Evolve a population of small neural-net sailors on the course.

Each generation every boat sails the same freshly drawn random course in a random
wind (see `courses`); the best few are kept as they are (elitism, and they are
re-scored on the next course, so only boats that hold up stay on top) and the
rest of the next generation are mutated copies of good boats. Every generation
the best boat is also sailed on a fixed held-out set of random courses and on the
old figure 8. Per-generation statistics and the
recorded runs of checkpoint generations are written as JSON for the viewer.
"""

import json
import multiprocessing as mp
import os
import time
from pathlib import Path

import numpy as np

from .courses import random_scenario
from . import _core
from .env import OBS_LEN, SailEnv

HIDDEN = 16
ACTIONS = 3  # rudder, sheet, hike
# Weights and biases of obs -> hidden -> actions, as one flat vector.
PARAM_COUNT = OBS_LEN * HIDDEN + HIDDEN + HIDDEN * ACTIONS + ACTIONS
RECORD_EVERY = 2  # keep every 2nd decision: 5 points a second


def act(params, obs):
    """The policy: a two-layer network with tanh outputs in [-1, 1]."""
    i = OBS_LEN * HIDDEN
    w1 = params[:i].reshape(OBS_LEN, HIDDEN)
    b1 = params[i : i + HIDDEN]
    j = i + HIDDEN
    w2 = params[j : j + HIDDEN * ACTIONS].reshape(HIDDEN, ACTIONS)
    b2 = params[j + HIDDEN * ACTIONS :]
    h = np.tanh(obs @ w1 + b1)
    return np.tanh(h @ w2 + b2)


_env = None


def _init_worker(time_limit, rudder_limit):
    global _env
    _env = SailEnv(time_limit=time_limit, hike=True, rudder_limit=rudder_limit)


def _pack(info, out):
    out += [
        round(info["x"] * 10),
        round(info["y"] * 10),
        round(info["heading"] * 100),
        round(info["heel"] * 100),
    ]


def _episode(params, record, scenario=None, policy=None):
    """Sail one episode; returns its summary and, when asked, the recorded path."""
    env = _env
    obs, info = env.reset(seed=0, options=scenario)
    path = []
    if record:
        _pack(info, path)
    done = False
    step = 0
    actions = []
    while not done:
        action = env.baseline_action() if policy == "baseline" else act(params, obs)
        actions.append(action)
        obs, _, terminated, truncated, info = env.step(action)
        done = terminated or truncated
        step += 1
        if record and (step % RECORD_EVERY == 0 or done):
            _pack(info, path)
    a = np.array(actions)
    sheet = (a[:, 1] + 1.0) / 2.0
    result = {
        "score": info["score"],
        "outcome": info["outcome"],
        "time": info["time"],
        # How the control was used this episode, each 0 to 1: mean rudder
        # deflection, how much of its range the sheet swept, mean hike lean.
        "controls": {
            "rudder": float(np.abs(a[:, 0]).mean()),
            "sheet": float(sheet.max() - sheet.min()),
            "hike": float(((a[:, 2] + 1.0) / 2.0).mean()),
        },
    }
    if record:
        result["p"] = path
    return result


def _evaluate(args):
    params, record, scenario = args
    return _episode(params, record, scenario)


def _baseline(scenario):
    return _episode(None, False, scenario, policy="baseline")


def _scenario_args(sc):
    flat = [float(v) for m in sc["marks"] for v in m]
    return flat, float(sc["wind_speed"]), float(sc["wind_dir"])


class _RustEvaluator:
    """Sails a batch of episodes inside Rust, in parallel across cores, no Python in the step loop."""

    def __init__(self, workers, time_limit, rudder_limit):
        self.workers, self.time_limit, self.rudder_limit = workers, time_limit, rudder_limit
        _init_worker(time_limit, rudder_limit)  # the scripted baseline still runs through the env

    def episodes(self, params, scenarios, record, hints=None):
        rows = _core.evaluate_episodes(
            [list(map(float, p)) for p in params],
            [_scenario_args(sc) for sc in scenarios],
            self.rudder_limit,
            self.time_limit,
            RECORD_EVERY if any(record) else 0,
            self.workers,
        )
        out = []
        for (score, outcome, t, rudder, sheet, hike, path), rec in zip(rows, record):
            r = {
                "score": score,
                "outcome": outcome,
                "time": t,
                "controls": {"rudder": rudder, "sheet": sheet, "hike": hike},
            }
            if rec:
                r["p"] = path
            out.append(r)
        return out

    def baseline(self, scenario):
        return _baseline(scenario)

    def close(self):
        pass


class _PythonEvaluator:
    """The reference path: one boat at a time through the Gymnasium env, over a process pool."""

    def __init__(self, workers, time_limit, rudder_limit):
        self.pool = None
        if workers > 1:
            self.pool = mp.get_context("spawn").Pool(workers, _init_worker, (time_limit, rudder_limit))
        else:
            _init_worker(time_limit, rudder_limit)

    def episodes(self, params, scenarios, record, hints=None):
        jobs = [(p, rec, sc) for p, sc, rec in zip(params, scenarios, record)]
        if self.pool is None:
            return [_evaluate(j) for j in jobs]
        # One episode at a time to whichever worker is free, the longest expected first.
        order = sorted(range(len(jobs)), key=lambda i: -(hints[i] if hints is not None else 0.0))
        out = [None] * len(jobs)
        for i, r in self.pool.imap_unordered(_indexed, [(i, jobs[i]) for i in order], chunksize=1):
            out[i] = r
        return out

    def baseline(self, scenario):
        return _baseline(scenario) if self.pool is None else self.pool.apply(_baseline, (scenario,))

    def close(self):
        if self.pool:
            self.pool.close()
            self.pool.join()


def _indexed(arg):
    i, job = arg
    return i, _evaluate(job)


# The old course and wind, kept as one held-out check.
FIGURE_8 = {"marks": SailEnv().course["marks"], "wind_speed": 13.0, "wind_dir": 0.0}
HELD_OUT = 10


def _stats(gen, results):
    scores = np.array([r["score"] for r in results])
    top = max(results, key=lambda r: r["score"])
    finished = [r["time"] for r in results if r["outcome"] == "finished"]
    n = len(results)
    return {
        "gen": gen,
        "best": float(scores.max()),
        "mean": float(scores.mean()),
        "median": float(np.median(scores)),
        "finish_rate": len(finished) / n,
        "capsize_rate": sum(r["outcome"] == "capsized" for r in results) / n,
        "mean_finish_time": float(np.mean(finished)) if finished else None,
        "fastest_finish_time": float(min(finished)) if finished else None,
        "best_controls": top["controls"],
    }


def checkpoint_gens(generations):
    """Early generations densely, later ones sparsely, always the last."""
    picks = {0, 1, 2, 3, 5, 8, 12, 18, 25, 35, 50, 70, 100, 140, 200, 280, 350, 450, 560, 700, 850, 1000, 1250, 1500, 1750}
    picks |= set(range(100, 2000, 100))  # then every 100th
    picks |= set(range(2000, generations, 500))  # and every 500th once runs get long
    gens = {g for g in picks if g < generations}
    gens.add(generations - 1)
    return sorted(gens)


def run(
    out_dir,
    population=100,
    generations=60,
    workers=None,
    seed=0,
    time_limit=150.0,
    sigma=0.3,
    sigma_final=0.03,
    rudder_limit=0.4,
    elite_fraction=0.1,
    parent_fraction=0.3,
    init_scale=0.5,
    randomize=True,
    gens_per_map=1,
    backend="rust",
):
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    rng = np.random.default_rng(seed)
    workers = workers or max(1, (os.cpu_count() or 2) - 1)
    elite = max(1, int(population * elite_fraction))
    parents = max(elite, int(population * parent_fraction))
    checkpoints = checkpoint_gens(generations)

    pop = rng.normal(0.0, init_scale, size=(population, PARAM_COUNT))
    probe = SailEnv(time_limit=time_limit, hike=True, rudder_limit=rudder_limit)
    course = probe.course
    held_out = [random_scenario(np.random.default_rng([seed, 1, i])) for i in range(HELD_OUT)]
    log = {
        "config": {
            "population": population,
            "generations": generations,
            "seed": seed,
            "sigma": sigma,
            "sigma_final": sigma_final,
            "rudder_limit": rudder_limit,
            "elite": elite,
            "time_limit": time_limit,
            "hidden": HIDDEN,
            "randomize": randomize,
            "gens_per_map": gens_per_map,
            "backend": backend,
            "wind_speed": probe.wind_speed,
            "wind_dir": probe.wind_dir,
        },
        "course": {"marks": course["marks"], "radius": course["radius"]},
        "checkpoints": checkpoints,
        "generations": [],
    }

    evaluator = (_RustEvaluator if backend == "rust" else _PythonEvaluator)(workers, time_limit, rudder_limit)
    baselines = {}  # the scripted baseline's score per course, scored once however many generations use it

    def baseline_score(key, scenario):
        if key not in baselines:
            baselines[key] = evaluator.baseline(scenario)["score"]
        return baselines[key]

    def held_out_stats(tests):
        return {
            "score": float(np.mean([t["score"] for t in tests[:-1]])),
            "finish_rate": float(np.mean([t["outcome"] == "finished" for t in tests[:-1]])),
            "figure8_score": tests[-1]["score"],
            "figure8_outcome": tests[-1]["outcome"],
        }

    suite = held_out + [FIGURE_8]
    pending = None  # (stats, best weights) of the last generation, whose held-out check is still to sail
    times = None  # each boat's episode time last generation, to order the next batch longest first
    try:
        baseline = evaluator.baseline(FIGURE_8)
        log["baseline"] = {k: baseline[k] for k in ("score", "time", "outcome", "controls")}
        for gen in range(generations):
            record = gen in checkpoints
            # One course and wind for the whole generation, so boats are ranked on equal terms;
            # the same one for `gens_per_map` generations in a row before a new one is drawn.
            map_id = gen // gens_per_map
            scenario = random_scenario(np.random.default_rng([seed, 2, map_id])) if randomize else FIGURE_8
            # The previous generation's best boat is tried on the held-out courses in the same batch.
            extra = suite if pending else []
            t0 = time.perf_counter()
            batch = evaluator.episodes(
                list(pop) + ([pending[1]] * len(extra) if extra else []),
                [scenario] * len(pop) + extra,
                [record] * len(pop) + [False] * len(extra),
                None if times is None else list(times) + [time_limit] * len(extra),
            )
            evaluate_s = time.perf_counter() - t0
            results, tests = batch[: len(pop)], batch[len(pop) :]
            if pending:
                pending[0]["held_out"] = held_out_stats(tests)
                pending[0]["timing"]["held_out_batched"] = True
                log["generations"].append(pending[0])
            order = np.argsort([-r["score"] for r in results], kind="stable")
            stats = _stats(gen, results)
            stats["wind_speed"], stats["wind_dir"] = scenario["wind_speed"], scenario["wind_dir"]
            t0 = time.perf_counter()
            stats["baseline_score"] = baseline_score(map_id, scenario)
            stats["timing"] = {"evaluate_s": evaluate_s, "baseline_s": time.perf_counter() - t0}
            pending = (stats, pop[order[0]].copy())
            if record:
                boats = [
                    {k: results[i][k] for k in ("score", "outcome", "time", "p")} for i in order
                ]
                (out / f"gen_{gen:04d}.json").write_text(
                    json.dumps(
                        {
                            "gen": gen,
                            "dt": 0.1 * RECORD_EVERY,
                            "course": {"marks": scenario["marks"], "radius": course["radius"]},
                            "wind": {"speed": scenario["wind_speed"], "dir": scenario["wind_dir"]},
                            "boats": boats,
                        }
                    )
                )
            (out / "training.json").write_text(json.dumps(log))
            (out / "best.json").write_text(
                json.dumps({"gen": gen, "score": results[order[0]]["score"], "params": pop[order[0]].tolist()})
            )
            # Next generation: keep the elite, fill the rest with mutated good boats.
            ranked = pop[order]
            parent = rng.integers(0, parents, size=population - elite)
            children = ranked[parent]
            # Wide steps early to explore, small ones late to polish.
            step = sigma + (sigma_final - sigma) * gen / max(1, generations - 1)
            children = children + rng.normal(0.0, step, size=children.shape)
            ranked_times = np.array([results[i]["time"] for i in order])
            times = np.concatenate([ranked_times[:elite], ranked_times[parent]])
            pop = np.vstack([ranked[:elite], children])
        # The last generation's held-out check has no next batch to ride along with.
        pending[0]["held_out"] = held_out_stats(
            evaluator.episodes([pending[1]] * len(suite), suite, [False] * len(suite))
        )
        log["generations"].append(pending[0])
        (out / "training.json").write_text(json.dumps(log))
    finally:
        evaluator.close()
    return log


def main():
    import argparse

    p = argparse.ArgumentParser(description="Evolve sailors on the course; writes JSON for the viewer.")
    p.add_argument("--out", default="web/public/runs/evolution-1")
    p.add_argument("--population", type=int, default=100)
    p.add_argument("--generations", type=int, default=60)
    p.add_argument("--workers", type=int, default=None)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--sigma", type=float, default=0.3)
    p.add_argument("--sigma-final", type=float, default=0.03)
    p.add_argument("--rudder-limit", type=float, default=0.4, help="largest rudder as a fraction of full (35 deg)")
    p.add_argument("--time-limit", type=float, default=150.0)
    p.add_argument("--gens-per-map", type=int, default=1, help="generations sailed on each random course before a new one")
    p.add_argument("--backend", choices=["rust", "python"], default="rust", help="rust sails whole populations in parallel inside Rust; python is the slower reference")
    p.add_argument("--fixed", action="store_true", help="always the figure 8 in a 13 m/s northerly, as before")
    args = p.parse_args()
    log = run(
        args.out, args.population, args.generations, args.workers, args.seed, args.time_limit, args.sigma,
        args.sigma_final,
        args.rudder_limit, randomize=not args.fixed, gens_per_map=args.gens_per_map, backend=args.backend,
    )
    g = log["generations"][-1]
    print(f"gen {g['gen']}: best {g['best']:.0f}, finish rate {g['finish_rate']:.0%}")


if __name__ == "__main__":
    main()
