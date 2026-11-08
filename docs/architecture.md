# Technical Architecture

## Stack

- **Rust** (edition 2024), **Bevy 0.20** (ECS, 2D sprites, `ImagePlugin::default_nearest()` for crisp pixels).
- Planned crates: `rand_chacha` (deterministic RNG), `noise` (overworld), `serde` + `ron` (content data, profile), `bincode` + `zstd` (saves), `bevy_ecs_tilemap` or custom chunked sprite batching for maps (decide in M1).

## Separation of logic and presentation

Game rules must be **deterministic and headless-testable**:

- `src/core/` — pure game logic (map, entities, scheduler, combat, skills, items, generation). No rendering or input. Operates on Bevy ECS `World` but is driven by explicit `Command`s (player intents), not by frame time.
- `src/render/` — sprites, tilemap, animation tweening, lighting, camera.
- `src/ui/` — HUD, menus, tooltips, message log.
- `src/input/` — key bindings → `PlayerCommand`.
- `src/data/` — loading RON content definitions (`assets/data/*.ron`).
- `src/save/` — save/load, profile (meta-progression), morgue writer.

The turn scheduler runs in a Bevy `FixedUpdate`-independent state machine:

```
AwaitingInput ──(PlayerCommand)──► ResolvingTurn ──► (ticks until player energy ≥ 100) ──► AwaitingInput
```

Animations play while in `ResolvingTurn` or after, but never gate game state (a "fast mode" skips them).

## Content as data

All races, classes, skills, abilities, synergies, spells, items, affixes, monsters, recipes, dungeons themes, feats and difficulty presets are defined in RON under `assets/data/`. Code implements **effect primitives** (damage, heal, status, summon, teleport, modify stat…) that data composes.

## Saves

- One save per character; written on quit, deleted on death (permadeath). Explorer mode keeps the save.
- Save = seed + settings + generated-level states + gameplay RNG state + entities.
- Versioned format; incompatible versions refuse to load gracefully.

## Testing

- Unit tests for formulas (gain rules, failure %, damage).
- **Determinism tests**: same seed + same command list ⇒ same world state hash.
- Generator validation tests (connectivity, stair reachability) across many seeds.
- CI: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` on Linux/Windows/macOS.
