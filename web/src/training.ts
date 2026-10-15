import { Application, Container, Graphics } from "pixi.js";
import { lineChart } from "./charts";
import { color, css, font, radius, stroke } from "./theme";

// Training page: graphs of an evolution run next to a replay of one generation,
// every boat of the population sailing the course at once as ghosts. The files
// come from `python -m sailing.evolve` (web/public/runs/<run>/).

const PARAMS = new URLSearchParams(location.search);
const RUN = PARAMS.get("run") ?? "rudder-0.4-1000";
// ?gen=N picks the nearest checkpoint, ?t=S freezes the replay at S seconds (for screenshots and links).
const START_GEN = PARAMS.has("gen") ? Number(PARAMS.get("gen")) : 0;
const FREEZE_T = PARAMS.has("t") ? Number(PARAMS.get("t")) : null;
const PANEL_W = 400;
const TRAIL_S = 12; // seconds of wake kept behind each boat
const HOLD_S = 2; // pause on the final frame before looping

interface GenStat {
  gen: number; best: number; mean: number; median: number;
  finish_rate: number; capsize_rate: number; mean_finish_time: number | null;
  best_controls: { rudder: number; sheet: number; hike: number };
}
interface Log {
  config: { population: number; generations: number; wind_speed: number; wind_dir: number; sigma: number; elite: number };
  course: { marks: [number, number][]; radius: number };
  checkpoints: number[];
  generations: GenStat[];
  baseline: { score: number; time: number; outcome: string };
}
interface Boat { score: number; outcome: string; time: number; n: number; x: Float32Array; y: Float32Array; h: Float32Array; heel: Float32Array }
interface Replay { gen: number; dt: number; boats: Boat[] }

const KN = 1.943844;
const pad4 = (n: number) => String(n).padStart(4, "0");
const card = (extra = "") =>
  `background:${css(color.card)};border:${stroke.outline}px solid ${css(color.ink)};border-radius:${radius.card}px;color:${css(color.ink)};${extra}`;

async function loadReplay(gen: number): Promise<Replay> {
  const raw = await (await fetch(`/runs/${RUN}/gen_${pad4(gen)}.json`)).json();
  const boats: Boat[] = raw.boats.map((b: { score: number; outcome: string; time: number; p: number[] }) => {
    const n = b.p.length / 4;
    const x = new Float32Array(n), y = new Float32Array(n), h = new Float32Array(n), heel = new Float32Array(n);
    for (let i = 0; i < n; i++) {
      x[i] = b.p[4 * i] / 10; y[i] = b.p[4 * i + 1] / 10; h[i] = b.p[4 * i + 2] / 100; heel[i] = b.p[4 * i + 3] / 100;
    }
    return { score: b.score, outcome: b.outcome, time: b.time, n, x, y, h, heel };
  });
  return { gen: raw.gen, dt: raw.dt, boats };
}

function message(text: string) {
  const d = document.createElement("div");
  d.style.cssText = card(`position:fixed;left:50%;top:40%;transform:translateX(-50%);padding:16px 20px;font:500 14px ${font.mono};max-width:520px;white-space:pre-wrap`);
  d.textContent = text;
  document.body.appendChild(d);
}

async function main() {
  await Promise.all([document.fonts.load("600 16px Fredoka"), document.fonts.load('500 14px "DM Mono"')]).catch(() => {});
  let log: Log;
  try {
    const res = await fetch(`/runs/${RUN}/training.json`);
    if (!res.ok) throw new Error(String(res.status));
    log = await res.json();
  } catch {
    document.body.style.background = css(color.water);
    message(`No run found at /runs/${RUN}/training.json.\nCreate one with:\n.venv/bin/python -m sailing.evolve --out web/public/runs/${RUN}`);
    return;
  }

  const app = new Application();
  await app.init({ resizeTo: window, background: color.water, antialias: true });
  document.body.appendChild(app.canvas);

  // World drawing: marks, trails, ghost boats. Fitted to the course in the area left of the panel.
  const world = new Container();
  const marksGfx = new Graphics();
  const trails = new Graphics();
  const boatLayer = new Container();
  world.addChild(marksGfx, trails, boatLayer);
  app.stage.addChild(world);

  const marks = log.course.marks;
  const pts = [[0, 0], ...marks];
  const minX = Math.min(...pts.map((p) => p[0])), maxX = Math.max(...pts.map((p) => p[0]));
  const minY = Math.min(...pts.map((p) => p[1])), maxY = Math.max(...pts.map((p) => p[1]));
  let ppm = 5; // pixels per metre, refit on resize
  const layout = () => {
    const w = Math.max(200, app.screen.width - PANEL_W), h = Math.max(200, app.screen.height - 150 - 80);
    const bw = maxX - minX + 4 * log.course.radius, bh = maxY - minY + 4 * log.course.radius;
    ppm = Math.min(w / bw, h / bh);
    world.scale.set(ppm, -ppm); // y north is up
    world.position.set(w / 2 - ((minX + maxX) / 2) * ppm, 150 + h / 2 + ((minY + maxY) / 2) * ppm);
  };
  layout();
  app.renderer.on("resize", layout);

  const drawMarks = () => {
    marksGfx.clear();
    const r = log.course.radius;
    marks.forEach(([mx, my]) => {
      marksGfx.circle(mx, my, r).fill({ color: color.mark, alpha: 0.12 }).stroke({ width: 1.5 / ppm, color: color.mark, alpha: 0.8 });
      marksGfx.circle(mx, my, 9 / ppm).fill(color.mark).stroke({ width: stroke.outline / ppm, color: color.ink });
    });
    // Start: a cream dot.
    marksGfx.circle(0, 0, 6 / ppm).fill(color.card).stroke({ width: stroke.outline / ppm, color: color.ink });
  };

  // One Graphics per boat, drawn once in boat metres and moved each frame.
  // World y points north, so the bow is at +y.
  const hullPts = (k: number) => [0, 2.1 * k, 0.6 * k, 0.7 * k, 0.66 * k, -1.0 * k, 0.4 * k, -1.8 * k, -0.4 * k, -1.8 * k, -0.66 * k, -1.0 * k, -0.6 * k, 0.7 * k];
  const makeBoat = (best: boolean) => {
    const g = new Graphics();
    g.poly(hullPts(1)).fill(color.hull).stroke({ width: best ? 0.28 : 0.22, color: color.ink, join: "round" });
    if (best) g.poly(hullPts(0.55)).fill(color.card);
    return g;
  };

  let replay: Replay | null = null;
  let boatGfx: Graphics[] = [];
  let t = 0, playing = FREEZE_T === null, speed = 1;
  let frozenApplied = false;

  const setReplay = (r: Replay) => {
    replay = r;
    boatLayer.removeChildren().forEach((c) => c.destroy());
    // Worst first so the best boat is drawn on top.
    boatGfx = r.boats.map((_, i) => makeBoat(i === 0));
    [...boatGfx].reverse().forEach((g) => boatLayer.addChild(g));
    if (FREEZE_T === null || !frozenApplied) { t = FREEZE_T ?? 0; frozenApplied = true; } else t = 0;
    drawMarks();
    updateInfo();
  };

  // ----- DOM: info card, controls, graphs -----
  const info = document.createElement("div");
  info.style.cssText = card(`position:fixed;left:12px;top:12px;padding:10px 14px;min-width:230px;box-sizing:border-box;font-family:${font.mono}`);
  document.body.appendChild(info);

  const controls = document.createElement("div");
  controls.style.cssText = card(`position:fixed;left:12px;bottom:12px;padding:8px 12px;display:flex;align-items:center;gap:10px;font:500 13px ${font.mono};width:${`calc(100% - ${PANEL_W + 36}px)`};max-width:640px;box-sizing:border-box`);
  const button = (label: string) => {
    const b = document.createElement("button");
    b.textContent = label;
    b.style.cssText = `font:600 14px ${font.display};background:${css(color.hull)};color:${css(color.ink)};border:${stroke.outline}px solid ${css(color.ink)};border-radius:${radius.pill}px;padding:3px 12px;cursor:pointer`;
    return b;
  };
  const playBtn = button(FREEZE_T === null ? "Pause" : "Play");
  const speedBtn = button("1x");
  const slider = document.createElement("input");
  slider.type = "range"; slider.min = "0"; slider.max = String(log.checkpoints.length - 1); slider.value = "0";
  slider.style.cssText = `flex:1;min-width:80px;accent-color:${css(color.ink)}`;
  const sliderLabel = document.createElement("span");
  sliderLabel.style.minWidth = "60px";
  controls.append(playBtn, speedBtn, slider, sliderLabel);
  document.body.appendChild(controls);

  const panel = document.createElement("div");
  panel.style.cssText = `position:fixed;right:0;top:0;bottom:0;width:${PANEL_W}px;box-sizing:border-box;padding:12px;display:flex;flex-direction:column;gap:12px;overflow-y:auto`;
  const head = document.createElement("div");
  head.style.cssText = card(`padding:10px 14px;font:500 12px/1.5 ${font.mono}`);
  const c = log.config;
  head.innerHTML = `<div style="font:600 18px ${font.display}">Run ${RUN}</div>${c.population} boats, ${log.generations.length} generations, top ${c.elite} kept, mutation ${c.sigma}`;
  panel.appendChild(head);

  const gens = log.generations;
  const charts = [
    lineChart({
      title: "Score",
      series: [
        { name: "best", values: gens.map((g) => g.best), color: color.ink, width: 3 },
        { name: "mean", values: gens.map((g) => g.mean), color: color.waterWave, width: 2.5 },
      ],
      reference: { name: "scripted baseline", value: log.baseline.score },
      checkpoints: log.checkpoints, onPick: (g) => pickGeneration(g),
    }),
    lineChart({
      title: "Finish and capsize rate",
      series: [
        { name: "finished", values: gens.map((g) => g.finish_rate * 100), color: color.ink, width: 3 },
        { name: "capsized", values: gens.map((g) => g.capsize_rate * 100), color: color.heel, width: 2.5 },
      ],
      yMin: 0, yMax: 100, unit: "%",
      checkpoints: log.checkpoints, onPick: (g) => pickGeneration(g),
    }),
    lineChart({
      title: "Controls the best boat uses",
      series: [
        { name: "rudder", values: gens.map((g) => g.best_controls.rudder * 100), color: color.ink, width: 2.5 },
        { name: "sheet range", values: gens.map((g) => g.best_controls.sheet * 100), color: 0x2a8f84, width: 2.5 },
        { name: "hike", values: gens.map((g) => g.best_controls.hike * 100), color: 0x8a8794, width: 2.5 },
      ],
      yMin: 0, yMax: 100, unit: "%",
      checkpoints: log.checkpoints, onPick: (g) => pickGeneration(g),
    }),
    lineChart({
      title: "Finish time",
      series: [{ name: "mean of finishers", values: gens.map((g) => g.mean_finish_time), color: color.ink, width: 3 }],
      reference: { name: "baseline", value: log.baseline.time },
      unit: " s",
      checkpoints: log.checkpoints, onPick: (g) => pickGeneration(g),
    }),
  ];
  charts.forEach((ch) => panel.appendChild(ch.card));
  document.body.appendChild(panel);

  const windText = `wind ${(c.wind_speed * KN).toFixed(0)} kn from ${c.wind_dir.toFixed(0)}°`;
  let current = 0;
  const updateInfo = () => {
    const g = gens[current];
    const boats = replay?.boats ?? [];
    const done = boats.filter((b) => b.outcome === "finished").length;
    const cap = boats.filter((b) => b.outcome === "capsized").length;
    info.innerHTML = `<div style="font:600 24px ${font.display}">Generation ${g.gen}</div>
      <div style="font:500 12px/1.6 ${font.mono}">${boats.length || c.population} boats: ${done} finished, ${cap} capsized<br>best ${g.best.toFixed(0)} (baseline ${log.baseline.score.toFixed(0)})<br>${windText}<br>time ${t.toFixed(0)} s</div>`;
    sliderLabel.textContent = `gen ${g.gen}`;
    charts.forEach((ch) => ch.setGeneration(g.gen));
  };

  const cache = new Map<number, Replay>();
  const select = async (cp: number) => {
    current = log.checkpoints[cp];
    slider.value = String(cp);
    let r = cache.get(current);
    if (!r) { r = await loadReplay(current); cache.set(current, r); }
    if (log.checkpoints[Number(slider.value)] === r.gen) setReplay(r);
  };
  const nearestCheckpoint = (g: number) => {
    let best = 0;
    log.checkpoints.forEach((cg, i) => { if (Math.abs(cg - g) < Math.abs(log.checkpoints[best] - g)) best = i; });
    return best;
  };
  const pickGeneration = (g: number) => void select(nearestCheckpoint(g));
  slider.addEventListener("input", () => void select(Number(slider.value)));
  playBtn.addEventListener("click", () => { playing = !playing; playBtn.textContent = playing ? "Pause" : "Play"; });
  speedBtn.addEventListener("click", () => { speed = speed === 1 ? 2 : speed === 2 ? 4 : 1; speedBtn.textContent = `${speed}x`; });
  addEventListener("keydown", (e) => {
    if (e.code === "Space") { e.preventDefault(); playBtn.click(); }
    if (e.code === "ArrowRight") void select(Math.min(log.checkpoints.length - 1, Number(slider.value) + 1));
    if (e.code === "ArrowLeft") void select(Math.max(0, Number(slider.value) - 1));
  });

  await select(nearestCheckpoint(START_GEN));
  updateInfo();

  // ----- animation -----
  const duration = () => (replay ? Math.max(...replay.boats.map((b) => (b.n - 1) * replay!.dt)) : 0);
  let infoClock = 0;
  app.ticker.add((ticker) => {
    if (!replay) return;
    if (playing) t += (ticker.deltaMS / 1000) * speed;
    const end = duration();
    if (t > end + HOLD_S) t = 0;
    const r = replay;
    const size = Math.max(1, 5 / ppm) * 1; // boats are at least 5 px per boat-metre
    trails.clear();
    r.boats.forEach((b, i) => {
      const f = Math.min(t / r.dt, b.n - 1);
      const i0 = Math.min(b.n - 2, Math.floor(f)), k = f - i0;
      const lerp = (a: Float32Array) => a[i0] + (a[Math.min(b.n - 1, i0 + 1)] - a[i0]) * k;
      let dh = b.h[Math.min(b.n - 1, i0 + 1)] - b.h[i0];
      dh = ((dh + Math.PI) % (2 * Math.PI) + 2 * Math.PI) % (2 * Math.PI) - Math.PI; // shortest way round
      const g = boatGfx[i];
      const ended = t >= (b.n - 1) * r.dt;
      const best = i === 0;
      g.position.set(lerp(b.x), lerp(b.y));
      g.rotation = -(b.h[i0] + dh * k); // world y is flipped
      const s = (best ? 1.35 : 1) * size;
      g.scale.set(s * Math.cos(lerp(b.heel)), s);
      g.alpha = best ? 1 : ended && b.outcome === "finished" ? 0.3 : ended ? 0.45 : 0.62;
      g.tint = ended && b.outcome === "capsized" ? 0x8a8794 : 0xffffff;
      // Wake: the last few seconds of this boat's path.
      const from = Math.max(0, i0 - Math.floor(TRAIL_S / r.dt));
      trails.moveTo(b.x[from], b.y[from]);
      for (let j = from + 1; j <= i0; j++) trails.lineTo(b.x[j], b.y[j]);
      trails.lineTo(g.position.x, g.position.y);
    });
    trails.stroke({ width: 2.5 / ppm, color: color.hull, alpha: 0.2, cap: "round", join: "round" });
    // Keep the time readout fresh without rebuilding the card every frame.
    infoClock += ticker.deltaMS;
    if (infoClock > 250) { infoClock = 0; updateInfo(); }
  });
}
main();
