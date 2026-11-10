// Synthetic interaction fixtures only. Never import this evaluator into the game.
export const WIDTH = 10;
export const HEIGHT = 6;

const item = (id, name, kind, location, extra = {}) => ({
  id, name, kind, location, width: 1, height: 1, weight: 100,
  known: true, rarity: "Normal", base: {}, affix: {}, ...extra,
});

export function createStudy(scenario = "normal") {
  const items = [
    item("i1", "Iron sword", "gear", "equipment", {
      slot: "Main hand", width: 1, height: 3, weight: 3000, base: { damage: 3 },
    }),
    item("i2", "Iron mail", "gear", "equipment", {
      slot: "Body", width: 2, height: 3, weight: 12000, base: { armor: 4 },
    }),
    item("i3", "Healing potion", "potion", "backpack", {
      cell: [0, 0], weight: 300, appearance: "Murky green potion", heal: 24,
    }),
    item("i4", "Healing potion", "potion", "backpack", {
      cell: [1, 0], weight: 300, appearance: "Murky green potion", heal: 24,
    }),
    item("i5", "Scroll of Identify", "scroll", "backpack", {
      cell: [2, 0], appearance: 'Scroll labelled "XOTH"', effect: "identify",
    }),
    item("i6", "Scroll of Identify", "scroll", "backpack", {
      cell: [3, 0], appearance: 'Scroll labelled "XOTH"', effect: "identify",
    }),
    item("i7", "Worn iron sword", "gear", "backpack", {
      cell: [5, 1], slot: "Main hand", width: 1, height: 3,
      weight: 2500, base: { damage: 2 },
    }),
    item("i8", "Iron sword", "gear", "ground", {
      slot: "Main hand", width: 1, height: 3, weight: 3000,
      rarity: "Magic", known: false, base: { damage: 3 },
      affix: { damage: 2 }, suffix: "of keen edges",
    }),
    item("i9", "Iron mail", "gear", "ground", {
      slot: "Body", width: 2, height: 3, weight: 12000,
      rarity: "Magic", known: false, base: { armor: 4 },
      affix: { armor: 2 }, suffix: "of warding",
    }),
  ];
  const state = {
    scenario, items, hp: scenario === "full-hp" ? 40 : 16, maxHp: 40,
    hostiles: scenario === "danger" ? 1 : 0, capacity: 60000,
    actions: 0, energy: 0, revision: 0,
  };
  if (scenario === "unknown") {
    for (const value of items.filter(value => ["potion", "scroll"].includes(value.kind))) {
      value.known = false;
    }
  }
  if (scenario === "space") {
    state.items = items.filter(value => value.location !== "backpack");
    // Thirty free cells, but no vertical/2x3 rectangle: not merely "bag full".
    for (let y = 0; y < HEIGHT; y++) {
      for (let x = 0; x < WIDTH; x++) {
        if ((x + y) % 2 === 0) {
          state.items.push(item(`parcel-${x}-${y}`, "Space-test parcel", "parcel", "backpack", {
            cell: [x, y], weight: 0,
          }));
        }
      }
    }
  }
  if (scenario === "weight") {
    state.items.push(item("test-weight", "Weight-test parcel", "parcel", "backpack", {
      cell: [9, 5], weight: 40000,
    }));
  }
  if (scenario === "empty") {
    state.items = items.filter(value => value.location === "equipment");
  }
  if (scenario === "long-name") {
    items.find(value => value.id === "i8").name =
      "Ancient ceremonial iron sword of the long-forgotten crypt watch";
  }
  return state;
}

export function label(value) {
  if (!value.known) return `${value.appearance ?? value.name} (?)`;
  return `${value.name}${value.suffix ? ` ${value.suffix}` : ""}`;
}

export function itemView(value) {
  return {
    id: value.id, label: label(value), kind: value.kind, location: value.location,
    width: value.width, height: value.height, weight: value.weight,
    known: value.known, rarity: value.rarity, slot: value.slot,
    base: { ...value.base }, bonuses: value.known ? { ...value.affix } : {},
    cell: value.cell ? [...value.cell] : null,
    effect: value.kind === "potion"
      ? value.known ? `Restore up to ${value.heal} HP` : "Unknown effect"
      : value.kind === "scroll"
        ? value.known ? "Identify one item" : "Unknown effect"
        : null,
    heal: value.known && value.kind === "potion" ? value.heal : null,
  };
}

export function carriedWeight(state) {
  return state.items.filter(value => value.location !== "ground")
    .reduce((sum, value) => sum + value.weight, 0);
}

export function bagItems(state) {
  return state.items.filter(value => value.location === "backpack")
    .sort((a, b) => a.cell[1] - b.cell[1] || a.cell[0] - b.cell[0] || a.id.localeCompare(b.id));
}

export function firstFit(items, candidate) {
  for (let y = 0; y <= HEIGHT - candidate.height; y++) {
    for (let x = 0; x <= WIDTH - candidate.width; x++) {
      if (items.every(other => {
        const [ox, oy] = other.cell;
        return x + candidate.width <= ox || ox + other.width <= x
          || y + candidate.height <= oy || oy + other.height <= y;
      })) return [x, y];
    }
  }
  return null;
}

export function knownTotals(state, candidate = null) {
  const totals = { damage: 6, armor: 0, accuracy: 10, evasion: 5 };
  let equipped = state.items.filter(value => value.location === "equipment");
  if (candidate) equipped = [...equipped.filter(value => value.slot !== candidate.slot), candidate];
  for (const value of equipped) {
    for (const stat of Object.keys(totals)) {
      totals[stat] += (value.base[stat] ?? 0) + (value.known ? value.affix[stat] ?? 0 : 0);
    }
  }
  return totals;
}

export function comparison(state, id) {
  const candidate = state.items.find(value => value.id === id);
  if (!candidate || candidate.kind !== "gear" || candidate.location === "equipment") return null;
  const incumbent = state.items.find(value => value.location === "equipment" && value.slot === candidate.slot);
  return {
    before: knownTotals(state), after: knownTotals(state, candidate),
    incumbent: incumbent ? label(incumbent) : "Empty slot",
    uncertain: !candidate.known || state.items.some(value => value.location === "equipment" && !value.known),
  };
}

export function identifySources(state) {
  return state.items.filter(value => value.location === "backpack"
    && value.kind === "scroll" && value.known && value.effect === "identify").map(itemView);
}

function transferReason(state, value, verb) {
  if (verb === "pickup" && carriedWeight(state) + value.weight > state.capacity) {
    return `Over pickup limit: ${((carriedWeight(state) + value.weight) / 1000).toFixed(1)} / 60.0 kg. Leave something behind.`;
  }
  const bag = bagItems(state).filter(other => other.id !== value.id);
  const incumbent = verb === "equip" ? state.items.find(other =>
    other.location === "equipment" && other.slot === value.slot) : null;
  const needsSpace = verb === "pickup" || verb === "unequip" ? value : incumbent;
  if (needsSpace && !firstFit(bag, needsSpace)) {
    return `No contiguous ${needsSpace.width}x${needsSpace.height} space for ${label(needsSpace)}. Nothing moved.`;
  }
  return "";
}

export function actionsFor(state, id) {
  const value = state.items.find(candidate => candidate.id === id);
  if (!value) return [];
  const actions = [];
  const add = (verb, title, cost, reason = "", review = false) =>
    actions.push({ verb, title, cost, reason, enabled: !reason, review });
  if (value.location === "ground") {
    add("pickup", "Pick up", 50, transferReason(state, value, "pickup"));
    return actions;
  }
  if (value.kind === "gear") {
    const verb = value.location === "equipment" ? "unequip" : "equip";
    add(verb, verb === "equip" ? "Equip" : "Unequip",
      value.slot === "Body" ? 300 : 100, transferReason(state, value, verb),
      value.slot === "Body" && state.hostiles > 0);
  }
  if (value.kind === "potion") {
    add("drink", "Drink", 100, value.known && state.hp >= state.maxHp
      ? "Already at full HP. This known healing potion would have no effect." : "", !value.known);
  }
  if (!value.known) {
    add("identify", "Identify...", 100, identifySources(state).length
      ? "" : "No known Identify scroll. Unknown scroll types are not revealed by this chooser.", true);
  }
  add("drop", "Drop...", value.location === "equipment" && value.slot === "Body" ? 300 : 100, "", true);
  return actions;
}

export function offer(state, id, verb, sourceId = null) {
  const action = actionsFor(state, id).find(value => value.verb === verb);
  if (!action) return { ok: false, message: "That action is not available for this item." };
  if (!action.enabled) return { ok: false, message: action.reason };
  const value = state.items.find(candidate => candidate.id === id);
  if (verb === "identify" && !identifySources(state).some(source => source.id === sourceId)) {
    return { ok: false, message: "Choose a known carried Identify scroll." };
  }
  return { ok: true, id, verb, sourceId, revision: state.revision, ...action,
    label: label(value), location: value.location };
}

export function commit(state, pending) {
  if (!pending?.ok) {
    return { ok: false, state, message: pending?.message ?? "Choose an available item action first." };
  }
  if (pending.revision !== state.revision) {
    return { ok: false, state, message: "The items changed. Review the current selection again." };
  }
  const checked = offer(state, pending.id, pending.verb, pending.sourceId);
  if (!checked.ok) return { ...checked, state };
  const next = structuredClone(state);
  const value = next.items.find(candidate => candidate.id === pending.id);
  const oldLabel = label(value);
  if (pending.verb === "drink" && next.hp >= next.maxHp) {
    return { ok: false, state, message: "No effect: already at full HP. No item or energy spent." };
  }
  if (pending.verb === "pickup" || pending.verb === "unequip") {
    value.cell = firstFit(bagItems(next), value);
    value.location = "backpack";
  } else if (pending.verb === "equip") {
    const incumbent = next.items.find(other => other.location === "equipment" && other.slot === value.slot);
    delete value.cell;
    value.location = "equipment";
    if (incumbent) {
      incumbent.cell = firstFit(bagItems(next), incumbent);
      incumbent.location = "backpack";
    }
  } else if (pending.verb === "drop") {
    value.location = "ground";
    delete value.cell;
  } else if (pending.verb === "drink") {
    next.hp = Math.min(next.maxHp, next.hp + value.heal);
    next.items = next.items.filter(other => other.id !== value.id);
    for (const other of next.items.filter(other => other.kind === "potion")) other.known = true;
  } else if (pending.verb === "identify") {
    next.items = next.items.filter(other => other.id !== pending.sourceId);
    value.known = true;
    for (const other of next.items) {
      if (other.kind === "scroll" || (value.kind === "potion" && other.kind === "potion")) other.known = true;
    }
  }
  next.actions++;
  next.energy += checked.cost;
  next.revision++;
  return {
    ok: true, state: next,
    message: `${checked.title.replace("...", "")}: ${oldLabel}. ${checked.cost} energy committed.`
      + (state.hostiles ? " In the game, enemies now act; this study does not simulate combat." : ""),
  };
}
