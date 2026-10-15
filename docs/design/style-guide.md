# Viewer style guide

Scope: the game viewer (canvas, HUD, sliders, banners). Tokens live in `web/src/theme.ts`; this file holds the rules. A visual specimen of the palette was reviewed on 2026-10-15.

## Feel

Playful but clear. Daylight world, flat shapes, thick outlines, rounded corners. Personality comes from shape and a little motion, not from texture or decoration.

## Colour

| Token | Hex | Use |
|---|---|---|
| `water` | `#4FC3B5` | the world; `waterWave` `#7DD8CC` for wave strokes |
| `card` | `#FFF3DC` | HUD cards, opaque, never translucent |
| `ink` | `#2B2A33` | outlines and text |
| `hull` | `#FFFFFF` | the boat |
| `signal` | `#FFC93C` | arrow to the next mark, wind gauge, speed gauge |
| `mark` | `#FFFFFF` | course marks |
| `heel` | `#FF4A3D` | heel ring and capsize banner only |

Rules:
- `heel` red means danger and nothing else. Nothing else on screen may be red.
- `signal` yellow means "information you steer by". Do not use it for decoration or buttons.
- Marks are white with an ink outline; the next mark gets a dashed ring.
- Every colour comes from `theme.ts`; no literals in `main.ts`.

## Glance priority

1. Next mark: yellow arrow around the boat, with distance on the mark label.
2. Wind: own gauge (direction and strength).
3. Heel: ring around the boat that grows as heel nears the capsize angle.
4. Speed: own gauge.
5. Sheet, rudder, time and settings: plain text and simple sliders on a card.

## Type

- Display: Fredoka 500/600 for labels and banners.
- Numbers: DM Mono with tabular digits so live values do not jitter.

## Shapes

- Boat ("toy dinghy"): smooth white hull with a cream deck inset, a puffy white sail that bellies to leeward, flat ink sailor on the windward side. Ink outlines 0.06 to 0.1 m. Drawn at 18 px per metre.
- Arrow to the next mark: small, thin, rounded chevron orbiting the boat; subtle, not a solid arrow.
- Cards: 16 px radius, 2.5 px ink outline, no shadow.
- Gauges: circular or semicircular, filled with `signal`, outlined in ink. They sit directly on the water, not in a card.

## Motion

Subtle. Cards and banners enter in about 180 ms with a small overshoot. No motion delays reading a value. Respect `prefers-reduced-motion` in the HTML overlay.

## Banned

Navy with white stripes, glossy neon game UI, and by default anchors, ship wheels, decorative compass roses, rope borders and parchment maps.

## Not yet decided

- Reference games or illustrators (none given).
- Exact gauge sizes and layout positions.
- Applying the tokens in `web/src/main.ts` (the viewer still uses its old look).
