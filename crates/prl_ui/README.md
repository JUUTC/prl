# P1 placeholder UI

`PrlUiPlugin` requires Bevy UI/text, the built-in default font, and
`PrlCorePlugin`. The binary must insert a loaded `CoreState` before running.
No external fonts or images are used; styling is explicitly temporary.

| Action | Keys |
| --- | --- |
| Eight directions | Arrows (cardinal), numpad 1–9, vi `y k u h l b j n` |
| Wait | `.`, numpad 5, Space |
| Explicit open door | `O`, then a direction |
| Reset fixture | `R` |
| Help | F1 or `?` (`/` also opens help) |
| Pause menu / close overlay | Escape |
| Independent UI scale | `[` / `]` (0.75–2× in 0.25 steps) |
| World zoom | `+` / `-` (render plugin, integer 1–4×) |

Only newly pressed keys generate intents; a stable priority accepts at most one
per frame. Held keys do not repeat. Help/menu and focus transitions discard
gameplay edges and cancel the pending door prompt. Closing an overlay cannot
execute a simultaneous direction. The first focused frame is intentionally
ignored, preventing focus-restoration input from spending a turn.

The HUD shows HP/stamina, seed, core ticks/elapsed seconds/action count and the
last five sequenced core messages. Commands run through `CoreCommandQueue` in
`CoreSet::Input`; changed-state feedback runs in `CoreSet::Presentation`.
Missing state displays an explicit startup/content-error prompt.

Combat, inventory, save/options screens, mouse gameplay and final art/fonts are
not implemented. No menu action advances core time.
