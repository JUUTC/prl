# Inventory and item UX

**Status: interaction proposal and executable study, 2026-11-10.**
Requested focus: make inventory and item decisions understandable and comfortable
with placeholder art. This is not a new art direction or an implemented game UI.
The current Bevy inventory remains the text overlay until I-034/I-035 ship.
The study is in [`tools/inventory-ux/`](../tools/inventory-ux/README.md).

Rules authority remains [items-and-crafting.md](items-and-crafting.md),
[time-and-turns.md](time-and-turns.md) and
[gear-and-storage.md](gear-and-storage.md). This document specifies how those
rules are communicated, not new inventory capacity, stacking, healing or costs.
Q-030/Q-031 track unapproved interaction defaults. Preserve the separate
[art/UX gates](art-production.md); placeholder icons and styling are disposable.

## 1. Player jobs and priorities

| Priority | Job | Success criterion |
|---|---|---|
| First | Heal without deciphering controls | Find a carried potion, see known effect/unknown risk and cost, explicitly use it; selection never consumes it |
| First | Understand carried versus worn versus ground | Distinct labelled regions; one item has one location; picking up does not equip |
| First | Decide whether to equip a find | Candidate and displaced equipment remain visible alongside known-only character changes |
| First | Recover from a failed action | Precise reason beside the action; item, selection and world state remain intact |
| Next | Collect a pile without repeated menu hunting | Ground list at the current cell, single-item actions, focus retained by persistent item ID |
| Next | Understand shape and capacity | True 10x6 geometry, item footprint and weight; distinguish no contiguous room from overweight |
| Later | Manage larger item sets | Same item card, focus and transfer language extend to stacks, bags and local storage when their rules ship |

Do not optimize for decorating a paper doll before healing, comparison and
safe selection work. Do not solve the short-run balance issue by adding free
healing or changing starting supplies.

## 2. Workspace: one place to make item decisions

Use a near-full-window modal with an opaque reading surface and a small visible
world margin. Keep the header and action footer fixed; scroll content panes,
not the entire modal. World zoom must not change its geometry.

```text
+ INVENTORY   Browsing: no time passes  HP 16/40  Carry 18.4/60 kg  [Close] +
| Threat: 1 hostile visible. Acting advances enemies; browsing does not.  |
+------------------+--------------------------------+--------------------+
| EQUIPMENT        | BACKPACK - 10 x 6               | ITEM DETAILS       |
| Main hand        | [Grid] [List]                   | Magic iron sword ? |
| [Iron sword]     |                                | carried / 1 x 3    |
|                  | real multi-cell item footprints | 3.0 kg             |
| Body             | selected outline + text label   |                    |
| [Iron mail]      | no auto-rearrange on selection  | KNOWN COMPARISON   |
|                  |                                | Now -> If equipped |
| Known totals     |                                | Damage 9 -> 9 (+0) |
| Damage / Armor   | Weight meter + exact text      | Affixes unknown    |
+------------------+--------------------------------+--------------------+
| AT YOUR FEET - current cell only [item] [item] [item] ...                |
+------------------------------------------------------------------------+
| Selected: Magic iron sword ?     [Equip - 100 energy] [Identify] [Drop]  |
| Action result / reason stays here.                                     |
+------------------------------------------------------------------------+
```

### Equipment

- Start with Main hand and Body, the two functional slice slots. Show a
  labelled slot even when empty. Slot identity matters more than a silhouette.
- Clicking/focusing an equipped item uses the **same details card** as the bag.
  Selecting a bag weapon highlights the slot it would replace; the incumbent's
  name and item knowledge are visible in comparison.
- Do not draw the current world sprite's shield as an equipped item. There is
  no functional shield slot/block mechanic in the slice.
- Future full paper doll groups Head/Body/Hands/Feet, jewellery and hand slots;
  show main/off hand anatomically, not as screen-left/screen-right guesses.
  Do not fill the current screen with disabled future slots.

### Backpack

- The grid is a packing view, not a row of icons. Draw each item once across
  its actual footprint. Use a short category glyph or ID, rarity marker and
  identification `?`; the selected item's full name is always outside the grid.
- Offer a **List** view with the same items, order, actions and details.
  Switching view is free and retains selection. Keyboard/list access is not a
  second-class alternative to dragging.
- MVP grid is read-only placement plus selection. No drag-to-repack illusion:
  current core disallows same-container transfers. See I-028/Q-031.
- Label both capacity measures. In the current game: `18.4 / 60.0 kg -
  playtest pickup limit`; do not call it STR-derived capacity or invent
  encumbrance tiers. Empty-cell count does not guarantee an item fits.
- Do not auto-sort/repack the grid on pickup, inspection or list sorting.
  Future list filters dim nonmatching grid items instead of collapsing their
  occupied cells. An item-list filter is not a ground loot filter.

### Ground

- `At your feet` lists every item at the current cell, including loot-filter
  hidden items. This is an explicit manual pickup view, not auto-pickup.
- Nearby visible loot belongs to examine/world labels. Do not let this panel
  remotely transfer an item, inspect remembered live contents or show unseen loot.
- Selecting ground gear may preview a comparison, but offers **Pick up**, not
  Equip/Drink. Pickup is a separate 50-energy action, with its own capacity check.
- MVP is one item per action. No `Take all` button or multi-turn queue until
  interruption, partial failure and total-cost UX are separately designed.

### Details and action strip

- Persistent details for explicit selection; no hover-only critical content.
  Pointer hover may show a small read-only name preview, never change the
  selected action target. No auto-equipping on hover or double-click.
- Keep actions and cost readable without scrolling through the full tooltip.
  Show unavailable actions with a specific reason, not just grey icons.
- A long name wraps in details; grid/list names may truncate only when the full
  name remains accessible. Use stable IDs internally, never letters/list indices.
- Empty states say `Backpack empty`, `No items at your feet` or `Select an item`;
  never fabricate an invalid item to trigger a generic core error.

## 3. Item card and comparison contract

Read in this order:

1. **Identity:** known name/appearance, rarity word + marker, known/unknown state.
2. **Location and shape:** Equipped / Backpack / Ground, compatible slot,
   footprint and weight. Quantity appears only after stacks exist.
3. **Known function:** base stats or the known effect, with an `Unidentified -
   effect unknown` warning when appropriate.
4. **Comparison:** current loadout versus the result, incumbent item(s), signed
   changes and explicit uncertainty.
5. **Available actions, time cost and failure reason.**
6. **Optional disclosure:** known mathematical breakdown, flavour and future
   durability/requirements only when those rules exist.

### Knowledge is a boundary, not a tooltip convention

An unidentified item must not leak its hidden affix, effect, curse, value,
score or true consumable type through name, icon, sort/filter order, ARIA label,
button enablement, preview, log or source-selector labels. A healing-looking
icon must not distinguish the true type of an unknown potion.

For a known potion, show e.g. `Restore up to 24 HP; at current HP, restores 24`.
For an unknown potion, show only its appearance and `Unknown effect. Using it
may reveal its type.` Do not preview the actual heal. Unknown item actions may
still be rejected by core; a reason must not be presented before the attempt if
it would reveal hidden knowledge.

Normal items remain known. Unknown Magic gear compares its **known base only**.
Show `Unknown affixes excluded on either side; actual result may differ`, not
`No upgrade` or a green overall score. Unknown curse state is not `Uncursed`.
Do not reveal hidden bonuses by subtracting actual current character stats.

### Comparison is a loadout result

- Use core's known-only evaluator for current and hypothetical equipment.
  Present `Now | If equipped | Change`; distinguish base weapon damage from
  character damage and raw armor points from mitigation percentage.
- Slice rows: damage, armor, accuracy, evasion. Do not invent DPS, crit chance
  against no target, resistances, block or requirement penalties.
- Body swaps show both items and their time cost. Losing one benefit while
  gaining another is visible; no automatic `Best` badge.
- Already equipped items show their known contribution, not an offer to
  compare the item against itself as an upgrade. Removal preview is a separate
  future evaluator request; never relabel equip comparison as unequip math.
- Later: ring destination picker; two-handed main+off-hand displacement;
  loadout A/B; sets/runewords and reference targets using I-013/Q-022.
- Icons, `+`/`-` and words carry meaning without colour. Better/lower depends
  on the stat, not on all positive numbers being universally desirable.

## 4. Interaction and commitment

**Proposed default (Q-030): one explicit action, one ordinary command, then
return to the world.** Opening inventory, changing selection/view, comparing
and reading the review step never advances time.

This intentionally preserves the current safe action boundary: item actions can
let enemies act. Improve reopening by retaining region, view, scroll and selection
by `ItemId`, not by leaving gameplay hidden behind the inventory.

| Intent | UX | Existing slice cost |
|---|---|---|
| Pick up | Ground selection -> explicit Pick up; first-fit bag destination | 50 energy |
| Equip / unequip | Show slot and incumbent/destination; explicit action | 100; Body 300 |
| Drink | Carried potion; known heal or unknown-effect warning | 100 |
| Identify | Choose target then a carried scroll source; inspect both before commit | 100 |
| Drop | Explicit selected item; additional review to prevent accidental loss | 100; worn Body 300 |
| Browse / cancel / switch view | No gameplay command | Free |
| Rejection | Remain in inventory; preserve selection; inline reason | No cost or item loss |

Show `300 energy - 3 standard actions; enemies may act multiple times`, not
`3 seconds` or `guaranteed 3 enemy attacks`. The energy scheduler and carried
energy determine actual elapsed ticks. Do not show a countdown for free browsing.

### Review policy

- Equip/pickup/known consumption normally commit from their labelled action
  button; no universal confirmation spam.
- A review step is required for Drop, unknown consumption, and a 300-energy
  Body change while hostiles are visible (provisional Q-030).
- The review includes item(s), source/destination, known result, energy cost,
  unknown risks and threat warning. No prechecked destructive defaults.
- Keyboard opening a review cannot also confirm it. Ignore key-repeat for
  item actions. Escape cancels the topmost mode first. Moving focus never acts.
- Only one command may be pending. Disable action controls until it resolves;
  revalidate the selected ID, location and target at commitment.
- Success: return to world, show a terse result in the message log, and clear
  held-input state. If the player dies, show death instead of reopening inventory.
- Failure: remain/reopen in inventory with the exact core reason and a useful
  next step. Do not silently auto-drop the displaced item or consume a fallback.
- Persistence errors remain visible in world/save status; do not describe an
  item operation as durably saved just because its command succeeded.

### Identification journey

1. Select the unknown target. `Identify...` opens a source chooser.
2. List carried scrolls by **known appearance/name**, not internal ItemKind.
   Never highlight which unknown scroll is secretly Identify.
3. MVP may offer known Identify scrolls; if none are known, explain how to
   select an unknown scroll and attempt Read with a target. That broader Read
   contract needs Q-030, since the current core only exposes `Identify`.
4. For the first implementation without general Read, do not silently select
   the true unknown Identify scroll. Show `No known Identify scroll` and keep
   target selection. Current healing/loot remains available without identifying.
5. A successful Identify changes target knowledge and scroll-type knowledge
   through the normal outcome; a gear preview then refreshes from known state.

This is a deliberate correction to current UI code, which discovers a scroll
by its hidden definition kind. It needs a focused regression before Bevy UX
delivery (B-010), not a new rendering-only workaround.

## 5. Pointer, keyboard and focus

| Input | Proposed behavior inside inventory |
|---|---|
| Click / Enter on focused item | Select and inspect only |
| Tab / Shift+Tab | Standard visible focus order across controls and regions |
| Arrow keys in item collections | Previous/next item in stable order; multi-cell item is visited once |
| Grid focus | MVP visits items in row-major anchor order, not each occupied cell; List offers equivalent traversal |
| E | Equip/unequip selected gear; ignored in text inputs or nested chooser/review |
| Q | Drink selected carried potion, following review policy |
| C | Identify target/source flow; never hidden automatic source selection |
| D | Drop review |
| Escape | Cancel chooser/review, then close inventory; never commit |
| I | Open; optionally close only from top-level inventory (Q-030) |

No double-click action, mandatory precision dragging or timed input gesture.
Right-click context menus and modifier quick-move are later conveniences,
not the only action path. Game movement bindings do not move the character while
inventory is open. Rebinding should eventually include inventory actions, not
only movement; the first UX increment can preserve existing keys.

On reopen, retain selected item if still reachable. If consumed/dropped,
select the nearest surviving item in the old region, then its empty state;
do not jump to a different region and arm an unrelated action. After pickup,
reopening ground selects the next ground item; after equip, retain the same
item now in Equipment. Focus is separate from selection; action buttons always
name the selected target. A focus outline and selected outline are distinct.
Pointer and keyboard can alternate without stealing each other's target.

## 6. Layout, readability and accessibility

- Comfortable desktop: equipment, backpack, details simultaneously visible;
  ground rail below. Aim for 1280x720 at normal UI scale, then 1920x1080.
- At larger text scales/narrower width: equipment and backpack above a full-width
  details/action region; stack sections as needed. Never shrink text/targets to
  force three columns. Scroll each content area and keep action cost reachable.
- For the browser study at narrow widths, stack all regions with normal document
  scrolling. This tests overflow, not a commitment to a mobile shipping target.
- Provisional baseline: 16 CSS/logical px body, 14 minimum secondary text,
  44x44 minimum action targets, clear 2 px focus outline, 48 px grid cells at
  comfortable width. Final Bevy font sizes need Pixelify/Silkscreen glyph review.
- Contrast target: 4.5:1 normal text, 3:1 large text and meaningful controls.
  Rarity has a word and marker; errors have a written reason. No colour-only
  improvement, unidentified, focus or invalid-placement signal.
- Critical information is persistent, not an expiring toast or hover-only tooltip.
  Tooltip fallback must be clamped to the viewport; details use bounded scrolling.
- Reduced motion: no item bounce, no mandatory transition. Selection changes
  immediately. Future drag has click-source/click-destination and keyboard paths.
- Test browser keyboard/zoom now; do not claim Bevy screen-reader support from an
  HTML study. Shipping accessibility needs testing in the actual game UI.

## 7. State and failure matrix

| State | Required message / behavior |
|---|---|
| Known potion, full HP | Explain no healing needed; no charge, no consumption |
| Unknown potion, full HP | Do not reveal type via preview/disabled state; core decides after an explicit attempt |
| Bag has cells, no fitting rectangle | `No contiguous 2x3 space for Iron mail`; do not say simply Full |
| Pickup overweight | Show current + item weight versus temporary limit; no mutation |
| Swap cannot return incumbent to bag | Name displaced item and missing space; original gear stays worn |
| No known Identify scroll | Explain absence; never pick a hidden true type for the player |
| Failed source/target access | `Item is no longer reachable`; refresh views, no substitute item |
| Unidentified candidate | Show known base comparison and uncertainty on both sides |
| Long name / many items | Wrap details; bounded collection scroll; full name on focus/select |
| Empty bag/ground/slot | Explicit empty text; no misleading action prompts |
| Focus loss / Escape during review | Cancel pending UI intent; never replay held input on return |
| Death during item action | World death state wins; item command cannot continue |

## 8. Implementation plan and seams

### UX-0 - Review the interaction study (I-033)

Deliver this document and a local dependency-free browser study. Synthetic
fixtures demonstrate equipment, true 10x6 footprints, list parity, ground pickup,
known/unknown comparison, known healing, identification, review, world return
and rejection. Include full bag, overweight, full HP and visible-threat states.
The study's numeric evaluator is **mock data**, never reusable gameplay logic.
No actual game save, RNG or assets are loaded.

### UX-1 - Read-only Bevy workspace (I-034)

Replace inventory text with real `bevy_ui` panels, labelled item cards, slot
widgets, grid/list, ground-at-feet and persistent comparison. Keep all existing
commands and action boundaries. `UiMode` retains stable IDs, not list indices.
Implement a knowledge-safe item view model for tooltips, names and action hints.
Prepare explicit source views for the B-010 correction in UX-2. Pure core APIs provide
known stats; presentation does not copy formulas from this study.

Exit: keyboard/pointer selection, responsive layout and reading all item
information cost zero time/RNG. Two unknown items with the same known projection
are indistinguishable regardless of hidden affixes/effects.

### UX-2 - Action journeys and recovery (I-035; depends on UX-1)

Wire costed action strip, source chooser, targeted review, result-driven world
return and rejection focus; fix B-010 with explicit knowledge-safe sources.
Reuse/extend core's planning boundary for typed
availability and costs; do not duplicate transfer/weight/space validation in UI.
Preview and command must share rule inputs; the command revalidates.

Exit: pick up -> equip -> heal -> identify -> drop can each be completed with
mouse **or** keyboard, without guessing keys; manual movement never leaks
through; repeated input produces at most one command. Reopening retains context.
Current game regression tests and save/restore conservation continue passing.

### Later - Packing and full-game surfaces (I-028, I-013 through I-018)

Do not add dummy live tabs for bags/belt/storage/optimizer now.
Explicit `TransferItems.cell` already supports desired bag pickup/unequip
placement, but same-bag repacking, rotation, stack splitting, locks and sorting
need core command semantics, costs and migration where applicable (Q-031).
Prove item conservation and interrupted operations before drag/drop delivery.
Then extend the same item details/actions to:

- Belt/hotbar with charges and known effects; stacks/split amounts.
- Full equipment slots and loadout choice with correct displaced-item previews.
- Local storage source/destination labels, guarded deposit/refill sequences.
- Full comparison/optimizer using the existing knowledge-safe evaluator.

Do not pull post-P5 optimizer/storage breadth into this UX improvement.

## 9. Review tasks and evidence

Use these scripts before visual polish:

| Task | Observe / pass |
|---|---|
| At 16/40 HP, heal with a known potion | Effect and 100-energy cost found without help; no accidental use on selection |
| Inspect an unknown potion and unknown sword | No hidden effect/affix leaks through details, comparison, sort or action state |
| Equip sword, then change Body with a hostile visible | Correct incumbent/cost; Body danger review; no multi-action backlog |
| Pick up gear with full bag or overweight fixture | Precise reason; no lost/duplicated item or spent action |
| Identify one candidate | Explicit known source, target and cost; type knowledge updates consistently |
| Cancel each nested mode and reopen | Escape is predictable; focus/selection preserved; world only resumes intentionally |
| Navigate keyboard-only, then use pointer | All tasks reachable; hover does not change action target |
| 1280x720, 1920x1080, 200% browser zoom/narrow layout | No horizontal document overflow; names/cost/actions readable and reachable |

Record task completion, misclicks, unexpected commits, reopening effort and
failure comprehension. Proposed usability target: zero unintended item actions
in these tasks; do not declare a timed usability benchmark without testers.

### Study verification, 2026-11-10

The dependency-free model's 11 regression tests pass, covering knowledge,
capacity, atomic ownership, cost, explicit identification and stale offers.
Chromium browser checks exercised healing/world return, pickup/equip,
identification source validation/reveal, danger review/cancel/commit, unknown
supplies, exact disabled reasons, Grid/List and empty/long-name fixtures.
Real keyboard input exercised arrows, Drop, Escape cancellation/focus, Drink
and reopening; synthetic repeated keydown was ignored.

At 1280x720 and 1920x1080 the workspace stays inside the viewport with pinned
actions and independent content scrolling. At 1280x720 not all grid rows/item
details fit simultaneously; scrolling is intentional rather than shrinking text.
A 640x360 CSS viewport tested the stacked large-zoom layout proxy, no horizontal
document overflow, reachable actions and a scrolling review dialog. This was
not actual browser-zoom, human usability, 4K, packaged Windows or Bevy testing.
Those acceptance checks remain open.
Browser/model evidence proves the study, not game integration or Windows/4K
acceptance. Confirm Q-030 after hands-on review before treating this as final UX.
