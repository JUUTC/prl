# P2 placeholder UI

`PrlUiPlugin` requires Bevy UI/text, the built-in default font, and
`PrlCorePlugin`. The binary must insert a loaded `CoreState` before running.
No external fonts or images are used; styling is explicitly temporary.

| Action | Keys |
| --- | --- |
| Move / bump-attack in eight directions | Arrows (cardinal), numpad 1–9, vi `y k u h l b j n` |
| Wait | `.`, numpad 5, Space |
| Explicit open door | `O`, then a direction |
| New run while alive | `R` twice within 2 s, or `R` then `Y` |
| New run after death | `R` once |
| Help | F1 or `?` (`/` also opens help) |
| Pause menu / close overlay | Escape |
| Independent UI scale | `[` / `]` (0.75–2× in 0.25 steps) |
| World zoom | `+` / `-` (render plugin, integer 1–4×) |
| Motion / sound settings | `T`, `M`, `9` / `0` (render/audio plugins; shown here) |

Only newly pressed keys generate intents; a stable priority accepts at most one
per frame. Held keys do not repeat. Help/menu and focus transitions discard
gameplay edges and cancel the pending door or new-run prompt. While a new-run
confirmation is armed, any other gameplay key cancels it and is consumed. While
the player is dead, only `R` submits a command. Closing an overlay cannot
execute a simultaneous direction. The first focused frame is intentionally
ignored, preventing focus-restoration input from spending a turn.

Translucent HUD panels: header (placeholder notice, run, seed, HP, ticks,
clock, actions, motion and sound) top-left; visible hostiles with HP and speed
class top-right; the last five core messages bottom-left; a centred death
banner. Controls live in the F1 help overlay with a shape legend (B-003).

The message log is an outcome consumer: it keeps its own last-seen outcome
sequence, appends only newer messages in order, warns once if retained
outcomes have a gap, and clears on `RunStarted`. Sibling crates may not depend
on each other, so the binary copies motion/sound labels into `SettingsStatus`.
Commands run through `CoreCommandQueue` in `CoreSet::Input`; changed-state
feedback runs in `CoreSet::Presentation`. Missing state displays an explicit
startup/content-error prompt.

Inventory, save/options screens, mouse gameplay and final art/fonts are not
implemented. No menu action advances core time.
