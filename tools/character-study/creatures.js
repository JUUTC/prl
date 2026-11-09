// Disposable playtest creatures with independent left/right sets and idle/walk/attack clips.
// Placeholders only (docs/art-production.md): not production anatomy or animation timing.
//
// Poses are authored in body-local coordinates where +x points the way the creature
// faces. Facing maps those coordinates to the screen, but lighting does not flip:
// lit edges are always placed on the screen-left side, and the far limbs are the
// darker ones drawn first. Nothing composites a finished sprite and mirrors it.

export const creatureIds = ["skeleton", "shambler", "rat"];
export const creatureActions = ["idle", "walk", "attack"];

const ink = "#111514";
const cycle = [0, 1, 2, 3, 4, 5].map(frame => Math.round(3 * Math.sin(frame / 6 * 2 * Math.PI)));
// Anticipation, wind-up, contact, follow-through and recovery over six frames.
const lunge = [-1, -3, 4, 6, 3, 0];

function pose(action, frame) {
  if (action === "walk") return { stride: cycle[frame], bob: frame % 3 === 1 ? -1 : 0, lean: 1, strike: null };
  if (action === "attack") return { stride: 0, bob: frame === 3 ? 1 : 0, lean: lunge[frame], strike: frame };
  return { stride: 0, bob: frame === 2 || frame === 3 ? -1 : 0, lean: 0, strike: null };
}

function painter(canvas, facing) {
  canvas.width = 64; canvas.height = 64;
  const ctx = canvas.getContext("2d");
  ctx.imageSmoothingEnabled = false;
  const dir = facing === "left" ? -1 : 1;
  const screen = (x, w) => (dir > 0 ? 32 + x : 32 - x - w);
  const r = (x, y, w, h, colour) => {
    const width = Math.max(1, Math.round(w));
    ctx.fillStyle = colour;
    ctx.fillRect(Math.round(screen(x, width)), Math.round(y), width, Math.max(1, Math.round(h)));
  };
  const line = (x, y, xx, yy, colour, w = 1) => {
    const n = Math.max(1, Math.ceil(Math.max(Math.abs(xx - x), Math.abs(yy - y))));
    for (let i = 0; i <= n; i++) r(x + (xx - x) * i / n - (w - 1) / 2, y + (yy - y) * i / n, w, w, colour);
  };
  // Key light from the upper-left of the screen, whichever way the body faces.
  const lit = (x, y, w, h, colour) => {
    const width = Math.max(1, Math.round(w));
    ctx.fillStyle = colour;
    ctx.fillRect(Math.round(screen(x, width)), Math.round(y), 1, Math.max(1, Math.round(h)));
  };
  return { r, line, lit };
}

function skeleton({ r, line, lit }, { stride, bob, lean, strike }) {
  const bone = "#ad9b75", shade = "#796b52", pale = "#c6c8a3", rust = "#929780", grip = "#756750";
  const hip = { x: lean / 2, y: 40 + bob };
  const neck = { x: lean * 0.7, y: 24 + bob };
  line(hip.x - 2, hip.y, -3 - stride, 56, shade, 2);
  r(-5 - stride, 56, 4, 1, shade);
  // Far arm swings opposite the near leg while walking and braces during attacks.
  const farHand = strike === null ? { x: -5 + stride, y: 37 + bob } : { x: -6 + lean / 2, y: 33 };
  line(neck.x - 3, neck.y + 2, farHand.x, farHand.y, shade, 2);
  r(hip.x - 5, hip.y - 2, 10, 3, bone);
  line(hip.x, hip.y - 2, neck.x, neck.y, bone, 2);
  for (let i = 0; i < 4; i++) {
    r(neck.x - 5 + i * 0.3 * Math.sign(lean), neck.y + 2 + i * 3, 11, 1, bone);
    lit(neck.x - 5, neck.y + 2 + i * 3, 11, 1, pale);
  }
  line(hip.x + 2, hip.y, 2 + stride, 56, bone, 2);
  r(1 + stride, 56, 5, 1, bone);
  const head = { x: neck.x + (strike === 3 ? 2 : 0), y: neck.y - 11 };
  r(head.x - 5, head.y, 11, 10, ink);
  r(head.x - 4, head.y + 1, 9, 8, bone);
  lit(head.x - 4, head.y + 1, 9, 6, pale);
  r(head.x + 1, head.y + 4, 2, 2, ink);
  r(head.x - 2, head.y + 4, 1, 2, ink);
  r(head.x - 2, head.y + 8, 6, 1, ink);
  // Near arm carries a rusted blade: rest, wind-up behind the skull, cut, follow-through.
  const swing = [
    { hand: [7, 37], tip: [11, 47] }, { hand: [-2, 22], tip: [-8, 11] }, { hand: [-3, 20], tip: [-10, 10] },
    { hand: [13, 27], tip: [26, 29] }, { hand: [12, 35], tip: [21, 44] }, { hand: [9, 36], tip: [13, 46] },
  ];
  const held = strike === null ? { hand: [7, 37 + bob], tip: [11, 47 + bob] } : swing[strike];
  line(neck.x + 3, neck.y + 2, held.hand[0], held.hand[1], bone, 2);
  line(held.hand[0], held.hand[1], held.tip[0], held.tip[1], rust, 2);
  r(held.hand[0] - 1, held.hand[1] - 1, 3, 3, grip);
}

function shambler({ r, line, lit }, { stride, bob, lean, strike }) {
  const skin = "#6f7562", far = "#4f5446", rags = "#3b4640", ragsLight = "#4c5650", trousers = "#39352e";
  // Slow shuffle: half the stride, a heavier lurch and a permanent forward stoop.
  const step = Math.round(stride * 0.6);
  const sway = bob + (Math.abs(stride) === 3 ? 1 : 0);
  const hip = { x: -2 + lean / 3, y: 41 + sway };
  line(hip.x - 1, hip.y, -4 - step, 56, far, 3);
  r(-6 - step, 56, 5, 1, ink);
  const reach = strike === null ? 0 : [-2, -4, 6, 9, 5, 1][strike];
  const raise = strike === null ? 0 : [0, 2, -9, -12, -6, -1][strike];
  line(hip.x + 1, hip.y - 13, 7 + lean + reach, 41 + sway + raise, far, 2);
  r(hip.x - 6 + lean / 2, 27 + sway, 14, 15, rags);
  r(hip.x - 5 + lean / 2, 25 + sway, 10, 3, rags);
  lit(hip.x - 6 + lean / 2, 27 + sway, 14, 13, ragsLight);
  r(hip.x - 6 + lean / 2, 42 + sway, 2, 2, rags);
  r(hip.x + 1 + lean / 2, 42 + sway, 3, 1, rags);
  line(hip.x + 2, hip.y, 1 + step, 56, trousers, 3);
  r(-1 + step, 56, 5, 1, ink);
  const head = { x: 4 + lean, y: 20 + sway };
  r(head.x - 1, head.y, 10, 10, ink);
  r(head.x, head.y + 1, 8, 8, skin);
  r(head.x, head.y + 1, 8, 2, "#493b2c");
  lit(head.x, head.y + 3, 8, 5, "#858a72");
  r(head.x + 5, head.y + 4, 2, 1, "#c6c8a3");
  r(head.x + 4, head.y + 7, 3 + (strike === 3 ? 1 : 0), 1 + (strike === 3 ? 1 : 0), ink);
  line(hip.x + 3, hip.y - 12, 10 + lean + reach, 42 + sway + raise, skin, 2);
  r(10 + lean + reach, 42 + sway + raise, 3, 2, skin);
}

function rat({ r, line, lit }, { stride, bob, lean, strike }) {
  const fur = "#4c4a43", far = "#38372f", light = "#65716b", tail = "#796b52";
  const hop = strike === null ? bob : [0, 1, -2, -3, -1, 0][strike];
  const body = { x: -12 + lean, y: 45 + hop };
  line(body.x + 3, body.y + 6, body.x + 2 + stride, 56, far);
  line(body.x + 17, body.y + 6, body.x + 18 - stride, 56, far);
  line(body.x, body.y + 4, -26, 46 + (stride > 0 ? 1 : 0) - hop, tail);
  r(body.x, body.y, 22, 8, ink);
  r(body.x + 1, body.y + 1, 20, 6, fur);
  r(body.x + 4, body.y + 1, 12, 1, light);
  lit(body.x + 1, body.y + 1, 20, 5, light);
  const bite = strike === 3;
  const head = { x: body.x + 20 + (bite ? 2 : 0), y: body.y - 2 };
  r(head.x, head.y, 9, 7, fur);
  r(head.x + 8, head.y + 3, 3, bite ? 2 : 3, fur);
  r(head.x + 10, head.y + 4, 1, 1, tail);
  if (bite) { r(head.x + 8, head.y + 5, 3, 1, ink); r(head.x + 9, head.y + 6, 2, 1, "#c6c8a3"); }
  r(head.x + 6, head.y + 2, 1, 1, "#af7449");
  r(head.x + 1, head.y - 2, 2, 2, tail);
  line(body.x + 5, body.y + 6, body.x + 5 - stride, 56, fur);
  line(body.x + 15, body.y + 6, body.x + 15 + stride, 56, fur);
}

const painters = { skeleton, shambler, rat };

export function drawCreature(canvas, id, { facing, action, frame }) {
  if (!painters[id] || !["left", "right"].includes(facing) || !creatureActions.includes(action) ||
      !Number.isInteger(frame) || frame < 0 || frame >= 6) {
    throw new Error("Invalid creature, facing, action or frame.");
  }
  painters[id](painter(canvas, facing), pose(action, frame));
}
