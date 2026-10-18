"""Engineering graphs of the sim's dynamics.

    cargo run --release -p sailsim --example dynamics_data -- <csv dir>
    .venv/bin/python python/scripts/dynamics_figures.py <csv dir> docs/img/dynamics

The curves come from the Rust sim itself (the example above); the markers are published points
quoted in docs/research/2026-10-14-dinghy-physics-validation.md. Needs only numpy and matplotlib.
"""

import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
from readme_figures import BLUE, GREEN, INK, MUTED, ORANGE  # noqa: E402  (also sets rcParams)

KN = 0.514444
G = 9.81
LWL = 3.81  # m, Day and Nixon (2014)
RHO = 1000.0

# Day and Nixon (2014) tank tests, drag area m^2 at kn (Fig. 2 of Day 2017, read off).
TANK = np.array([(2, .0125), (3, .0132), (4, .0157), (5, .0207), (6.5, .0245), (8, .0221), (9, .0206)])
# Day (2017) VPP, upwind speed at 12 kn (Fig. 5): TWA deg, kn.
DAY_UPWIND_SPEED_12 = np.array([(40, 4.9), (42, 5.0), (46.5, 5.4), (50, 5.6), (60, 6.3)])
# Binns et al. (2002) measured polar at 12 kn, via Day (2017) Fig. 3 (+-0.3 kn): TWA deg, kn.
BINNS_12 = np.array([(37, 4.7), (90, 6.5), (180, 6.6)])
# Day (2017) VPP upwind VMG against wind (Fig. 8): wind kn, VMG kn.
DAY_VMG = np.array([(5, 2.3), (8, 3.25), (10, 3.6), (12, 3.75), (14, 3.8), (16, 3.8)])

TANK_LABEL = "Day and Nixon (2014) tank test"
DAY_LABEL = "Day (2017) VPP"
BINNS_LABEL = "Binns et al. (2002) measured, via Day (2017)"


def load(d, name):
    return np.genfromtxt(d / name, delimiter=",", names=True, dtype=None, encoding=None)


def params(d):
    return {str(r["name"]): float(r["value"]) for r in load(d, "params.csv")}


def save(fig, out, name):
    fig.savefig(out / f"{name}.png", dpi=150, bbox_inches="tight", facecolor="white")
    plt.close(fig)


def title(fig, text, sub=None):
    fig.suptitle(text, x=0.01, ha="left", fontsize=12, fontweight="bold", y=1.09 if sub else 1.02)
    if sub:
        fig.text(0.01, 1.045, sub, ha="left", va="top", fontsize=9, color=MUTED, linespacing=1.4)


def hull(d, out):
    t = load(d, "hull.csv")
    p = params(d)
    kn = t["kn"]
    fig, (a, b) = plt.subplots(2, 1, figsize=(8, 7), sharex=True, gridspec_kw={"hspace": 0.12})
    tail = kn >= 9
    for ax, per, unit in ((a, False, "hull resistance (N)"), (b, True, "drag area R / (0.5 rho v^2)  (m^2)")):
        def f(r, k=kn):
            return r / (0.5 * RHO * (k * KN) ** 2) if per else r
        ax.fill_between(kn[tail], f(t["r_p0"][tail], kn[tail]), f(t["r_p1"][tail], kn[tail]), color=ORANGE, alpha=0.22, lw=0)
        ax.plot(kn[tail], f(t["r_p0"][tail], kn[tail]), color=ORANGE, lw=1, ls="--")
        ax.plot(kn[tail], f(t["r_p1"][tail], kn[tail]), color=ORANGE, lw=1, ls="--")
        ax.plot(kn, f(t["r_p05"]), color=BLUE, lw=2, label="sim (tank table, tail p = 0.5)")
        ax.axvline(9, color=MUTED, lw=1, ls=":")
        ax.set_ylabel(unit)
        ax.grid(alpha=0.15)
    v = TANK[:, 0] * KN
    a.plot(TANK[:, 0], 0.5 * RHO * TANK[:, 1] * v ** 2, "s", color=INK, ms=6, mfc="white", mew=1.5, label=TANK_LABEL)
    b.plot(TANK[:, 0], TANK[:, 1], "s", color=INK, ms=6, mfc="white", mew=1.5, label=TANK_LABEL)
    a.text(16.5, 495, "tail: assumption,\nno data above 9 kn\n(band: p = 0 to 1,\nR ~ v^p beyond 9 kn)", color=ORANGE, fontsize=9, va="center", ha="center", fontweight="bold")
    b.text(9.3, 0.0245, "last tank point, 9 kn", color=MUTED, fontsize=8.5, va="bottom")
    a.annotate("hump: planing starts\n(about 6.5 kn, Fr 0.55)", xy=(6.5, 0.5 * RHO * 0.0245 * (6.5 * KN) ** 2), xytext=(3.2, 220),
               arrowprops=dict(arrowstyle="->", color=MUTED), color=INK, fontsize=9)
    a.legend(frameon=False, loc="upper left", fontsize=9)
    b.set_xlabel("boat speed (kn)")
    b.set_xlim(2, 24)
    b.set_ylim(0, 0.032)
    top = a.secondary_xaxis("top", functions=(lambda k: k * KN / np.sqrt(G * LWL), lambda f: f * np.sqrt(G * LWL) / KN))
    top.set_xlabel("length Froude number (LWL 3.81 m)")
    title(fig, "Resistance climbs to a hump near 6.5 kn, then the hull starts to plane")
    save(fig, out, "01_hull_resistance")


def foils(d, out):
    t = load(d, "foils.csv")
    p = params(d)
    fig, ax = plt.subplots(2, 3, figsize=(12, 7), gridspec_kw={"hspace": 0.28, "wspace": 0.25, "height_ratios": [1.2, 1]})
    panels = [("Rudder (0.45 m blade, AR 4.5)", ["rudder", "rudder_old"], "rudder_stall_deg"),
              ("Centreboard (AR 2)", ["board"], "board_stall_deg"),
              ("Sail (trimmed, depowering aside)", ["sail"], "sail_stall_deg")]
    style = {"rudder": (BLUE, "-", "new rudder"), "rudder_old": (ORANGE, "--", "old rudder"),
             "board": (BLUE, "-", "centreboard"), "sail": (BLUE, "-", "sail")}
    for k, (name, foil_names, stall_key) in enumerate(panels):
        top, bot = ax[0, k], ax[1, k]
        for fn in foil_names:
            r = t[t["foil"] == fn]
            col, ls, lab = style[fn]
            alpha, cl, cd = r["deg"], r["cl"], r["cd"]
            top.plot(alpha, cl, color=col, ls=ls, lw=2, label=f"CL, {lab}")
            top.plot(alpha, cd, color=col, ls=ls, lw=1, alpha=0.55, label=f"CD, {lab}")
            m = (alpha > 0) & (alpha <= 30)
            bot.plot(alpha[m], cl[m] / cd[m], color=col, ls=ls, lw=2, label=lab)
        stall = p["rudder_old_stall_deg"] if k == 0 else None
        for a_ in (top, bot):
            a_.axvline(p[stall_key], color=BLUE, lw=1, ls=":", label=f"stall angle {p[stall_key]:.1f} deg" + (" (new)" if k == 0 else ""))
        if k == 0:
            for a_ in (top, bot):
                a_.axvline(p["rudder_old_stall_deg"], color=ORANGE, lw=1, ls=":", label=f"stall angle {p['rudder_old_stall_deg']:.1f} deg (old)")
                a_.axvspan(12, 14, color=GREEN, alpha=0.25, lw=0, label="NACA 0012 stall, 12-14 deg")
        top.set_xlim(0, 60)
        top.set_title(name, fontsize=10)
        top.set_xlabel("angle of attack (deg)")
        top.set_ylabel("coefficient (CL thick, CD thin)")
        top.legend(frameon=False, fontsize=7.5, loc="upper right")
        top.set_ylim(-0.05, 1.9)
        bot.set_xlim(0, 30)
        bot.set_xlabel("angle of attack (deg)")
        bot.set_ylabel("lift / drag (CL / CD)")
        bot.legend(frameon=False, fontsize=7.5, loc="upper right")
        bot.set_ylim(top=bot.get_ylim()[1] * 1.5)
        for a_ in (top, bot):
            a_.grid(alpha=0.15)
    title(fig, "The new rudder has far better lift/drag but stalls sharply near 15 degrees; the old one never really stalled",
          None)
    save(fig, out, "02_foil_curves")


def rudder(d, out):
    t = load(d, "rudder_sweep.csv")
    p = params(d)
    x = t["rudder_deg"]
    fig, (a, b) = plt.subplots(2, 1, figsize=(8, 7), sharex=True, gridspec_kw={"hspace": 0.12})
    a.plot(x, t["yaw_steady_dps"], color=BLUE, lw=2, label="steady turn (2 s): keeps rising")
    a.plot(x, t["yaw_entry_dps"], color=ORANGE, lw=2, ls="--", label="turn entry (0.2 s): peaks, then falls")
    i = int(np.argmax(t["yaw_entry_dps"]))
    a.plot(x[i], t["yaw_entry_dps"][i], "o", color=ORANGE)
    a.annotate(f"entry peak {t['yaw_entry_dps'][i]:.1f} deg/s\nat {x[i]:.0f} deg", xy=(x[i], t["yaw_entry_dps"][i]), xytext=(11, 3.5),
               arrowprops=dict(arrowstyle="->", color=MUTED), fontsize=9)
    a.annotate(f"full rudder {t['yaw_entry_dps'][-1]:.1f} deg/s", xy=(x[-1], t["yaw_entry_dps"][-1]), xytext=(26, 7), arrowprops=dict(arrowstyle="->", color=MUTED), fontsize=9)
    a.set_ylabel("yaw rate (deg/s)")
    a.legend(frameon=False, loc="upper left", fontsize=9)
    lost = t["speed_lost_ms"] - t["speed_lost_ms"][0]
    b.plot(x, lost, color=GREEN, lw=2)
    b.set_ylabel("extra speed lost in 5 s (m/s)")
    b.set_xlabel("rudder angle (deg; 35 deg is full)")
    for ax in (a, b):
        ax.axvline(p["rudder_stall_deg"], color=BLUE, lw=1, ls=":")
        ax.axvspan(12, 14, color=GREEN, alpha=0.22, lw=0)
        ax.grid(alpha=0.15)
    a.text(p["rudder_stall_deg"] + 0.5, 0.97, "stall angle 14.9 deg\n(green: NACA 12-14)", fontsize=8.5, color=BLUE, va="top", transform=a.get_xaxis_transform())
    b.text(p["rudder_stall_deg"] + 0.5, 0.9, "speed cost grows once the blade stalls", fontsize=8.5, color=BLUE, va="top", transform=b.get_xaxis_transform())
    a.set_xlim(0, 35)
    title(fig, "Past the stall angle, a harder pull gives less turn at first and costs more speed",
          "No wind, 2.5 m/s start. In a steady turn the swinging stern cuts the blade's angle of attack, so yaw rate still rises.\nSpeed lost: heading locked, minus the coast-down at rudder 0.")
    save(fig, out, "03_rudder_effect")


def polar(d, out):
    t = load(d, "polar.csv")
    fig = plt.figure(figsize=(12, 5.8))
    ax = fig.add_subplot(1, 2, 1, projection="polar")
    ax.set_theta_zero_location("N")
    ax.set_theta_direction(-1)
    ax.set_thetamin(0)
    ax.set_thetamax(180)
    cols = {8.0: (GREEN, ":"), 12.0: (BLUE, "-"), 16.0: (ORANGE, "--")}
    for w, (c, ls) in cols.items():
        r = t[t["wind_kn"] == w]
        ax.plot(np.radians(r["twa_deg"]), r["speed_ms"] / KN, color=c, ls=ls, lw=2, label=f"sim, {w:.0f} kn wind")
    ax.plot(np.radians(DAY_UPWIND_SPEED_12[:, 0]), DAY_UPWIND_SPEED_12[:, 1], "D", color=INK, ms=6, mfc="white", mew=1.5, label=DAY_LABEL + ", 12 kn")
    ax.plot(np.radians(BINNS_12[:, 0]), BINNS_12[:, 1], "s", color=INK, ms=7, clip_on=False, label=BINNS_LABEL + ", 12 kn")
    ax.set_rlabel_position(200)
    ax.set_rticks([2, 4, 6, 8, 10])
    ax.set_rlim(0, 11)
    ax.set_title("Boat speed (kn) by true wind angle, best sheet", fontsize=10, pad=14)
    ax.legend(frameon=False, loc="upper center", bbox_to_anchor=(0.5, -0.04), fontsize=8.5)
    b = fig.add_subplot(1, 2, 2)
    for w, (c, ls) in cols.items():
        r = t[t["wind_kn"] == w]
        v = r["vmg_ms"] / KN
        b.plot(r["twa_deg"], v, color=c, ls=ls, lw=2, label=f"{w:.0f} kn wind")
        up, dn = np.argmax(v), np.argmin(v)
        b.plot(r["twa_deg"][up], v[up], "o", color=c)
        b.plot(r["twa_deg"][dn], v[dn], "o", color=c)
        b.annotate(f"{v[up]:.1f}", (r["twa_deg"][up], v[up]), textcoords="offset points", xytext=(14, 4), ha="center", fontsize=8.5)
        b.annotate(f"{-v[dn]:.1f}", (r["twa_deg"][dn], v[dn]), textcoords="offset points", xytext=(-4, -14), ha="center", fontsize=8.5)
    b.axhline(0, color=MUTED, lw=1)
    b.set_xlabel("true wind angle (deg from the wind)")
    b.set_ylabel("speed made good along the wind (kn)\n+ toward the wind, - away from it")
    b.set_xlim(30, 180)
    b.grid(alpha=0.15)
    b.legend(frameon=False, loc="upper right", fontsize=9)
    b.set_title("Velocity made good: best angle upwind and downwind (dots, kn)", fontsize=10)
    title(fig, "Speed peaks on a beam to broad reach; dead downwind the sim is well below the measured 6.6 kn")
    save(fig, out, "04_polar")


def wind_sweep(d, out):
    t = load(d, "wind_sweep.csv")
    w = t["wind_kn"]
    fig, (a, b) = plt.subplots(1, 2, figsize=(12, 4.8), gridspec_kw={"wspace": 0.22})
    a.plot(w, t["reach_speed_ms"] / KN, color=BLUE, lw=2, label="sim, best sheet")
    a.plot([12], [BINNS_12[1, 1]], "s", color=INK, ms=7, label=BINNS_LABEL)
    a.set(xlabel="true wind speed (kn)", ylabel="beam-reach speed (kn)", title="Beam reach (90 deg to the wind)")
    b.plot(w, t["upwind_vmg_ms"] / KN, color=BLUE, lw=2, label="sim, best angle 34-54 deg and best sheet")
    b.plot(DAY_VMG[:, 0], DAY_VMG[:, 1], "D", color=INK, ms=6, mfc="white", mew=1.5, label=DAY_LABEL)
    b.set(xlabel="true wind speed (kn)", ylabel="upwind VMG (kn)", title="Upwind velocity made good")
    for ax in (a, b):
        ax.grid(alpha=0.15)
        ax.legend(frameon=False, loc="upper left", fontsize=9)
        ax.set_xlim(4, 24)
        ax.set_ylim(bottom=0)
    title(fig, "Upwind VMG is low in light air and keeps rising above 14 kn, where the published curve flattens near 3.8 kn")
    save(fig, out, "05_speed_vs_wind")


def heel(d, out):
    r = load(d, "righting.csv")
    e = load(d, "equilibrium.csv")
    p = params(d)
    fig, (a, b) = plt.subplots(1, 2, figsize=(12, 4.8), gridspec_kw={"wspace": 0.2, "width_ratios": [1.3, 1]})
    for ax in (a, b):
        ax.plot(r["heel_deg"], r["rm_hiking"], color=BLUE, lw=2, label="righting moment, hiking fully")
        ax.plot(r["heel_deg"], r["rm_sitting"], color=ORANGE, lw=2, ls="--", label="righting moment, sitting")
        ax.axhline(p["depower_moment_nm"], color=GREEN, lw=1.2, ls=":")
        ax.plot(e["heel_deg"], e["moment_nm"], "o", color=INK, ms=7, label="sail heeling moment at close-hauled equilibrium")
        ax.set_xlabel("heel (deg)")
        ax.set_ylabel("moment (N m)")
        ax.grid(alpha=0.15)
    a.axvline(p["capsize_heel_deg"], color=INK, lw=1, ls="-.")
    a.text(p["capsize_heel_deg"] - 1, 0.45, "capsize at 70 deg", rotation=90, ha="right", fontsize=8.5, transform=a.get_xaxis_transform())
    a.axvline(p["vanishing_heel_deg"], color=MUTED, lw=1, ls=":")
    a.text(p["vanishing_heel_deg"] - 1, 0.45, "righting moment gone at 85 deg", rotation=90, ha="right", fontsize=8.5, color=MUTED, transform=a.get_xaxis_transform())
    a.text(5, p["depower_moment_nm"] - 65, "sail depowers above 450 N m (tuned, no source)", color=GREEN, fontsize=8.5)
    a.set_xlim(0, 90)
    a.set_ylim(-150, 1150)
    a.legend(frameon=False, loc="upper left", bbox_to_anchor=(0.15, 1.0), fontsize=8.5)
    b.set_xlim(0, 6)
    b.set_ylim(0, 1000)
    b.text(0.05, p["depower_moment_nm"] + 15, "depower limit 450 N m", color=GREEN, fontsize=8.5)
    for w, h, m in zip(e["wind_kn"], e["heel_deg"], e["moment_nm"]):
        b.annotate(f"{w:.0f} kn", (h, m), textcoords="offset points", xytext=(8, -12), fontsize=9)
    b.set_title("Zoom on the first 6 deg (close-hauled equilibria)", fontsize=10)
    a.set_title("Full range", fontsize=10)
    title(fig, "Heel stays near 2 degrees even in 20 kn: the hiking righting moment is already 750 N m at 2 degrees",
          "Equilibrium points: where the sail's heeling moment equals the righting moment. The sim does not export a heeling curve, so none is drawn.")
    save(fig, out, "06_heel_moments")


def time_responses(d, out):
    t = load(d, "time.csv")
    fig = plt.figure(figsize=(12, 7.5))
    gs = fig.add_gridspec(6, 2, hspace=0.9, wspace=0.22)
    r = t[t["case"] == "reach"]
    a1, a2 = fig.add_subplot(gs[0:3, 0]), fig.add_subplot(gs[3:6, 0])
    a1.plot(r["t"], r["surge"] / KN, color=BLUE, lw=2)
    a1.set(ylabel="boat speed (kn)", xlabel="time (s)", title="Acceleration from rest, beam reach, 12 kn wind, sheet 0.55")
    a2.plot(r["t"], np.degrees(np.abs(r["heel"])), color=ORANGE, lw=2)
    a2.set(ylabel="heel (deg)", xlabel="time (s)", ylim=(0, 1))
    a2.text(0.5, 0.9, "heel stays under 1 deg: the crew is hiking and the sail depowers", transform=a2.transAxes, fontsize=8.5, color=MUTED, ha="center")
    ts = [fig.add_subplot(gs[2 * k:2 * k + 2, 1]) for k in range(3)]
    cases = [("turn_035", BLUE, "-", "rudder 0.35 (12 deg)"), ("turn_100", ORANGE, "--", "rudder 1.0 (35 deg)")]
    for case, c, ls, lab in cases:
        s = t[t["case"] == case]
        ts[0].plot(s["t"], np.degrees(s["heading"]), color=c, ls=ls, lw=2, label=lab)
        ts[1].plot(s["t"], np.degrees(s["yaw_rate"]), color=c, ls=ls, lw=2)
        ts[2].plot(s["t"], s["surge"], color=c, ls=ls, lw=2)
        ts[2].annotate(f"{s['surge'][-1]:.2f}", (s["t"][-1], s["surge"][-1]), textcoords="offset points", xytext=(-4, 6), ha="right", fontsize=8.5)
    ts[0].set(ylabel="heading (deg)", title="Turn at 2.5 m/s, no wind, rudder held")
    ts[1].set(ylabel="yaw rate (deg/s)")
    ts[2].set(ylabel="boat speed (m/s)", xlabel="time (s)")
    ts[0].legend(frameon=False, loc="upper left", fontsize=9)
    for ax in (a1, a2, *ts):
        ax.grid(alpha=0.15)
    title(fig, "Full rudder spins the boat several times faster than 0.35 but it almost stops in 12 s; 0.35 keeps 0.8 m/s")
    save(fig, out, "07_time_responses")


def main():
    d, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    for f in (hull, foils, rudder, polar, wind_sweep, heel, time_responses):
        f(d, out)


if __name__ == "__main__":
    main()
