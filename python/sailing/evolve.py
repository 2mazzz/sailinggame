"""Evolve a population of small neural-net sailors on the course.

Each generation every boat sails the course once; the best few are kept as they
are (elitism, so the best score never drops) and the rest of the next
generation are mutated copies of good boats. Per-generation statistics and the
recorded runs of checkpoint generations are written as JSON for the viewer.
"""

import json
import multiprocessing as mp
import os
from pathlib import Path

import numpy as np

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


def _episode(params, record, policy=None):
    """Sail one episode; returns its summary and, when asked, the recorded path."""
    env = _env
    obs, info = env.reset(seed=0)
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
    params, record = args
    return _episode(params, record)


def _baseline(_):
    return _episode(None, False, policy="baseline")


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
        "best_controls": top["controls"],
    }


def checkpoint_gens(generations):
    """Early generations densely, later ones sparsely, always the last."""
    picks = {0, 1, 2, 3, 5, 8, 12, 18, 25, 35, 50, 70, 100, 140, 200, 280, 400, 560, 800, 1100, 1500}
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
            "wind_speed": probe.wind_speed,
            "wind_dir": probe.wind_dir,
        },
        "course": {"marks": course["marks"], "radius": course["radius"]},
        "checkpoints": checkpoints,
        "generations": [],
    }

    pool = None
    if workers > 1:
        pool = mp.get_context("spawn").Pool(workers, _init_worker, (time_limit, rudder_limit))
        evaluate, base = pool.map, pool.map
    else:
        _init_worker(time_limit, rudder_limit)
        evaluate, base = map, map
    try:
        baseline = list(base(_baseline, [0]))[0]
        log["baseline"] = {k: baseline[k] for k in ("score", "time", "outcome", "controls")}
        for gen in range(generations):
            record = gen in checkpoints
            results = list(evaluate(_evaluate, [(p, record) for p in pop]))
            order = np.argsort([-r["score"] for r in results], kind="stable")
            log["generations"].append(_stats(gen, results))
            if record:
                boats = [
                    {k: results[i][k] for k in ("score", "outcome", "time", "p")} for i in order
                ]
                (out / f"gen_{gen:04d}.json").write_text(
                    json.dumps({"gen": gen, "dt": 0.1 * RECORD_EVERY, "boats": boats})
                )
            (out / "training.json").write_text(json.dumps(log))
            (out / "best.json").write_text(
                json.dumps({"gen": gen, "score": results[order[0]]["score"], "params": pop[order[0]].tolist()})
            )
            # Next generation: keep the elite, fill the rest with mutated good boats.
            ranked = pop[order]
            children = ranked[rng.integers(0, parents, size=population - elite)]
            # Wide steps early to explore, small ones late to polish.
            step = sigma + (sigma_final - sigma) * gen / max(1, generations - 1)
            children = children + rng.normal(0.0, step, size=children.shape)
            pop = np.vstack([ranked[:elite], children])
    finally:
        if pool:
            pool.close()
            pool.join()
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
    args = p.parse_args()
    log = run(
        args.out, args.population, args.generations, args.workers, args.seed, args.time_limit, args.sigma,
        args.sigma_final,
        args.rudder_limit,
    )
    g = log["generations"][-1]
    print(f"gen {g['gen']}: best {g['best']:.0f}, finish rate {g['finish_rate']:.0%}")


if __name__ == "__main__":
    main()
