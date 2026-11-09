// Pure race, appearance and overhead sprite construction; no DOM side effects.
import { drawEquipment } from "./equipment.js";

export const directions = [
  { id: "ashen-pilgrim", name: "Ashen Pilgrim", note: "Gaunt travellers in torn wool and dark leather. Soot, bone, and dying embers.", number: "01" },
  { id: "gravebound", name: "Gravebound", note: "Shrouded figures, sunken faces, ragged charcoal cloaks. Quiet dread.", number: "02" },
  { id: "iron-remnant", name: "Iron Remnant", note: "Scarred survivors in tarnished iron and frayed mantles. Nothing clean or new.", number: "03" },
];

export const races = [
  { id: "human", name: "Human", height: 1, width: 1, skin: "warm", note: "Adaptable people; balanced proportions and rounded ears." },
  { id: "elf", name: "Elf", height: 1.08, width: .86, skin: "warm", ears: "long", note: "Ancient and watchful; tall, fine frames and long tapered ears." },
  { id: "dwarf", name: "Dwarf", height: .81, width: 1.23, skin: "warm", note: "Stoneborn; low, broad frames, strong brows and substantial hands." },
  { id: "halfling", name: "Halfling", height: .66, width: .86, skin: "warm", feet: true, note: "Small and nimble; rounded faces, cropped trousers and broad bare feet." },
  { id: "orc", name: "Orc", height: 1.06, width: 1.2, skin: "green", tusks: true, ears: "short", note: "Relentless; powerful frames, pronounced brows, pointed ears and tusks." },
  { id: "gnome", name: "Gnome", height: .68, width: .8, skin: "warm", ears: "short", locked: true, note: "Unlockable study: compact frames, large ears, expressive noses. Details provisional." },
  { id: "half-elf", name: "Half-Elf", height: 1.03, width: .94, skin: "warm", ears: "short", locked: true, note: "Unlockable study: human proportions with subtle tapered ears. Details provisional." },
  { id: "dark-elf", name: "Dark Elf", height: 1.08, width: .86, skin: "ash", ears: "long", locked: true, note: "Unlockable study: fine frames, ash-violet skin and pale accents. Details provisional." },
  { id: "half-giant", name: "Half-Giant", height: 1.12, width: 1.35, skin: "warm", locked: true, note: "Unlockable study: the tallest, heaviest frame inside this tile. Details provisional." },
  { id: "revenant", name: "Revenant", height: 1, width: .92, skin: "dead", undead: true, locked: true, note: "Unlockable study: sunken features, pallid skin and a spectral gaze. Details provisional." },
];

export const looks = [
  { name: "Wayfarer", description: "Close-cropped hair, clear features, an unadorned travelling tunic.", hair: 0, skin: 1, hairTone: 1 },
  { name: "Ember", description: "Swept hair, a nick through the brow, copper and warm earth.", hair: 1, skin: 2, hairTone: 2 },
  { name: "Thorn", description: "Braided lengths, freckled cheeks, a dark forest accent.", hair: 2, skin: 3, hairTone: 0 },
  { name: "Silver", description: "A tied-back mane, weathered features, silver against stone.", hair: 3, skin: 0, hairTone: 4 },
  { name: "Dusk", description: "Shaved sides, a high crest, a fine cheek marking.", hair: 4, skin: 4, hairTone: 3 },
];

// These original ramps are provisional, not the future locked Lospec-derived PRL palette.
export const skinRamps = {
  warm: [
    ["#775448", "#ad7c63", "#d4a282", "#efd0a5"],
    ["#5e4034", "#97674c", "#c39267", "#ddb58a"],
    ["#4c322b", "#794d36", "#a6724e", "#cb9668"],
    ["#342923", "#563b2c", "#82573b", "#ad7c55"],
    ["#251e1c", "#3d2c27", "#624333", "#906347"],
  ],
  green: [
    ["#354337", "#5e7453", "#8a9e6b", "#b2bd88"],
    ["#2d382b", "#4f6240", "#788957", "#9eab72"],
    ["#30382c", "#646543", "#8c8958", "#b0a775"],
    ["#242e29", "#3b5144", "#57735c", "#80967a"],
    ["#1d2926", "#31443c", "#49604f", "#6d8264"],
  ],
  ash: [
    ["#3d3b51", "#65607b", "#928aa5", "#b7afc5"],
    ["#302f43", "#4b4b66", "#73728f", "#9998af"],
    ["#3a3044", "#5a475f", "#856a85", "#af92a5"],
    ["#262938", "#3a4052", "#5c6577", "#88939f"],
    ["#231f30", "#383044", "#574965", "#7e6d89"],
  ],
  dead: [
    ["#474c40", "#767c63", "#a5aa8b", "#cacfad"],
    ["#39453f", "#5b7161", "#849888", "#abb9a4"],
    ["#493f43", "#736268", "#9d8588", "#c0aaa7"],
    ["#333c3c", "#546665", "#7c908a", "#a4b3a5"],
    ["#2c3033", "#454f54", "#6b7c7f", "#95a3a0"],
  ],
};
export const hairRamps = [
  ["#171d1c", "#29332e", "#465047"],
  ["#252321", "#493b2c", "#786047"],
  ["#392824", "#744331", "#af7449"],
  ["#50422e", "#948154", "#c7b77a"],
  ["#4c5350", "#8c9890", "#cad2bb"],
];
const clothRamps = [
  ["#242d2c", "#3e514c", "#687e67", "#97a180"],
  ["#362c29", "#60483a", "#927054", "#b4996c"],
  ["#222d28", "#3b513b", "#627653", "#94a074"],
  ["#2d3035", "#515b60", "#7a8987", "#aab6a2"],
  ["#302b37", "#504350", "#7f6373", "#ad8a90"],
];
export function drawSprite(canvas, selection) {
  canvas.width = 64;
  canvas.height = 64;
  const ctx = canvas.getContext("2d");
  const race = races[selection.race];
  const style = selection.equipment ? 1 : selection.direction;
  const variant = selection.look;
  const skin = skinRamps[race.skin][selection.skin];
  const hair = hairRamps[selection.hair];
  const cloth = clothRamps[variant];
  const ink = "#111514";
  const step = 1;
  const rect = (x, y, w, h, colour) => {
    ctx.fillStyle = colour;
    ctx.fillRect(Math.round(x / step) * step, Math.round(y / step) * step,
      Math.max(step, Math.round(w / step) * step), Math.max(step, Math.round(h / step) * step));
  };
  const poly = (points, colour) => {
    // Scanline fill keeps every edge on the native pixel grid, without canvas antialiasing.
    const low = Math.floor(Math.min(...points.map(p => p[1])));
    const high = Math.ceil(Math.max(...points.map(p => p[1])));
    for (let y = low; y < high; y += step) {
      const intersections = [];
      for (let i = 0; i < points.length; i++) {
        const a = points[i], b = points[(i + 1) % points.length];
        if ((a[1] <= y && b[1] > y) || (b[1] <= y && a[1] > y)) {
          intersections.push(a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]));
        }
      }
      intersections.sort((a, b) => a - b);
      for (let i = 0; i + 1 < intersections.length; i += 2) {
        rect(intersections[i], y, intersections[i + 1] - intersections[i], step, colour);
      }
    }
  };
  const feminine = selection.body === 1;
  const worldView = selection.perspective === "overhead-quarter";
  const top = Math.round(59 - (worldView ? 41 : 48) * race.height);
  const headW = Math.round((feminine ? 8 : 9) * Math.min(race.width, 1.18));
  const headH = worldView ? 7 : race.height < .85 ? 10 : 9;
  const headX = 32 - headW / 2;
  const chin = top + headH;
  const shoulder = Math.round((feminine ? 7 : 9) * race.width);
  const waist = Math.round((feminine ? 4 : 6) * race.width);
  const hip = Math.round((feminine ? 7 : 6) * race.width);
  const beltY = Math.round(chin + (57 - chin) * .45);
  const hipY = beltY + 6;
  const shirtTop = chin + 2;
  const eyeY = top + 4;

  poly([[21, 59], [27, 57], [38, 57], [44, 59], [38, 61], [27, 61]], "#1b211e");
  // A broken cloak outline and asymmetric folds avoid a rigid, toy-like silhouette.
  const mantle = style === 1 ? ["#151b1c", "#222b2c", "#394344"] :
    style === 2 ? ["#211f1d", "#39352e", "#575041"] : ["#23201b", "#3e382c", "#5d513c"];
  if (!selection.equipment || selection.equipment.cloak.type !== "none") {
  poly([[32 - shoulder - 2, shirtTop + 2], [28, chin], [36, chin],
    [34 + shoulder, shirtTop + 2], [35 + shoulder, 53],
    [34 + shoulder, 56], [31 + shoulder, 54], [30 + shoulder, 57],
    [36, 55], [31, 57], [26, 54], [29 - shoulder, 56],
    [27 - shoulder, 52]], ink);
  poly([[31 - shoulder, shirtTop + 3], [28, chin + 1], [36, chin + 1],
    [33 + shoulder, shirtTop + 3], [33 + shoulder, 52],
    [30 + shoulder, 55], [34, 53], [29, 55], [26, 52],
    [29 - shoulder, 54]], mantle[0]);
  poly([[31 - shoulder, shirtTop + 4], [29, chin + 2], [28, 48],
    [29 - shoulder, 53]], mantle[1]);
  poly([[33 + shoulder, shirtTop + 5], [36, chin + 2], [38, 50],
    [31 + shoulder, 54]], mantle[1]);
  poly([[30 - shoulder, shirtTop + 5], [29, chin + 4], [28 - shoulder, 48],
    [29 - shoulder, 51]], mantle[2]);
  }
  // Back hair is independent of silhouette.
  if (variant === 2 || variant === 3) {
    rect(headX - 2, top + 3, headW + 4, headH + 12, hair[0]);
    rect(headX - 1, top + 7, 3, headH + 8, hair[1]);
    rect(headX + headW - 2, top + 7, 3, headH + 8, hair[1]);
  }
  // Legs share the same kit; silhouette affects their spacing, not equipment.
  const legW = Math.round((feminine ? 3 : 4) * race.width);
  const legStart = hipY - 3;
  for (const side of [-1, 1]) {
    const x = side < 0 ? 31 - legW : 33;
    const knee = Math.round((legStart + 58) / 2);
    poly([[x - 1, legStart], [x + legW + 1, legStart], [x + legW, knee],
      [x + legW + side, 58], [x + side - 1, 59], [x - 1, knee]], ink);
    poly([[x, legStart], [x + legW, legStart], [x + legW - 1, knee],
      [x + legW + side - 1, 57], [x + side, 57], [x, knee]], cloth[0]);
    poly([[x, legStart + 1], [x + 2, legStart + 1], [x + 1, knee],
      [x + side + 1, 54], [x + side, 54]], cloth[1]);
    const boot = race.feet ? skin : ["#211f1d", "#39352e", "#575041", "#756750"];
    poly([[x + side, knee + 2], [x + legW + side, knee + 1],
      [x + legW + side, 56], [x + legW + side + (side > 0 ? 2 : 0), 58],
      [x + side - (side < 0 ? 2 : 0), 58]], boot[0]);
    rect(x + side, knee + 3, Math.max(1, legW - 1), 58 - knee - 5, boot[1]);
    rect(x + side, knee + 3, 1, 4, boot[2]);
    rect(x + side, 56, legW, 1, boot[2]);
  }
  // Arms and hands.
  for (const side of [-1, 1]) {
    const x = 32 + side * shoulder - 2;
    const elbow = shirtTop + Math.round((beltY - shirtTop) * .65);
    poly([[x, shirtTop + 2], [x + 4, shirtTop + 3], [x + 4 + side, elbow],
      [x + 3 + side, beltY + 6], [x + side, beltY + 7], [x + side - 1, elbow]], ink);
    poly([[x + 1, shirtTop + 3], [x + 3, shirtTop + 4], [x + 3 + side, elbow],
      [x + 2 + side, beltY + 4], [x + side, beltY + 4], [x + side, elbow]], cloth[1]);
    rect(x + 1, shirtTop + 4, 1, Math.max(1, elbow - shirtTop - 5), cloth[2]);
    rect(x + side, elbow, 3, 2, mantle[2]);
    rect(x + side, beltY + 4, 2, 3, skin[0]);
    rect(x + side, beltY + 4, 1, 2, skin[2]);
  }
  poly([[32 - shoulder, shirtTop], [32 - 5, chin], [37, chin], [32 + shoulder, shirtTop],
    [32 + waist + 1, beltY], [32 + hip, hipY], [32 - hip, hipY], [31 - waist, beltY]], ink);
  poly([[33 - shoulder, shirtTop + 1], [28, chin + 1], [36, chin + 1], [31 + shoulder, shirtTop + 1],
    [32 + waist - 1, beltY], [31 + hip, hipY - 1], [33 - hip, hipY - 1], [33 - waist, beltY]], cloth[1]);
  poly([[33 - shoulder, shirtTop + 2], [29, chin + 3], [30, beltY - 1], [33 - waist, beltY - 1]], cloth[2]);
  poly([[36, chin + 3], [31 + shoulder, shirtTop + 2], [31 + waist, beltY - 1], [35, beltY - 1]], cloth[0]);
  rect(30, chin - 1, 4, 4, skin[0]);
  rect(31, chin, 2, 2, skin[1]);
  poly([[28, chin + 1], [31, chin + 5], [30, chin + 7], [26, chin + 3]], mantle[2]);
  poly([[36, chin + 1], [33, chin + 5], [34, chin + 7], [38, chin + 3]], mantle[1]);
  rect(32 - waist, beltY - 1, waist * 2, 3, "#252321");
  rect(31, beltY - 1, 2, 2, "#756750");
  rect(32, beltY, 1, 1, "#ad9b75");
  rect(32 + waist, beltY, 3, 5, "#39352e");
  rect(32 + waist, beltY, 2, 1, "#756750");
  // Diagonal leather strap, fraying, and seams break up the flat torso.
  poly([[27, chin + 3], [29, chin + 3], [35, beltY], [33, beltY]], "#39352e");
  for (let y = chin + 5; y < beltY; y += 3) rect(28 + (y - chin) * .3, y, 1, 1, "#756750");
  for (let x = 33 - hip; x < 31 + hip; x += 3) {
    rect(x, hipY - 1 + (x % 2), 1, 2, mantle[2]);
  }
  rect(34, beltY - 4, 2, 1, ink);
  rect(35, beltY - 3, 1, 2, cloth[2]);
  if (style === 1 && !selection.equipment) {
    poly([[28, chin], [32, chin + 2], [36, chin], [38, chin + 5],
      [33, chin + 7], [26, chin + 5]], mantle[0]);
    rect(29, chin + 3, 6, 1, mantle[2]);
    poly([[31 - shoulder, shirtTop], [29, chin + 2], [29, hipY + 5],
      [25, hipY + 3], [28 - shoulder, hipY + 7]], mantle[1]);
    rect(27, beltY + 2, 1, 4, mantle[2]);
  } else if (style === 2) {
    const iron = ["#22292a", "#3f4948", "#68726b", "#9b9d87"];
    poly([[30 - shoulder, shirtTop + 2], [28, shirtTop + 1],
      [29, shirtTop + 5], [31 - shoulder, shirtTop + 7], [29 - shoulder, shirtTop + 5]], iron[0]);
    poly([[31 - shoulder, shirtTop + 2], [27, shirtTop + 2],
      [28, shirtTop + 4], [31 - shoulder, shirtTop + 5]], iron[2]);
    poly([[36, shirtTop + 1], [33 + shoulder, shirtTop + 3],
      [33 + shoulder, shirtTop + 6], [36, shirtTop + 5]], iron[1]);
    poly([[29, chin + 5], [35, chin + 5], [36, beltY - 2],
      [29, beltY - 2]], iron[1]);
    rect(29, chin + 6, 1, Math.max(1, beltY - chin - 9), iron[2]);
    rect(33, chin + 7, 2, 1, iron[2]);
    rect(34, beltY - 5, 1, 2, "#744331");
    rect(31, beltY - 3, 2, 1, iron[0]);
  }
  // Ears remain visible outside the hair.
  for (const side of [-1, 1]) {
    const x = side < 0 ? headX : headX + headW;
    if (race.ears) {
      const reach = race.ears === "long" ? 5 : 3;
      poly([[x, eyeY - 2], [x + side * reach, eyeY - 4], [x + side * (reach - 2), eyeY + 1], [x, eyeY + 3]], ink);
      poly([[x, eyeY - 1], [x + side * (reach - 1), eyeY - 3], [x + side * 2, eyeY + 1], [x, eyeY + 2]], skin[2]);
      rect(x + (side < 0 ? -2 : 0), eyeY, 2, 2, skin[0]);
    } else {
      rect(x - (side < 0 ? 2 : 0), eyeY - 1, 2, 5, skin[0]);
      rect(x - (side < 0 ? 1 : 0), eyeY, 2, 3, skin[2]);
    }
  }
  const jawInset = feminine ? 2 : 1;
  poly([[headX + 2, top], [headX + headW - 2, top], [headX + headW, top + 3],
    [headX + headW - 1, chin - 3], [headX + headW - jawInset, chin],
    [headX + jawInset, chin], [headX, chin - 3], [headX, top + 3]], ink);
  poly([[headX + 2, top + 1], [headX + headW - 2, top + 1], [headX + headW - 1, top + 4],
    [headX + headW - 2, chin - 3], [headX + headW - jawInset - 1, chin - 1],
    [headX + jawInset + 1, chin - 1], [headX + 1, chin - 4], [headX + 1, top + 4]], skin[1]);
  rect(headX + 2, top + 2, headW - 4, 2, skin[2]);
  rect(headX + 1, eyeY + 2, 2, 2, skin[2]);
  rect(headX + headW - 3, eyeY + 2, 2, 2, skin[0]);
  rect(31, eyeY + 1, 2, 2, skin[0]);
  rect(31, eyeY + 1, 1, 2, skin[3]);
  const eyeColour = race.undead ? "#97c8b8" : skin[3];
  rect(headX + 1, eyeY, 2, 2, skin[0]);
  rect(headX + headW - 3, eyeY, 2, 2, ink);
  rect(headX + 2, eyeY, 1, 1, eyeColour);
  if (race.undead) rect(headX + headW - 3, eyeY, 1, 1, eyeColour);
  rect(headX + 1, eyeY - 1, 2, 1, hair[0]);
  rect(headX + headW - 3, eyeY - 1, 2, 1, hair[0]);
  rect(30, chin - 2, 3, 1, skin[0]);
  if (race.tusks) {
    rect(29, chin - 3, 1, 2, "#ad9b75");
    rect(34, chin - 3, 1, 2, "#ad9b75");
  }
  if (race.id === "gnome") {
    rect(30, eyeY + 2, 4, 3, skin[2]);
    rect(31, eyeY + 2, 2, 1, skin[3]);
  }
  if (race.undead) {
    rect(headX + 1, eyeY + 2, 2, 2, skin[0]);
    rect(headX + headW - 3, eyeY + 2, 2, 2, skin[0]);
  }
  // Five hair constructions, not recolours of one head.
  if (variant === 0) {
    rect(headX + 1, top, headW - 2, 2, hair[0]);
    rect(headX + 2, top, headW - 4, 1, hair[1]);
    rect(headX + 1, top + 3, 2, 3, hair[1]);
    rect(headX + headW - 3, top + 3, 2, 3, hair[0]);
  } else if (variant === 1) {
    poly([[headX, top + 4], [headX + 2, top - 2], [headX + headW - 3, top - 3],
      [headX + headW + 1, top + 2], [headX + 6, top + 3], [headX + 3, top + 7]], hair[0]);
    poly([[headX + 2, top + 2], [headX + 4, top - 1], [headX + headW - 3, top - 2],
      [headX + headW - 1, top], [headX + 5, top + 3]], hair[1]);
    rect(headX + 3, top - 1, Math.max(1, headW - 6), 1, hair[2]);
    rect(headX + headW - 4, eyeY - 2, 1, 4, skin[3]);
  } else if (variant === 2) {
    rect(headX, top, headW, 3, hair[0]);
    rect(headX + 2, top, headW - 4, 1, hair[2]);
    for (const x of [headX - 1, headX + headW - 1]) {
      for (let y = top + 3; y < chin + 10; y += 3) {
        rect(x, y, 2, 2, hair[1]);
        rect(x + 1, y, 1, 1, hair[2]);
      }
      rect(x, chin + 8, 2, 1, "#756750");
    }
    rect(headX + 3, eyeY + 3, 1, 1, skin[0]);
    rect(headX + 5, eyeY + 4, 1, 1, skin[0]);
    rect(headX + headW - 4, eyeY + 3, 1, 1, skin[0]);
  } else if (variant === 3) {
    rect(headX, top, headW, 4, hair[0]);
    rect(headX + 1, top, headW - 2, 2, hair[1]);
    rect(headX + 2, top, headW - 4, 1, hair[2]);
    rect(headX, top + 3, 2, 5, hair[1]);
    rect(headX + headW - 2, top + 3, 2, 5, hair[1]);
    rect(headX + 1, eyeY + 2, 2, 1, skin[0]);
    rect(headX + headW - 3, eyeY + 2, 2, 1, skin[0]);
    if (!feminine && race.id === "dwarf") {
      poly([[headX + 1, chin - 5], [31, chin - 2], [headX + headW - 1, chin - 5],
        [headX + headW - 2, chin + 4], [32, chin + 7], [headX + 2, chin + 4]], hair[1]);
      rect(30, chin + 1, 1, 4, hair[2]);
    }
  } else {
    rect(headX + 1, top + 1, headW - 2, 3, skin[0]);
    poly([[30, top + 3], [30, top - 2], [33, top - 3], [35, top - 1], [34, top + 3]], hair[0]);
    rect(31, top - 1, 2, 3, hair[1]);
    rect(32, top - 2, 1, 2, hair[2]);
    rect(headX + 3, eyeY + 3, 1, 3, "#7f6373");
  }
  rect(33 - waist, beltY - 4, 1, 2, cloth[2]);
  rect(35, hipY - 2, 1, 1, mantle[2]);
  if (selection.equipment) {
    drawEquipment(selection.equipment,
      { top, chin, headX, headW, shoulder, waist, hip, beltY, hipY, shirtTop }, rect, poly);
  }
}

export function drawPortrait(canvas, selection) {
  canvas.width = 128;
  canvas.height = 128;
  const ctx = canvas.getContext("2d");
  const sprite = document.createElement("canvas");
  drawSprite(sprite, selection);
  const race = races[selection.race];
  const top = Math.round(59 - 48 * race.height);
  ctx.imageSmoothingEnabled = false;
  ctx.fillStyle = "#242d2c";
  ctx.fillRect(0, 0, 128, 128);
  // A four-times face crop is a study, not a finished high-resolution portrait.
  ctx.drawImage(sprite, 16, Math.max(0, top - 6), 32, 32, 0, 0, 128, 128);
}
