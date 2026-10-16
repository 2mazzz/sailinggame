# Sailing game

A small sailing simulator in Rust where neural-net sailors learn to race a course by evolution. A viewer in `web/` replays every boat of every checkpoint generation as ghosts with wakes.

## Play it yourself

You can sail the boat yourself in the browser, on the same physics the neural nets use. You steer, trim the sheet and hike out, round the marks in order, and try to beat the clock. The wind, its direction, gusts and wind shifts have sliders.

![The game](docs/img/game.png)

| Key | Action |
|---|---|
| `A` / `D` or `←` / `→` | rudder |
| `W` / `S` or `↑` / `↓` | trim the sheet in / ease it out |
| `Space` or `Shift` (hold) | hike: lean out to keep the boat flat |
| `P` | hand over to the scripted autopilot (or open with `?auto=1`) |
| `C` | capsizing on or off |
| `R` | restart |

The yellow line is the apparent wind, the arrow around the boat points to the next mark, and the ring shows heel. Run it from `web/` with `npm install` and `npm run dev` (it builds the Rust core to WebAssembly first, so you need the `wasm32-unknown-unknown` target and `wasm-bindgen`), then open the address Vite prints. The trained boats are not playable in the browser yet. They are replayed on the Training page (`/training.html`), and the autopilot there is the hand-written baseline.

![Boats at four points of training](docs/img/tracks.png)

*The same population at generations 0, 500, 3000 and 9999, each on its own random course. Blue: all 100 boats. Orange: the best one. Grey circles: the marks.*

## What it does

- Each boat is a 16-unit network that reads 14 inputs (boat state and the marks ahead) and outputs rudder, sheet and hike (leaning out).
- Every generation all 100 boats sail the same random course (8 legs) in a random wind (8 to 16 m/s, any direction). The best 10 % carry over unchanged and the rest are mutated copies of the best 30 %.
- A course is kept for 10 generations, then a new one is drawn. 10,000 generations is 1,000 courses.
- Score rewards progress, mark bonuses and a fast finish, and charges a time cost and a capsize penalty. The rudder is limited to 0.4 of full deflection.

## The boat

The simulator is a deterministic Rust core (`crates/sailsim`) for a single-handed ILCA/Laser-class dinghy: 4.2 m long, 7 m² sail, 160 kg with the sailor. It is 2D (position, heading, surge and sway speed, yaw rate) plus heel angle, stepped with a fixed-step RK4 integrator. The same code runs in training (via PyO3) and in the browser (via WASM).

![Forces on the boat](docs/img/physics.png)

*A sketch of what the simulator models, not output from it.*

- **Apparent wind:** the true wind minus the boat's own velocity, in the boat's frame. It swings forward as the boat speeds up.
- **Sail:** lift and drag from the angle of attack, with stall. The sheet sets the boom angle. The force acts at the sail's centre of effort, which gives the heeling moment. Once that moment is large the sail depowers (flattens), as a sailor would.
- **Hull:** drag that follows a tank-test table, with its hump near 6.5 kn. There is no planing.
- **Centreboard and rudder:** foils with lift and drag. The board resists sideways force, so leeway is small but not zero. The rudder is at the stern, so spinning and sideslip change what it sees. The board sits ahead of the sail's centre of effort, so a released rudder rounds the boat up (weather helm).
- **Heel:** a righting moment from a hiking-crew curve. Past 70° of heel the boat capsizes and the run ends.
- **Controls:** rudder, sheet and hike (how far the sailor leans out). These are the three network outputs.

At 12 kn of wind the model gives about 5.3 kn and a VMG of 3.4 kn at 44° close-hauled, and 6.8 kn on a beam reach. Published figures for the class are 5.4 kn, 3.75 kn and 6.5 kn. Light-air VMG is about 20 % low. Several parameters are tuned without a source and the design doc lists them: [`docs/plans/2026-10-14-dinghy-sim-design.md`](docs/plans/2026-10-14-dinghy-sim-design.md), with the source check in [`docs/research/`](docs/research/2026-10-14-dinghy-physics-validation.md). There are no waves, gusts or wind shifts in these runs.

## How it learns

There is no gradient and no value function. The learner is a plain evolution strategy (neuroevolution) over the weights of a fixed network, so the only learning signal is the episode score.

- **Policy:** 14 inputs, 16 tanh hidden units, 3 tanh outputs (rudder, sheet, hike). That is 291 numbers, stored as one flat vector per boat.
- **Fitness:** the total `sailcourse` score of one episode (progress, mark bonuses, a finish bonus for speed, a time cost, a capsize penalty). Every boat in a generation sails the same course and wind, so they are ranked on equal terms.
- **Selection:** rank the 100 boats by score. The top 10 (elitism) carry over unchanged. The other 90 are copies of boats picked at random from the top 30, each with Gaussian noise added to every weight.
- **Step size:** the noise starts at σ = 0.3 to explore and shrinks linearly to 0.03 to polish.
- **New course every 10 generations:** the elite are re-scored on each new course, so a boat only stays on top if it holds up on the next one.
- **Held-out check:** each generation the best boat also sails 10 fixed unseen courses and the old figure 8. This is the generalisation curve in the results below.

```
random weights ─► sail the course ─► score ─► keep top 10, mutate top 30 ─► next generation
        ▲                                                                        │
        └────────────── new course and wind every 10 generations ◄──────────────┘
```

The loop is `python/sailing/evolve.py`. Episodes run in parallel across processes through a Gymnasium environment (`python/sailing/env.py`) over the same Rust code as the game. A scripted baseline sailor, not trained, is the reference the scores are compared with.

## Results

![Learning curves](docs/img/learning.png)

![The Training page, all 100 boats mid-run](docs/img/midrun.png)

*The game's Training page at generation 3000, 30 s into the run: all 100 boats at once with wakes, rounding the marks, and the training graphs on the right. Open it yourself with `/training.html?run=maps-1000x10&gen=3000&t=30`.*

| Generations | Finish rate (training course) | Finish rate (best boat, unseen courses) |
|---|---|---|
| 0 to 999 | 41 % | 32 % |
| 3000 to 3999 | 46 % | 59 % |
| 6000 to 6999 | 66 % | 70 % |
| 9000 to 9999 | 82 % | 71 % |

- From 0 finishers to about 80 % of the population finishing a course it is currently being trained on. By the last generation all 100 finish.
- The best boat finishes about 70 % of courses it has never seen. That is flat since about generation 6000, so the population is getting better at the current course faster than it is generalising.
- It does not yet beat the scripted baseline. The baseline scores about 940 on a typical course, the best evolved boat about 700 on unseen ones.
- Early on the finish rate drops sharply (around generation 2300). A few unlucky courses can push the population toward a strategy that works only in some winds.

## Run it

```sh
cd python
.venv/bin/python -m sailing.evolve --out ../web/public/runs/maps-1000x10 \
    --generations 10000 --population 100 --gens-per-map 10
```

On 8 cores this takes a while (the 10,000-generation run). Then, in `web/`, `npm run dev` and open `/training.html`. See `python/README.md` for the options.

Regenerate the figures above from a finished run:

```sh
.venv/bin/python python/scripts/readme_figures.py web/public/runs/maps-1000x10 docs/img
```

`docs/img/midrun.png` is a screenshot of the page itself (with `npx vite` running in `web/`):

```sh
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --use-angle=swiftshader --enable-unsafe-swiftshader \
    --force-device-scale-factor=2 --window-size=1500,950 --virtual-time-budget=15000 \
    --screenshot=docs/img/midrun.png "http://localhost:5173/training.html?run=maps-1000x10&gen=3000&t=30"
```

## License

[MIT](LICENSE)
