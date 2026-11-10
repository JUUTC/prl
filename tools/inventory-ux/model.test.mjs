import test from "node:test";
import assert from "node:assert/strict";
import {
  createStudy, itemView, bagItems, firstFit, knownTotals, comparison,
  identifySources, actionsFor, offer, commit, carriedWeight,
} from "./model.mjs";

function act(state, id, verb, source) {
  return commit(state, offer(state, id, verb, source));
}

function assertOwnership(state) {
  const bag = bagItems(state);
  assert.equal(new Set(state.items.map(value => value.id)).size, state.items.length);
  const occupied = new Set();
  const slots = new Set();
  for (const value of state.items) {
    if (value.location === "equipment") {
      assert(!slots.has(value.slot));
      slots.add(value.slot);
    }
    if (value.location !== "backpack") {
      assert.equal(value.cell, undefined);
      continue;
    }
    const [x, y] = value.cell;
    assert(x >= 0 && y >= 0 && x + value.width <= 10 && y + value.height <= 6);
    for (let dy = 0; dy < value.height; dy++) {
      for (let dx = 0; dx < value.width; dx++) {
        const key = `${x + dx},${y + dy}`;
        assert(!occupied.has(key));
        occupied.add(key);
      }
    }
  }
  assert.equal(bag.length, state.items.filter(value => value.location === "backpack").length);
}

test("browsing, comparison and availability are read-only", () => {
  const state = createStudy();
  const before = structuredClone(state);
  for (const value of state.items) {
    itemView(value); comparison(state, value.id); actionsFor(state, value.id);
  }
  assert.deepEqual(state, before);
});

test("unknown views and comparisons cannot reveal hidden affix changes", () => {
  const a = createStudy();
  const b = structuredClone(a);
  const hidden = b.items.find(value => value.id === "i8");
  hidden.affix = { damage: 900, armor: 900 };
  hidden.suffix = "SECRET";
  assert.deepEqual(itemView(a.items.find(value => value.id === "i8")), itemView(hidden));
  assert.deepEqual(comparison(a, "i8"), comparison(b, "i8"));
  assert(!JSON.stringify(itemView(hidden)).includes("SECRET"));
});

test("unknown supplies have no effect preview or hidden identification source", () => {
  const state = createStudy("unknown");
  assert.equal(itemView(state.items.find(value => value.id === "i3")).effect, "Unknown effect");
  assert.equal(itemView(state.items.find(value => value.id === "i3")).heal, null);
  assert.deepEqual(identifySources(state), []);
  const identify = actionsFor(state, "i8");
  assert.equal(identify.some(action => action.verb === "identify"), false, "ground offers pickup only");
  const picked = act(state, "i8", "pickup").state;
  assert.match(actionsFor(picked, "i8").find(action => action.verb === "identify").reason, /No known/);
  const another = structuredClone(state);
  another.hp = 40;
  assert.deepEqual(actionsFor(state, "i3"), actionsFor(another, "i3"),
    "full HP availability cannot identify unknown potions");
});

test("healing spends one action and known full-health rejection consumes nothing", () => {
  const start = createStudy();
  const result = act(start, "i3", "drink");
  assert(result.ok);
  assert.equal(result.state.hp, 40);
  assert.equal(result.state.energy, 100);
  assert.equal(result.state.actions, 1);
  assert.equal(result.state.items.some(value => value.id === "i3"), false);
  const fail = act(result.state, "i4", "drink");
  assert.equal(fail.ok, false);
  assert.deepEqual(fail.state, result.state);
});

test("unknown consumption reviews, reveals the type and updates all remaining copies", () => {
  const state = createStudy("unknown");
  const pending = offer(state, "i3", "drink");
  assert.equal(pending.review, true);
  const result = commit(state, pending);
  assert(result.ok);
  assert(result.state.items.filter(value => value.kind === "potion").every(value => value.known));
  assert(result.state.items.filter(value => value.kind === "scroll").every(value => !value.known));
  const full = createStudy("unknown");
  full.hp = 40;
  const failed = act(full, "i3", "drink");
  assert.equal(failed.ok, false);
  assert.deepEqual(failed.state, full);
});

test("pickup, swap and drop preserve unique ownership and cost the documented actions", () => {
  let state = createStudy();
  const ids = state.items.map(value => value.id).sort();
  for (const [id, verb] of [["i8", "pickup"], ["i8", "equip"], ["i8", "drop"]]) {
    const result = act(state, id, verb);
    assert(result.ok);
    state = result.state;
    assertOwnership(state);
    assert.deepEqual(state.items.map(value => value.id).sort(), ids);
  }
  assert.equal(state.energy, 250);
  assert.equal(state.actions, 3);
  assert.equal(state.items.find(value => value.id === "i1").location, "backpack");
});

test("fragmented space and weight failures are distinct, atomic and free", () => {
  for (const [scenario, id, reason] of [["space", "i9", /No contiguous 2x3/], ["weight", "i8", /Over pickup limit/]]) {
    const state = createStudy(scenario);
    const result = act(state, id, "pickup");
    assert.equal(result.ok, false);
    assert.match(result.message, reason);
    assert.deepEqual(result.state, state);
    assertOwnership(result.state);
  }
});

test("failed displaced-item placement leaves incumbent equipment untouched", () => {
  const state = createStudy("space");
  const candidate = state.items.find(value => value.id === "i9");
  candidate.location = "backpack";
  // Create exactly one 1x1 vacancy for a synthetic small candidate.
  candidate.width = 1; candidate.height = 1; candidate.cell = [1, 0];
  const result = act(state, "i9", "equip");
  assert.equal(result.ok, false);
  assert.match(result.message, /No contiguous 2x3 space for Iron mail/);
  assert.deepEqual(result.state, state);
});

test("identify requires an explicit known source, consumes one and refreshes known comparison", () => {
  let state = act(createStudy(), "i8", "pickup").state;
  assert.equal(offer(state, "i8", "identify").ok, false);
  assert.equal(offer(state, "i8", "identify", "i3").ok, false);
  assert.equal(comparison(state, "i8").after.damage, 9);
  const result = act(state, "i8", "identify", "i5");
  assert(result.ok);
  state = result.state;
  assert.equal(state.items.some(value => value.id === "i5"), false);
  assert.equal(state.items.find(value => value.id === "i6").known, true);
  assert.equal(comparison(state, "i8").after.damage, 11);
  assert.equal(comparison(state, "i8").uncertain, false);
  assertOwnership(state);
});

test("body changes cost 300 and require danger review; stale commitments do not repeat", () => {
  const state = createStudy("danger");
  const pending = offer(state, "i2", "unequip");
  assert.equal(pending.cost, 300);
  assert.equal(pending.review, true);
  const result = commit(state, pending);
  assert(result.ok);
  assert.equal(result.state.actions, 1);
  assert.equal(result.state.energy, 300);
  const again = commit(result.state, pending);
  assert.equal(again.ok, false);
  assert.deepEqual(again.state, result.state);
});

test("all starting fixtures have valid packing, and list order is deterministic", () => {
  for (const scenario of ["normal", "unknown", "danger", "space", "weight", "full-hp", "empty", "long-name"]) {
    const state = createStudy(scenario);
    assertOwnership(state);
    assert(carriedWeight(state) <= state.capacity, "starting fixture is not already overweight");
    assert.deepEqual(bagItems(state), bagItems(structuredClone(state)));
  }
  assert.deepEqual(firstFit([], { width: 2, height: 3 }), [0, 0]);
  assert.equal(firstFit([], { width: 11, height: 1 }), null);
  assert.equal(knownTotals(createStudy()).damage, 9);
});
