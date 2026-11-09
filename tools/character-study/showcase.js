export const studies = [
  ["fire", "Fire bolt", "Ember trail → impact", "spell"],
  ["frost", "Frost lance", "Crystal shards → shatter", "spell"],
  ["lightning", "Lightning", "Jagged discharge → sparks", "spell"],
  ["poison", "Poison", "Droplets → lingering cloud", "spell"],
  ["ward", "Ward", "Rune ring → protective pulse", "spell"],
  ["restore", "Restoration", "Rising motes → healing pulse", "spell"],
  ["arrow", "Broadhead", "Physical arrow", "arrow"],
  ["bodkin", "Bodkin", "Narrow armour-piercing study", "arrow"],
  ["fire-arrow", "Fire arrow", "Burning tip and ember trail", "arrow"],
  ["frost-arrow", "Frost arrow", "Ice tip and shard trail", "arrow"],
  ["poison-arrow", "Poison arrow", "Coated tip and droplets", "arrow"],
  ["skeleton", "Skeleton", "Undead enemy", "actor"],
  ["goblin", "Goblin", "Small knife-bearing enemy", "actor"],
  ["wraith", "Wraith", "Shrouded spectral monster", "actor"],
  ["smith", "Town smith", "Apron and hammer", "actor"],
  ["merchant", "Town merchant", "Hood, satchel and ledger", "actor"],
  ["wolf", "Wolf", "Lean quadruped", "animal"],
  ["deer", "Deer", "Antlers and long legs", "animal"],
  ["mule", "Pack mule", "Panniers and long ears", "animal"],
  ["tree", "Tree felling", "Standing → falling → stump/logs", "harvest"],
  ["ore", "Ore vein", "Intact → fractured → rubble/ore", "harvest"],
  ["forge", "Forge & anvil", "Bellows ember cycle", "bench"],
  ["alchemy", "Alchemy bench", "Vials, burner and bubbling flask", "bench"],
  ["loom", "Loom", "Frame, threads and shuttle", "bench"],
  ["food", "Meal", "Bread and bowl", "consumable"],
  ["drink", "Drink", "Cup and potion bottle", "consumable"],
];
const ink = "#111514";

export function drawStudy(canvas, id, frame) {
  const study = studies.find(([key]) => key === id);
  if (!study || !Number.isInteger(frame) || frame < 0 || frame >= 6) {
    throw new Error("Invalid showcase study or frame.");
  }
  canvas.width = 96; canvas.height = 64;
  const ctx = canvas.getContext("2d");
  const r = (x, y, w, h, c) => {
    ctx.fillStyle = c; ctx.fillRect(Math.round(x), Math.round(y), Math.max(1, Math.round(w)), Math.max(1, Math.round(h)));
  };
  const line = (x, y, xx, yy, c, w = 1) => {
    const n = Math.max(1, Math.ceil(Math.max(Math.abs(xx - x), Math.abs(yy - y))));
    for (let i = 0; i <= n; i++) r(x + (xx - x) * i / n, y + (yy - y) * i / n, w, w, c);
  };
  const ring = (x, y, radius, c) => {
    for (let a = 0; a < 32; a++) r(x + Math.cos(a * Math.PI / 16) * radius,
      y + Math.sin(a * Math.PI / 16) * radius * .6, 1, 1, c);
  };
  const phase = [0, 1, 2, 3, 2, 1][frame];
  const colours = id.includes("fire") ? ["#744331", "#af7449", "#efd0a5"] :
    id.includes("frost") ? ["#3d606c", "#71929a", "#b0c7c3"] :
    id.includes("poison") ? ["#3b513b", "#788957", "#b2bd88"] : ["#577d79", "#97c8b8", "#c6c8a3"];
  if (study[3] === "spell" || study[3] === "arrow") {
    const x = 18 + frame * 10;
    if (study[3] === "arrow") {
      line(x - 13, 31, x + 6, 31, "#756750");
      line(x - 12, 28, x - 8, 31, "#ad9b75");
      line(x - 12, 34, x - 8, 31, "#ad9b75");
      if (id === "bodkin") line(x + 5, 31, x + 11, 31, "#929780");
      else { line(x + 5, 28, x + 9, 31, "#929780"); line(x + 5, 34, x + 9, 31, "#929780"); }
      if (!["arrow", "bodkin"].includes(id)) {
        r(x + 5, 29, 3, 3, colours[1]);
        for (let i = 0; i < 4; i++) r(x - 17 + i * 4, 29 + (i + frame) % 5, 2, 1, colours[i % 3]);
      }
    } else if (["ward", "restore"].includes(id)) {
      ring(48, 37, 12 + phase * 2, colours[1]);
      for (let i = 0; i < 5; i++) r(36 + i * 6, 37 - (frame * 4 + i * 3) % 24, 1, 3, colours[2]);
      if (id === "restore") { r(47, 24, 2, 9, colours[2]); r(44, 27, 8, 2, colours[2]); }
      else { r(47, 24, 2, 2, colours[2]); r(33, 36, 2, 2, colours[2]); }
    } else if (id === "lightning") {
      for (let i = 0; i < 6; i++) line(16 + i * 10, 25 + (i + frame) % 3 * 5,
        26 + i * 10, 25 + (i + frame + 1) % 3 * 5, colours[2], 2);
    } else {
      if (id === "fire") {
        for (let i = 0; i < 6; i++) r(x - i * 3, 29 + (i + frame) % 5, 4 - Math.min(i, 2), 3, colours[i % 3]);
        r(x, 29, 4, 4, colours[2]);
        if (frame >= 4) for (let i = 0; i < 8; i++) {
          const angle = i * Math.PI / 4;
          line(x + 3, 31, x + 3 + Math.cos(angle) * 9, 31 + Math.sin(angle) * 8, colours[i % 3]);
        }
      } else if (id === "frost") {
        line(x - 9, 31, x + 8, 31, colours[2], 2);
        line(x - 2, 25, x + 8, 31, colours[1]);
        line(x - 2, 37, x + 8, 31, colours[1]);
        if (frame >= 4) for (let i = 0; i < 5; i++) {
          line(x - 5 + i * 3, 26 + i % 2 * 8, x - 3 + i * 3, 23 + i % 2 * 14, colours[2]);
        }
      } else {
        for (let i = 0; i < 5; i++) {
          r(x - i * 4, 28 + (i + frame) % 7, 2, 2, colours[i % 3]);
        }
        if (frame >= 4) {
          ring(x, 31, 7, colours[1]); ring(x - 5, 34, 5, colours[0]); ring(x + 5, 29, 4, colours[0]);
        }
      }
    }
    return;
  }
  r(25, 57, 47, 2, "#1b211e");
  if (study[3] === "actor") {
    const bob = frame % 3 === 1 ? -1 : 0;
    const colour = id === "skeleton" ? "#ad9b75" : id === "goblin" ? "#788957" : "#515b60";
    const short = id === "goblin" ? 8 : 0;
    if (id === "wraith") {
      for (let y = 20; y < 56; y++) {
        const width = 5 + (y - 20) / 4;
        r(48 - width, y + bob, width * 2, 1, y % 4 === 0 ? "#394344" : "#222b2c");
      }
      r(43, 24 + bob, 10, 9, ink); r(45, 28 + bob, 1, 1, "#97c8b8"); r(50, 28 + bob, 1, 1, "#97c8b8");
      r(36 + phase, 53, 3, 4, "#394344");
    } else {
      const face = ["smith", "merchant"].includes(id) ? "#97674c" : colour;
      r(41, 16 + short + bob, 12, 10, ink); r(43, 17 + short + bob, 8, 7, face);
      if (id !== "skeleton") { r(43, 16 + short + bob, 8, 2, "#493b2c"); r(51, 20 + short + bob, 2, 2, face); }
      r(45, 21 + short + bob, 2, 2, ink); r(50, 21 + short + bob, 1, 2, ink);
      line(43, 29 + short + bob, 40, 54, colour, 3);
      line(51, 29 + short + bob, 55, 54, colour, 3);
      if (id === "skeleton") {
        line(47, 27 + bob, 47, 42, colour, 2);
        for (let i = 0; i < 4; i++) r(42, 29 + i * 3 + bob, 11, 1, colour);
        r(45, 24 + bob, 6, 1, ink);
      } else {
        r(40, 28 + short + bob, 16, 15 - short / 2, id === "smith" ? "#493b2c" : "#3b4640");
        if (id === "smith") { r(44, 28 + bob, 8, 21, "#756750"); r(45, 36 + bob, 6, 5, "#39352e"); }
        if (id === "merchant") { r(40, 15 + bob, 16, 5, "#222b2c"); r(53, 38 + bob, 6, 10, "#756750"); r(44, 31 + bob, 1, 12, "#ad9b75"); }
      }
      line(40, 30 + short + bob, 34, 41 + phase, colour, 2);
      line(55, 31 + short + bob, 62, 40 - phase, colour, 2);
      if (id === "smith") { line(62, 40 - phase, 63, 28 - phase, "#756750", 2); r(59, 27 - phase, 9, 3, "#65716b"); }
      if (id === "goblin") { r(39, 20 + short, 3, 3, colour); line(62, 40 - phase, 68, 33 - phase, "#929780"); }
    }
  } else if (study[3] === "animal") {
    const fur = id === "wolf" ? "#65716b" : "#796b52";
    const bob = frame % 2;
    r(29, 34 + bob, 32, 11, ink); r(30, 35 + bob, 30, 8, fur);
    r(58, 27 + bob, 11, 13, fur); r(66, 32 + bob, 7, 5, fur); r(66, 30 + bob, 1, 1, ink);
    for (let i = 0; i < 4; i++) line(32 + i * 8, 43, 31 + i * 8 + (i % 2 ? phase : -phase), 56, fur, 2);
    line(29, 37, 20, 32 + phase, fur, 2);
    if (id === "deer") { line(62, 27, 59, 16, "#ad9b75"); line(64, 27, 69, 17, "#ad9b75"); line(59, 19, 55, 17, "#ad9b75"); }
    else { r(59, id === "mule" ? 17 : 23, 2, id === "mule" ? 12 : 6, fur); r(65, 21, 2, 7, fur); }
    if (id === "mule") { r(35, 33, 17, 13, "#39352e"); r(36, 35, 6, 10, "#756750"); r(46, 35, 6, 10, "#756750"); }
  } else if (id === "tree") {
    r(43, 49, 10, 8, "#39352e"); r(44, 49, 8, 2, "#ad9b75");
    if (frame < 4) {
      const lean = [0, 1, 4, 15][frame];
      line(48, 50, 48 + lean, 18 + lean, "#756750", 5);
      for (let i = 0; i < 4; i++) r(34 + lean - i * 2, 12 + i * 7 + lean, 28 + i * 3, 8, i % 2 ? "#3b513b" : "#303d30");
    } else { r(56, 51, 22, 5, "#756750"); r(55, 51, 3, 5, "#ad9b75"); r(67, 43, 14, 5, "#3b513b"); }
  } else if (id === "ore") {
    for (let i = 0; i < 4; i++) r(28 + i * 9, 39 - (i % 2) * 7 + (frame > 3 ? 10 : 0),
      frame > 3 ? 7 : 14, frame > 3 ? 5 : 17, i % 2 ? "#515545" : "#394344");
    for (let i = 0; i < 4; i++) r(34 + i * 8, 44 + i % 2 * 4, 3, 2, "#929780");
    if (frame >= 2 && frame < 4) line(48, 33, 43, 52, ink, 2);
  } else if (study[3] === "bench") {
    r(24, 39, 48, 5, "#756750"); r(27, 44, 4, 13, "#39352e"); r(65, 44, 4, 13, "#39352e");
    if (id === "forge") {
      r(26, 23, 20, 17, "#394344"); r(29, 28, 14, 10, ink);
      r(31, 33 - phase, 10, 5 + phase, "#af7449"); r(34, 34, 4, 3, "#efd0a5");
      r(53, 34, 19, 3, "#65716b"); r(59, 37, 7, 4, "#3d4847");
    } else if (id === "alchemy") {
      r(29, 29, 5, 9, "#577d79"); r(30, 27, 3, 2, "#ad9b75");
      r(45, 28, 3, 6, "#97c8b8"); r(42, 33, 10, 6, "#577d79");
      r(45, 35, 2, 2, "#b2bd88"); r(46, 23 - phase, 1, 1, "#97c8b8");
      r(58, 32, 6, 7, "#744331");
    } else {
      r(31, 17, 3, 22, "#756750"); r(61, 17, 3, 22, "#756750"); r(31, 17, 33, 3, "#756750");
      for (let i = 0; i < 8; i++) line(36 + i * 3, 20, 36 + i * 3, 37, "#929780");
      r(35, 30, 26, 7, "#3b4640"); r(36 + frame * 3, 29, 6, 2, "#ad9b75");
    }
  } else {
    if (id === "food") {
      r(29, 43, 20, 7, "#796b52"); r(30, 41, 18, 3, "#ad9b75");
      r(55, 43, 18, 3, "#65716b"); r(57, 46, 14, 6, "#3d4847");
    } else {
      r(29, 39, 12, 14, "#3d4847"); r(28, 38, 14, 2, "#929780");
      r(41, 41, 4, 7, "#65716b"); r(60, 29, 4, 5, "#ad9b75");
      r(56, 35, 12, 17, "#577d79"); r(58, 40, 8, 10, "#744331");
    }
    r(61, 33 - phase, 1, 2, "#65716b");
  }
}

export function buildShowcase(container) {
  for (const [id, name, note] of studies) {
    const card = document.createElement("article");
    const heading = document.createElement("h3"); heading.textContent = name;
    const canvas = document.createElement("canvas");
    canvas.dataset.study = id; canvas.setAttribute("aria-label", `${name}: ${note}`);
    const description = document.createElement("p"); description.textContent = note;
    card.append(heading, canvas, description); container.append(card);
  }
}

export function renderShowcase(container, frame) {
  container.querySelectorAll("canvas[data-study]").forEach(canvas => drawStudy(canvas, canvas.dataset.study, frame));
}
