import { classes, slots, labels, materials, startingEquipment, validEquipment } from "./equipment.js";
import { drawWorld, canMove } from "./world.js";
import { actions, frameCount, frameDuration, twoHanded, facingForMove, drawAnimated } from "./animation.js";
import { studies, drawStudy, buildShowcase, renderShowcase } from "./showcase.js";
import { directions, races, looks, skinRamps, hairRamps, drawSprite, drawPortrait } from "./sprite.js";

const state = { direction: 1, race: 0, body: 0, look: 0, skin: 1, hair: 1,
  class: 0, equipment: startingEquipment(0) };
const position = { x: 2, y: 2 };
const playback = { facing: "right", action: "idle", frame: 0 };
let playing = !matchMedia("(prefers-reduced-motion: reduce)").matches;
let walkUntil = 0;
let lastFrameTime = 0;
let atmospherePhase = 0;
const $ = id => document.getElementById(id);
const notice = message => { $("notice").textContent = message; };
const makeCanvas = (selection, label) => {
  const canvas = document.createElement("canvas");
  canvas.setAttribute("aria-label", label);
  drawSprite(canvas, selection);
  return canvas;
};
const makeButton = (label, selected, action) => {
  const button = document.createElement("button");
  button.type = "button";
  button.setAttribute("aria-label", label);
  button.setAttribute("aria-pressed", String(selected));
  button.addEventListener("click", action);
  return button;
};
function reference() {
  const gear = slots.filter(([slot]) => state.equipment[slot].type !== "none")
    .map(([slot]) => `${slot}:${state.equipment[slot].type}@${state.equipment[slot].level}/t${state.equipment[slot].material + 1}`).join(", ");
  return `gravebound / ${races[state.race].id} / ${state.body + 1}-${state.look + 1} / skin ${state.skin + 1} / hair ${state.hair + 1} / ${classes[state.class].name} / ${gear || "unequipped"}`;
}
function setLook(index) {
  state.look = index;
  state.skin = looks[index].skin;
  state.hair = looks[index].hairTone;
}
function update() {
  const focusedLabel = document.activeElement?.getAttribute("aria-label");
  document.querySelectorAll("[data-direction]").forEach(button => {
    button.setAttribute("aria-pressed", String(Number(button.dataset.direction) === state.direction));
  });
  document.querySelectorAll("[data-race]").forEach(button => {
    button.setAttribute("aria-pressed", String(Number(button.dataset.race) === state.race));
  });
  $("bodies").replaceChildren(...[0, 1].map(body => {
    const button = makeButton(`Silhouette ${body + 1}`, state.body === body, () => {
      state.body = body; update();
    });
    button.append(makeCanvas({ ...state, body, look: 0 }, `Silhouette ${body + 1} preview`));
    return button;
  }));
  $("looks").replaceChildren(...looks.map((look, index) => {
    const button = makeButton(`Look ${index + 1}: ${look.name}`, state.look === index, () => {
      setLook(index); update();
    });
    button.append(makeCanvas({ ...state, look: index, skin: look.skin, hair: look.hairTone }, look.name));
    const label = document.createElement("span");
    label.textContent = String(index + 1).padStart(2, "0");
    button.append(label);
    return button;
  }));
  for (const [id, ramps, key] of [["skin", skinRamps[races[state.race].skin], "skin"], ["hair", hairRamps, "hair"]]) {
    $(id).replaceChildren(...ramps.map((ramp, index) => {
      const button = makeButton(`${id === "skin" ? "Skin tone" : "Hair colour"} ${index + 1}`, state[key] === index, () => {
        state[key] = index; update();
      });
      button.style.backgroundColor = ramp[2];
      return button;
    }));
  }
  $("race-note").textContent = races[state.race].note;
  $("preview-title").textContent = `Gravebound / ${races[state.race].name} / ${classes[state.class].name}`;
  $("look-title").textContent = looks[state.look].name;
  $("look-description").textContent = looks[state.look].description;
  $("reference").textContent = reference();
  for (const id of ["hero", "native", "double"]) drawSprite($(id), state);
  drawPortrait($("portrait"), state);
  renderAnimation();
  $("position").textContent = `Hidden grid position: ${position.x}, ${position.y} · facing ${playback.facing} · 8 directions`;
  $("class").value = String(state.class);
  for (const [slot] of slots) {
    $(`${slot}-type`).value = state.equipment[slot].type;
    $(`${slot}-level`).value = String(state.equipment[slot].level);
    $(`${slot}-material`).value = String(state.equipment[slot].material);
    $(`${slot}-level`).disabled = state.equipment[slot].type === "none";
    $(`${slot}-material`).disabled = state.equipment[slot].type === "none";
  }
  $("off-type").disabled = twoHanded.has(state.equipment.main.type);
  $("grip-note").textContent = twoHanded.has(state.equipment.main.type) ?
    "Two-handed grip: off hand reserved for the main-hand item." :
    "Main and off hand stay anatomically assigned in both facings.";
  $("progression").replaceChildren(...[1, 25, 50, 100].map(level => {
    const selection = { ...state, equipment: Object.fromEntries(slots.map(([slot]) =>
      [slot, { ...state.equipment[slot], level }])) };
    const card = document.createElement("div");
    card.append(makeCanvas(selection, `Same equipped items at item level ${level}`));
    const label = document.createElement("span");
    label.textContent = `ilvl ${level}`;
    card.append(label);
    return card;
  }));
  if (!$("catalog").hidden) renderCatalog();
  if (focusedLabel) {
    Array.from(document.querySelectorAll("button[aria-label]"))
      .find(button => button.getAttribute("aria-label") === focusedLabel)?.focus({ preventScroll: true });
  }
}
function renderCatalog() {
  $("catalog").replaceChildren(...races.map((race, raceIndex) => {
    const row = document.createElement("div");
    row.className = "catalog-row";
    const heading = document.createElement("h3");
    heading.textContent = race.name + (race.locked ? " · Unlockable study" : "");
    const grid = document.createElement("div");
    grid.className = "catalog-looks";
    for (const body of [0, 1]) {
      looks.forEach((look, lookIndex) => {
        const selection = { ...state, race: raceIndex, body, look: lookIndex, skin: look.skin, hair: look.hairTone };
        const button = makeButton(`${race.name}, silhouette ${body + 1}, ${look.name}`,
          state.race === raceIndex && state.body === body && state.look === lookIndex,
          () => {
            Object.assign(state, selection);
            update();
            $("appearance-heading").scrollIntoView({ behavior: "smooth" });
          });
        button.append(makeCanvas(selection, `${race.name} ${look.name}`));
        const label = document.createElement("span");
        label.textContent = `${body + 1}-${lookIndex + 1}`;
        button.append(label);
        grid.append(button);
      });
    }
    row.append(heading, grid);
    return row;
  }));
}
function download(blob, name) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
function validSelection(value) {
  return value !== null && typeof value === "object" &&
    value.direction === 1 &&
    [["race", 10], ["body", 2], ["look", 5], ["skin", 5], ["hair", 5], ["class", classes.length]]
      .every(([key, count]) => Number.isInteger(value[key]) && value[key] >= 0 && value[key] < count) &&
    validEquipment(value.equipment);
}

classes.forEach((template, index) => {
  const option = document.createElement("option");
  option.value = String(index);
  option.textContent = template.name;
  $("class").append(option);
});
$("class").addEventListener("change", () => {
  state.class = Number($("class").value);
  state.equipment = startingEquipment(state.class);
  update();
});
for (const [slot, name, types] of slots) {
  const row = document.createElement("div");
  row.className = "equipment-row";
  const label = document.createElement("label");
  label.htmlFor = `${slot}-type`;
  label.textContent = name;
  const type = document.createElement("select");
  type.id = `${slot}-type`;
  for (const value of types) {
    const option = document.createElement("option");
    option.value = value; option.textContent = labels[value]; type.append(option);
  }
  const level = document.createElement("input");
  level.id = `${slot}-level`; level.type = "number"; level.min = "1"; level.max = "200"; level.step = "1";
  level.setAttribute("aria-label", `${name} item level`);
  const material = document.createElement("select");
  material.id = `${slot}-material`;
  material.setAttribute("aria-label", `${name} material`);
  materials.forEach((value, index) => {
    const option = document.createElement("option");
    option.value = String(index); option.textContent = `T${index + 1} · ${value.name}`;
    material.append(option);
  });
  type.addEventListener("change", () => {
    state.equipment[slot].type = type.value;
    if (slot === "main" && twoHanded.has(type.value)) {
      if (state.equipment.off.type !== "none") notice("Off-hand item unequipped for a two-handed grip.");
      state.equipment.off.type = "none";
    }
    update();
  });
  material.addEventListener("change", () => {
    state.equipment[slot].material = Number(material.value); update();
  });
  level.addEventListener("change", () => {
    if (!level.checkValidity()) {
      level.reportValidity();
      notice(`${name} item level must be a whole number from 1 to 200. The preview is unchanged.`);
      return;
    }
    state.equipment[slot].level = Number(level.value); notice(""); update();
  });
  level.required = true;
  row.append(label, type, level, material);
  $("equipment").append(row);
}
$("reset-kit").addEventListener("click", () => {
  state.equipment = startingEquipment(state.class); update();
});
$("occlusion").addEventListener("change", () => {
  if ($("occlusion").checked && position.x === 2 && position.y === 3) {
    $("occlusion").checked = false;
    notice("Move off the pillar's cell before showing the pillar.");
    return;
  }
  update();
});
const moves = [
  ["NW", -1, -1], ["N", 0, -1], ["NE", 1, -1], ["W", -1, 0],
  ["E", 1, 0], ["SW", -1, 1], ["S", 0, 1], ["SE", 1, 1],
];
function move(dx, dy) {
  const x = position.x + dx, y = position.y + dy, pillar = $("occlusion").checked;
  if (!canMove(x, y, pillar) ||
    (dx !== 0 && dy !== 0 && (!canMove(position.x + dx, position.y, pillar) ||
      !canMove(position.x, position.y + dy, pillar)))) {
    notice("The way is blocked in this preview.");
    return;
  }
  position.x = x; position.y = y;
  playback.facing = facingForMove(playback.facing, dx);
  walkUntil = performance.now() + frameCount * frameDuration;
  playback.frame = 0; lastFrameTime = performance.now();
  notice(""); update();
}
for (const [name, dx, dy] of moves) {
  const button = document.createElement("button");
  button.type = "button"; button.textContent = name;
  button.setAttribute("aria-label", `Move ${name}`);
  button.addEventListener("click", () => move(dx, dy));
  $("movement").append(button);
}
$("world-quarter").addEventListener("keydown", event => {
  const keys = {
    ArrowUp: [0, -1], ArrowDown: [0, 1], ArrowLeft: [-1, 0], ArrowRight: [1, 0],
    y: [-1, -1], k: [0, -1], u: [1, -1], h: [-1, 0],
    l: [1, 0], b: [-1, 1], j: [0, 1], n: [1, 1],
  };
  const numpad = { Numpad7: [-1, -1], Numpad8: [0, -1], Numpad9: [1, -1],
    Numpad4: [-1, 0], Numpad6: [1, 0], Numpad1: [-1, 1], Numpad2: [0, 1], Numpad3: [1, 1] };
  const delta = numpad[event.code] ?? keys[event.key];
  if (!delta || event.ctrlKey || event.metaKey || event.altKey) return;
  event.preventDefault(); move(...delta);
});
races.forEach((race, index) => {
  const button = makeButton(race.name + (race.locked ? " (unlockable study)" : ""), index === state.race, () => {
    state.race = index; update();
  });
  button.dataset.race = index;
  button.append(document.createTextNode(race.name));
  if (race.locked) {
    const badge = document.createElement("small");
    badge.textContent = "UNLOCK";
    button.append(badge);
  }
  $("races").append(button);
});
$("catalog-toggle").addEventListener("click", () => {
  const open = $("catalog").hidden;
  $("catalog").hidden = !open;
  $("catalog-toggle").setAttribute("aria-expanded", String(open));
  $("catalog-toggle").textContent = open ? "Hide full company" : "Show all 100 looks";
  if (open) renderCatalog();
});
function appearance() {
  return { race: races[state.race], skin: skinRamps[races[state.race].skin][state.skin], hair: hairRamps[state.hair] };
}
function animatedSprite(canvas, selection) {
  drawAnimated(canvas, selection, { ...playback,
    action: performance.now() < walkUntil ? "walk" : "idle" }, drawSprite, appearance());
}
function renderAnimation() {
  for (const facing of ["left", "right"]) {
    drawAnimated($(`animation-${facing}`), state, { ...playback, facing }, drawSprite, appearance());
  }
  drawWorld($("world-quarter"), { ...state, position, pillar: $("occlusion").checked,
    atmosphere: $("atmosphere").value, atmospherePhase,
    cameraOffset: { x: (position.x - 2) * 8, y: (position.y - 2) * 8 } }, animatedSprite);
  renderShowcase($("showcase"), playback.frame);
  $("animation-frame").textContent = `${playback.action} · frame ${playback.frame + 1}/${frameCount}`;
  $("animation-play").textContent = playing ? "Pause" : "Play";
  $("animation-play").setAttribute("aria-pressed", String(playing));
  $("facing-left").setAttribute("aria-pressed", String(playback.facing === "left"));
  $("facing-right").setAttribute("aria-pressed", String(playback.facing === "right"));
  const working = ["craft", "mine", "chop"].includes(playback.action);
  $("action-note").textContent = working ?
    "Action-prop study: crafting hammer, pickaxe, or wood axe temporarily replaces held weapons without changing equipped items." :
    ["gather", "eat", "drink", "heal", "cast"].includes(playback.action) ?
      "Held weapons are temporarily stowed for this action. Consumables and effects are demonstration props, not inventory changes." :
      playback.action === "shoot" ?
        "Archery demonstration: bow grip and drawing hand are separate; projectile visuals do not resolve damage or consume arrows." :
        "Current main/off-hand items are attached to their anatomical hands. Attack is a melee pose proof.";
}
for (const [id, name] of actions) {
  const option = document.createElement("option");
  option.value = id; option.textContent = name; $("animation-action").append(option);
}
$("animation-action").addEventListener("change", () => {
  playback.action = $("animation-action").value; playback.frame = 0; lastFrameTime = performance.now(); renderAnimation();
});
$("animation-play").addEventListener("click", () => {
  playing = !playing; lastFrameTime = performance.now(); renderAnimation();
});
$("animation-step").addEventListener("click", () => {
  playing = false; playback.frame = (playback.frame + 1) % frameCount; renderAnimation();
});
for (const facing of ["left", "right"]) {
  $(`facing-${facing}`).addEventListener("click", () => {
    playback.facing = facing; update();
  });
}
const proofs = [
  ["Sword & shield", () => { state.equipment = startingEquipment(0); playback.action = "attack"; }],
  ["Dual daggers", () => { state.equipment = startingEquipment(1); playback.action = "attack"; }],
  ["Two-handed sword", () => {
    state.equipment = startingEquipment(0); state.equipment.main.type = "greatsword";
    state.equipment.off.type = "none"; playback.action = "attack";
  }],
  ["Mining tool", () => {
    state.equipment = startingEquipment(1); state.equipment.main.type = "pickaxe";
    state.equipment.off.type = "none"; playback.action = "mine";
  }],
  ["Casting", () => { state.equipment = startingEquipment(2); playback.action = "cast"; }],
  ["Archery", () => { state.equipment = startingEquipment(4); playback.action = "shoot"; }],
  ["Drinking", () => { playback.action = "drink"; }],
];
for (const [name, apply] of proofs) {
  const button = document.createElement("button");
  button.type = "button"; button.textContent = name;
  button.addEventListener("click", () => {
    apply(); playback.frame = 0; $("animation-action").value = playback.action;
    notice("Applied demonstration equipment. Starting-class selection is unchanged; reset class kit to restore it.");
    update();
  });
  $("animation-proofs").append(button);
}
$("animation-export").addEventListener("click", () => {
  const sheet = document.createElement("canvas");
  sheet.width = frameCount * 64; sheet.height = 128;
  const ctx = sheet.getContext("2d");
  for (const [row, facing] of [[0, "left"], [1, "right"]]) {
    for (let frame = 0; frame < frameCount; frame++) {
      const sprite = document.createElement("canvas");
      drawAnimated(sprite, state, { facing, action: playback.action, frame }, drawSprite, appearance());
      ctx.drawImage(sprite, frame * 64, row * 64);
    }
  }
  sheet.toBlob(blob => {
    if (!blob) { notice("Animation export failed: the browser could not encode the sheet."); return; }
    download(blob, `prl-gravebound-${playback.action}-left-right.png`);
    notice("Exported six 64×64 frames per facing: left row first, right row second, 140ms per frame. Prototype art only.");
  }, "image/png");
});
buildShowcase($("showcase"));
$("atmosphere").value = playing ? "fog-ash" : "off";
$("atmosphere").addEventListener("change", renderAnimation);
$("showcase-export").addEventListener("click", () => {
  const sheet = document.createElement("canvas");
  sheet.width = 96 * frameCount; sheet.height = studies.length * 64;
  const ctx = sheet.getContext("2d");
  studies.forEach(([id], row) => {
    for (let frame = 0; frame < frameCount; frame++) {
      const tile = document.createElement("canvas"); drawStudy(tile, id, frame);
      ctx.drawImage(tile, frame * 96, row * 64);
    }
  });
  sheet.toBlob(blob => {
    if (!blob) { notice("Showcase export failed: the browser could not encode the sheet."); return; }
    download(blob, "prl-gravebound-world-showcase.png");
    download(new Blob([JSON.stringify({ frameWidth: 96, frameHeight: 64,
      framesPerRow: frameCount, frameDurationMs: frameDuration,
      rows: studies.map(([id, name, note, category], row) => ({ row, id, name, note, category })),
      productionAssetsApproved: false }, null, 2)], { type: "application/json" }), "prl-gravebound-world-showcase.json");
    notice("Exported representative study sheet and row manifest. These are prototype visuals, not game definitions.");
  }, "image/png");
});
$("save").addEventListener("click", () => {
  const study = { version: 3, baseline: "gravebound", selection: structuredClone(state),
    reference: reference(), baselineApproved: true, productionAssetsApproved: false };
  try {
    localStorage.setItem("prl-character-study", JSON.stringify(study));
  } catch (error) {
    notice(`Could not keep the study in this browser: ${error.message}. Settings will still download.`);
    download(new Blob([JSON.stringify(study, null, 2)], { type: "application/json" }), "prl-character-study.json");
    return;
  }
  download(new Blob([JSON.stringify(study, null, 2)], { type: "application/json" }), "prl-character-study.json");
  notice(`Gravebound equipment study kept: ${reference()}.`);
});
$("export").addEventListener("click", () => {
  $("hero").toBlob(blob => {
    if (!blob) {
      notice("Sprite export failed: the browser could not encode the image.");
      return;
    }
    download(blob, `prl-gravebound-${races[state.race].id}-${state.body + 1}-${state.look + 1}-${classes[state.class].name.toLowerCase()}.png`);
    notice("Native 64 × 64 transparent sprite exported. This is a study, not a final asset.");
  }, "image/png");
});
try {
  const saved = localStorage.getItem("prl-character-study");
  if (saved) {
    const study = JSON.parse(saved);
    if (study?.version !== 3 || study?.baseline !== "gravebound" || !validSelection(study?.selection)) {
      notice("The saved study is invalid or from an unsupported version. Choose a new study below.");
    } else {
      Object.assign(state, study.selection);
      notice(`Restored your Gravebound equipment study: ${reference()}.`);
    }
  }
} catch (error) {
  notice(`Could not restore a saved study: ${error.message}. The atelier remains available.`);
}
update();
function tick(now) {
  if (playing && !document.hidden && now - lastFrameTime >= frameDuration) {
    playback.frame = (playback.frame + 1) % frameCount;
    atmospherePhase = (atmospherePhase + 1) % 192;
    lastFrameTime = now; renderAnimation();
  }
  requestAnimationFrame(tick);
}
requestAnimationFrame(tick);
