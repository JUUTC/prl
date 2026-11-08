# Technical Architecture

## Stack

- **Rust** (edition 2024), **Bevy 0.20** (ECS, 2D sprites, `ImagePlugin::default_nearest()` and integer zoom for crisp 64×64 pixels at any resolution up to 4K).
- **UI:** `bevy_ui` with a shared theme/widget module for all player-facing UI; `bevy_egui` only for dev/debug tools behind a Cargo feature.
- Planned libraries: `rand_chacha` (deterministic RNG), `noise` (overworld), `serde` + `ron` (content data, profile), `bincode` + `zstd` (saves), `bevy_ecs_tilemap` or custom chunked sprite batching for maps (decide in M1).

## Workspace layout

PRL is a **Cargo workspace** with a thin binary and focused library crates (standard Rust naming: `snake_case` crate names under `crates/`):

| Crate | Responsibility |
|---|---|
| `prl` | Thin binary: builds the Bevy `App` and wires plugins together |
| `prl_core` | Pure game logic: map, entities, scheduler, combat, skills, items. No rendering or input |
| `prl_content` | Content definitions, RON loading, content packs (mods/DLC), namespaced IDs |
| `prl_gen` | Seeded world, town and dungeon generation |
| `prl_render` | Sprites, tilemap, animation tweening, lighting, camera |
| `prl_ui` | HUD, menus, tooltips, message log, theme/widgets, key bindings → `PlayerCommand` |
| `prl_audio` | Sound effects (music later) |
| `prl_save` | Save/load, profile (meta-progression), morgue writer |

## Separation of logic and presentation

Game rules must be **deterministic and headless-testable**. `prl_core` and `prl_gen` operate on the Bevy ECS `World` but are driven by explicit `Command`s (player intents), not by frame time, and never depend on rendering, UI or audio crates.

The turn scheduler runs in a Bevy `FixedUpdate`-independent state machine:

```
AwaitingInput ──(PlayerCommand)──► ResolvingTurn ──► (ticks until player energy ≥ 100) ──► AwaitingInput
```

Animations play while in `ResolvingTurn` or after, but never gate game state (a "fast mode" skips them).

## Content as data, mods & DLC

All races, classes, skills, abilities, synergies, spells, items, affixes, monsters, recipes, dungeon themes, feats and difficulty presets are defined in RON. Code implements **effect primitives** (damage, heal, status, summon, teleport, modify stat…) that data composes.

- **Content packs (v1, data-only):** a pack is a folder of RON + assets with a manifest. The base game is itself a pack (namespace `prl`). Full rules: [standards/extensibility.md](standards/extensibility.md).
- **Namespaced IDs:** every definition has an ID like `prl:longsword` or `my_mod:frost_wolf`, so packs never collide.
- **Load order:** packs load in a defined order; later packs may add or override earlier definitions by ID.
- **DLC** ships as first-party content packs.
- **Scripting:** WASM scripting is designed-for (stable effect/event boundaries) but deferred past v1.
- The save records the active packs and their versions.

## Saves

- One save per character; **auto-save on quit** plus a **periodic checkpoint** (every N turns and on level transitions) for crash safety. The checkpoint is overwritten in place and is deleted with the save on death (permadeath). Explorer mode keeps the save.
- Save = seed + settings + NG+ tier + dungeon instance states (incl. reset counts) + overworld state (blight, houses, pack animals en route) + gameplay RNG state + entities.
- Writes are atomic (write temp file, then rename).
- Versioned format; incompatible versions refuse to load gracefully.

## Testing

- Unit tests for formulas (gain rules, failure %, damage).
- **Determinism tests**: same seed + same command list ⇒ same world state hash.
- Generator validation tests (connectivity, stair reachability) across many seeds.

## Code quality gates

- `rustfmt` formatting; `clippy` with `all` + `pedantic` lints as **errors**.
- No `unsafe` code.
- No `unwrap`/`expect` outside tests; errors are propagated or handled.
- Every public item has doc comments; `cargo doc` builds with warnings denied.
- CI: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, docs build, on Linux/Windows/macOS.
- Asset checks: sprites use only the locked PRL palette; every third-party asset is listed in `assets/CREDITS.md`.
