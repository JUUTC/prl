// Builds the disposable in-game placeholder atlas from the approved Gravebound studies.
// Output is data, not a shipping art format: see docs/art-production.md.
import { startingEquipment } from "./equipment.js";
import { drawAnimated, frameCount, frameDuration } from "./animation.js";
import { creatureActions, drawCreature } from "./creatures.js";
import { races, skinRamps, hairRamps, drawSprite } from "./sprite.js";

const cell = 64;
const wallHeight = 18;
// Player and three creatures, each with three clips × two facings.
const tileRow = 4 * 3 * 2 * cell;
const atlas = document.getElementById("atlas");
atlas.width = frameCount * cell;
atlas.height = tileRow + cell + wallHeight;
const ctx = atlas.getContext("2d");
ctx.imageSmoothingEnabled = false;
const rect = (x, y, w, h, colour) => { ctx.fillStyle = colour; ctx.fillRect(x, y, w, h); };

// Human Gravebound baseline wearing the Warrior starting kit.
const selection = { direction: 1, race: 0, body: 0, look: 0, skin: 1, hair: 1,
  class: 0, equipment: startingEquipment(0), perspective: "overhead-quarter" };
const appearance = { race: races[0], skin: skinRamps[races[0].skin][1], hair: hairRamps[1] };
// Matches the atelier world: feet sit 7px below the cell centre on a soft shadow.
const offsetY = 7;

let row = 0;
const strip = draw => {
  const frames = [];
  for (let frame = 0; frame < frameCount; frame++) {
    const x = frame * cell, y = row * cell;
    draw(x, y, frame);
    frames.push([x, y, cell, cell]);
  }
  row++;
  return frames;
};
const shadow = (x, y) => rect(x + 19, y + 56, 26, 3, "#171e19");
const heroClip = action => {
  const clip = { frame_ms: frameDuration };
  for (const facing of ["right", "left"]) {
    clip[facing] = strip((x, y, frame) => {
      const sprite = document.createElement("canvas");
      drawAnimated(sprite, selection, { facing, action, frame }, drawSprite, appearance);
      shadow(x, y);
      ctx.drawImage(sprite, x, y);
    });
  }
  return clip;
};
// Creatures author each facing separately (creatures.js); nothing is mirrored.
const creatureClip = (id, action) => {
  const clip = { frame_ms: frameDuration };
  for (const facing of ["right", "left"]) {
    clip[facing] = strip((x, y, frame) => {
      const sprite = document.createElement("canvas");
      drawCreature(sprite, id, { facing, action, frame });
      shadow(x, y);
      ctx.drawImage(sprite, x, y);
    });
  }
  return clip;
};

const actors = {
  "prl:player": { offset_y: offsetY, idle: heroClip("idle"), walk: heroClip("walk"), attack: heroClip("attack") },
};
for (const [content, study] of [["prl:skeleton", "skeleton"], ["prl:shambler", "shambler"], ["prl:grave_rat", "rat"]]) {
  const [idle, walk, attack] = creatureActions.map(action => creatureClip(study, action));
  actors[content] = { offset_y: offsetY, idle, walk, attack };
}

// Seamless floor: speckles wrap inside one cell so adjacent tiles show no grid.
const floor = (x, y) => {
  rect(x, y, cell, cell, "#2a3028");
  for (let i = 0; i < 40; i++) {
    const w = i % 4 === 0 ? 3 : 1;
    rect(x + (i * 37 % (cell - w + 1)), y + (i * 23 % cell), w, 1, i % 3 === 0 ? "#343b31" : "#252c25");
  }
};
// Half-height wall block: top face overhangs the cell to the north by wallHeight.
const wall = (x, y) => {
  rect(x, y, cell, cell, "#515545");
  rect(x, y + cell, cell, wallHeight, "#30372e");
  rect(x + 2, y + 3, cell - 4, 1, "#696b55");
  rect(x + 32, y + 4, 1, 24, "#3c4337");
  rect(x + 8, y + 20, 15, 1, "#454d3e");
  rect(x + 40, y + 42, 15, 1, "#454d3e");
};
const posts = (x, y) => {
  rect(x, y, 8, cell + wallHeight, "#30372e"); rect(x + cell - 8, y, 8, cell + wallHeight, "#30372e");
  rect(x, y, 8, cell, "#515545"); rect(x + cell - 8, y, 8, cell, "#515545");
};
const tileRects = [];
const tileCell = draw => {
  const x = tileRects.length * cell;
  draw(x, tileRow);
  tileRects.push([x, tileRow, cell, cell + wallHeight]);
  return tileRects.at(-1);
};
const floorRect = (() => { floor(0, tileRow); tileRects.push([0, tileRow, cell, cell]); return tileRects[0]; })();
const wallRect = tileCell(wall);
const closedRect = tileCell((x, y) => {
  posts(x, y);
  rect(x + 8, y + 4, cell - 16, cell - 4, "#493b2c");
  for (let i = 0; i < 4; i++) rect(x + 12 + i * 11, y + 6, 1, cell - 8, "#39352e");
  rect(x + 8, y + cell, cell - 16, wallHeight, "#39352e");
  rect(x + 44, y + cell + 6, 3, 3, "#929780");
});
const openRect = tileCell((x, y) => {
  floor(x, y + wallHeight);
  posts(x, y);
  rect(x + 8, y + wallHeight, 4, cell, "#493b2c");
});
// Block centres sit (wallHeight / 2)px north of the logical cell centre.
const tiles = {
  "prl:floor": { rect: floorRect, offset_y: 0, open: null },
  "prl:wall": { rect: wallRect, offset_y: wallHeight / 2, open: null },
  "prl:door": { rect: closedRect, offset_y: wallHeight / 2, open: openRect },
};

const manifest = {
  schema_version: 1,
  status: "placeholder",
  source: "tools/character-study/export_placeholder_atlas.py",
  image: "gravebound_placeholder.png",
  size: [atlas.width, atlas.height],
  actors,
  tiles,
};
document.getElementById("result").textContent = JSON.stringify({ manifest, png: atlas.toDataURL("image/png") });
