import { Application, Container, Graphics, Text } from "pixi.js";
import init, { Game } from "./wasm/sailsim_wasm.js";

// Wind speed in knots from the URL, e.g. ?wind=24 (default 12 kn).
const WIND_KN = Number(new URLSearchParams(location.search).get("wind")) || 12;
let windKn = WIND_KN;
let windFromDeg = Number(new URLSearchParams(location.search).get("dir")) || 0; // wind blows from the north
const PX_PER_M = 14;
const RUDDER_RATE = 2.5; // per second while a key is held
const RUDDER_RETURN = 4.0; // per second back to centre when released
const SHEET_RATE = 0.5;
const HIKE_RATE = 2.0; // per second: the sailor takes about half a second to lean out or in
const KN = 1.943844;

// Snapshot field indices (see sailsim-wasm).
const X = 0, Y = 1, HEADING = 2, HEEL = 3, SURGE = 4, SWAY = 5, AWA = 7, AWS = 8, CAPSIZED = 9, TIME = 10, TWS = 11, TWD = 12;

const deg = (r: number) => (r * 180) / Math.PI;

async function main() {
  await init();
  const game = new Game(windKn / 1.943844, windFromDeg);
  const app = new Application();
  await app.init({ resizeTo: window, background: 0x0b3d5c, antialias: true });
  document.body.appendChild(app.canvas);

  // World: x east, y north -> screen y is flipped. The camera follows the boat.
  const world = new Container();
  const water = new Graphics();
  const boat = new Container();
  const hull = new Graphics()
    .poly([0, -2.1, 0.7, -0.6, 0.65, 1.8, -0.65, 1.8, -0.7, -0.6])
    .fill(0xf2efe6).stroke({ width: 0.06, color: 0x333333 });
  const sailPivot = new Container();
  sailPivot.y = -0.5;
  const sail = new Graphics().moveTo(0, 0).lineTo(0, 2.1).stroke({ width: 0.1, color: 0xe74c3c });
  sailPivot.addChild(sail);
  const rudderGfx = new Graphics();
  rudderGfx.y = 1.8;
  const sailor = new Graphics();
  boat.addChild(hull, sailPivot, rudderGfx, sailor);
  boat.scale.set(PX_PER_M);
  const trail = new Graphics();
  world.addChild(water, trail, boat);
  app.stage.addChild(world);

  const windArrow = new Graphics().poly([0, -26, 9, 4, 0, -2, -9, 4]).fill(0xffffff);
    app.stage.addChild(windArrow);

  // Course marks (world frame, drawn on the water); the next one is highlighted.
  const markGfx = new Graphics();
  world.addChildAt(markGfx, 1);
  const marks = Array.from(game.course_marks());
  const markRadius = game.mark_radius();
  const finishBanner = new Text({ text: "", style: { fill: 0x7dffb0, fontFamily: "monospace", fontSize: 36, align: "center" } });
  finishBanner.anchor.set(0.5);
  finishBanner.visible = false;
  app.stage.addChild(finishBanner);

  // Apparent wind: a line from the boat toward where the wind comes from.
  const apparent = new Graphics();
  world.addChild(apparent);

  const hud = new Text({ text: "", style: { fill: 0xffffff, fontFamily: "monospace", fontSize: 14 } });
  hud.position.set(12, 10);
  app.stage.addChild(hud);
  const banner = new Text({ text: "CAPSIZED - press R", style: { fill: 0xff6b6b, fontFamily: "monospace", fontSize: 40 } });
  banner.anchor.set(0.5);
  banner.visible = false;
  app.stage.addChild(banner);

  // Wind sliders (HTML overlay). Moving a slider changes the wind live.
  const panel = document.createElement("div");
  panel.style.cssText = "position:fixed;right:12px;bottom:12px;padding:10px 14px;background:#0008;color:#fff;font:13px monospace;border-radius:6px";
  const mkSlider = (label: string, min: number, max: number, value: number, onInput: (v: number) => void) => {
    const row = document.createElement("label");
    row.style.cssText = "display:flex;align-items:center;gap:8px;margin:4px 0";
    const text = document.createElement("span");
    text.style.width = "130px";
    const input = document.createElement("input");
    input.type = "range";
    input.min = String(min); input.max = String(max); input.step = "1"; input.value = String(value);
    const show = () => { text.textContent = `${label} ${input.value}`; };
    input.addEventListener("input", () => { show(); onInput(Number(input.value)); input.blur(); });
    show();
    row.append(text, input);
    panel.appendChild(row);
  };
  const applyWind = () => game.set_wind(windKn / KN, windFromDeg);
  mkSlider("wind kn", 0, 35, Math.round(windKn), (v) => { windKn = v; applyWind(); });
  mkSlider("wind from deg", 0, 359, Math.round(windFromDeg), (v) => { windFromDeg = v; applyWind(); });
  mkSlider("gust %", 0, 60, Math.round(game.gust_intensity() * 100), (v) => game.set_gust_intensity(v / 100));
  mkSlider("shift deg", 0, 30, Math.round(game.shift_deg()), (v) => game.set_shift_deg(v));
  document.body.appendChild(panel);

  const keys = new Set<string>();
  addEventListener("keydown", (e) => {
    keys.add(e.code);
    if (e.code === "KeyR") { game.reset(); trailPts.length = 0; }
    if (e.code === "KeyC") game.set_capsize_enabled(!game.capsize_enabled());
    if (e.code.startsWith("Arrow") || e.code === "Space") e.preventDefault();
  });
  addEventListener("keyup", (e) => keys.delete(e.code));

  const trailPts: number[] = [];
  let rudder = 0;
  let sheet = 0.5;
  let hike = 0;
  let sailAngle = 0;

  app.ticker.add((ticker) => {
    const dt = ticker.deltaMS / 1000;
    const left = keys.has("ArrowLeft") || keys.has("KeyA");
    const right = keys.has("ArrowRight") || keys.has("KeyD");
    const steer = (right ? 1 : 0) - (left ? 1 : 0);
    if (steer !== 0) rudder += steer * RUDDER_RATE * dt;
    else rudder -= Math.sign(rudder) * Math.min(Math.abs(rudder), RUDDER_RETURN * dt);
    rudder = Math.max(-1, Math.min(1, rudder));
    // Up/W trims the sheet in (towards 1); Down/S eases it out.
    if (keys.has("ArrowUp") || keys.has("KeyW")) sheet += SHEET_RATE * dt;
    if (keys.has("ArrowDown") || keys.has("KeyS")) sheet -= SHEET_RATE * dt;
    sheet = Math.max(0, Math.min(1, sheet));
    // Hold Space or Shift to lean out; releasing sits the sailor back in.
    const hiking = keys.has("Space") || keys.has("ShiftLeft") || keys.has("ShiftRight");
    hike = Math.max(0, Math.min(1, hike + (hiking ? 1 : -1) * HIKE_RATE * dt));

    game.advance(dt, rudder, sheet, hike);
    const s = game.snapshot();
    const awaNow = s[AWA];

    const { width, height } = app.screen;
    const bx = s[X] * PX_PER_M, by = -s[Y] * PX_PER_M;
    world.position.set(width / 2 - bx, height / 2 - by);

    // Water grid so motion is visible.
    const grid = 10 * PX_PER_M;
    const x0 = Math.floor((bx - width) / grid) * grid, y0 = Math.floor((by - height) / grid) * grid;
    water.clear();
    for (let gx = x0; gx < bx + width; gx += grid) water.moveTo(gx, by - height).lineTo(gx, by + height);
    for (let gy = y0; gy < by + height; gy += grid) water.moveTo(bx - width, gy).lineTo(bx + width, gy);
    water.stroke({ width: 1, color: 0x1a5a82, alpha: 0.7 });

    trailPts.push(bx, by);
    if (trailPts.length > 1200) trailPts.splice(0, 2);
    trail.clear();
    if (trailPts.length >= 4) {
      trail.moveTo(trailPts[0], trailPts[1]);
      for (let i = 2; i < trailPts.length; i += 2) trail.lineTo(trailPts[i], trailPts[i + 1]);
      trail.stroke({ width: 2, color: 0xffffff, alpha: 0.35 });
    }

    const awDir = s[HEADING] + awaNow; // world direction the apparent wind comes from
    const len = 4 + Math.min(s[AWS], 12) * 1.5;
    apparent.clear().moveTo(bx, by).lineTo(bx + Math.sin(awDir) * len * PX_PER_M, by - Math.cos(awDir) * len * PX_PER_M)
      .stroke({ width: 3, color: 0x7fe0ff, alpha: 0.9 });

    boat.position.set(bx, by);
    boat.rotation = s[HEADING]; // heading is clockwise from north = Pixi's clockwise rotation
    // The boom lies on the leeward side, at a fixed angle set by the sheet:
    // boom angle runs 90 deg (eased) to 5 deg (hard in), as in the sim.
    const boom = ((90 - 85 * sheet) * Math.PI) / 180;
    const awa = s[AWA]; // positive: wind from starboard, so the boom goes to port
    sailAngle = awa >= 0 ? boom : -boom;
    sailPivot.rotation = sailAngle; // the sail is drawn aft of the mast, so 0 lies along the centreline
    // Visual heel: squash the beam.
    boat.scale.x = PX_PER_M * Math.cos(s[HEEL]);
    // The sailor sits on the windward side (opposite the boom) and leans out
    // past the gunwale as `hike` rises.
    const windward = awa >= 0 ? 1 : -1;
    const sx = windward * (0.45 + 0.75 * hike);
    sailor.clear().moveTo(windward * 0.3, 0.6).lineTo(sx, 0.6)
      .stroke({ width: 0.22, color: 0x2c6fbb, cap: "round" })
      .circle(sx + windward * 0.1, 0.6, 0.16).fill(0xf1c27d);
    rudderGfx.clear().moveTo(0, 0).lineTo(Math.sin(rudder * 0.6) * 0.5, Math.cos(rudder * 0.6) * 0.5)
      .stroke({ width: 0.1, color: 0x333333 });

    const [nextMark, markDist, markBearing] = game.course_progress();
    markGfx.clear();
    for (let i = 0; i < marks.length / 2; i++) {
      const next = i === nextMark;
      const done = nextMark < 0 || i < nextMark;
      const mx = marks[2 * i] * PX_PER_M, my = -marks[2 * i + 1] * PX_PER_M;
      markGfx.circle(mx, my, markRadius * PX_PER_M)
        .fill({ color: next ? 0xffd23f : 0xffffff, alpha: next ? 0.3 : 0.1 })
        .stroke({ width: next ? 3 : 1.5, color: next ? 0xffd23f : 0xffffff, alpha: done ? 0.3 : 0.9 });
      markGfx.circle(mx, my, 5).fill({ color: next ? 0xffd23f : 0xffffff, alpha: done ? 0.3 : 1 });
    }
    const finished = game.course_status() === 1;
    finishBanner.visible = finished;
    finishBanner.text = `FINISHED  ${game.finish_time().toFixed(1)} s\nR to restart`;
    finishBanner.position.set(width / 2, height / 2 - 80);
    const courseLine = nextMark < 0
      ? `course   finished in ${game.finish_time().toFixed(1)} s`
      : `mark ${nextMark + 1}/${marks.length / 2}  bearing ${(((deg(markBearing) % 360) + 360) % 360).toFixed(0)} deg  dist ${markDist.toFixed(0)} m`;

    banner.visible = s[CAPSIZED] === 1;
    banner.position.set(width / 2, height / 2 - 80);
    windArrow.position.set(width - 50, 50);
    windArrow.rotation = (s[TWD] * Math.PI) / 180 + Math.PI; // points where the wind blows to
    const twa = Math.abs(((deg(s[HEADING]) - s[TWD] + 540) % 360) - 180);
    hud.text = [
      `time     ${(finished ? game.finish_time() : s[TIME]).toFixed(1)} s`,
      courseLine,
      `speed    ${(s[SURGE] * KN).toFixed(1)} kn`,
      `heel     ${deg(s[HEEL]).toFixed(0)} deg`,
      `heading  ${((deg(s[HEADING]) % 360) + 360) % 360 | 0} deg`,
      `TWA      ${twa.toFixed(0)} deg   wind ${(s[TWS] * KN).toFixed(1)} kn (mean ${windKn.toFixed(0)}) from ${(((s[TWD] % 360) + 360) % 360).toFixed(0)} (mean ${windFromDeg.toFixed(0)})`,
      `AWA/AWS  ${deg(awa).toFixed(0)} deg / ${(s[AWS] * KN).toFixed(1)} kn`,
      `rudder   ${rudder.toFixed(2)}   sheet ${sheet.toFixed(2)} (1 = hard in)   hike ${hike.toFixed(2)}`,
      `capsize  ${game.capsize_enabled() ? "on" : "off"}`,
      ``,
      `A/D or <-/->  rudder    W/S or up/down  sheet in/out    hold Space/Shift  hike`,
      `cyan line = apparent wind direction`,
      `R reset    C toggle capsize`,
    ].join("\n");
  });
}
main();
