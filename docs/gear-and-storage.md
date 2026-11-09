# Gear Comparison, Optimizer & Storage

Design for comparing gear, finding a good loadout on request, and moving items
between the character and storage: custom storage tabs, auto-sort, inventory
refills and crafting from storage. Items, rarity, identification and crafting
rules live in [items-and-crafting.md](items-and-crafting.md). The backpack,
pack animals and houses are in [housing-and-logistics.md](housing-and-logistics.md).

Status: design baseline, added 2026-11-09. Rows marked with a `Q-NNN` ID use
provisional defaults (see [open-questions.md](open-questions.md)). Delivery
stages are in [playtest-plan.md](playtest-plan.md#gear-and-storage-delivery).

## Principles

1. **One rules path.** Comparison, the optimizer, tooltips and real combat use
   the same pure stat-derivation function in `prl_core`. Nothing in the UI
   re-implements formulas, so a comparison can never disagree with play.
2. **Only known information.** Comparisons and the optimizer see only what the
   character knows. Hidden affixes and curse status of unidentified items stay
   hidden; such items are scored on their known base and flagged "unknown".
3. **Previews are free; changes follow normal rules.** Comparing and optimizing
   are read-only and cost no game time. Applying a result issues ordinary
   equip/transfer commands with their usual validation and time costs.
4. **Deterministic.** Same state and settings give the same comparison, the
   same optimizer result and the same sort order. Ties break on stable IDs.
5. **Location matters.** Storage is reached only where it physically is (bank in
   a town, a house's storage at that house, a pack animal next to you). This keeps
   pack-animal logistics and housing meaningful.

## Gear comparison

- Hovering or selecting an item shows a **comparison against what is equipped**
  in the slot(s) it would occupy. Rings compare against both ring slots.
  Two-handed weapons compare against main hand plus off hand together.
- The comparison shows **derived results**, not just affix text: change in
  damage per action, crit/glance chances against a reference target, armor %,
  resistances (after the difficulty penalty and caps), HP/mana/stamina, speed,
  weight/encumbrance and requirement penalties. Increases and decreases use
  shape and sign, not colour alone.
- **Set bonuses, runewords and requirements** are evaluated on the whole
  resulting loadout, so losing a set bonus shows as a loss.
- A comparison can target either of the **two quick-swap weapon loadouts**.
- The reference target used for crit/glance/armor numbers is selectable: a
  training dummy at the current area level by default (Q-022).

## Optimizer

The player triggers the optimizer explicitly (a button or key on the equipment
screen). It never runs automatically and never changes gear by itself.

- **Goal:** pick a preset objective (balanced, offence, defence, resist caps,
  carry weight) or a custom weighting. Constraints include: keep slot X, never
  equip unidentified or possibly cursed items, respect stat requirements, stay
  under an encumbrance tier, and keep set bonus Y (Q-022).
- **Candidate pool:** equipped and carried items by default. When standing at
  storage, the player may include that storage's tabs.
- **Result:** a preview of the proposed loadout with the same comparison view,
  a per-slot change list and the total time cost of applying it. The player can
  accept all, accept per slot, or cancel. Applying it runs normal equip and
  transfer commands; if one fails validation, the remaining steps stop and the
  reason is shown.
- **Bounded work:** the search uses the pure evaluator with a deterministic
  budget and a "best found" result if the budget runs out. It runs outside the
  turn scheduler so the game never waits on it, and it is not available while
  hostiles are in view.

## Storage

### Storage locations

| Location | Shared? | Access |
|---|---|---|
| **Bank** | Shared across all towns | At any town bank |
| **House storage** | One per house, not shared | At that house only |
| **Pack animal cargo** | Per animal | When the animal is adjacent, or when it delivers |

### Tabs (custom locations)

Each storage location holds a list of **tabs**, similar to Path of Exile stash
tabs, but bought **with in-game gold or earned in play, never with real money**
(Q-023).

- Every tab has a player-chosen **name, colour and icon**, and its position
  in the tab list can be changed.
- **Tab types:**

  | Type | Layout | Use |
  |---|---|---|
  | General | 12 × 12 grid with item shapes | Anything |
  | Material | Fixed slot per material/reagent definition, high stack limits | Crafting resources by tier |
  | Consumable | Fixed slot per potion/scroll/ammo type | Supplies |
  | Gear | Grid with a filter for slot/rarity | Equipment sets and spares |
  | Favorites | Small grid; items here are never auto-sorted or sold | Treasured items |

- House tiers grant a number of free tabs instead of "grid pages" (Small 2,
  Medium 4, Large 8, Castle 16). More tabs can be bought, up to a cap per
  location (Q-023).
- On New Game+, house tabs move into the bank as tabs, keeping their name and
  contents (see [endgame.md](endgame.md#new-game)).

### Moving items in and out

- **Deposit / withdraw** single items, stacks (with a split amount) or a
  selection. Ctrl-click style quick-move sends an item to the best matching tab.
- **Deposit all** moves every unlocked backpack item into storage using the
  auto-sort rules. Equipped items, belt items and **locked** backpack items
  stay.
- Transfers at storage cost no game time when no hostiles are in view; inside
  hostile range they are not allowed (Q-024).
- Every transfer is **atomic**: an item is never duplicated or lost, even if
  the destination fills part-way. A partial transfer reports what stayed behind.

### Auto-sort rules

- Each tab can have **rules**, which are filters on item data (category, slot,
  rarity, material tier, tags such as `reagent` or `undead_bane`, identified
  or unidentified). Rules are data, so new content works without code changes.
- Rules have a **priority order** across tabs. An item goes to the first
  matching tab with room, then to a General tab, otherwise it stays in the
  backpack and is reported.
- **Sort tab** reorders items inside a tab (by type, then rarity, then name,
  then ID) with the same deterministic result every time.

### Kits: refill favorites into the character's inventory

A **kit** is a saved template of what the character wants to carry and where:
for example 5 healing potions on belt slot 1, 40 arrows in the quiver,
10 bandages and a repair kit in fixed backpack cells.

- Up to several named kits per character (Q-025). A kit records **item
  definitions and quantities** (any matching item) and can also **pin a
  specific item** by its instance ID.
- **Refill** (one action at storage) tops up the inventory to the kit from the
  storage there. It uses the lowest-quality identical stacks first unless the
  kit says otherwise, and reports any shortfall.
- **Save current as kit** captures the current backpack and belt layout.
- Kit cells act as a **lock**, so Deposit all does not remove kit items.

## Crafting from storage

- A crafting station can use materials from the **carried inventory plus the
  storage at the same site**: the house's storage for a house station, and
  the bank for a station in that town. Pack-animal cargo counts when the animal
  is adjacent (Q-026).
- The recipe view shows available quantities per source and the total, and
  which source will be used. By default it uses carried materials first, then
  storage (Material tabs before General). The player can change this.
- Consumption is atomic with the craft: materials are reserved when the craft
  starts and only consumed on completion or on a failure that destroys
  materials, so nothing is lost if the craft is interrupted. Output goes to the
  backpack, or to storage if the backpack is full.
- Repair and enchanting use the same source rules.

## Foundation contracts

These keep the design possible without building it before it's scheduled:

| Need | Contract |
|---|---|
| Comparison/optimizer equal to real play | `prl_core` exposes a pure `derive_stats(actor, loadout, context) -> DerivedStats`. Combat reads `DerivedStats`; P2's definition-supplied stats (Q-019) become its first input |
| Hypothetical loadouts | A loadout is a value (slot → `ItemId`), not ECS components, so it can be evaluated without changing state |
| Knowledge filtering | Evaluation takes the character's knowledge of each item, so unknown affixes are excluded by construction |
| Storage anywhere | A `ContainerId` with an owner (character, world site, animal), a location (map/site), a layout (grid, slotted, equipment) and an ordered tab list. The backpack is just one container |
| No duplication or loss | One atomic `TransferItems` command validates the source, destination, capacity and access location; it is property-tested for item conservation |
| Data-driven sorting | Tab rules and kits are serializable filters over item data and tags; no Rust `match` on content IDs |
| Crafting sources | Recipe resolution queries "accessible containers at this site" for materials, using the same access rule as transfers |
| Persistence | Tabs, rules, kits and locks are versioned save data, owned by the profile/world as described in [foundation-plan.md](foundation-plan.md) |
