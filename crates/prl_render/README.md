# P2 placeholder view

`PrlRenderPlugin` requires Bevy's 2D renderer and text, and `PrlCorePlugin`.
Startup must insert a successfully loaded `CoreState`; the renderer never
constructs rules or writes gameplay state. The binary composes these plugins.

All visuals are repository-owned procedural shapes, not final PRL assets or
palette. Map cells occupy 64 logical world units. The camera follows the player;
`+`/`-` (including numpad) select integer 1–4× world zoom. UI scaling is independent.
Terrain uses core's remembered definition/door state and current visibility;
actors come exclusively from `visible_actors()`. Unknown cells stay black.

Actor shapes are chosen data-agnostically from behaviour and speed class
(no content-ID matches): player cyan square (grey at 0 HP), slow hostile green
wide block, normal hostile orange tall block, fast hostile pink diamond,
passive grey square. Damaged actors get an HP bar.

`CombatEffects` is an outcome consumer with its own last-seen sequence: it
processes only newer outcomes in order, never replays, warns once on a gap,
and resets on `RunStarted`. `Attacked` produces an attacker lunge, a target
flash and a floating number (`-6`, `CRIT -11`, `glance -2`); `ActorDied` a
fading ghost. Only attacks involving the player or visible actors are shown.

`AnimationSpeed` (`T`: Normal ≈150 ms lunge/flash and 600 ms numbers, Fast
40% of that, Off static) is presentation-only. New outcomes snap in-flight
effects; nothing waits for animation. The test
`effects::tests::animation_speed_never_changes_core_outcomes` proves identical
core snapshots across all three speeds.
