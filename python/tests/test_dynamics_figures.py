"""The engineering graphs build end to end. Standalone: it only shells out to cargo and to the
plotting script, so it does not need the compiled `sailing._core` extension."""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPECTED = [
    "01_hull_resistance", "02_foil_curves", "03_rudder_effect", "04_polar",
    "05_speed_vs_wind", "06_heel_moments", "07_time_responses",
]


def test_every_figure_is_produced(tmp_path):
    data = tmp_path / "data"
    subprocess.run(
        ["cargo", "run", "--release", "-q", "-p", "sailsim", "--example", "dynamics_data", "--", str(data)],
        cwd=ROOT, check=True,
    )
    out = tmp_path / "img"
    subprocess.run(
        [sys.executable, str(ROOT / "python/scripts/dynamics_figures.py"), str(data), str(out)],
        check=True,
    )
    for name in EXPECTED:
        assert (out / f"{name}.png").stat().st_size > 10_000, name
