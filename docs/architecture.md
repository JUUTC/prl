# Technical Architecture

For proposed foundation contracts and the concrete build order through P1, see
[foundation-plan.md](foundation-plan.md). It separates what must be implemented
now from the extension points reviewed before later systems are added.

## Stack

- **Rust** (edition 2024), **Bevy 0.20** (ECS, 2D sprites, `ImagePlugin::default_nearest()` and integer zoom for crisp 64×64 pixels at any resolution up to 4K).
- **World rendering:** three-quarter overhead over hidden square cells, no drawn floor grid, half-height foreground walls. Eight-direction movement stays in game logic; depth ordering and character/equipment facing belong to rendering.
- **Animation contract:** start with authored left/right poses, with anatomical main/off-hand attachments and per-facing depth order; never mirror the final equipped character. Actions specify grips, temporary props, and cosmetic effects separately from equipped items. Two-handed grips reserve both attachments. North/south travel retains facing. Action definitions must allow future facing sets; animation timing never drives rules.
- **Effects/world assets:** projectile visuals, spell travel/impact, harvest-state transitions, stations, consumables and creature/NPC animations are presentation assets. Archery separates bow grip, drawing hand, and projectile release. Resource depletion and item consumption are explicit rule events, never inferred from animation frames.
- **Atmospheric depth:** optional low-opacity fog/ash layers use differential camera offsets; interactable geometry and actors never parallax. Exclude readable actor/target regions from obscuring effects, support reduced motion/off, and keep all drift independent of game time.
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

Game rules must be **deterministic and headless-testable**. The current
`prl_core` owns logical actors/maps in a `CoreState` resource, not actor entities.
The Bevy plugin resolves explicit `Command`s (player intents); headless callers
can use the same resolver without an ECS world. Core/generation never depend on
frame time, rendering, UI or audio crates. Presentation entities map to logical
IDs at runtime and are not serialized.

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

### Current P4 save lifecycle (I-029/I-030)

`prl_save::SnapshotStore` is a headless single-slot storage API used by the
playable P4 save controller. The binary selects a per-user `PRL/character.ron`
path (`%LOCALAPPDATA%` on Windows, `$XDG_DATA_HOME` or `$HOME/.local/share` on
Linux, and `~/Library/Application Support` on macOS) and creates that directory.
`write` validates a living committed `CoreState`, encodes it and atomically
replaces that slot; `load` returns `None` only for an absent slot and reports
I/O, malformed data, incompatible versions, checksum mismatch, dead state and
invalid logical state as explicit errors. It never silently starts a new run,
deletes a corrupt save or promotes abandoned temporary files.

Pre-release file format 1 is compact RON with `format`, exact Cargo package
`engine` version, `checksum` and `snapshot` (the compact core RON as a string).
BLAKE3 context `PRL save snapshot bytes v1` hashes those snapshot UTF-8 bytes
before parsing/migration. This catches accidental corruption, not deliberate
tampering; normal core/content invariant validation still runs. Reads stop at
16 MiB plus one overflow-detection byte; oversized input/output is rejected.
The existing core v2-arena-to-v3 migration applies after checksum validation.
Engine/file incompatibility fails explicitly; there is no promise that identical
pre-release package versions imply compatibility across arbitrary code changes.
The planned bincode/zstd production encoding is not implemented: this narrow
store deliberately reuses the already tested snapshot encoding without changing
the core DTO or its canonical hash.

`tempfile` 3.27 (MIT/Apache-2.0) provides unique same-directory temporary files and
cross-platform atomic overwrite, including Windows; its OS randomness affects
only temporary names, never gameplay. Bytes are written and `sync_all` completes
before `persist` replaces the destination. Write/rename failure preserves the
old slot and drops the temporary file. Unix also synchronizes the parent directory
after replacement; failure at that stage explicitly means the new save may
already exist with durability unconfirmed. Windows metadata durability through
sudden power loss is not guaranteed. This assumes one game writer in a normal
local writable directory; it is not a multi-process save-slot coordinator.

Storage and lifecycle regressions cover overwrite, lower-floor disk restoration/return with
loot and open doors preserved, subsequent command/RNG/hash equivalence, abandoned
temporary writes, failed replacement/cleanup, corrupt/oversized files, version
rejection/migration and dead-state rejection. These are storage/API tests, not
power-cut tests or packaged Windows Continue acceptance.

The controller checkpoints after each provisional 20 committed player actions
and every floor transition, saves from Bevy's `OnAppExitSystems`, and exposes
storage errors in the HUD. Startup recovers a death barrier first, otherwise
continues a valid living slot, and only starts seed 1 when the slot is absent.
Loaded generated content is rebuilt from the installed crypt settings and seed;
mismatches fail visibly rather than replacing the save. The runtime `RunFactory`
is reinstalled from current content/configuration and is never serialized.

On observed death, the controller writes a dead-run barrier before removing the
old living slot. Startup gives that barrier precedence, builds seed + 1 through
`RunFactory`, commits the new living run and then clears the barrier. A failed
write is surfaced and the barrier remains whenever it was successfully written.
Text morgues are now preserved before removing the old living slot and before
clearing the barrier (I-031). Explorer respawn, profile and skills remain later
work.

Q-012's interval remains provisional. A hard crash can roll back up to 19
committed actions (or progress since the most recent floor transition); there
is also a small interval after death resolves in memory and before the next
update durably writes the barrier. The implementation is not a write-ahead
command protocol and does not claim crash-proof permadeath or sudden-power-loss
durability. A save failure is shown while the application remains open and
reported in logs; a failed death-barrier write cannot establish the normal
no-resurrection guarantee.

### Current terminal-run morgues (I-031/I-032)

Each save slot owns a sibling directory, for example
`PRL/character.morgues/`. Reports are UTF-8 text with a format-2 heading and
`run-<number>-seed-<seed>-<canonical-terminal-snapshot-hash>.txt` filenames.
This keeps separate slots and different terminal snapshots from overwriting
one another. The report includes engine version and snapshot fingerprint, seed,
run, pack/map/floor, clock/actions, crypt objective status, fatal attacker when
retained, equipped/backpack items using known-only labels, and recent messages.
It does not expose ground loot or unknown affixes.

The bounded outcome/message log is not a full replay. Snapshot v4 stores the
player's lifetime kill count within each run separately; only player-attributed
deaths increment it, and new runs reset it. Legacy v2/v3 migrations start counting
at migration and explicitly label earlier kills unknown. These are not
profile-wide counters. Reports label unavailable cause information and
unimplemented skills/settings rather than inventing values. Future skill/
settings/victory records and an in-game history screen remain I-032.

The death barrier is committed **first**. The report then uses the existing
bounded temporary-write/file-sync mechanism with a no-clobber publication;
Unix directory sync also covers directory creation and report publication.
Retrying an identical snapshot accepts only an identical existing report.
A conflicting/corrupt report is preserved and produces an explicit error.
On startup, a barrier left before archiving recreates the report before building
the new run; a living new-run save cannot clear the barrier until the dead run's
report is preserved. Archive failures surface through the existing save
error/HUD path and leave the barrier in place for recovery. No RNG input or
game-time cost is introduced. The existing Q-012
hard-crash and Windows power-loss limitations still apply.

For confirmed abandonment, an optional runtime `RunExitGuard` is installed by
`prl_save`, keeping filesystem dependencies out of core. The core dispatcher
builds the proposed replacement **first**, then invokes the archive boundary
before discarding the old run. Build/archive failures reject explicitly without
changing rules state; the message log explains the failure. Headless direct
commands remain deterministic without a persistence guard.

A living terminal report is followed by a checksummed `.abandoned` snapshot
barrier. Until a newer run is saved, Continue refuses the old checkpoint.
`recover_death` now handles both terminal barrier kinds; if both exist, all
reports are preserved and the newest terminal run determines recovery. The
new run is committed before barriers are cleared. Archive failures leave the
old living slot intact. A crash/failed publication before the abandonment
barrier can leave a report of the confirmed request while the old checkpoint
remains continuable; this is not atomic history or a write-ahead protocol.
Existing format-1 reports remain untouched. Snapshot migration can change
fingerprints and produce an additional report for a migrated terminal snapshot.

### Current held input and walking (I-023/B-009)

`prl_ui` owns held-action timing, interruption checks and shared remappable
movement/wait bindings. It submits at most one ordinary intent when the queue
is empty. It never writes gameplay positions, time or RNG. Exact provisional
timings and stops are in [time-and-turns.md](time-and-turns.md#current-held-movementwait-i-023).
Input settings/remapping are session-only, not character save state.

`ActorView` keeps logical destination separate from a wall-clock cosmetic base.
Walking updates that base before combat effects compose lunges; HP bars and the
camera follow its unlunged centre. Walk clips fit the traversal duration and
return to idle when it ends. New actions replace rather than queue tweens;
floor/run transitions snap, including reused logical IDs. Normal/Fast/Off
change only presentation, never occupancy, FOV, scheduling or outcomes.

### Current snapshot proof (I-008, Q-016)

The version-4 in-memory DTO round-trips through compact RON. It is not a disk
save implementation. A mid-combat snapshot is restored into a fresh headless
Bevy app with different entity allocation for logical-ID stand-ins; queued
commands produce identical RNG positions, ordered outcomes/messages and snapshots,
including continuation through new run. No authoritative actor entity remapping
is needed because rules state contains no `Entity` references.

Version 3 added the active/inactive floor registry, guardian objective,
item/container ownership, catalog/filter and seeded consumable appearance map.
Version-2 arena snapshots migrate with both new optional fields absent; a v2
snapshot claiming either field is rejected. Registry, dimensions, cross-floor
actor IDs, placement, item bounds/overlap/slots, knowledge and allocators are
validated on restore. Command-only crypt tests compare multi-floor continuations
against RON-restored state, including kills, loot, consumables and endpoint.
Version 4 adds kill counts and a completeness flag; v2/v3 migrate with zero
newly tracked kills and incomplete historical totals. Restore validates the
counter against allocated non-player IDs. Counter continuation and reset are
tested independently of retained feedback.

`prl_gen::CryptPlaytest` constructs a `RunFactory` runtime resource; the core
dispatcher uses it to rebuild geometry/content for retry at seed + 1 without a
core-to-generation dependency. Build failures report a rejection and preserve
the old run. The closure is not serialized: startup reinstalls it from the
current crypt content/configuration before loading or recovering. Direct
`CoreState::issue(NewRun)` rejects dungeon retries without this dispatcher; the
fixed arena remains usable for headless combat tests.

The provisional canonical digest is BLAKE3 with derive-key context
`PRL core snapshot RON hash v2` over the UTF-8 bytes from `CoreSnapshot::to_ron()`
(RON 0.10 compact serialization). It includes the DTO version, full validated
content, ordered logical state, allocator, visibility/memory, portable RNG
position, clock and retained feedback. Struct field order and `BTreeMap` ordering
are canonical; presentation settings, handles and ECS IDs are absent. A base-arena
golden digest detects encoding/content drift, and mutation tests cover future
rule inputs. BLAKE3 is MIT/Apache-2.0/CC0 licensed and already in the dependency
graph; core now uses it directly.
The v4 base-arena golden is
`de928b7f48b5f4e35cf5afd6ffd00b7cfd3267cf7bbbd274d443dcdc5aa3745c`.
The former v2 DTO used hash context v1; hashes are not compared across migrations.

This digest is for pre-release regression/replay comparisons, not a permanent
cross-version save checksum. Content, DTO or message changes can change it.
Review the golden fixture on such changes; version the hash context if the
encoding changes. Q-016 still tracks generation-domain encoding before more
seeded streams are introduced.

## Testing

- Unit tests for formulas (gain rules, failure %, damage).
- **Determinism tests**: same seed + same command list ⇒ same world state hash.
- Generator validation tests (connectivity, stair reachability) across many seeds.

## Code quality gates

Production asset metadata, source/export requirements, validation and approval
gates are specified in [art-production.md](art-production.md#3-asset-and-animation-contract).
The browser atelier is an isolated placeholder experiment; it is not the Bevy
asset pipeline, and its timing, palette and hardcoded item bands are not contracts.

- `rustfmt` formatting; `clippy` with `all` + `pedantic` lints as **errors**.
- No `unsafe` code.
- No `unwrap`/`expect` outside tests; errors are propagated or handled.
- Every public item has doc comments; `cargo doc` builds with warnings denied.
- CI: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, docs build, on Linux/Windows/macOS.
- Asset checks: sprites use only the locked PRL palette; every third-party asset is listed in `assets/CREDITS.md`.
