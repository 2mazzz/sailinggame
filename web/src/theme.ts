// Visual tokens for the viewer. Rules and rationale: docs/design/style-guide.md.
// Colours are 0xRRGGBB numbers for Pixi; use `css()` for HTML overlays.

export const color = {
  water: 0x4fc3b5,
  waterWave: 0x7dd8cc,
  card: 0xfff3dc,
  ink: 0x2b2a33,
  hull: 0xffffff,
  // Roles: each is used for one meaning and nothing else.
  signal: 0xffc93c, // information you steer by: next-mark arrow, wind gauge, speed gauge
  mark: 0xffffff, // course marks
  heel: 0xff4a3d, // heel and capsize danger
} as const;

export const css = (c: number) => `#${c.toString(16).padStart(6, "0")}`;

export const font = {
  display: '"Fredoka", "Trebuchet MS", sans-serif', // labels, banners
  mono: '"DM Mono", ui-monospace, monospace', // live values, tabular digits
} as const;

export const stroke = {
  outline: 2.5, // px for cards and gauges, 0.06 m on the boat
  ring: 3,
} as const;

export const radius = { card: 16, pill: 999 } as const;

export const motion = {
  // Subtle: short and springy, never delays reading a value.
  enterMs: 180,
  easing: "cubic-bezier(0.34, 1.4, 0.64, 1)",
} as const;
