"""Draw the figures used in the top-level README from a finished evolution run.

    .venv/bin/python python/scripts/readme_figures.py web/public/runs/h16-5k web/public/runs/h16-10k docs/img

Every run but the last is followed by a run that was started from its best boat (`--init-from`), so the
runs are drawn as one, generations counting on from the end of the previous one.
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


def learning(log, out, joins):
    g = log["generations"]
    x = [s["gen"] + 1 for s in g]
    fig, ax = plt.subplots(1, 2, figsize=(11, 3.8))
    for s, c, label in (("finish_rate", BLUE, "whole population, course being trained on"),):
        ax[0].plot(x, smooth([r[s] for r in g]) * 100, color=c, label=label)
    ax[0].plot(x, smooth([r["held_out"]["finish_rate"] for r in g]) * 100, color=ORANGE,
               label="best boat, 10 unseen courses")
    for a in ax:
        for j in joins:
            a.axvline(j, color=MUTED, ls=":", lw=1)
    ax[0].set(title="Share of boats that finish the course", xlabel="generation", ylabel="%", ylim=(0, 100))
    ax[0].legend(frameon=False, loc="lower right")
    ax[1].plot(x, smooth([r["held_out"]["score"] for r in g]), color=ORANGE, label="best boat, unseen courses")
    ax[1].plot(x, smooth([r["median"] for r in g]), color=BLUE, label="median boat, training course")
    ax[1].plot(x, smooth([r["baseline_score"] for r in g]), color=GREEN, ls="--", label="scripted baseline")
    ax[1].set(title="Score (higher is better)", xlabel="generation", ylabel="score")
    ax[1].legend(frameon=False, loc="lower right")
    fig.text(0.5, -0.02, "200-generation moving averages. Every generation brings new random courses. Dotted line: the run was restarted from its best boat.",
             ha="center", color=MUTED, fontsize=9)
    fig.tight_layout()
    fig.savefig(out / "learning.png", dpi=160, bbox_inches="tight")
    plt.close(fig)


def tracks(runs, gens, out):
    """`gens` count over all the runs together; a replay is read from the run that holds it."""
    sizes = [len(json.loads((run / "training.json").read_text())["generations"]) for run in runs]
    fig, axes = plt.subplots(1, len(gens), figsize=(3.6 * len(gens), 5.2))
    for ax, overall in zip(axes, gens):
        i, gen = 0, overall
        while gen >= sizes[i]:
            gen -= sizes[i]
            i += 1
        r = json.loads((runs[i] / f"gen_{gen:04d}.json").read_text())
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
        ax.set_title(f"generation {overall + 1}\n{done}/100 finished", fontsize=10)
        ax.set_xlabel(f"wind {w['speed'] * 1.944:.0f} kn from {w['dir']:.0f}°", color=MUTED, fontsize=9)
        ax.set_aspect("equal")
        ax.set_xticks([]); ax.set_yticks([])
        for s in ax.spines.values():
            s.set_visible(False)
    fig.tight_layout()
    fig.savefig(out / "tracks.png", dpi=160, bbox_inches="tight")
    plt.close(fig)


def physics(out):
    """A sketch of the forces the simulator models, top-down and from behind."""
    fig, (a, b) = plt.subplots(1, 2, figsize=(11, 5.2), gridspec_kw={"width_ratios": [1.15, 1]})
    # Top-down: boat heading up the page, wind from the right-hand bow.
    a.set_xlim(-3.2, 4.4); a.set_ylim(-3.4, 4.2); a.set_aspect("equal"); a.axis("off")
    hull = np.array([[0, 2.1], [0.7, 0.6], [0.7, -1.8], [-0.7, -1.8], [-0.7, 0.6]])
    a.add_patch(plt.Polygon(hull, fc="#e8edf2", ec=INK, lw=1.5))
    a.plot([0, 0.15], [-1.8, -2.45], color=INK, lw=3, solid_capstyle="round")  # rudder blade at the stern
    boom = 0.30
    tip = np.array([-np.sin(boom) * 2.0, -np.cos(boom) * 2.0]) + [0, 0.1]
    a.plot([0, tip[0]], [0.1, tip[1]], color=INK, lw=3)
    sail_mid = np.array([0.5 * -tip[0], 0.1 + 0.5 * (tip[1] - 0.1)]) * 1
    def arrow(ax, x0, y0, dx, dy, c, text, tx=0, ty=0, lw=2):
        ax.annotate("", xy=(x0 + dx, y0 + dy), xytext=(x0, y0),
                    arrowprops=dict(arrowstyle="-|>", color=c, lw=lw, shrinkA=0, shrinkB=0))
        ax.text(x0 + dx + tx, y0 + dy + ty, text, color=c, fontsize=9.5, ha="center", va="center")
    arrow(a, 3.9, 3.7, -1.3, -0.95, MUTED, "true wind", -0.9, 0.3)
    arrow(a, 2.4, 2.2, -1.1, -1.1, BLUE, "apparent wind", 2.05, 0.5)
    a.text(3.4, 0.95, "(true wind minus\nboat velocity)", color=BLUE, fontsize=8.5, ha="center", va="center")
    arrow(a, 0.0, -0.3, 0.0, 1.3, GREEN, "drive", -0.5, 0.15, 2.5)
    arrow(a, 0.0, -0.3, -1.5, 0.0, ORANGE, "side force\n(resisted by\ncentreboard)", -0.35, -0.85, 2.5)
    a.annotate("", xy=(0.0, -2.2), xytext=(0.0, -3.0), arrowprops=dict(arrowstyle="-|>", color=INK, lw=1.5))
    a.text(0.0, -3.2, "heading", ha="center", fontsize=9.5, color=INK)
    a.annotate("rudder (yaw moment,\nweather helm)", xy=(0, -1.85), xytext=(-2.4, -2.3), fontsize=9.5,
               arrowprops=dict(arrowstyle="-", color=MUTED), color=INK)
    a.annotate("sail: lift and drag from\nangle of attack, trimmed\nby the sheet", xy=(0.45, -0.7), xytext=(1.5, -2.4),
               fontsize=9.5, arrowprops=dict(arrowstyle="-", color=MUTED), color=INK)
    a.set_title("From above", fontsize=11)
    # From behind: heel against the righting moment.
    b.set_xlim(-3, 3); b.set_ylim(-2.8, 4.6); b.set_aspect("equal"); b.axis("off")
    th = np.radians(-18)  # heeled to leeward (left): the sail force pushes the mast top away from the wind
    R = np.array([[np.cos(th), np.sin(th)], [-np.sin(th), np.cos(th)]])
    def tf(pts): return np.asarray(pts) @ R.T
    water = np.array([[-3, 0], [3, 0]])
    b.fill_between([-3, 3], -2.1, 0, color="#cfe0f0", alpha=0.7)
    b.plot(*water.T, color=BLUE, lw=1)
    hullp = tf([[-1.0, 0.35], [1.0, 0.35], [0.7, -0.35], [-0.7, -0.35]])
    b.add_patch(plt.Polygon(hullp, fc="#e8edf2", ec=INK, lw=1.5))
    mast = tf([[0, 0.35], [0, 4.4]])
    b.plot(*mast.T, color=INK, lw=3)
    b.add_patch(plt.Polygon(tf([[0.0, 0.8], [0.0, 4.2], [-1.2, 1.1]]), fc=BLUE, alpha=0.25, ec=BLUE))
    b.plot(*tf([[0, -0.35], [0, -1.5]]).T, color=INK, lw=3)  # centreboard
    ce = tf([[-0.3, 2.3]])[0]
    b.annotate("", xy=(ce[0] - 1.6, ce[1]), xytext=ce, arrowprops=dict(arrowstyle="-|>", color=ORANGE, lw=2.5))
    b.text(ce[0] - 0.8, ce[1] + 0.3, "sail force", color=ORANGE, ha="center", fontsize=9.5)
    b.annotate("", xy=(2.0, -0.9), xytext=(0.45, -0.9), arrowprops=dict(arrowstyle="-|>", color=ORANGE, lw=2.5))  # the water pushes back, towards the wind
    b.text(1.85, -1.35, "board resists\nthe side force", color=ORANGE, ha="center", fontsize=9.5, va="center")
    cg = tf([[0.0, 0.7]])[0]
    b.annotate("", xy=(cg[0] + 1.7, cg[1] + 0.55), xytext=(cg[0] + 0.05, cg[1]), arrowprops=dict(arrowstyle="-|>", color=GREEN, lw=2))
    b.text(cg[0] + 2.1, cg[1] + 1.05, "crew hikes out:\nlonger righting arm", color=GREEN, ha="left", fontsize=9.5, va="center")
    b.annotate("", xy=(2.55, 0.0), xytext=(2.55, 0.0))
    b.text(0, -2.6, "heeling moment (sail force) against righting moment (hull, crew weight)", ha="center", fontsize=9, color=MUTED)
    b.set_title("From behind", fontsize=11)
    fig.tight_layout()
    fig.savefig(out / "physics.png", dpi=160, bbox_inches="tight")
    plt.close(fig)


def main():
    runs, out = [Path(a) for a in sys.argv[1:-1]], Path(sys.argv[-1])
    out.mkdir(parents=True, exist_ok=True)
    generations, joins = [], []
    for run in runs:
        for row in json.loads((run / "training.json").read_text())["generations"]:
            generations.append({**row, "gen": len(generations)})
        joins.append(len(generations))
    learning({"generations": generations}, out, joins[:-1])
    tracks(runs, [0, 500, 2500, len(generations) - 1], out)
    physics(out)


if __name__ == "__main__":
    main()
