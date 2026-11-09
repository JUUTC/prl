import { materials } from "./equipment.js";

export const actions = [
  ["idle", "Idle"], ["walk", "Walk"], ["attack", "Attack"],
  ["craft", "Craft / hammer"], ["mine", "Mine"], ["chop", "Chop wood"],
  ["gather", "Gather"], ["eat", "Eat"], ["drink", "Drink"],
  ["heal", "Heal / bandage"], ["cast", "Cast"], ["shoot", "Archery release"],
];
export const frameCount = 6;
export const frameDuration = 140;
export const twoHanded = new Set(["greatsword", "pickaxe", "axe", "bow"]);

// Independently authored anatomical attachments: main hand is near in right-facing
// art and far in left-facing art. Item identity never changes with facing.
const facings = {
  right: {
    head: [33, -1], near: "main", mainShoulder: [30, 0], offShoulder: [35, -2],
    main: [[28, 13], [29, 12], [30, 13], [31, 13], [30, 14], [29, 14]],
    off: [[39, 10], [38, 11], [37, 11], [38, 10], [39, 9], [39, 10]],
    strike: [[31, 12], [25, 5], [30, -1], [43, 9], [42, 13], [33, 13]],
    support: [[36, 10], [32, 9], [33, 6], [39, 13], [38, 14], [36, 11]],
    shaft: [[.1, -1], [-.7, -.7], [.1, -1], [.9, .35], [.8, .1], [.2, -1]],
    feet: [[-2, 2], [0, 0], [3, -3], [2, -2], [0, 0], [-3, 3]],
  },
  left: {
    head: [30, 0], near: "off", mainShoulder: [29, -2], offShoulder: [34, 0],
    main: [[23, 10], [24, 10], [25, 11], [24, 12], [23, 11], [22, 10]],
    off: [[35, 14], [34, 13], [33, 13], [34, 14], [35, 15], [36, 14]],
    strike: [[25, 10], [30, 3], [26, -2], [18, 8], [19, 12], [25, 11]],
    support: [[29, 13], [32, 8], [29, 5], [23, 12], [24, 15], [30, 13]],
    shaft: [[-.2, -1], [.65, -.75], [-.1, -1], [-.9, .3], [-.8, .15], [-.3, -1]],
    feet: [[3, -2], [1, -1], [-2, 3], [-3, 2], [-1, 0], [2, -3]],
  },
};

export function facingForMove(current, dx) {
  return dx < 0 ? "left" : dx > 0 ? "right" : current;
}

export function poseFor(selection, facing, action, frame, race) {
  const f = facings[facing];
  if (!f || !actions.some(([id]) => id === action) ||
    !Number.isInteger(frame) || frame < 0 || frame >= frameCount) {
    throw new Error("Invalid animation facing, action, or frame.");
  }
  const top = Math.round(59 - 41 * race.height);
  const chin = top + 7;
  const active = frame >= 2 && frame <= 4;
  const crouch = ["gather", "heal"].includes(action) ? [0, 1, 4, 5, 3, 1][frame] : 0;
  const bob = action === "walk" ? [0, -1, 0, 0, -1, 0][frame] : crouch;
  const baseY = chin + 4 + bob;
  const at = pair => [pair[0], baseY + pair[1] * Math.min(1, race.height)];
  let main = at(f.main[frame]), off = at(f.off[frame]);
  let vector = f.shaft[0];
  let mainType = selection.equipment.main.type, offType = selection.equipment.off.type;
  const working = ["mine", "chop", "craft"].includes(action);
  if (working || action === "attack") {
    main = at(f.strike[frame]); vector = f.shaft[frame];
    if (working) { mainType = action === "mine" ? "pickaxe" : action === "chop" ? "axe" : "hammer"; offType = "none"; }
    if (working && frame === 3) {
      main = [facing === "right" ? 36 : 24, action === "craft" ? 44 : 43];
    }
    if (twoHanded.has(mainType)) {
      // Both grips lie on the same shaft, with the support hand below the main grip.
      off = [main[0] - vector[0] * 5, main[1] - vector[1] * 5];
    } else if (action === "craft") off = at(f.support[frame]);
  } else if (twoHanded.has(mainType)) {
    off = [main[0] - vector[0] * 5, main[1] - vector[1] * 5];
  }
  if (action === "shoot") {
    mainType = "bow"; offType = "none";
    main = [facing === "right" ? 41 : 22, Math.min(45, chin + 10)];
    off = [main[0] + (facing === "right" ? -1 : 1) * [1, 4, 8, 10, 1, 0][frame], main[1]];
  }
  if (["gather", "eat", "drink", "heal", "cast"].includes(action)) {
    mainType = action === "eat" ? "food" : action === "drink" ? "cup" : action === "heal" ? "bandage" : "none";
    offType = "none";
    if (action === "eat" || action === "drink") main = [f.head[0] + (facing === "right" ? 5 : -5), chin - 1 + (active ? 0 : 9)];
    if (action === "heal") { main = [32, chin + 13 + bob]; off = [34, chin + 12 + bob]; }
    if (action === "gather") { main = [facing === "right" ? 42 : 21, 50 + (active ? 3 : 0)]; off = [32, chin + 14 + bob]; }
    if (action === "cast") {
      main = [facing === "right" ? 44 : 18, chin + (active ? 3 : 10)];
      off = [facing === "right" ? 36 : 27, chin + (active ? -1 : 11)];
    }
  }
  return { facing, action, frame, top, chin, bob, main, off, vector, mainType, offType,
    mainShoulder: at(f.mainShoulder), offShoulder: at(f.offShoulder), near: f.near,
    head: [f.head[0], top + f.head[1] + bob], feet: action === "walk" ? f.feet[frame] : [0, 0] };
}

export function drawAnimated(canvas, selection, playback, drawSprite, appearance) {
  canvas.width = 64; canvas.height = 64;
  const { race, skin, hair } = appearance;
  const p = poseFor(selection, playback.facing, playback.action, playback.frame, race);
  const ctx = canvas.getContext("2d");
  ctx.imageSmoothingEnabled = false;
  const ink = "#111514";
  const rect = (x, y, w, h, colour) => {
    ctx.fillStyle = colour;
    ctx.fillRect(Math.round(x), Math.round(y), Math.max(1, Math.round(w)), Math.max(1, Math.round(h)));
  };
  const line = (a, b, colour, width = 1) => {
    const steps = Math.max(1, Math.ceil(Math.max(Math.abs(b[0] - a[0]), Math.abs(b[1] - a[1]))));
    for (let i = 0; i <= steps; i++) rect(a[0] + (b[0] - a[0]) * i / steps,
      a[1] + (b[1] - a[1]) * i / steps, width, width, colour);
  };
  const g = selection.equipment;
  const base = document.createElement("canvas");
  const stripped = structuredClone(g);
  for (const slot of ["main", "off", "hands", "feet"]) stripped[slot].type = "none";
  drawSprite(base, { ...selection, equipment: stripped, perspective: "overhead-quarter" });
  const half = Math.round((selection.body === 0 ? 7 : 5) * race.width);
  const belt = Math.round(p.chin + (57 - p.chin) * .45);
  // Keep the existing equipment torso; limbs and head are separately posed.
  const bodyBottom = g.body.type === "robe" ? 56 : belt + 5;
  if (g.cloak.type !== "none") {
    rect(26 - half, p.chin + 5 + p.bob, 5, 53 - p.chin, "#222b2c");
    rect(33 + half, p.chin + 6 + p.bob, 4, 50 - p.chin, "#151b1c");
    rect(27 - half, 53 + p.bob, 2, 2, "#394344");
  }
  for (const [i, x] of [[0, 28], [1, 34]]) {
    const foot = p.feet[i];
    line([x, belt + 4 + p.bob], [x + foot, 53], "#30372e", 3);
    const c = g.feet.type === "none" ? skin : materials[g.feet.material][g.feet.type === "greaves" ? "metal" : "cloth"];
    line([x + foot, 52], [x + foot, 57], c[1], 3);
    rect(x + foot + (p.facing === "right" ? 1 : -2), 57, 5, 1, c[2]);
  }
  function arm(slot) {
    const start = p[`${slot}Shoulder`], hand = p[slot];
    const colours = materials[g.body.material].cloth;
    const elbow = [(start[0] + hand[0]) / 2 + (slot === "main" ? -2 : 2), (start[1] + hand[1]) / 2 + 2];
    line(start, elbow, ink, 4); line(elbow, hand, ink, 3);
    line(start, elbow, colours[1], 3); line(elbow, hand, colours[2], 2);
    const glove = g.hands.type === "none" ? skin :
      materials[g.hands.material][g.hands.type === "gauntlets" ? "metal" : "cloth"];
    rect(hand[0], hand[1], 2, 3, glove[1]);
    rect(hand[0], hand[1], 1, 1, glove[2]);
  }
  function held(slot) {
    const type = p[`${slot}Type`];
    if (type === "none") return;
    const h = p[slot], item = g[slot], metal = materials[item.material].metal;
    const v = slot === "main" ? p.vector : [0, -1];
    const length = type === "dagger" ? 10 : ["pickaxe", "axe", "staff", "greatsword"].includes(type) ? 16 : 13;
    const end = [h[0] + v[0] * length, h[1] + v[1] * length];
    const side = [-v[1], v[0]];
    const offset = (point, amount) => [point[0] + side[0] * amount, point[1] + side[1] * amount];
    if (["kite", "buckler", "heater"].includes(type)) {
      const size = type === "buckler" ? 4 : 6;
      const bottom = Math.min(9, 60 - Math.round(h[1]));
      for (let y = -5; y <= bottom; y++) {
        const w = Math.max(1, size - Math.max(0, y - 1) * .55);
        rect(h[0] - w, h[1] + y, w * 2, 1, ink);
        rect(h[0] - w + 1, h[1] + y, Math.max(1, w * 2 - 2), 1, metal[1]);
      }
      line([h[0], h[1] - 3], [h[0], h[1] + 5], metal[2]);
      if (item.level >= 50) rect(h[0] - 3, h[1], 6, 1, metal[3]);
    } else if (type === "book") {
      rect(h[0] - 2, h[1] - 3, 6, 8, ink);
      rect(h[0] - 1, h[1] - 2, 4, 6, materials[item.material].cloth[1]);
      rect(h[0] + 2, h[1] - 1, 1, 5, "#ad9b75");
    } else if (type === "cup") {
      rect(h[0] - 1, h[1] - 2, 4, 5, "#3d4847");
      rect(h[0] - 1, h[1] - 2, 4, 1, "#929780");
      rect(h[0] + 3, h[1] - 1, 2, 3, "#65716b");
    } else if (type === "food" || type === "bandage") {
      rect(h[0] - 1, h[1] - 1, 4, 3, type === "food" ? "#927054" : "#ad9b75");
      rect(h[0], h[1], 2, 1, "#c6c8a3");
    } else if (type === "bow") {
      line([h[0] + 3, h[1] - 17], [h[0] - 3, h[1]], "#756750", 2);
      line([h[0] - 3, h[1]], [h[0] + 3, h[1] + 15], "#756750", 2);
      if (p.action === "shoot") {
        line([h[0] + 3, h[1] - 17], p.off, "#ad9b75");
        line(p.off, [h[0] + 3, h[1] + 15], "#ad9b75");
      } else line([h[0] + 3, h[1] - 17], [h[0] + 3, h[1] + 15], "#ad9b75");
    } else {
      const wooden = ["pickaxe", "axe", "staff", "hammer", "mace"].includes(type);
      line([h[0] - v[0] * 7, h[1] - v[1] * 7], end, ink, 3);
      line([h[0] - v[0] * 6, h[1] - v[1] * 6], end, wooden ? "#756750" : metal[2], 2);
      if (["pickaxe", "axe", "hammer", "mace"].includes(type)) {
        const extent = type === "pickaxe" ? 6 : type === "axe" ? 5 : 3;
        line(offset(end, -extent), offset(end, extent), metal[1], type === "pickaxe" ? 2 : 4);
        line(offset(end, -extent), offset(end, extent - 1), metal[2]);
      } else if (type !== "staff") {
        line(offset([h[0] + v[0] * 3, h[1] + v[1] * 3], -3),
          offset([h[0] + v[0] * 3, h[1] + v[1] * 3], 3), metal[1]);
      }
      if (item.level >= 50) rect(end[0], end[1], 1, 2, metal[3]);
    }
  }
  const far = p.near === "main" ? "off" : "main";
  arm(far); held(far);
  ctx.drawImage(base, 32 - half, p.chin + 2, half * 2, bodyBottom - p.chin - 2,
    32 - half, p.chin + 2 + p.bob, half * 2, bodyBottom - p.chin - 2);
  // Left and right heads have distinct nose, ear, eye, and light placement.
  const [hx, hy] = p.head;
  const headW = Math.round(8 * Math.min(1.2, race.width));
  ctx.drawImage(base, 20, p.top - 4, 24, 12, hx - 12, hy - 4, 24, 12);
  if (g.head.type !== "helm") {
    rect(hx - 3, hy + 2, headW - 2, 5, skin[0]);
    if (p.facing === "right") {
      rect(hx, hy + 2, 3, 4, skin[1]); rect(hx + 3, hy + 3, 2, 2, skin[2]);
      rect(hx + 1, hy + 3, 1, 1, ink); rect(hx - 3, hy + 4, 1, 2, skin[2]);
    } else {
      rect(hx - 3, hy + 2, 3, 4, skin[1]); rect(hx - 5, hy + 4, 2, 1, skin[2]);
      rect(hx - 2, hy + 3, 1, 1, ink); rect(hx + 2, hy + 3, 1, 2, skin[2]);
    }
    if (g.head.type === "none") rect(hx - 3, hy, 6, 2, hair[1]);
    if (race.tusks) rect(hx + (p.facing === "right" ? 2 : -4), hy + 6, 1, 2, "#ad9b75");
  }
  arm(p.near); held(p.near);
  if (twoHanded.has(p.mainType)) arm(far); // Both hands visibly meet the shared shaft or bowstring.
  if (p.action === "shoot") {
    const dir = p.facing === "right" ? 1 : -1;
    const x = p.main[0] + dir * (p.frame >= 4 ? 9 : -3);
    line([x - dir * 8, p.main[1]], [x + dir * 4, p.main[1]], "#ad9b75");
    rect(x + dir * 4, p.main[1] - 1, 2, 2, "#929780");
  }
  const targetX = p.facing === "right" ? 50 : 10;
  if (p.action === "mine") {
    rect(targetX - 5, 49, 10, 9, "#515545");
    rect(targetX - 3, 49, 6, 2, "#696b55");
  } else if (p.action === "chop") {
    rect(targetX - 2, 24, 5, 34, "#39352e"); rect(targetX - 1, 24, 1, 30, "#756750");
  } else if (p.action === "craft") {
    rect(targetX - 7, 47, 14, 3, "#65716b"); rect(targetX - 3, 50, 6, 7, "#3d4847");
  } else if (p.action === "gather") {
    for (let i = 0; i < 3; i++) line([targetX + i * 2, 58], [targetX - 3 + i * 3, 52], "#657164", 2);
  }
  if (p.frame === 3 && ["mine", "chop", "craft"].includes(p.action)) {
    rect(targetX - 4, 44, 1, 2, "#ad9b75"); rect(targetX + 3, 45, 2, 1, "#756750");
  }
  if (p.action === "cast" && p.frame >= 2 && p.frame <= 4) {
    const c = ["#577d79", "#97c8b8", "#c6c8a3"];
    for (let i = 0; i < 4; i++) rect(p.main[0] + (p.facing === "right" ? 3 : -4) + i % 2,
      p.main[1] - 4 + i * 2, 2, 1, c[(i + p.frame) % 3]);
  }
  return p;
}
