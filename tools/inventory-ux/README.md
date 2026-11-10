# Inventory interaction study

Low-fidelity companion to [the UX design](../../docs/inventory-ux.md).
Synthetic items, known-only fixture comparisons and simulated action costs;
**not** the Bevy game, a gameplay evaluator or a production UI/art dependency.
No assets, accounts, remote services, saves or packages are needed.

From the repository root:

```sh
python3 -m http.server 8765 --bind 127.0.0.1
```

Open `http://127.0.0.1:8765/tools/inventory-ux/`. Serve HTTP rather than opening
the HTML file directly because the study uses native ES modules.

Run fixture/interaction-model tests:

```sh
node --test tools/inventory-ux/model.test.mjs
```

## Review journeys

- **Heal:** default situation -> selected Healing potion -> Drink -> world
  return -> reopen inventory. Selection alone must never consume the potion.
- **Compare/equip:** select a ground Magic sword -> read known comparison ->
  Pick up -> reopen -> select the sword in the bag -> Equip. It swaps with the
  incumbent only when that item fits back into the bag.
- **Identify:** picked-up unknown gear -> Identify -> explicitly choose a known
  scroll -> Commit. The newly known affix changes the fixture comparison.
- **Danger:** Hostile visible -> select worn Iron mail -> Unequip -> inspect the
  300-energy warning -> cancel. No simulated action count changes.
- **Failures:** Fragmented backpack -> ground mail; Near pickup weight limit ->
  ground sword; Full health -> Healing potion. Reasons must be different and
  no items/actions may change.
- **Knowledge:** Unidentified supplies -> unknown potion has no heal preview;
  no known Identify source is available. Drink requires review. This fixture
  eventually reveals its single potion type; it does not add new game content.
- **Input/layout:** use only Tab/Shift+Tab, arrows, Enter, Escape and the
  displayed action keys. Switch Grid/List; resize to 1280x720, 1920x1080 and
  a 640 CSS-pixel-wide view (a proxy for large browser text/zoom).
- **Empty/long names:** use the named situations to check explicit empty
  states, wrapped details and reachable actions without horizontal page overflow.

## Recorded verification

2026-11-10: all 11 Node model tests and module syntax checks pass.
Chromium interaction checks cover the journeys above, explicit source validation,
review cancellation/commitment and context retention. Real keyboard input covers
arrows, Drop, Escape, Drink and reopen; a synthetic repeated keydown is ignored.
Desktop checks at 1280x720 and 1920x1080 keep actions in view with scrolling
content panes. At 640x360 CSS pixels, sections stack with no horizontal document
overflow and actions/dialog contents remain reachable by scrolling. This is a
zoom-layout proxy, not actual browser zoom or human usability acceptance.

Known study boundaries: no drag/drop, packing edits, stacks, scrolling-world
interaction, enemy scheduling, core command validation, save persistence,
controller input, final fonts/art or Bevy accessibility claim. All fixtures reset
on reload. The study's base stats/evaluator are mock examples, never imported
into the game. Only the production core may calculate gameplay comparisons.
