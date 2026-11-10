# Open Questions

Canonical running decision register for design and implementation. Delivery stages
are defined in [playtest-plan.md](playtest-plan.md). Review this register at each
stage handoff; add discoveries as they arise, not just at the next interview.

## How to use the register

Keep stable `Q-NNN` IDs. **Open** needs a decision; **Provisional** has a reversible
working default, not an accepted design answer; **Blocked** prevents only the
named dependent task; **Deferred** is outside the current slice; **Resolved**
records an accepted answer and its source. Preserve resolved rows so the rationale
is not lost. New questions must name their impact and next action.

Working defaults below are planning proposals dated 2026-11-08. They are not
claims about answers in an unrecovered interview. When a decision is accepted,
update the owning system doc and retain the ID, resolution and source here.

## Active register

| ID | Question | Status | Impact / needed by | Working default and next action |
|---|---|---|---|---|
| Q-001 | World name, pre-war history and invading host? | Deferred | Campaign lore; after P5 | Keep existing Malgareth history; decide in [lore.md](lore.md) before campaign writing |
| Q-002 | Final game title? | Deferred | Branding; not a playtest blocker | Use PRL; revisit after systems/playtest feedback |
| Q-003 | Names/personalities of the other five gods? | Deferred | Cleric/Paladin content; after P5 | Keep drafts; validate before those character templates are playable |
| Q-004 | Area-level ranges, NG+ multipliers and house costs? | Open | Slice encounter tuning at P3; wider balance later | Use explicitly provisional crypt encounter data; record deaths/damage/run length; defer NG+/house values |
| Q-005 | Does counting crafting/gathering in PL over-match crafters? | Open | Multi-skill/overworld slice after P5 | Preserve all trained skills in PL; later compare combat-only and crafter-heavy characters in identical areas |
| Q-006 | Where are the latest animation/UX session decisions, and what changed? | Resolved | Gameplay/art boundary | Integrated the concurrent art session's [art-production.md](art-production.md) and [Gravebound reference](style/gravebound-p0.md); approved direction/facing/equipment contracts preserved, final art and timing still gated |
| Q-007 | Movement/attack timing, key repeat, input buffering and motion options? | Provisional | Input/presentation integration at P1/P2 | I-023/B-009 implement reversible defaults: enabled held movement/wait after 500 ms, then 180 ms; F2/F3/F4 adjust, F5/F6 remap/reset (session-only). Stop at visible hostiles, damage, HP <=25%, notable cells and modal/focus/outcome interruptions; never repeat attacks or catch up missed intervals. Walk tween 150 ms Normal / 60 ms Fast / instant Off. Confirm comfort/timing and future persisted options through hands-on feedback; no final art/UX approval |
| Q-008 | Can diagonal movement squeeze past blocked corners; do invalid actions cost time? | Provisional | Movement/pathfinding tests at P1 | No corner squeezing when either orthogonal neighbour blocks passage; invalid intents explain rejection and consume no energy; confirm and document in time-and-turns.md |
| Q-009 | How are facing/frontal shield blocks and physical mitigation ordering defined? | Provisional | Block/armor implementation at P2 | P2 working default: no shields in P2 content, so the block step and facing are not implemented; armor% uses `raw` after crit/glance and before flat armor; single-type physical only. Still needed: facing/arc rules and mixed-damage handling before shields ship; numeric cases in combat.md |
| Q-010 | What challenge probability drives combat skill gains when attacks always hit? | Blocked | Combat skill-gain integration at P4 only | Do not feed hit probability 1 into `4p(1-p)`; decide a trained-skill-versus-target challenge model and repeat window in skills-and-progression.md |
| Q-011 | Which Windows build profile/linker gives the fastest useful iteration? | Open | Distribution at P0 | Native Windows local builds and one Windows artifact target; benchmark incremental/link times and package size before enabling dynamic linking or changing linker |
| Q-012 | Checkpoint interval and crash recovery contract? | Provisional | Persistence at P4 | Working default implemented in I-030: every 20 committed player actions and floor transition, plus quit-save; confirm interval through playtest/write-cost evidence. Death uses a durable tombstone before removing the living slot, but the post-command write leaves a small hard-crash window; checkpoints may roll back progress since the last save. I-029/I-030 cover atomic storage and runtime lifecycle, not write-ahead death atomicity. Same-directory file sync then atomic replacement; Unix directory sync follows. Windows sudden-power-loss metadata durability remains unproven; do not call interrupted-temp-file tests a power-cut guarantee |
| Q-013 | Minimum Windows version/GPU and performance/readability budgets? | Open | Launch compatibility at P0; acceptance at P5 | Target Windows x86_64; test 1080p/4K and multiple UI scales; obtain tester hardware, measure latency/frame time, then set supported minimums and budgets |
| Q-014 | Final palette base, placeholder asset subset and title font? | Open | Asset integration at P0/P1; visual sign-off later | Existing 64x64/ramp palette/font decisions stand; temporary owned shapes unblock launch; verify CC0 subset/OFL files and credits before importing; record palette exceptions for placeholders |
| Q-015 | Persistent IDs and character/world/map ownership across reset and NG+? | Provisional | P3 local instance implemented; review before overworld/resets | P3 stores one run-local instance: MapIds 1/2 are floor ordinals, actor IDs allocate across both floors, item/container allocators are separate. Inactive floors persist but freeze without catch-up/respawn. This is not a world/reset descriptor; review scope and migrate before resets/NG+ ([foundation-plan.md](foundation-plan.md)) |
| Q-016 | Stable seed/domain encoding, RNG state format and canonical snapshot hash? | Provisional | Review before further domains/formats | Combat retains ChaCha8 seed/stream/word-position. Snapshot v4 adds per-run kill counters/completeness; v2 arenas/v3 crypts migrate with past kills unknown. Canonical BLAKE3 compact-RON context remains `PRL core snapshot RON hash v2` ([architecture.md](architecture.md#current-snapshot-proof-i-008-q-016)); v4 golden reviewed. Crypt layout seed v1 remains unchanged. Consumable appearance v1 hashes seed LE-u64 plus definition ID; multi-type appearance permutation remains open. These are golden-tested reversible defaults, not interview answers |
| Q-017 | How do render, audio, logs and later feats consume ordered outcomes without loss? | Provisional | Foundation F3/F5 | Sequenced core outcomes, independent consumer fan-out and bounded presentation retention; snapshot contains state, not animations; define reset, overflow and lag handling before integrating consumers |
| Q-018 | Production art lock: palette, anatomy/perspective, authoring tools, frame/timing and improvement variants? | Open | Art P0-P2, not gameplay placeholder P1 | Follow [art-production.md](art-production.md); [P0 reference package](style/gravebound-p0.md) is ready for review but not approved. Original palette is not locked/verified Lospec-derived; LibreSprite export feasibility passed; palette reuse terms and author/tool selection remain open |
| Q-019 | Where do combat numbers come from before stats and skills exist (P4)? | Provisional | P2 base/P3 equipment; replace base derivation at P4 | Actor definitions supply placeholder HP/accuracy/evasion/armor/damage/crit multiplier (150); pure `derive_stats` now adds P3 gear bonuses. Actual combat includes unknown affixes; known-only comparisons do not. No damage variance roll. Fixed item bonuses, heal 24 and temporary 60 kg hard pickup cap are slice tuning, not approved balance; replace base/carry derivation with stats/skills at P4 and tune with Q-004 |
| Q-020 | P2 enemy awareness, chase and door rules? | Provisional | P2 AI | An enemy is aware while it stands in a cell the player can see (shared FOV), remembers the player's last seen cell, chases by deterministic 8-way shortest path with the Q-008 corner rule, attacks when adjacent, waits when unaware or path-blocked, and cannot open doors. Revisit with noise/factions after P2 playtest |
| Q-021 | What does retry/new run mean before saves and character creation? | Provisional | P2/P3 death and retry | Death ends the run; only new run is accepted. Runtime RunFactory regenerates both crypt floors/supplies at previous seed + 1 and increments run number, preserving outcome sequence continuity. Failed construction rejects explicitly and preserves the old state. Guardian completion is a one-time slice outcome, not campaign victory. Replace setup/death handling with character creation/morgue at P4 |
| Q-022 | Optimizer objectives, constraints and comparison reference target? | Provisional | Gear tools after P5; `derive_stats` shape at P3 | Presets (balanced, offence, defence, resist caps, carry weight) plus custom weights; constraints for locked slots, unknown/cursed items, requirements, encumbrance and set bonuses; reference target = training dummy at current area level. Confirm presets and weights with the user before building the optimizer |
| Q-023 | How are storage tabs obtained, priced and capped, and which tab types exist? | Provisional | First town/housing slice | In-game gold or play rewards only, never real money; types General, Material, Consumable, Gear, Favorites; houses grant free tabs by tier (2/4/8/16); a per-location cap and prices to be set in balance. Confirm types, cap and costs |
| Q-024 | Do storage transfers and refills cost game time? | Provisional | First town/housing slice | Free at storage when no hostiles are in view; disallowed in hostile range; any time cost would follow time-and-turns.md |
| Q-025 | How many kits per character, and do kits pin specific items or only definitions? | Provisional | First town/housing slice | Several named kits; definition + quantity by default, with optional pinned item IDs; kit cells act as locks for Deposit all |
| Q-026 | Which storage can a crafting station draw from? | Provisional | First crafting slice | Carried inventory plus storage at the same site (house storage at a house, the bank at a town station) and an adjacent pack animal; carried first, then Material tabs, then General; materials reserved at start, consumed on completion |
| Q-027 | Loot filter presets, protections, scope and auto-pickup defaults? | Provisional | `ItemPredicate` + Default preset at P3; custom filters after P5 | Built-in read-only presets (Default, Strict, Very strict, Crafter, Show all, Manual) as content data; custom filters copied from presets, stored in the profile, chosen per character, plain-text import/export; Unique/Set/quest/wanted items cannot be hidden; auto-pickup uses normal pickup commands and time, stops when a hostile is in view or before over-encumbrance; gold auto-pickup on by default. Confirm preset list and default auto-pickup set |
| Q-028 | Is a 64×64 source cell enough fidelity for recognisable sprites? | Open | Art P0/P1 gate; distinguish logical tile size from sprite texture resolution | Keep 64×64 logical tiles (SPEC). Current procedural placeholders do not establish the quality ceiling. With no dedicated artist, compare rigged-3D bakes at 64/128 source pixels at identical in-game size (I-024), including crisp versus selectively softened edges. Higher-resolution art need not change the grid. Oversized monster/action bounds remain available; no resolution or filtering change accepted yet |
| Q-029 | Which no-dedicated-artist production pipeline and edge treatment should replace the rejected procedural art? | Open | Art P0 authoring decision, before asset-family expansion | [Art-production.md §8](art-production.md#8-artist-light-pipeline-investigation-2026-11-09) recommends an offline rigged-3D/Blender-to-sprite proof; Tripo first AI candidate, animated licensed assets as control, eligible Azure services for references/geometry. No vendor or soft-edge style approved; I-025 checks subscription eligibility, I-024 measures accepted quality, equipment feasibility and actual cost. Existing crisp-edge SPEC and separate UX gates remain in force |
| Q-030 | Inventory commitment, targeted reviews and unknown-scroll reading policy? | Resolved and UX implemented; unknown Read deferred | I-034/I-035 delivered | User interview 2026-11-10: keep inventory open after safe actions; after resolution close on any visible hostile or damage, but allow free browsing in danger. Separate persistent warnings for Drop, unknown consumables and threatened Body changes; checked "Warn me next time" disables that category when unchecked, Options re-enables it. Explicit known Identify source fixes B-010. Unknown-scroll Read, skill gain/risk-reward/discovery deferred for brainstorming; no new Read rules approved. [Contract/evidence](inventory-ux.md) |
| Q-031 | What are same-bag packing/rotation, drag, sorting and stack-split command/cost semantics? | Organization resolved and implemented; stacks deferred | I-036 implemented; remaining breadth I-028 | User interview 2026-11-10: free rearrangement/90-degree rotation even in danger; drag/drop, click-place and keyboard parity. Escape cancels, invalid placement leaves original intact. Explicit free Organize only, never automatic on pickup: deterministic best effort may rotate; all items must fit or preserve whole layout and explain. Guaranteed optimal packing not required. Implemented with atomic core commands and snapshot v5 orientation/v4 migration; stack/lock semantics remain deferred |

## Resolution history

Inventory interview, 2026-11-10 (user): keep inventory open after actions and
close automatically if danger appears. This supersedes Q-030's proposed
one-action/world-return default. After an action resolves, any visible hostile
or damage taken closes inventory; opening/browsing while threatened is allowed
and free. Remaining review/read policies are still being interviewed; update
the owning UX contract together once those answers are captured.

Inventory warning policy (same interview): confirmations must be configurable.
Warn on the first and subsequent occurrences unless the player unchecks a
checked "Warn me next time" box. Separate settings for dropping, unknown
consumables and slow Body changes in danger persist across runs and restarts;
Options can re-enable each. Do not turn this into a one-time-only warning.

Unknown-scroll reading (same interview): deferred for brainstorming, including
skill gain and risk/reward. User prefers target-first use as a starting direction;
the scroll is destroyed on use and the message describes what happened, but
the effect is not necessarily a clear identification. This is not approval of
automatic type discovery or a complete consumption/failure contract. Do not
implement new unknown-scroll reading rules in the inventory UX increment.

Packing (same interview): rearranging within the backpack is free organization,
including while enemies are visible. Equip/use/pickup/drop retain their normal
costs. Allow 90-degree rotation during placement, also free. Support drag/drop,
click-to-pick-up/click-to-place and keyboard placement/rotation through the same
rules. Escape cancels; invalid placement leaves the item at its original
position. Automatic packing may move existing items only through an explicit
Organize button; never silently on pickup. Organize is free, deterministic
best-effort packing and may rotate items. Commit only if every item fits;
otherwise preserve the entire original layout and explain the failure. No
guarantee to find every mathematically possible arrangement is required.

Q-006 resolved by integrating the concurrent art session's committed documents.
Q-007/Q-014/Q-018 remain open/provisional: prototype timing, colours and procedural
shapes are not production approvals. Diagonal collision remains Q-008.

Q-012 follow-up: confirmed abandonment now archives the report before committing
an `.abandoned` barrier; after that barrier, the old run cannot Continue.
A crash or failed barrier publication before commitment can leave a report of
the confirmed request while the old checkpoint remains continuable. This is
not an atomic history/write-ahead guarantee. Both death and abandonment barriers
are recovered before Continue; reports from both are retained if a replacement
run dies before its first successful save.


## Decided

- Big bad: Lich-Emperor with spreading undead blight.
- Blight spreads over time (difficulty-scaled), can be pushed back.
- Pantheon: required only for Clerics/Paladins.
- Art: 64×64 tiles with integer zoom (4K-ready); DCSS CC0 32×32 tiles as 2× placeholders until original art (not yet imported; licence verification pending, Q-014).
- Palette: locked ramp-based PRL palette (~24 ramps × 6–8 shades, ~150–190 colours), CI-checked. This is the target; no palette is locked yet (Q-014/Q-018).
- Fonts: Pixelify Sans + Silkscreen (OFL); title font later. Integrated in the placeholder UI (I-010); final type scale belongs to the UX effort.
- UI: bevy_ui for the game; bevy_egui only for dev tools behind a feature flag.
- Tone: grimdark-hopeful high fantasy; terse, archaic-lite prose.
- Character art baseline: Gravebound; two unlabeled silhouettes, five looks per race/silhouette, actual equipped items and gear improvements visible independently of skill-derived PL.
- Production quality: existing procedural sprites are placeholders, not shippable art. A finished character in a representative world scene must be approved before batch expansion; follow the gated [art production plan](art-production.md).
- World perspective: three-quarter overhead, hidden square grid, no visible floor grid, eight-direction movement, half-height foreground walls for character readability.
- Initial animation: independent left/right facing sets, no mirrored equipped composites; north/south retains facing, main/off hands remain assigned, dual wield and coordinated two-handed grips supported. Dedicated work, gathering, consumable and casting actions.
- Visual coverage: archery draw/release, arrow variants, spell effects, monsters/enemies, townsfolk, animals, tree-felling and ore-depletion states, crafting benches, eating and drinking.
- Atmosphere: optional subtle fog/ash parallax, no shifting interactables or terrain; preserve actor/target readability and offer reduced-motion/off controls.
- Mods/DLC: v1 data-only content packs (namespaced IDs, load order); DLC = first-party packs; WASM scripting deferred.
- Code: Cargo workspace (prl, prl_core, prl_content, prl_gen, prl_render, prl_ui, prl_audio, prl_save); strict lints (clippy all + pedantic as errors, no unsafe, no unwrap/expect outside tests, documented pub items).
- Overworld: separate scaled-down map (ADOM style).
- Meta-progression: everything earnable in Explorer mode.
- Pets: stabled in towns; only control-slot pets travel.
- Audio: basic SFX in v1, music later.
- Skill pacing: ~20+ hours of focused use to reach Master (100).
- No reputation system.
- No skill/stat decay.
- Keyboard + mouse only for v1.
- Malgareth the Undying: betrayed champion; Kaedric demanded a sacrifice he refused (secret quest line).
- Combat: always hit; evasion → glancing blows; armor flat + %; resist cap 75% (90% max) with difficulty penalty.
- Damage types: Physical + Fire, Cold, Lightning, Poison.
- Bestiary: 60–80 base monsters, ~10 families.
- Dungeons: instanced, resettable at entrance; Dungeon Sigil modifiers (risk/reward, rare pure bonus).
- Final dungeon resets grow the blight; victory → win screen → New Game+ (endless tiers, keep character/gear/bank, retempering).
- Skill gain: Hard 150%, Nightmare 250%.
- Overworld encounters: visible + random, area level vs Power Level, weak monsters avoid, repel/attraction consumables, evade prompt.
- Power Level = total trained skill points / 10.
- Inventory: Diablo grid + weight, bags; pack animals (ambush risk shown, trainable, barding).
- Housing: unlimited houses (Small/Medium/Large/Castle), clear area first, separate storage, fast travel, blight sieges.
- Quests: seeded main line + procedural side quests; light narrative.
- Light: dark dungeons, fuelled light sources, darkness aids stealth.
- Saves: auto-save on quit + overwritten crash checkpoint.
