# Foundation Specification and P1 Build Plan

> Proposed implementation contracts, 2026-11-08. No foundation task below is
> implemented merely because it is documented. [SPEC.md](SPEC.md) governs game
> rules; [playtest-plan.md](playtest-plan.md) defines delivery stages. This document
> details the foundation through **P1 - Walk and see** ("stage one").

## Objective

Build a launchable Windows room in P0, then movement, turns, doors, FOV, light and
HUD in P1. Establish the boundaries needed to add the rest of the design without
replacing the scheduler, map identity, content references or presentation model.

We cannot guarantee zero future rewrites while design decisions remain open.
We can prevent foreseeable structural rewrites by testing extension points early,
keeping APIs small and making uncertainty explicit. Do not implement every future
system or a universal rules engine before the first playable room.

Current foundation now includes the initial P1 code, agent instruction entry
points and a Windows packaging workflow. The fixed room demonstrates logical IDs,
validated fixture definitions, command scheduling, recursive shadowcasting,
placeholder presentation and versioned RON snapshot/restore. P2 added a portable
combat RNG stream; I-026 adds a separate headless crypt-generation stream and
rooms/corridors proof. I-027 now adds a persistent two-floor playtest registry;
I-012/I-017 add logical item/container ownership and shared stats/predicates.
P4 now adds provisional local checkpoint, quit-save, death-barrier recovery and
death/abandonment text morgues and durable per-run kill counters around the
snapshot seam. Held movement/wait and cosmetic walking now exercise the input/
presentation boundary; no presentation timer drives gameplay. These do not close
the profile, future skill/settings/victory history or Windows
launch acceptance gates. Production pack manifests and general world/reset
ownership remain later work or open contracts.

## Contracts to establish before gameplay grows

These are architecture proposals consistent with the existing standards, not
additional settled interview answers. Names below describe responsibilities;
choose concrete types at implementation review.

| Boundary | Build through P1 | Invariant / later extension |
|---|---|---|
| Content identity | Existing `ContentId`; minimal versioned base-pack manifest and tile/actor definitions | Definition IDs are not runtime instance IDs; add monsters/items/skills without matching specific IDs in Rust |
| Runtime identity | Typed `ActorId`, `MapId`, deterministic ID allocator; runtime ID-to-Entity lookup | Persistent references never store Bevy `Entity`; adding saves does not change gameplay identity |
| Run context | Seed, settings, content/version descriptor, clock and gameplay RNG state | Same inputs reproduce a run; difficulty, mods and NG+ enter explicit context, not globals |
| Map state | Dense logical grid, integer cell coordinates, separate terrain/occupancy and current map ID | Camera and pixels never decide movement; multiple maps can be added without replacing coordinates |
| Command path | Typed move/wait/door intents, validation, cost and rejection outcome | Keyboard, mouse, AI and later abilities enter rule-owned resolution; UI never writes actor position directly |
| Time | Energy scheduler, stable tie ordering and committed-action boundary | Multiple speeds/costs, status ticks and world-time work do not depend on frames |
| Outcomes | Ordered typed outcomes with sequence IDs and committed state | Render/audio/UI consume the same outcomes independently; animation cannot consume a message needed by saves or feats |
| Visibility | Core FOV/remembered terrain; explicit movement and sight-blocking properties | Lighting appearance is presentation; gameplay visibility, stealth and targeting have shared rule inputs |
| Presentation | Actor/map ID-based visual mapping, camera, shared theme and separate settings | Original art, animation timing and 4K/UI scaling can change without touching rules |
| Save seam | Explicit core-state snapshot DTO and restore proof for P1 state | Disk saves come in P4; render handles, tweens, widgets and ECS allocation order are not serialized |

### Command and turn transaction

Use this path rather than scattered mutation in `Update` systems:

```text
input / UI intent
  -> validate against authoritative state
  -> reject with reason, or resolve and charge documented action cost
  -> run eligible actors/ticks in stable order until awaiting player input
  -> publish ordered outcomes and expose a coherent committed state
  -> presentation/log consumers; later checkpoint and feat consumers
```

P1 implements only move, wait and door interaction. Add attack, equip, consume,
craft and cast when their slices arrive; do not implement placeholder handlers
that silently succeed. Browsing UI is not a gameplay command.

Validation precedes energy/RNG/state mutation. Apply Q-008's provisional invalid
action/corner rules explicitly; a future accepted rule can change one validator
and its tests. The scheduler contract must cover expensive actions, carried
energy, ties, multiple NPC actions and no-time menu activity.

Presentation outcomes are data, not callbacks into widgets. They carry logical
IDs and relevant resolved values; consumers must not recompute damage or reroll
RNG. Do not rely solely on short-lived Bevy messages for durable gameplay history.
Retain a bounded, sequenced presentation queue and reset it on loading/new run;
authoritative state/snapshots remain sufficient without replaying that queue.
Consumer lifecycle/retention is tracked in Q-017.

### Maps and time beyond the first room

P1 needs one active map, not a complete overworld. Use `MapId` and logical
coordinates from the outset; map dimensions and tile scale are data/settings,
not constants embedded across rules and UI.

Distinguish walking blockage from sight blockage (a window or smoke need not
behave like a wall). Occupants are actors/objects, not baked into terrain cells.
Remembered terrain is keyed by map and viewer; do not expose unseen live actors
through the renderer.

Keep world elapsed time distinct from render time and action counters. Local
ticks use the spec's six-second scale. Future overworld travel advances the same
world clock in larger increments; it does not fake thousands of rendered steps.
Define an elapsed-time input for later off-screen catch-up; do not build the
blight, trips or siege simulation now. Checkpoint counters count committed player
actions, not clock ticks.

At P3, extend map identity with world/dungeon/reset-count/floor ownership and
persist visited instances; do not make a scene transition erase game-state maps.
At NG+, keep character-owned state distinct from world-owned state. The exact
scope and ID rules are Q-015. The current bounded P3 implementation stores
one local instance: MapIds 1/2 are floor ordinals, actors allocate monotonically
across both floors, and items/containers have their own run-local allocators.
Inactive terrain, memory, actors and ground ownership persist without simulation
or respawning. No claim is made that these ordinals identify future
world/dungeon/reset instances; that expansion needs a new reviewed identity
descriptor and snapshot migration.

### Reproducibility and save readiness

Use explicit domain-separated seeded streams for generation; a separate gameplay
stream is part of the snapshot. Document the hash/encoding and RNG/version choice
in Q-016. Avoid unspecified language hashes, ECS iteration order, wall-clock time
and random UUIDs for rules. IDs and allocator state must round-trip too.

P1's snapshot test must serialize/restore supported state into a new headless world
with different ECS entity allocation, then check the next command's outcomes and
canonical state hash. This is a narrow compatibility proof, not a production
save file or a promise to preserve every pre-release format. **Current status:**
I-008 is implemented: a mid-combat snapshot continues through the command plugin
in a fresh headless app with different logical-ID entity allocations; outcomes,
RNG positions, snapshots and canonical hashes match. Core actors are logical
records in a resource, so entity stand-ins are not authoritative state.
The provisional hash is domain-separated BLAKE3 over versioned compact RON,
with a base-arena golden fixture; see [architecture.md](architecture.md#current-snapshot-proof-i-008-q-016).
Q-016 remains provisional for future stream/domain encoding.

P4 adds atomic file writes, recovery and death invalidation around that boundary.
Version snapshots from their first use; incompatible state must fail explicitly.
Do not serialize the full Bevy `World` or duplicate authoritative gameplay state
in the renderer.

## How the full design fits

This traceability table is the review checklist, not a requirement to code each
system before P1.

| Planned design | Foundation it reuses | Add later / compatibility proof |
|---|---|---|
| Races, classes, stats, skills, PL | Definition IDs, actor state, fixed-point rules, typed outcomes | P2/P4 attach stats/skills; class supplies starting data, never a locked actor subtype |
| Combat, ranged, magic, status effects | Commands, scheduler, target IDs, outcome order | P2 damage; later effects/tick hooks in core; no ability logic in hotbar widgets |
| Items, grid inventory, affixes, crafting | Definition vs instance IDs, cost/validation, actor-owned state | P3 adds `ItemId`/`ContainerId` and atomic transfers; equipment remains separate from bag grid coordinates. Combat reads stats from one pure `derive_stats(actor, loadout, context)` so gear comparison and the optimizer reuse it; loot filters, auto-sort and optimizer constraints share one `ItemPredicate` rule type ([gear-and-storage.md](gear-and-storage.md#foundation-contracts)) |
| Banks, house storage, pack cargo | Container ownership and map/world identity | Shared transfer rules with per-container size/weight; don't assume all storage belongs to the player backpack. Storage locations hold ordered, named tabs with data-driven sort rules; kits and crafting query accessible containers by site |
| Overworld, towns, instanced dungeons, resets | Map identity, dense grids, generation domains, world clock | P3 map registry; later overworld/coarse simulation; reset only the selected instance scope |
| Pets, summons, undead, escorts | Actor IDs, scheduler eligibility, relationships | Add ownership/faction/control-slot data; NPC scheduler must not assume every non-player is hostile |
| Housing, trips, sieges, blight | World-owned objects and elapsed time | Add coarse world-time jobs; inactive maps don't need active sprites or per-frame actor simulation |
| Sigils, difficulty, world knobs | Run context, content registries, generic rule parameters | Modify generation/rules from data; include descriptor in determinism and save checks |
| Death, Explorer, meta feats, NG+ | Outcome stream, snapshots, character/world separation | P4 persistence; profile remains outside run state; separate death handling and campaign victory from slice completion |
| Animation, light, SFX, accessibility | Read-only state/outcomes, theme, independent presentation settings | Add sprites/effects/widgets without changing command results; replay comparison with motion/audio off |
| Mods, DLC, future WASM | Namespaced registries, pack versions, effect boundaries | Start with base pack; later patches and validation; no scripting runtime or general expression VM in P1 |

Before introducing P3 containers or P4 profile/world carryover, review their own
data model. Those choices can be additive because P1 has not hard-coded item
ownership, player-only actor logic or single-world IDs.

## Ordered implementation work through P1

Each row is a small implementation unit with an observable gate. The table
defines target contracts, not automatic completion: the initial P1 implementation
proves movement/presentation and logical snapshot continuity, but F0/F7 Windows
acceptance and F1's full manifest/seed descriptor remain incomplete. Run independent
work in parallel only when it does not invent an unsettled shared API.

| Task | Depends on | Owner / output | Acceptance evidence |
|---|---|---|---|
| F0 - Build/quality baseline | None | Existing manifests; Windows artifact workflow; headless test command; agent rules pointing to standards | Fresh ZIP launch; lint/format/docs gates really execute; targeted core test excludes render/window/audio features |
| F1 - IDs, context and minimal content | None; Q-015/Q-016 review | `prl_content`, `prl_core`: typed runtime IDs, seed/settings descriptor, pack manifest/tile/actor loader | Malformed required content errors identify file/ID; same fixture allocates the same IDs; content identity/version is recorded |
| F2 - Logical room and snapshot seam | F1 | `prl_core`: terrain, actor occupancy, map reference, core-state DTO | Bounds/sight/walk tests; snapshot round trip remaps Entities but preserves logical references |
| F3 - Authoritative command/scheduler | F2; Q-008 default documented | `prl_core`: validator, energy/cost resolution, clock, ordered outcomes | Speed/cost/tie tests; rejected action leaves rules/RNG unchanged; repeated command stream yields equal state hash |
| F4 - Camera and theme shell (P0) | F0, F2 | `prl_render`, `prl_ui`: room/player view, boot errors, UI/world scale | Packaged Windows room displays; resize/zoom independent; no render-to-rule writes |
| F5 - Input and feedback | F3, F4; Q-007 default documented | `prl_ui`, `prl_render`: bindings, focus/menu handling, move/wait/door feedback | One accepted intent gives intended turn effects; no stale buffered move after menu/focus change; instant vs tween replay equality |
| F6 - FOV/light/HUD (P1) | F3, F4 | `prl_core`, `prl_ui`, `prl_render`: sight, memory, stats/clock/log | Doors change FOV; unseen actors stay hidden; idle/menu test unchanged after 60 seconds |
| F7 - Foundation/extension review | F5, F6 | Headless fixtures, packaged Windows checklist, question updates | All gates below pass and unresolved decisions have specific dependent tasks |

F4 closes P0 after its package smoke test. F7 closes P1. F1 starts the minimal
content path earlier than P3 in the general playtest plan: this deliberately
avoids building a hard-coded room format that must be replaced with a pack loader.
Only tile/actor definitions are required; the remaining content schema is added
when used.

## Foundation acceptance and change discipline

Before approving P1:

1. Restore a snapshot into a fresh ECS world; issue the same next command and
   compare outcomes, RNG and logical state.
2. Run two fixture maps with different dimensions/IDs without changing movement,
   input or render code. Verify a changed tile definition's sight/walk properties
   affect the shared rule queries.
3. Add a second scheduled actor in headless tests without adding player-only
   assumptions; exercise all specified speeds/action costs and stable ties.
4. Swap placeholder visuals/theme, zoom and animation speed while replaying
   commands; core hashes remain equal.
5. Pause at input, browse UI and lose focus; rules don't advance. Missing content
   and invalid intents have explicit feedback, not success-shaped fallbacks.
   Packaged launches require adjacent assets in every build profile; only
   explicit `--dev-assets` allows checkout assets when no adjacent assets exist
   (B-006). An incomplete adjacent package never falls back.
6. Launch the ZIP outside the source checkout on Windows. Record build identity,
   hardware, check results and unresolved questions in the playtest stage record.

Review future feature changes against the traceability table. Changes to identity,
clock semantics, snapshot shape or the command contract need their associated
tests and docs updated in the same change. Do not freeze APIs prematurely:
prefer private modules and narrowly exported types to a sprawling plugin ABI.

Do **not** build a universal container/effect framework, full mod patch engine,
WASM layer, off-screen simulation, full save system or speculative new crates
before P1. Leave those responsibilities in the existing crates and implement
their minimum proven requirements when each slice reaches them.

## Decisions that can affect this foundation

Track answers in [open-questions.md](open-questions.md), not a second question list:

- Q-006/Q-007: recovered UX choices and provisional input/presentation policy.
- Q-008: movement/corner and rejected-action semantics.
- Q-011/Q-013/Q-014: Windows delivery, target hardware and asset sourcing.
- Q-015: persistent identity and character/world/map ownership.
- Q-016: reproducible streams, seed encoding and snapshot representation.
- Q-017: ordered outcome fan-out, retention and consumer lifecycle.

Q-009 (combat) and Q-010 (combat skill gain) still need answers for their later
tasks, but do not block building the movement foundation.
