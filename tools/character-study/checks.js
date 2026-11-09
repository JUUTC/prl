import { actions, drawAnimated, facingForMove, frameCount, poseFor, twoHanded } from "./animation.js";
import { startingEquipment, validEquipment } from "./equipment.js";
import { studies, drawStudy } from "./showcase.js";

// Run in the atelier's browser console: await (await import("./checks.js")).runChecks()
export async function runChecks() {
  const { drawSprite, races, skinRamps, hairRamps } = await import("./study.js");
  let count = 0;
  const failures = [];
  const check = (condition, message) => { if (!condition) failures.push(message); };
  check(facingForMove("left", 0) === "left" && facingForMove("right", 0) === "right", "Vertical facing retention");
  check(facingForMove("right", -1) === "left" && facingForMove("left", 1) === "right", "Horizontal facing choice");
  for (let race = 0; race < races.length; race++) {
    for (const body of [0, 1]) for (const [action] of actions) for (const facing of ["left", "right"]) {
      const hashes = new Set();
      for (let frame = 0; frame < frameCount; frame++) {
        const equipment = startingEquipment(race % 6);
        const selection = { race, body, direction: 1, look: 2, skin: 1, hair: 1, equipment };
        const canvas = document.createElement("canvas");
        const pose = drawAnimated(canvas, selection, { facing, action, frame }, drawSprite,
          { race: races[race], skin: skinRamps[races[race].skin][1], hair: hairRamps[1] });
        const data = canvas.getContext("2d").getImageData(0, 0, 64, 64).data;
        let invalid = false;
        for (let i = 0; i < data.length; i += 4) {
          if (!data[i + 3]) continue;
          const x = i / 4 % 64, y = Math.floor(i / 256);
          if (data[i + 3] !== 255 || x === 0 || x === 63 || y === 0 || y === 63) invalid = true;
        }
        check(!invalid, `Clipped or antialiased frame: ${race}/${body}/${action}/${facing}/${frame}`);
        if (twoHanded.has(pose.mainType) && action !== "shoot") {
          check(Math.abs(pose.off[0] - (pose.main[0] - pose.vector[0] * 5)) < .01 &&
            Math.abs(pose.off[1] - (pose.main[1] - pose.vector[1] * 5)) < .01, "Shared two-handed shaft");
        }
        if (action === "shoot") {
          check(pose.mainType === "bow" && pose.offType === "none" &&
            pose.main[1] === pose.off[1], "Bow grip and drawing hand");
        }
        hashes.add(canvas.toDataURL());
        count++;
      }
      check(hashes.size > 1, `Static action: ${race}/${body}/${action}/${facing}`);
    }
  }
  const gear = startingEquipment(0);
  for (const facing of ["left", "right"]) {
    const pose = poseFor({ equipment: gear }, facing, "walk", 0, races[0]);
    check(pose.mainType === "sword" && pose.offType === "kite", `Hand identity: ${facing}`);
    check(pose.near === (facing === "left" ? "off" : "main"), `Facing-specific depth order: ${facing}`);
  }
  gear.main.type = "greatsword";
  check(!validEquipment(gear), "Reject two-handed plus shield loadout");
  gear.off.type = "none";
  check(validEquipment(gear), "Accept two-handed loadout");
  const left = document.createElement("canvas"), right = document.createElement("canvas");
  const sel = { race: 0, body: 0, direction: 1, look: 0, skin: 1, hair: 1, equipment: startingEquipment(0) };
  for (const [canvas, facing] of [[left, "left"], [right, "right"]]) {
    drawAnimated(canvas, sel, { facing, action: "idle", frame: 0 }, drawSprite,
      { race: races[0], skin: skinRamps.warm[1], hair: hairRamps[1] });
  }
  const l = left.getContext("2d").getImageData(0, 0, 64, 64).data;
  const r = right.getContext("2d").getImageData(0, 0, 64, 64).data;
  let mirrored = true;
  for (let y = 0; y < 64; y++) for (let x = 0; x < 64; x++) for (let channel = 0; channel < 4; channel++) {
    if (l[(y * 64 + x) * 4 + channel] !== r[(y * 64 + 63 - x) * 4 + channel]) mirrored = false;
  }
  check(!mirrored, "Left/right must not be mirrored composites");
  for (const [id] of studies) {
    const hashes = new Set();
    for (let frame = 0; frame < frameCount; frame++) {
      const canvas = document.createElement("canvas");
      drawStudy(canvas, id, frame);
      const data = canvas.getContext("2d").getImageData(0, 0, 96, 64).data;
      let invalid = false;
      for (let i = 0; i < data.length; i += 4) {
        if (!data[i + 3]) continue;
        const x = i / 4 % 96, y = Math.floor(i / 384);
        if (data[i + 3] !== 255 || x === 0 || x === 95 || y === 0 || y === 63) invalid = true;
      }
      check(!invalid, `Clipped or antialiased showcase: ${id}/${frame}`);
      hashes.add(canvas.toDataURL());
    }
    check(hashes.size > 1, `Static showcase: ${id}`);
  }
  if (failures.length) throw new Error(`${failures.length} animation checks failed:\n${failures.join("\n")}`);
  return { frames: count, showcaseFrames: studies.length * frameCount, checks: "passed" };
}
