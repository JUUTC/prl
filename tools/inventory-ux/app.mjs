import {
  createStudy, itemView, label, bagItems, carriedWeight, knownTotals,
  comparison, actionsFor, identifySources, offer, commit,
} from "./model.mjs";

const $ = id => document.getElementById(id);
let state = createStudy();
let selected = "i3";
let view = "grid";
let pending = null;
let returnFocus = null;
let feedback = "Select an item to inspect. No action has been committed.";

function node(tag, text, className) {
  const element = document.createElement(tag);
  if (text !== undefined) element.textContent = text;
  if (className) element.className = className;
  return element;
}

function kg(grams) { return `${(grams / 1000).toFixed(1)} kg`; }

function itemButton(value, region) {
  const data = itemView(value);
  const button = node("button", undefined, `item ${data.rarity.toLowerCase()}${selected === data.id ? " selected" : ""}`);
  button.type = "button";
  button.dataset.item = data.id;
  button.dataset.region = region;
  button.setAttribute("aria-label", `${data.label}, ${data.rarity}, ${data.location}, ${data.width} by ${data.height}, ${kg(data.weight)}`);
  button.setAttribute("aria-pressed", String(selected === data.id));
  if (region === "equipment") button.append(node("span", data.slot, "slot"));
  const glyph = node("span", data.kind === "gear" ? data.slot === "Body" ? "AR" : "SW"
    : data.kind === "potion" ? "P" : data.kind === "scroll" ? "S" : "X", "glyph");
  glyph.setAttribute("aria-hidden", "true");
  const rarity = data.rarity === "Magic" ? "*" : "o";
  const marker = node("span", region === "grid" ? rarity : `${rarity} ${data.rarity}`, "marker");
  if (!data.known && region === "grid") marker.append(" ?");
  marker.setAttribute("aria-label", `${data.rarity}${data.known ? "" : ", unidentified"}`);
  button.append(glyph, marker,
    node("span", data.label, "item-name"));
  button.title = `${data.label} / ${data.width}x${data.height} / ${kg(data.weight)}`;
  if (region === "grid") {
    button.style.gridColumn = `${data.cell[0] + 1} / span ${data.width}`;
    button.style.gridRow = `${data.cell[1] + 1} / span ${data.height}`;
  }
  button.addEventListener("click", () => select(data.id, region));
  return button;
}

function select(id, region) {
  selected = id;
  feedback = "Inspecting only. No time passes.";
  render();
  document.querySelector(`[data-region="${region}"][data-item="${id}"]`)?.focus();
}

function descriptionList(values) {
  const list = node("dl");
  for (const [title, value] of values) list.append(node("dt", title), node("dd", String(value)));
  return list;
}

function renderDetails() {
  const area = $("detail-content");
  area.replaceChildren();
  const value = state.items.find(candidate => candidate.id === selected);
  if (!value) {
    area.append(node("p", "Select an item to see its known properties and actions."));
    return;
  }
  const data = itemView(value);
  area.append(node("h3", data.label, "item-title"),
    node("p", `${data.rarity} / ${data.known ? "Identified" : "Unidentified"}`),
    descriptionList([
      ["Location", data.location], ["Footprint", `${data.width} x ${data.height}`],
      ["Weight", kg(data.weight)], ...(data.slot ? [["Slot", data.slot]] : []),
    ]));
  if (data.effect) {
    area.append(node("h3", "Known effect"), node("p", data.effect));
    if (data.heal !== null) area.append(node("p",
      `At current HP: restore ${Math.min(data.heal, state.maxHp - state.hp)} HP, up to ${state.maxHp} maximum.`));
  }
  if (!data.known) {
    area.append(node("p", data.kind === "gear"
      ? "Unknown affixes excluded. Curse state unknown; no safety claim."
      : "Unknown effect. Using it may reveal its type; no effect preview is available.", "notice"));
  }
  const compared = comparison(state, selected);
  if (compared) {
    area.append(node("h3", "Known loadout comparison"), node("p", `Replaces: ${compared.incumbent}`));
    const table = node("table");
    const caption = node("caption", "Synthetic character totals (not game calculations)");
    const header = node("tr");
    for (const text of ["Stat", "Now", "If equipped", "Change"]) {
      const th = node("th", text); th.scope = "col"; header.append(th);
    }
    const head = node("thead"); head.append(header);
    const body = node("tbody");
    for (const stat of Object.keys(compared.before)) {
      const delta = compared.after[stat] - compared.before[stat];
      const row = node("tr");
      row.append(node("th", stat), node("td", compared.before[stat]),
        node("td", compared.after[stat]), node("td", `${delta > 0 ? "+" : ""}${delta}`));
      body.append(row);
    }
    table.append(caption, head, body);
    area.append(table);
    if (compared.uncertain) area.append(node("p",
      "Known base comparison only. Unknown affixes on either side are excluded; actual result may differ.", "notice"));
    if (data.location === "ground") area.append(node("p", "Preview only: pick up before equipping."));
  } else if (data.kind === "gear") {
    area.append(node("h3", "Known item contribution"),
      descriptionList(Object.entries(data.base).map(([key, amount]) =>
        [key, `+${amount + (data.bonuses[key] ?? 0)}`])));
  }
  const breakdown = node("details");
  breakdown.append(node("summary", "What these numbers mean"),
    node("p", "Damage and armor are character totals; armor is raw points, not a percent. Accuracy/evasion are ratings, not hit chances. This fixture has no reference target, DPS, durability or requirements."));
  area.append(breakdown);
}

function renderActions() {
  const value = state.items.find(candidate => candidate.id === selected);
  $("action-target").textContent = value ? `Selected: ${label(value)}` : "No item selected";
  $("actions").replaceChildren();
  for (const action of actionsFor(state, selected)) {
    const wrapper = node("div", undefined, "action-option");
    const button = node("button", `${action.title} - ${action.cost} energy`);
    button.id = `action-${action.verb}`;
    button.disabled = !action.enabled;
    if (action.verb !== "drop") button.classList.add("primary");
    button.addEventListener("click", () => requestAction(action.verb));
    wrapper.append(button);
    if (action.reason) {
      const reason = node("p", action.reason);
      reason.id = `reason-${action.verb}`;
      button.setAttribute("aria-describedby", reason.id);
      wrapper.append(reason);
    }
    $("actions").append(wrapper);
  }
  $("feedback").textContent = feedback;
}

function render() {
  const focus = document.activeElement?.id;
  const totals = knownTotals(state);
  $("vitals").textContent = `HP ${state.hp}/${state.maxHp} | ${state.actions} simulated item actions`;
  $("threat").textContent = state.hostiles
    ? "1 hostile visible. Browsing is free; acting advances enemies in the game. Body changes cost 3 standard actions."
    : "No hostiles visible in this fixture. Browsing is free; item actions still spend game time.";
  $("equipment-items").replaceChildren();
  const candidate = state.items.find(value => value.id === selected);
  for (const slot of ["Main hand", "Body"]) {
    const value = state.items.find(value => value.location === "equipment" && value.slot === slot);
    if (value) {
      const button = itemButton(value, "equipment");
      if (candidate?.slot === slot && candidate.location !== "equipment") {
        button.querySelector(".slot").classList.add("target");
        button.querySelector(".slot").textContent += " / comparison target";
      }
      $("equipment-items").append(button);
    } else {
      $("equipment-items").append(node("p", `${slot}: Empty`));
    }
  }
  $("totals").replaceChildren(...descriptionList(Object.entries(totals)).childNodes);
  const bag = bagItems(state);
  $("bag-grid").replaceChildren(...bag.map(value => itemButton(value, "grid")));
  $("bag-list").replaceChildren(...bag.map(value => itemButton(value, "list")));
  $("bag-grid").hidden = view !== "grid";
  $("bag-list").hidden = view !== "list";
  if (!bag.length) {
    $("bag-grid").append(node("p", "Backpack empty"));
    $("bag-list").append(node("p", "Backpack empty"));
  }
  $("grid-view").setAttribute("aria-pressed", String(view === "grid"));
  $("list-view").setAttribute("aria-pressed", String(view === "list"));
  const weight = carriedWeight(state);
  $("weight-label").textContent = `${kg(weight)} / ${kg(state.capacity)} - playtest pickup limit`;
  $("weight").value = weight / 1000;
  const occupied = bag.reduce((sum, value) => sum + value.width * value.height, 0);
  $("space").textContent = `${60 - occupied} / 60 cells free. Shape matters: free cells do not guarantee a fit.`;
  const ground = state.items.filter(value => value.location === "ground");
  $("ground-items").replaceChildren(...ground.map(value => itemButton(value, "ground")));
  if (!ground.length) $("ground-items").append(node("p", "No items at your feet."));
  renderDetails();
  renderActions();
  if (focus) $(focus)?.focus({ preventScroll: true });
}

function openInventory() {
  $("world").hidden = true;
  $("inventory").hidden = false;
  render();
  const region = state.items.find(value => value.id === selected)?.location;
  const targetRegion = region === "backpack" ? view : region;
  const target = document.querySelector(`[data-item="${selected}"][data-region="${targetRegion}"]`);
  (target ?? $("close")).focus();
}

function showWorld(message) {
  $("inventory").hidden = true;
  $("world").hidden = false;
  $("world-result").textContent = message;
  $("world-stats").textContent = `HP ${state.hp}/${state.maxHp} | ${state.actions} item actions | ${state.energy} total energy committed`;
  $("open").focus();
}

function finish(pendingAction) {
  const old = state.items.find(value => value.id === selected);
  const region = old?.location;
  const oldOrder = region === "backpack" ? bagItems(state)
    : state.items.filter(value => value.location === region);
  const index = oldOrder.findIndex(value => value.id === selected);
  const result = commit(state, pendingAction);
  feedback = result.message;
  if (!result.ok) {
    render();
    $("feedback").focus();
    return;
  }
  state = result.state;
  const kept = state.items.find(value => value.id === selected);
  if (!kept || pendingAction.verb === "pickup" || pendingAction.verb === "drop") {
    const remaining = region === "backpack" ? bagItems(state)
      : state.items.filter(value => value.location === region);
    selected = remaining[Math.min(Math.max(0, index), remaining.length - 1)]?.id ?? null;
  }
  showWorld(feedback);
}

function requestAction(verb) {
  if (pending || $("inventory").hidden) return;
  const action = actionsFor(state, selected).find(value => value.verb === verb);
  if (!action) return;
  if (!action.enabled) {
    feedback = action.reason;
    renderActions();
    return;
  }
  if (!action.review) {
    finish(offer(state, selected, verb));
    return;
  }
  const value = state.items.find(candidate => candidate.id === selected);
  pending = { id: selected, verb };
  returnFocus = `action-${verb}`;
  $("review-title").textContent = `${action.title.replace("...", "")} one item`;
  $("review-target").textContent = `${label(value)} / from ${value.location}`;
  $("review-cost").textContent = `${action.cost} energy - ${action.cost / 100} standard action${action.cost === 100 ? "" : "s"}. Nothing is spent until commitment.`;
  $("review-warning").textContent = verb === "drink" ? "Unknown effect. This is a commitment to use it, not to inspect it."
    : verb === "drop" ? "Item will be left on the current cell. No undo of the committed turn."
      : verb === "identify" ? "Choose a known carried scroll. One source is consumed to identify this target."
        : "Enemies may act multiple times during this Body change. No attacks are simulated here.";
  $("review-error").textContent = "";
  $("source-label").hidden = verb !== "identify";
  $("source").replaceChildren();
  if (verb === "identify") {
    const placeholder = node("option", "Choose a known scroll...");
    placeholder.value = "";
    $("source").append(placeholder);
    for (const source of identifySources(state)) {
      const option = node("option", `${source.label} (${source.id})`);
      option.value = source.id;
      $("source").append(option);
    }
  }
  $("review").showModal();
  $("cancel").focus();
}

$("confirm").addEventListener("click", () => {
  if (!pending) return;
  const checked = offer(state, pending.id, pending.verb, $("source").value || null);
  if (!checked.ok) {
    $("review-error").textContent = checked.message;
    return;
  }
  pending = null;
  $("review").close("commit");
  finish(checked);
});
$("review").addEventListener("close", () => {
  pending = null;
  if (!$("inventory").hidden) $(returnFocus)?.focus();
});
$("open").addEventListener("click", openInventory);
$("close").addEventListener("click", () => showWorld("Inventory closed. No item action committed."));
for (const display of ["grid", "list"]) {
  $(`${display}-view`).addEventListener("click", () => { view = display; render(); });
}
function reset() {
  if ($("review").open) $("review").close("cancel");
  pending = null;
  state = createStudy($("scenario").value);
  selected = state.scenario === "space" ? "i9"
    : ["weight", "long-name"].includes(state.scenario) ? "i8"
      : state.scenario === "empty" ? null : "i3";
  feedback = "Situation reset. These are synthetic fixtures, not a game save.";
  $("world").hidden = true;
  $("inventory").hidden = false;
  render();
}
$("scenario").addEventListener("change", reset);
$("reset").addEventListener("click", reset);

document.addEventListener("keydown", event => {
  if (event.repeat || event.ctrlKey || event.altKey || event.metaKey) return;
  if ($("review").open || ["SELECT", "INPUT", "TEXTAREA"].includes(event.target.tagName)) return;
  if ($("inventory").hidden) {
    if (event.key.toLowerCase() === "i") { event.preventDefault(); openInventory(); }
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    showWorld("Inventory closed. No item action committed.");
    return;
  }
  const direction = ["ArrowRight", "ArrowDown"].includes(event.key) ? 1
    : ["ArrowLeft", "ArrowUp"].includes(event.key) ? -1 : 0;
  if (direction && event.target.dataset.item) {
    event.preventDefault();
    const collection = event.target.closest(".collection");
    const buttons = [...collection.querySelectorAll("button[data-item]")];
    const index = buttons.indexOf(event.target);
    const next = buttons[Math.min(buttons.length - 1, Math.max(0, index + direction))];
    select(next.dataset.item, next.dataset.region);
    return;
  }
  const verb = { e: state.items.find(value => value.id === selected)?.location === "equipment" ? "unequip" : "equip",
    q: "drink", c: "identify", d: "drop" }[event.key.toLowerCase()];
  if (verb) { event.preventDefault(); requestAction(verb); }
});
render();
