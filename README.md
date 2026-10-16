# Sailing game

A small sailing simulator in Rust where neural-net sailors learn to race a course by evolution. A viewer in `web/` replays every boat of every checkpoint generation as ghosts with wakes.

![Boats at four points of training](docs/img/tracks.png)

*The same population at generations 0, 500, 3000 and 9999, each on its own random course. Blue: all 100 boats. Orange: the best one. Grey circles: the marks.*

## What it does

- Each boat is a 16-unit network that reads 14 inputs (boat state and the marks ahead) and outputs rudder, sheet and hike (leaning out).
- Every generation all 100 boats sail the same random course (8 legs) in a random wind (8 to 16 m/s, any direction). The best 10 % carry over unchanged and the rest are mutated copies of the best 30 %.
- A course is kept for 10 generations, then a new one is drawn. 10,000 generations is 1,000 courses.
- Score rewards progress, mark bonuses and a fast finish, and charges a time cost and a capsize penalty. The rudder is limited to 0.4 of full deflection.

## Results

![Learning curves](docs/img/learning.png)

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
