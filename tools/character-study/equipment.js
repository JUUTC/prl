export const slots = [
  ["head", "Head", ["none", "hood", "coif", "helm"]],
  ["body", "Body", ["none", "leather", "robe", "chain", "scale", "banded"]],
  ["cloak", "Cloak", ["none", "shroud"]],
  ["hands", "Hands", ["none", "gloves", "gauntlets"]],
  ["feet", "Feet", ["none", "boots", "greaves"]],
  ["main", "Main hand", ["none", "sword", "dagger", "staff", "mace", "bow", "greatsword", "pickaxe", "axe", "hammer"]],
  ["off", "Off hand", ["none", "dagger", "kite", "buckler", "heater", "book"]],
  ["ammo", "Ammo", ["none", "quiver"]],
  ["amulet", "Amulet", ["none", "symbol"]],
];
export const labels = {
  none: "Unequipped", hood: "Hood", coif: "Mail coif", helm: "Helm",
  leather: "Leather armour", robe: "Robe", chain: "Chainmail", scale: "Scale mail",
  banded: "Banded mail", shroud: "Charcoal shroud", gloves: "Leather gloves",
  gauntlets: "Gauntlets", boots: "Leather boots", greaves: "Armoured boots",
  sword: "Longsword", dagger: "Dagger", staff: "Quarterstaff", mace: "Mace",
  bow: "Shortbow", kite: "Kite shield", buckler: "Buckler", heater: "Heater shield",
  book: "Spellbook", quiver: "Quiver", symbol: "Holy symbol",
  greatsword: "Two-handed sword", pickaxe: "Pickaxe (two hands)", axe: "Wood axe (two hands)", hammer: "Crafting hammer",
};
export const materials = [
  { name: "Iron / linen / rawhide", metal: ["#202727", "#3d4847", "#65716b", "#8e9483"], cloth: ["#222725", "#3b4640", "#657164", "#929780"] },
  { name: "Steel / wool / leather", metal: ["#20292e", "#44535a", "#78878a", "#a6b1a6"], cloth: ["#25231f", "#494237", "#796b52", "#a39371"] },
  { name: "Silver / silk / hardened leather", metal: ["#293034", "#58676b", "#98aba8", "#c2cbc0"], cloth: ["#24252b", "#454859", "#747a90", "#aaa9b1"] },
  { name: "Mithril / spidersilk / wyvern hide", metal: ["#202c33", "#3d606c", "#71929a", "#b0c7c3"], cloth: ["#22282b", "#3b5159", "#627b81", "#92a8a1"] },
  { name: "Adamantine / shadowweave / drake scale", metal: ["#1b1c27", "#393a53", "#68637b", "#a097a8"], cloth: ["#1c1e23", "#343943", "#575c6b", "#85858e"] },
  { name: "Starmetal / celestial silk / dragon scale", metal: ["#252a2a", "#535f5a", "#909b83", "#c6c8a3"], cloth: ["#272629", "#4c444d", "#7f6b7d", "#b4a0a5"] },
];
export const classes = [
  { name: "Warrior", body: "chain", head: "coif", hands: "gauntlets", feet: "greaves", main: "sword", off: "kite" },
  { name: "Rogue", body: "leather", hands: "gloves", feet: "boots", main: "dagger", off: "dagger" },
  { name: "Mage", body: "robe", head: "hood", main: "staff", off: "book" },
  { name: "Cleric", body: "scale", main: "mace", off: "buckler", amulet: "symbol" },
  { name: "Ranger", body: "leather", hands: "gloves", feet: "boots", main: "bow", ammo: "quiver" },
  { name: "Paladin", body: "banded", main: "sword", off: "heater", amulet: "symbol" },
];

export function startingEquipment(index) {
  return Object.fromEntries(slots.map(([slot]) => [slot, {
    type: classes[index][slot] ?? "none", level: 1, material: 0,
  }]));
}

export function validEquipment(gear) {
  return gear !== null && typeof gear === "object" && slots.every(([slot, , types]) => {
    const item = gear[slot];
    return item && types.includes(item.type) &&
      Number.isInteger(item.level) && item.level >= 1 && item.level <= 200 &&
      Number.isInteger(item.material) && item.material >= 0 && item.material < materials.length;
  }) && (!["greatsword", "pickaxe", "axe", "bow"].includes(gear.main.type) || gear.off.type === "none");
}

export function drawEquipment(gear, anchors, rect, poly) {
  const { top, chin, headX, headW, shoulder, waist, hip, beltY, hipY, shirtTop } = anchors;
  const ink = "#111514";
  const item = slot => gear[slot];
  const ramp = (slot, metal = false) => materials[item(slot).material][metal ? "metal" : "cloth"];
  // These are art-study sample bands, not game balance or affix thresholds.
  const detail = slot => item(slot).level >= 100 ? 3 : item(slot).level >= 50 ? 2 : item(slot).level >= 25 ? 1 : 0;
  const trim = (slot, x, y, width) => {
    if (detail(slot)) rect(x, y, width, 1, ramp(slot, true)[2]);
    if (detail(slot) >= 2) for (let dx = 0; dx < width; dx += 3) rect(x + dx, y + 2, 1, 1, ramp(slot, true)[3]);
  };
  const torso = (colour, inset = 0) => poly([
    [33 - shoulder + inset, shirtTop + 2], [29, chin + 3], [35, chin + 3],
    [31 + shoulder - inset, shirtTop + 2], [31 + waist, beltY - 1],
    [31 + hip, hipY - 1], [33 - hip, hipY - 1], [33 - waist, beltY - 1],
  ], colour);
  if (item("body").type !== "none") {
    const type = item("body").type;
    const metal = ["chain", "scale", "banded"].includes(type);
    const colours = ramp("body", metal);
    torso(colours[1]);
    poly([[33 - shoulder, shirtTop + 3], [29, chin + 4], [30, beltY - 2],
      [33 - waist, beltY - 2]], colours[2]);
    poly([[35, chin + 4], [31 + shoulder, shirtTop + 3], [31 + waist, beltY - 2],
      [34, beltY - 2]], colours[0]);
    if (type === "robe") {
      poly([[33 - waist, beltY - 2], [31 + waist, beltY - 2], [32 + hip + 1, 56],
        [35, 55], [33, 58], [30, 56], [31 - hip, 57]], ink);
      poly([[34 - waist, beltY - 1], [30 + waist, beltY - 1], [31 + hip, 55],
        [33, 56], [30, 54], [32 - hip, 55]], colours[1]);
      poly([[30, beltY], [31, beltY], [29, 55], [27, 54]], colours[2]);
      rect(33, beltY + 2, 1, 53 - beltY, colours[0]);
      trim("body", 30 - hip, 54, hip * 2);
    } else if (metal) {
      for (let y = chin + 5; y < hipY - 2; y += type === "banded" ? 3 : 2) {
        const half = y < beltY ? waist : hip - 1;
        for (let x = 33 - half; x < 31 + half; x += 3) {
          rect(x + (y % 2), y, type === "banded" ? 3 : 1, 1, colours[2]);
          if (type === "scale") rect(x + (y % 2), y + 1, 2, 1, colours[0]);
        }
      }
    } else {
      rect(29, chin + 5, 1, beltY - chin - 6, colours[0]);
      rect(34, chin + 5, 1, beltY - chin - 6, colours[2]);
    }
    trim("body", 33 - waist, beltY - 3, waist * 2 - 2);
    if (detail("body") >= 2) {
      for (const side of [-1, 1]) {
        const x = 32 + side * shoulder - 2;
        poly([[x - 1, shirtTop + 2], [x + 4, shirtTop + 1], [x + 5, shirtTop + 5],
          [x + 1, shirtTop + 7], [x - 2, shirtTop + 5]], ink);
        poly([[x, shirtTop + 2], [x + 3, shirtTop + 2], [x + 4, shirtTop + 4],
          [x + 1, shirtTop + 5]], colours[2]);
        if (detail("body") === 3) rect(x + 1, shirtTop + 3, 1, 1, colours[3]);
      }
    }
    rect(32 - waist, beltY - 1, waist * 2, 2, "#211f1d");
    rect(31, beltY - 1, 2, 2, "#756750");
  }
  if (item("cloak").type === "shroud") {
    const c = ramp("cloak");
    poly([[31 - shoulder, chin + 2], [29, chin + 1], [27, hipY + 8],
      [25, hipY + 5], [28 - shoulder, 55]], c[1]);
    poly([[35, chin + 1], [33 + shoulder, chin + 3], [34 + shoulder, 55],
      [38, 53]], c[0]);
    rect(29, chin + 3, 6, 2, c[2]);
    trim("cloak", 29, chin + 5, 6);
  }
  for (const side of [-1, 1]) {
    if (item("hands").type !== "none") {
      const x = 32 + side * shoulder - 2 + side;
      const c = ramp("hands", item("hands").type === "gauntlets");
      rect(x, beltY + 2, 3, 5, ink);
      rect(x, beltY + 3, 2, 3, c[1]);
      rect(x, beltY + 3, 1, 2, c[2]);
      trim("hands", x, beltY + 2, 3);
    }
    if (item("feet").type !== "none") {
      const x = side < 0 ? 28 - Math.round(waist / 3) : 34;
      const c = ramp("feet", item("feet").type === "greaves");
      rect(x, 50, 3, 7, c[0]);
      rect(x, 51, 2, 5, c[1]);
      rect(x, 51, 1, 4, c[2]);
      rect(x - (side < 0 ? 1 : 0), 57, 4, 1, c[1]);
      trim("feet", x, 50, 3);
    }
  }
  const head = item("head").type;
  if (head !== "none") {
    const c = ramp("head", head !== "hood");
    poly([[headX - 2, chin - 1], [headX - 2, top + 1], [headX + 1, top - 3],
      [headX + headW - 1, top - 3], [headX + headW + 2, top + 1],
      [headX + headW + 2, chin], [headX + headW - 1, chin + 1],
      [headX + headW - 1, top + 2], [headX + 1, top + 2], [headX + 1, chin + 1]], ink);
    poly([[headX - 1, chin], [headX - 1, top + 1], [headX + 1, top - 2],
      [headX + headW - 1, top - 2], [headX + headW + 1, top + 1],
      [headX + headW + 1, chin], [headX + headW, chin],
      [headX + headW - 1, top + 1], [headX + 1, top + 1], [headX, chin]], c[1]);
    rect(headX + 1, top - 1, headW - 2, 1, c[2]);
    if (head === "coif") {
      for (const x of [headX - 1, headX + headW]) {
        for (let y = top + 2; y < chin; y += 2) rect(x, y, 1, 1, c[2]);
      }
    } else if (head === "helm") {
      rect(headX, top + 3, headW, 3, c[0]);
      rect(headX + 1, top + 4, headW - 2, 1, ink);
      rect(32, top + 3, 1, 5, c[2]);
    }
    trim("head", headX, top, headW);
  }
  if (item("ammo").type === "quiver") {
    rect(39 + Math.round(shoulder / 2), shirtTop - 1, 4, 16, "#39352e");
    rect(39 + Math.round(shoulder / 2), shirtTop, 1, 14, ramp("ammo")[2]);
    for (let dx = 0; dx < 4; dx += 2) {
      rect(39 + Math.round(shoulder / 2) + dx, shirtTop - 6, 1, 8, "#756750");
      rect(38 + Math.round(shoulder / 2) + dx, shirtTop - 6, 2, 2, "#ad9b75");
    }
    trim("ammo", 39 + Math.round(shoulder / 2), shirtTop + 1, 4);
    if (detail("ammo") >= 2) rect(40 + Math.round(shoulder / 2), shirtTop + 8, 2, 1, ramp("ammo", true)[2]);
  }
  if (item("amulet").type === "symbol") {
    rect(31, chin + 3, 1, 4, "#756750");
    rect(30, chin + 7, 3, 1, ramp("amulet", true)[2]);
    rect(31, chin + 6, 1, 3, ramp("amulet", true)[3]);
    if (detail("amulet")) rect(29, chin + 7, 1, 1, ramp("amulet", true)[2]);
    if (detail("amulet") >= 2) rect(33, chin + 7, 1, 1, ramp("amulet", true)[2]);
    if (detail("amulet") === 3) rect(31, chin + 9, 1, 1, ramp("amulet", true)[3]);
  }
  function weapon(slot, side) {
    const type = item(slot).type;
    if (type === "none") return;
    const x = 32 + side * (shoulder + 4);
    const handY = beltY + 4;
    const c = ramp(slot, true);
    const d = detail(slot);
    if (["kite", "buckler", "heater"].includes(type)) {
      const w = type === "buckler" ? 4 : 6;
      const bottom = Math.min(60, handY + (type === "buckler" ? 6 : 13));
      const crest = type === "kite" ? handY - 9 : handY - 6;
      poly([[x - w, handY - 6], [x, crest], [x + w, handY - 6], [x + w, handY + 3],
        [x, bottom], [x - w, handY + 3]], ink);
      poly([[x - w + 1, handY - 5], [x, crest + 1], [x + w - 1, handY - 5], [x + w - 1, handY + 2],
        [x, bottom - 2], [x - w + 1, handY + 2]], c[1]);
      rect(x - w + 1, handY - 5, 1, 7, c[2]);
      rect(x - 1, handY - 1, 2, 3, c[2]);
      if (d) rect(x, handY - 4, 1, bottom - handY, c[2]);
      if (d >= 2) rect(x - 3, handY, 6, 1, c[3]);
      return;
    }
    if (type === "book") {
      rect(x - 3, handY - 6, 7, 10, ink);
      rect(x - 2, handY - 5, 5, 8, ramp(slot)[1]);
      rect(x + 2, handY - 4, 1, 6, "#ad9b75");
      trim(slot, x - 1, handY - 3, 3);
      return;
    }
    if (type === "bow") {
      const bowBottom = Math.min(60, handY + 12);
      poly([[x + 1, handY - 16], [x - 3, handY - 12], [x - 5, handY - 3],
        [x - 4, Math.min(57, handY + 7)], [x + 1, bowBottom], [x - 2, handY + 5],
        [x - 3, handY - 3], [x - 1, handY - 11]], "#756750");
      rect(x + 1, handY - 15, 1, bowBottom - handY + 15, "#ad9b75");
      if (d) rect(x - 4, handY - 6, 1, 5, c[2]);
      if (d >= 2) rect(x - 2, handY - 12, 2, 2, c[3]);
      return;
    }
    if (type === "staff") {
      rect(x, Math.max(3, top - 1), 2, 59 - Math.max(3, top - 1), ink);
      rect(x, Math.max(4, top), 1, 58 - Math.max(4, top), "#756750");
      if (d) rect(x - 1, top + 3, 3, 3, c[2]);
      if (d >= 2) {
        poly([[x, top - 1], [x + 3, top + 2], [x, top + 5], [x - 2, top + 2]], c[2]);
        rect(x, top + 1, 1, 2, "#97c8b8");
      }
      return;
    }
    const length = type === "dagger" ? 10 + d : ["mace", "hammer"].includes(type) ? 14 : 21 + d;
    const tip = Math.max(3, handY - length);
    rect(x - 1, tip, 3, handY - tip + 5, ink);
    rect(x, tip + 1, 1, handY - tip - 2, c[2]);
    rect(x - 2, handY - 2, 5, 1, c[1]);
    rect(x, handY, 1, 4, "#756750");
    if (["mace", "hammer", "pickaxe", "axe"].includes(type)) {
      poly([[x - 3, tip], [x + 3, tip], [x + 4, tip + 4], [x + 2, tip + 7],
        [x - 2, tip + 7], [x - 4, tip + 4]], ink);
      rect(x - 2, tip + 1, 4, 5, c[1]);
      rect(x - 2, tip + 1, 1, 4, c[2]);
      if (d >= 2) rect(x + 1, tip + 2, 1, 3, c[3]);
      if (type === "pickaxe") rect(x - 5, tip + 2, 11, 2, c[2]);
      if (type === "axe") rect(x - 4, tip + 1, 4, 6, c[2]);
    } else if (d >= 2) {
      rect(x - 1, tip + 3, 1, handY - tip - 6, c[1]);
      rect(x, tip + 2, 1, 2, c[3]);
    }
    if (d === 3) rect(x, handY + 3, 1, 1, c[3]);
  }
  weapon("main", -1);
  weapon("off", 1);
}
