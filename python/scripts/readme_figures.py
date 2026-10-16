"""Draw the figures used in the top-level README from a finished evolution run.

    .venv/bin/python python/scripts/readme_figures.py web/public/runs/maps-1000x10 docs/img
"""

import json
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

INK, MUTED = "#1f2933", "#8a94a0"
BLUE, ORANGE, GREEN = "#2f6fb0", "#d9822b", "#2e9e6b"
plt.rcParams.update({"font.size": 10, "axes.spines.top": False, "axes.spines.right": False,
                     "axes.edgecolor": MUTED, "axes.labelcolor": INK, "text.color": INK})


def smooth(v, w=200):
    v = np.asarray(v, float)
    c = np.cumsum(np.insert(v, 0, 0))
    out = np.empty_like(v)
    for i in range(len(v)):
        lo = max(0, i - w + 1)
        out[i] = (c[i + 1] - c[lo]) / (i + 1 - lo)
    return out


def learning(log, out):
    g = log["generations"]
    x = [s["gen"] for s in g]
    fig, ax = plt.subplots(1, 2, figsize=(11, 3.8))
    for s, c, label in (("finish_rate", BLUE, "whole population, course being trained on"),):
        ax[0].plot(x, smooth([r[s] for r in g]) * 100, color=c, label=label)
    ax[0].plot(x, smooth([r["held_out"]["finish_rate"] for r in g]) * 100, color=ORANGE,
               label="best boat, 10 unseen courses")
    ax[0].set(title="Share of boats that finish the course", xlabel="generation", ylabel="%", ylim=(0, 100))
    ax[0].legend(frameon=False, loc="upper left")
    ax[1].plot(x, smooth([r["held_out"]["score"] for r in g]), color=ORANGE, label="best boat, unseen courses")
    ax[1].plot(x, smooth([r["median"] for r in g]), color=BLUE, label="median boat, training course")
    ax[1].plot(x, smooth([r["baseline_score"] for r in g]), color=GREEN, ls="--", label="scripted baseline")
    ax[1].set(title="Score (higher is better)", xlabel="generation", ylabel="score")
    ax[1].legend(frameon=False, loc="lower right")
    fig.text(0.5, -0.02, "200-generation moving averages. A new course is drawn every 10 generations.",
             ha="center", color=MUTED, fontsize=9)
    fig.tight_layout()
    fig.savefig(out / "learning.png", dpi=160, bbox_inches="tight")
    plt.close(fig)


def tracks(run, gens, out):
    fig, axes = plt.subplots(1, len(gens), figsize=(3.6 * len(gens), 5.2))
    for ax, gen in zip(axes, gens):
        r = json.loads((run / f"gen_{gen:04d}.json").read_text())
        for i, b in reversed(list(enumerate(r["boats"]))):
            p = np.array(b["p"]).reshape(-1, 4)
            best = i == 0
            ax.plot(p[:, 0] / 10, p[:, 1] / 10, color=ORANGE if best else BLUE,
                    lw=1.8 if best else 0.5, alpha=1 if best else 0.25, zorder=3 if best else 2)
        m = np.array(r["course"]["marks"])
        for mx, my in m:
            ax.add_patch(plt.Circle((mx, my), r["course"]["radius"], color=INK, alpha=0.15, zorder=1))
        ax.plot(*m.T, "o", color=INK, ms=3, zorder=4)
        done = sum(b["outcome"] == "finished" for b in r["boats"])
        w = r["wind"]
        ax.set_title(f"generation {gen}\n{done}/100 finished", fontsize=10)
        ax.set_xlabel(f"wind {w['speed'] * 1.944:.0f} kn from {w['dir']:.0f}°", color=MUTED, fontsize=9)
        ax.set_aspect("equal")
        ax.set_xticks([]); ax.set_yticks([])
        for s in ax.spines.values():
            s.set_visible(False)
    fig.tight_layout()
    fig.savefig(out / "tracks.png", dpi=160, bbox_inches="tight")
    plt.close(fig)


def main():
    run, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    log = json.loads((run / "training.json").read_text())
    learning(log, out)
    tracks(run, [0, 500, 3000, 9999], out)


if __name__ == "__main__":
    main()
