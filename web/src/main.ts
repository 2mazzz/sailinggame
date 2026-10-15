import { Application, Container, Graphics, Text, type TextStyleOptions } from "pixi.js";
import init, { Game } from "./wasm/sailsim_wasm.js";
import { color, css, font, motion, radius, stroke } from "./theme";

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
const CAPSIZE_HEEL = (70 * Math.PI) / 180; // matches BoatParams::capsize_heel
const SPEED_GAUGE_MAX_KN = 8;
const ARROW_ORBIT_PX = 62; // radius of the next-mark arrow around the boat
const HEEL_RING_PX = 44;
const GAUGE_W = 156;

// Snapshot field indices (see sailsim-wasm).
const X = 0, Y = 1, HEADING = 2, HEEL = 3, SURGE = 4, SWAY = 5, AWA = 7, AWS = 8, CAPSIZED = 9, TIME = 10, TWS = 11, TWD = 12;

const deg = (r: number) => (r * 180) / Math.PI;

// Overshoot ease for the subtle pop-in of cards and banners.
const easeOutBack = (t: number) => {
  const c = 1.7;
  return 1 + (c + 1) * Math.pow(t - 1, 3) + c * Math.pow(t - 1, 2);
};

const monoStyle = (size: number, fill: number = color.ink): TextStyleOptions => ({
  fill, fontFamily: font.mono, fontSize: size, fontWeight: "500",
});
const displayStyle = (size: number, fill: number = color.ink): TextStyleOptions => ({
  fill, fontFamily: font.display, fontSize: size, fontWeight: "600", align: "center",
});

/** A cream card with an ink outline: Graphics background plus a Text, resized to fit. */
function makeCard(style: TextStyleOptions, pad = 12) {
  const box = new Container();
  const bg = new Graphics();
  const text = new Text({ text: "", style });
  text.position.set(pad, pad - 2);
  box.addChild(bg, text);
  return {
    box,
    text,
    set(t: string) {
      if (text.text === t) return;
      text.text = t;
      bg.clear().roundRect(0, 0, text.width + pad * 2, text.height + pad * 2 - 4, radius.card)
        .fill(color.card).stroke({ width: stroke.outline, color: color.ink });
    },
  };
}

async function main() {
  // Pixi renders text once with whatever font is ready, so wait for ours.
  await Promise.all([
    document.fonts.load(`600 16px Fredoka`),
    document.fonts.load(`500 14px "DM Mono"`),
  ]).catch(() => {});
  await init();
  const game = new Game(windKn / 1.943844, windFromDeg);
  const app = new Application();
  await app.init({ resizeTo: window, background: color.water, antialias: true });
  document.body.appendChild(app.canvas);

  // World: x east, y north -> screen y is flipped. The camera follows the boat.
  const world = new Container();
  const water = new Graphics();
  const boat = new Container();
  const hull = new Graphics()
    .poly([0, -2.1, 0.7, -0.6, 0.65, 1.8, -0.65, 1.8, -0.7, -0.6])
    .fill(color.hull).stroke({ width: 0.09, color: color.ink, join: "round" });
  const sailPivot = new Container();
  sailPivot.y = -0.5;
  const sail = new Graphics()
    .moveTo(0, 0).lineTo(0, 2.1).stroke({ width: 0.22, color: color.ink, cap: "round" })
    .moveTo(0, 0).lineTo(0, 2.1).stroke({ width: 0.11, color: color.card, cap: "round" });
  sailPivot.addChild(sail);
  const rudderGfx = new Graphics();
  rudderGfx.y = 1.8;
  const sailor = new Graphics();
  boat.addChild(hull, sailPivot, rudderGfx, sailor);
  boat.scale.set(PX_PER_M);
  const trail = new Graphics();
  world.addChild(water, trail, boat);
  app.stage.addChild(world);

  // Course marks (world frame, drawn on the water); the next one is ringed.
  const markGfx = new Graphics();
  world.addChildAt(markGfx, 1);
  const marks = Array.from(game.course_marks());
  const markRadius = game.mark_radius();

  // Apparent wind: a thin line from the boat toward where the wind comes from.
  const apparent = new Graphics();
  world.addChild(apparent);

  // Screen-space overlays: heel ring and next-mark arrow follow the boat.
  const heelRing = new Graphics();
  const nextArrow = new Graphics()
    .moveTo(-9, 5).lineTo(0, -6).lineTo(9, 5)
    .stroke({ width: 4, color: color.signal, cap: "round", join: "round" });
  const distLabel = makeCard(monoStyle(12), 6);
  app.stage.addChild(heelRing, nextArrow, distLabel.box);

  // HUD card, top left: plain text for everything that is not a gauge.
  const hud = makeCard(monoStyle(13));
  hud.box.position.set(12, 12);
  app.stage.addChild(hud.box);

  // Wind gauge (top right, drawn straight on the water): yellow dial with an ink arrow pointing where the wind blows to.
  const windGauge = new Container();
  const windDial = new Graphics().circle(0, 0, 23).fill(color.signal).stroke({ width: stroke.outline, color: color.ink });
  windDial.position.set(36, 33);
  const windPointer = new Graphics().poly([0, -15, 8, 9, 0, 4, -8, 9]).fill(color.ink);
  windPointer.position.set(36, 33);
  const windValue = new Text({ text: "", style: { ...displayStyle(22), align: "left" } });
  windValue.position.set(70, 10);
  const windLabel = new Text({ text: "wind kn", style: monoStyle(11) });
  windLabel.position.set(70, 38);
  windGauge.addChild(windDial, windPointer, windValue, windLabel);

  // Speed gauge: a half dial that fills with yellow.
  const speedGauge = new Container();
  const speedArc = new Graphics();
  speedArc.position.set(36, 42);
  const speedValue = new Text({ text: "", style: { ...displayStyle(22), align: "left" } });
  speedValue.position.set(70, 6);
  const speedLabel = new Text({ text: "boat kn", style: monoStyle(11) });
  speedLabel.position.set(70, 34);
  speedGauge.addChild(speedArc, speedValue, speedLabel);
  app.stage.addChild(windGauge, speedGauge);

  // Banners pop in with a short overshoot; heel red is reserved for capsize.
  const capsizeBanner = makeCard(displayStyle(36, color.heel), 18);
  const finishBanner = makeCard(displayStyle(36), 18);
  for (const b of [capsizeBanner, finishBanner]) { b.box.visible = false; app.stage.addChild(b.box); }
  const shownAt = new Map<Container, number>();
  const pop = (card: { box: Container }, on: boolean, cx: number, cy: number) => {
    const box = card.box;
    if (!on) { box.visible = false; shownAt.delete(box); return; }
    if (!shownAt.has(box)) shownAt.set(box, performance.now());
    const t = Math.min(1, (performance.now() - shownAt.get(box)!) / motion.enterMs);
    const k = easeOutBack(t);
    // Scale about the card's centre: measure at scale 1, then place.
    box.scale.set(1);
    const w = box.width, h = box.height;
    box.scale.set(k);
    box.position.set(cx - (w * k) / 2, cy - (h * k) / 2);
    box.visible = true;
  };

  // Settings sliders (HTML overlay on a cream card). Moving a slider changes the wind live.
  const panel = document.createElement("div");
  panel.style.cssText = `position:fixed;right:12px;bottom:12px;padding:10px 14px;background:${css(color.card)};color:${css(color.ink)};font:500 13px ${font.mono};border-radius:${radius.card}px;border:${stroke.outline}px solid ${css(color.ink)}`;
  const mkSlider = (label: string, min: number, max: number, value: number, onInput: (v: number) => void) => {
    const row = document.createElement("label");
    row.style.cssText = "display:flex;align-items:center;gap:8px;margin:4px 0";
    const text = document.createElement("span");
    text.style.width = "130px";
    const input = document.createElement("input");
    input.type = "range";
    input.style.accentColor = css(color.ink);
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

    // Little wave squiggles on a jittered grid so motion is visible.
    const grid = 7 * PX_PER_M;
    const gx0 = Math.floor((bx - width) / grid), gx1 = Math.ceil((bx + width) / grid);
    const gy0 = Math.floor((by - height) / grid), gy1 = Math.ceil((by + height) / grid);
    water.clear();
    for (let gx = gx0; gx < gx1; gx++) {
      for (let gy = gy0; gy < gy1; gy++) {
        const h = Math.sin(gx * 127.1 + gy * 311.7) * 43758.5453;
        const r1 = h - Math.floor(h), r2 = Math.abs(Math.sin(h * 7.3));
        const wx = (gx + r1) * grid, wy = (gy + r2) * grid;
        water.moveTo(wx, wy).quadraticCurveTo(wx + 9, wy - 8, wx + 18, wy).quadraticCurveTo(wx + 27, wy + 8, wx + 36, wy);
      }
    }
    water.stroke({ width: 3, color: color.waterWave, cap: "round" });

    trailPts.push(bx, by);
    if (trailPts.length > 1200) trailPts.splice(0, 2);
    trail.clear();
    if (trailPts.length >= 4) {
      trail.moveTo(trailPts[0], trailPts[1]);
      for (let i = 2; i < trailPts.length; i += 2) trail.lineTo(trailPts[i], trailPts[i + 1]);
      trail.stroke({ width: 5, color: color.hull, alpha: 0.35, cap: "round", join: "round" });
    }

    const awDir = s[HEADING] + awaNow; // world direction the apparent wind comes from
    const len = 4 + Math.min(s[AWS], 12) * 1.5;
    apparent.clear().moveTo(bx, by).lineTo(bx + Math.sin(awDir) * len * PX_PER_M, by - Math.cos(awDir) * len * PX_PER_M)
      .stroke({ width: 2, color: color.signal, alpha: 0.7, cap: "round" });

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
      .stroke({ width: 0.3, color: color.ink, cap: "round" })
      .circle(sx + windward * 0.1, 0.6, 0.2).fill(color.hull).stroke({ width: 0.07, color: color.ink });
    rudderGfx.clear().moveTo(0, 0).lineTo(Math.sin(rudder * 0.6) * 0.5, Math.cos(rudder * 0.6) * 0.5)
      .stroke({ width: 0.12, color: color.ink, cap: "round" });

    const [nextMark, markDist, markBearing] = game.course_progress();
    markGfx.clear();
    for (let i = 0; i < marks.length / 2; i++) {
      const next = i === nextMark;
      const done = nextMark < 0 || i < nextMark;
      const mx = marks[2 * i] * PX_PER_M, my = -marks[2 * i + 1] * PX_PER_M;
      const r = markRadius * PX_PER_M;
      if (next) {
        // Dashed ring: 16 short arcs.
        for (let k = 0; k < 16; k++) {
          const a0 = (k / 16) * Math.PI * 2, a1 = a0 + (Math.PI * 2) / 32;
          markGfx.moveTo(mx + Math.cos(a0) * r, my + Math.sin(a0) * r).arc(mx, my, r, a0, a1);
        }
        markGfx.stroke({ width: 3, color: color.mark, cap: "round" });
      }
      markGfx.circle(mx, my, 9).fill({ color: color.mark, alpha: done ? 0.45 : 1 })
        .stroke({ width: stroke.outline, color: color.ink, alpha: done ? 0.45 : 1 });
    }

    // Heel ring around the boat: grows (longer and thicker) as heel nears the capsize angle.
    const heelFrac = Math.min(1, Math.abs(s[HEEL]) / CAPSIZE_HEEL);
    const cx = width / 2, cy = height / 2;
    heelRing.clear();
    if (heelFrac > 0.06) {
      const dir = s[HEEL] >= 0 ? 1 : -1;
      const a0 = -Math.PI / 2;
      heelRing.moveTo(cx + Math.cos(a0) * HEEL_RING_PX, cy + Math.sin(a0) * HEEL_RING_PX)
        .arc(cx, cy, HEEL_RING_PX, a0, a0 + dir * heelFrac * Math.PI * 1.9, dir < 0)
        .stroke({ width: 4 + heelFrac * 6, color: color.heel, cap: "round" });
    }

    // Yellow chevron orbiting the boat towards the next mark, with its distance.
    nextArrow.visible = distLabel.box.visible = nextMark >= 0;
    if (nextMark >= 0) {
      nextArrow.position.set(cx + Math.sin(markBearing) * ARROW_ORBIT_PX, cy - Math.cos(markBearing) * ARROW_ORBIT_PX);
      nextArrow.rotation = markBearing;
      distLabel.set(`${markDist.toFixed(0)} m`);
      distLabel.box.position.set(
        cx + Math.sin(markBearing) * (ARROW_ORBIT_PX + 30) - distLabel.box.width / 2,
        cy - Math.cos(markBearing) * (ARROW_ORBIT_PX + 30) - distLabel.box.height / 2,
      );
    }

    const finished = game.course_status() === 1;
    finishBanner.set(`FINISHED  ${game.finish_time().toFixed(1)} s\nR to restart`);
    capsizeBanner.set("CAPSIZED\nR to restart");
    pop(finishBanner, finished, width / 2, height / 2 - 80);
    pop(capsizeBanner, s[CAPSIZED] === 1, width / 2, height / 2 - 80);

    // Gauges.
    windGauge.position.set(width - GAUGE_W - 12, 12);
    speedGauge.position.set(width - GAUGE_W - 12, 12 + 66 + 10);
    windPointer.rotation = (s[TWD] * Math.PI) / 180 + Math.PI; // points where the wind blows to
    windValue.text = (s[TWS] * KN).toFixed(1);
    const kn = Math.max(0, s[SURGE] * KN);
    speedValue.text = kn.toFixed(1);
    speedArc.clear()
      .arc(0, 0, 24, Math.PI, Math.PI * 2).stroke({ width: 9, color: color.ink, alpha: 0.18, cap: "round" })
      .arc(0, 0, 24, Math.PI, Math.PI + Math.PI * Math.min(1, kn / SPEED_GAUGE_MAX_KN)).stroke({ width: 9, color: color.signal, cap: "round" });

    const twa = Math.abs(((deg(s[HEADING]) - s[TWD] + 540) % 360) - 180);
    const courseLine = nextMark < 0
      ? `course   finished in ${game.finish_time().toFixed(1)} s`
      : `mark ${nextMark + 1}/${marks.length / 2}  bearing ${(((deg(markBearing) % 360) + 360) % 360).toFixed(0)} deg`;
    hud.set([
      `time     ${(finished ? game.finish_time() : s[TIME]).toFixed(1)} s`,
      courseLine,
      `heading  ${((deg(s[HEADING]) % 360) + 360) % 360 | 0} deg   heel ${deg(s[HEEL]).toFixed(0)} deg`,
      `TWA      ${twa.toFixed(0)} deg   wind from ${(((s[TWD] % 360) + 360) % 360).toFixed(0)} (mean ${windFromDeg.toFixed(0)}, ${windKn.toFixed(0)} kn)`,
      `rudder   ${rudder.toFixed(2)}   sheet ${sheet.toFixed(2)}   hike ${hike.toFixed(2)}`,
      `capsize  ${game.capsize_enabled() ? "on" : "off"}`,
      ``,
      `A/D or <-/->  rudder   W/S or up/down  sheet`,
      `hold Space/Shift  hike   R reset   C capsize`,
      `yellow line = apparent wind`,
    ].join("\n"));
  });
}
main();
