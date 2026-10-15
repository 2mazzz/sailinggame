import { color, css, font, stroke } from "./theme";

export interface Series {
  name: string;
  values: (number | null)[]; // one per generation
  color: number;
  width?: number;
}

export interface ChartOptions {
  title: string;
  series: Series[];
  /** A constant reference line, drawn dashed. */
  reference?: { name: string; value: number };
  yMin?: number;
  yMax?: number;
  unit?: string;
  checkpoints: number[];
  onPick: (generation: number) => void;
}

const W = 360, H = 150, ML = 42, MR = 10, MT = 10, MB = 24;
const NS = "http://www.w3.org/2000/svg";
const el = (name: string, attrs: Record<string, string | number> = {}, text?: string) => {
  const e = document.createElementNS(NS, name);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, String(v));
  if (text !== undefined) e.textContent = text;
  return e;
};

/** Round a range to tidy axis ticks. */
function niceTicks(min: number, max: number, count = 3): number[] {
  const span = max - min || 1;
  const raw = span / count;
  const mag = Math.pow(10, Math.floor(Math.log10(raw)));
  const step = [1, 2, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? raw;
  const first = Math.ceil(min / step) * step;
  const ticks: number[] = [];
  for (let v = first; v <= max + 1e-9; v += step) ticks.push(Math.round(v * 1e6) / 1e6);
  return ticks;
}

/** A cream card holding a titled line chart; `setGeneration` moves the marker. */
export function lineChart(opts: ChartOptions) {
  const n = Math.max(...opts.series.map((s) => s.values.length));
  const all = opts.series.flatMap((s) => s.values.filter((v): v is number => v !== null));
  if (opts.reference) all.push(opts.reference.value);
  const lo = opts.yMin ?? Math.min(...all), hi = opts.yMax ?? Math.max(...all);
  const pad = opts.yMin === undefined || opts.yMax === undefined ? (hi - lo) * 0.05 : 0;
  const y0 = opts.yMin ?? lo - pad, y1 = opts.yMax ?? hi + pad;
  const x = (g: number) => ML + (g / Math.max(1, n - 1)) * (W - ML - MR);
  const y = (v: number) => MT + (1 - (v - y0) / (y1 - y0 || 1)) * (H - MT - MB);

  const card = document.createElement("section");
  card.style.cssText = `background:${css(color.card)};border:${stroke.outline}px solid ${css(color.ink)};border-radius:16px;padding:10px 12px 6px;color:${css(color.ink)}`;
  const head = document.createElement("div");
  head.style.cssText = `display:flex;justify-content:space-between;align-items:baseline;gap:8px;flex-wrap:wrap;font:600 15px ${font.display}`;
  head.append(opts.title);
  const legend = document.createElement("span");
  legend.style.cssText = `font:500 11px ${font.mono};display:flex;gap:10px;flex-wrap:wrap`;
  for (const s of opts.series) {
    const item = document.createElement("span");
    item.innerHTML = `<i style="display:inline-block;width:10px;height:10px;border-radius:50%;background:${css(s.color)};border:1.5px solid ${css(color.ink)};margin-right:4px"></i>${s.name}`;
    legend.append(item);
  }
  if (opts.reference) legend.append(`- - ${opts.reference.name}`);
  head.append(legend);

  const svg = el("svg", { viewBox: `0 0 ${W} ${H}`, width: "100%" }) as SVGSVGElement;
  svg.style.cssText = "display:block;cursor:pointer;touch-action:none";
  for (const t of niceTicks(y0, y1)) {
    svg.append(el("line", { x1: ML, x2: W - MR, y1: y(t), y2: y(t), stroke: css(color.ink), "stroke-opacity": 0.15 }));
    svg.append(el("text", { x: ML - 6, y: y(t) + 4, "text-anchor": "end", "font-size": 10, "font-family": font.mono, fill: css(color.ink) }, `${t}${opts.unit ?? ""}`));
  }
  svg.append(el("line", { x1: ML, x2: W - MR, y1: H - MB, y2: H - MB, stroke: css(color.ink), "stroke-width": 1.5 }));
  for (const g of opts.checkpoints) svg.append(el("circle", { cx: x(g), cy: H - MB, r: 2.5, fill: css(color.card), stroke: css(color.ink), "stroke-width": 1.5 }));
  svg.append(el("text", { x: ML, y: H - 6, "font-size": 10, "font-family": font.mono, fill: css(color.ink) }, "gen 0"));
  svg.append(el("text", { x: W - MR, y: H - 6, "text-anchor": "end", "font-size": 10, "font-family": font.mono, fill: css(color.ink) }, `gen ${n - 1}`));
  if (opts.reference) {
    svg.append(el("line", { x1: ML, x2: W - MR, y1: y(opts.reference.value), y2: y(opts.reference.value), stroke: css(color.ink), "stroke-width": 1.5, "stroke-dasharray": "5 4" }));
  }
  for (const s of opts.series) {
    const pts = s.values
      .map((v, g) => (v === null ? "" : `${x(g).toFixed(1)},${y(v).toFixed(1)}`))
      .filter(Boolean)
      .join(" ");
    svg.append(el("polyline", { points: pts, fill: "none", stroke: css(s.color), "stroke-width": s.width ?? 2.5, "stroke-linejoin": "round", "stroke-linecap": "round" }));
  }
  const marker = el("line", { y1: MT, y2: H - MB, stroke: css(color.ink), "stroke-width": 1.5 });
  const dots = opts.series.map((s) => el("circle", { r: 4, fill: css(s.color), stroke: css(color.ink), "stroke-width": 1.5 }));
  svg.append(marker, ...dots);

  const pick = (e: PointerEvent) => {
    const r = svg.getBoundingClientRect();
    const px = ((e.clientX - r.left) / r.width) * W;
    const g = Math.round(((px - ML) / (W - ML - MR)) * (n - 1));
    opts.onPick(Math.max(0, Math.min(n - 1, g)));
  };
  svg.addEventListener("pointerdown", (e) => { svg.setPointerCapture(e.pointerId); pick(e); });
  svg.addEventListener("pointermove", (e) => { if (e.buttons) pick(e); });

  card.append(head, svg);
  return {
    card,
    setGeneration(g: number) {
      marker.setAttribute("x1", String(x(g)));
      marker.setAttribute("x2", String(x(g)));
      opts.series.forEach((s, i) => {
        const v = s.values[g];
        dots[i].setAttribute("visibility", v === null || v === undefined ? "hidden" : "visible");
        if (v !== null && v !== undefined) {
          dots[i].setAttribute("cx", String(x(g)));
          dots[i].setAttribute("cy", String(y(v)));
        }
      });
    },
  };
}
