export const worldSize = 5;

export function canMove(x, y, pillar) {
  return x >= 1 && y >= 1 && x < worldSize && y < worldSize &&
    !(pillar && x === 2 && y === 3);
}

export function drawWorld(canvas, selection, drawSprite) {
  canvas.width = 384;
  canvas.height = 384;
  const ctx = canvas.getContext("2d");
  ctx.imageSmoothingEnabled = false;
  const rect = (x, y, w, h, colour) => {
    ctx.fillStyle = colour;
    ctx.fillRect(Math.round(x), Math.round(y), Math.max(1, Math.round(w)), Math.max(1, Math.round(h)));
  };
  const wallHeight = 18;
  rect(0, 0, 384, 384, "#101412");
  rect(32, 32, 320, 320, "#2a3028");
  // Deterministic floor texture is continuous across the hidden movement cells.
  for (let i = 0; i < 160; i++) {
    const x = 34 + (i * 73 % 314), y = 34 + (i * 97 % 314);
    rect(x, y, i % 4 === 0 ? 3 : 1, 1, i % 3 === 0 ? "#343b31" : "#252c25");
  }
  for (let y = 0; y < worldSize; y++) for (let x = 0; x < worldSize; x++) {
    const px = 64 + x * 64, py = 32 + y * 64;
    if (x === 0 || y === 0 || (selection.pillar && x === 2 && y === 3)) {
      rect(px - 32, py - wallHeight, 64, 64, "#515545");
      rect(px - 32, py + 64 - wallHeight, 64, wallHeight, "#30372e");
      rect(px - 30, py + 3 - wallHeight, 60, 1, "#696b55");
      rect(px, py + 4 - wallHeight, 1, 24, "#3c4337");
      rect(px - 24, py + 20 - wallHeight, 15, 1, "#454d3e");
    }
    if (x === selection.position.x && y === selection.position.y) {
      const sprite = document.createElement("canvas");
      drawSprite(sprite, { ...selection, perspective: "overhead-quarter" });
      const feetY = py + 52;
      rect(px - 13, feetY - 3, 26, 3, "#171e19");
      ctx.drawImage(sprite, px - 32, feetY - 59);
    }
  }
  if (selection.atmosphere && selection.atmosphere !== "off") {
    const phase = selection.atmospherePhase ?? 0;
    const camera = selection.cameraOffset ?? { x: 0, y: 0 };
    ctx.save();
    // Only non-interactive atmosphere uses differential camera offsets.
    // Leave a clear area around the player, preserving silhouettes and hand gear.
    const playerX = 64 + selection.position.x * 64;
    const playerY = 32 + selection.position.y * 64 + 28;
    const clear = (x, y) => Math.abs(x - playerX) < 34 && Math.abs(y - playerY) < 38;
    for (let layer = 0; layer < 2; layer++) {
      ctx.globalAlpha = layer === 0 ? .09 : .14;
      const factor = layer === 0 ? .25 : .65;
      for (let i = 0; i < 12; i++) {
        const x = ((i * 83 + phase * (layer + 1) - camera.x * factor) % 384 + 384) % 384;
        const y = ((i * 47 + layer * 31 - camera.y * factor) % 384 + 384) % 384;
        if (!clear(x, y) && !clear(x + 22, y)) rect(x, y, 24, 3, "#929780");
      }
    }
    if (selection.atmosphere === "fog-ash") {
      ctx.globalAlpha = .35;
      for (let i = 0; i < 18; i++) {
        const x = ((i * 59 + phase - camera.x * .85) % 384 + 384) % 384;
        const y = ((i * 41 + phase * .5 - camera.y * .85) % 384 + 384) % 384;
        if (!clear(x, y)) rect(x, y, 1, 1, "#ad9b75");
      }
    }
    ctx.restore();
  }
}
