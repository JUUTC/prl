# P2 placeholder view

`PrlRenderPlugin` requires Bevy's 2D renderer and text, and `PrlCorePlugin`.
Startup must insert a successfully loaded `CoreState`; the renderer never
constructs rules or writes gameplay state. The binary composes these plugins.

Insert `PlaceholderArtSource(pack_dir)` to draw the disposable Gravebound atlas
described by `<pack_dir>/sprites/placeholder/gravebound_placeholder.ron`
(regenerate with `python3 tools/character-study/export_placeholder_atlas.py`).
Sprites are looked up by content ID from that pack manifest; adding a
definition's sprite needs no engine change. If the source is absent or the
manifest/PNG is invalid, an error is logged and procedural shapes are drawn.
Neither is final PRL art or palette, and neither completes art P1.

Map cells occupy 64 logical world units with no drawn grid. The camera follows
the player; `+`/`-` (including numpad) select integer 1–4× world zoom. UI scaling
is independent. Terrain uses core's remembered definition/door state and current
visibility; actors come exclusively from `visible_actors()`. Unknown cells stay
black. Walls, doors and actors are depth-sorted by row, so a half-height wall
hides only the lower part of an actor directly north of it.

Fallback actor shapes are chosen data-agnostically from behaviour and speed class
(no content-ID matches): player cyan square (grey at 0 HP), slow hostile green
wide block, normal hostile orange tall block, fast hostile pink diamond,
passive grey square. Damaged actors get an HP bar.

Facing and walk/attack clips are presentation-only. A separate outcome cursor
reads `ActorMoved`/`Attacked`: horizontal steps and attacks choose left/right,
vertical steps keep the last facing. Left and right sets are independently
authored; single-facing creature studies are never mirrored. Frames advance on
wall-clock time and never feed back into rules; with motion Off they hold a
still idle frame.

`CombatEffects` is an outcome consumer with its own last-seen sequence: it
processes only newer outcomes in order, never replays, warns once on a gap,
and resets on `RunStarted`. `Attacked` produces an attacker lunge, a target
flash (toward white for shapes, red for atlas sprites) and a floating number (`-6`, `CRIT -11`, `glance -2`); `ActorDied` a
fading ghost of the last drawn sprite. Only attacks involving the player or visible actors are shown.

`AnimationSpeed` (`T`: Normal ≈150 ms lunge/flash and 600 ms numbers, Fast
40% of that, Off static) is presentation-only. New outcomes snap in-flight
effects; nothing waits for animation. The test
`effects::tests::animation_speed_never_changes_core_outcomes` proves identical
core snapshots across all three speeds.
