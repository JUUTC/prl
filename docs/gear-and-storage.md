# Gear Comparison, Optimizer & Storage

Design for comparing gear, finding a good loadout on request, loot filters with
preset defaults, and moving items between the character and storage: custom
storage tabs, auto-sort, inventory refills and crafting from storage. Items, rarity, identification and crafting
rules live in [items-and-crafting.md](items-and-crafting.md). The backpack,
pack animals and houses are in [housing-and-logistics.md](housing-and-logistics.md).

Status: design baseline, added 2026-11-09. Rows marked with a `Q-NNN` ID use
provisional defaults (see [open-questions.md](open-questions.md)). Delivery
stages are in [playtest-plan.md](playtest-plan.md#gear-and-storage-delivery).

Current P3 foundation: `derive_stats(actor, loadout, known_only)` is pure;
combat uses actual affixes, while comparison uses only known bonuses. Actor
definitions still provide the provisional base until P4 stats/skills (Q-019).
`ItemId`/`ContainerId` ownership, atomic transfers, a 10x6 first-fit backpack,
Main hand/Body and selected-item comparisons are implemented, as are the shared
`ItemPredicate` and read-only content-defined Default ground-label preset.
The rest of this document remains target design; see the exact
[slice limits](items-and-crafting.md#current-p3-subset).

The implemented [inventory/item workspace](inventory-ux.md) provides equipment,
backpack, ground and persistent item-card workspace, including knowledge-safe
comparison and action flows. Its browser study is mock data, not another stats
path. I-034/I-035 are delivered; free packing/rotation/Organize is approved I-036.
Optimizer/storage breadth remains after P5; final art and Windows acceptance
remain separate.

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

## Loot filters

Loot filters decide how items on the ground are **shown and picked up**, in the
style of Path of Exile item filters. They use the same item-rule language as
the auto-sort rules, so one rule can describe "rare rings" for both.

### What a filter does

A filter is an **ordered list of rules**. Each rule has conditions and actions.
The first rule that matches an item applies (later rules are ignored), and a
final catch-all rule is always present.

- **Conditions** use item data and tags: category, slot, base type, rarity,
  item level, material tier, sockets, stack size, value, set/unique, tags
  (`reagent`, `undead_bane`, ...), and identified or unidentified. Conditions
  see **only what the character knows**: an unidentified item matches on its
  base type and rarity colour, never on hidden affixes or curse status.
- **Actions:**
  - **Display:** show, minimise (small dim label) or hide the ground label.
  - **Emphasis:** label style and a minimap/edge marker. Placeholder styles
    use shape and outline as well as colour, so emphasis never relies on
    colour alone. Final styles belong to the UX/art effort.
  - **Sound:** optional alert sound from a fixed set.
  - **Auto-pickup:** on or off (see below).

### Safety rules

- Filters **never destroy, move or alter items**. A hidden item is still on the
  ground and still in the game state. A "show all" key temporarily reveals every
  item, and the inspect/look command always lists hidden items.
- Filters cannot hide **Unique, Set or quest items**, or items the character
  has marked as wanted; the rule editor rejects a rule that tries (Q-027).
- Filters are **presentation and command helpers only**. They never change core
  rules, RNG, drops or the clock, so a filter cannot change what drops.

### Auto-pickup

- When the player steps onto a tile, items whose matching rule has auto-pickup
  are picked up by **normal pickup commands**. They cost the usual pickup time
  and are recorded like any other command, so replays and saves stay exact.
- Auto-pickup stops when a hostile comes into view, stops before going
  over-encumbered or running out of space, and reports what it left behind.
- Gold auto-pickup follows the same rule (default on in every preset) (Q-027).

### Presets and custom filters

- The game ships **built-in presets** as content data (a filter file in the
  content pack). Presets are read-only:

  | Preset | Shows | Auto-pickup |
  |---|---|---|
  | **Default** | Everything; Magic and better emphasised | Gold, potions, scrolls, ammo for the equipped launcher |
  | **Strict** | Hides Normal non-material gear below the area level; minimises Magic gear | As Default plus identified Rare jewellery |
  | **Very strict** | Hides Normal and Magic gear; shows Rare and better, materials and consumables | As Strict |
  | **Crafter** | Emphasises materials, reagents and socketed bases | Default plus materials and reagents |
  | **Show all** | Every item with equal emphasis | Gold only |
  | **Manual** | As Default | Nothing |

- **Custom filters:** the player copies a preset (or starts empty) and edits
  it: add, reorder, enable/disable and delete rules, with a live preview of
  items on the current screen. Custom filters can be renamed.
- Filters are stored in the **profile**, so every character can use them. Each
  character selects its active filter; a quick key cycles a short list of
  favourites (Q-027).
- **Import/export** as a plain-text versioned file so filters can be shared.
  Imported filters are validated; unknown definition IDs or tags are kept but
  shown as warnings, so a filter written for newer content still loads.
- When content changes, a preset is updated with the game; a custom filter is
  never rewritten silently.

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
| One rule language | Loot filters, auto-sort rules and optimizer constraints share one pure, serializable `ItemPredicate` evaluated against the character's knowledge of an item |
| Filters cannot affect play | Filter evaluation is read-only; auto-pickup only submits ordinary pickup commands, so the command log stays the source of truth |
| Crafting sources | Recipe resolution queries "accessible containers at this site" for materials, using the same access rule as transfers |
| Persistence | Tabs, rules, kits and locks are versioned save data, owned by the profile/world as described in [foundation-plan.md](foundation-plan.md) |
