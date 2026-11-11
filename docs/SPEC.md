# PRL — Game Design Specification

> Status: **Draft v0.2**. This document is the source of truth for design. Sub-documents in `docs/` expand on individual systems. Items marked **[OPEN]** need a decision (see [open-questions.md](open-questions.md)).

## 1. Vision

PRL is a single-player, high-fantasy, pixel-art roguelike written in Rust. It combines:

- **Classic roguelike rules** — turn-based, grid-based, permadeath, procedural generation, deep emergent systems.
- **ADOM-style world** — a seeded overworld with towns and many dungeons, culminating in a final dungeon where the big bad waits.
- **Ultima Online-style progression** — no character levels; every skill improves by *using* it, with no skill cap. Any character can eventually master everything; what you *do* is defined by the gear you wear and the skills you've trained.
- **Diablo-style itemization** — rarity tiers, randomized affixes, uniques, sets, sockets, and identification.

### Design pillars

1. **Your choices are your build.** Class is just a starting template; gear and practice define you.
2. **Every action moves the world.** Time advances only when the player acts.
3. **Same seed, same world.** Worlds are fully reproducible from a seed + settings.
4. **Readable depth.** Complex systems, but the UI always explains numbers and outcomes.
5. **Death matters (by default).** Permadeath, with an opt-in Explorer mode.

## 2. Platform & Technology

| Item | Decision |
|---|---|
| Language | Rust (edition 2024) |
| Engine | Bevy 0.20 (ECS, 2D sprites) |
| Platforms | Desktop: Windows, macOS, Linux |
| Art | Pixel art on a **64×64 logical tile**, nearest-neighbour **integer zoom**; source sprite dimensions depend on the selected tileset |
| Display | Resolution-independent; integer world zoom, UI scale setting |
| UI | `bevy_ui` with a shared theme/widget module; `bevy_egui` for dev/debug tools only (feature-flagged) |
| Fonts | **Pixelify Sans** (UI/body) + **Silkscreen** (small labels/numbers), both OFL; title display font TBD |
| Input | Keyboard-first (arrows, numpad, vi-keys), mouse supported; no gamepad in v1 |
| Players | Single-player only |
| Content data | Data-driven (RON) content packs; the base game is itself a pack |
| Mods / DLC | v1: data-only content packs (RON + assets, namespaced IDs, load order); DLC = first-party packs; WASM scripting designed-for, deferred |
| Code | Cargo workspace of focused crates; strict lint gates |

See [architecture.md](architecture.md).

## 3. Core Loop

1. **Create character** — pick race, class, point-buy stats, world seed, world knobs, difficulty, and (unlocked) legacy perks.
2. **Explore the overworld** — travel between towns and dungeon entrances; random encounters.
3. **Delve dungeons** — fight, loot, gather, find shrines and quests; reset instances (optionally with Dungeon Sigils) to re-run them.
4. **Haul and return** — send loot home on pack animals; in town or at your houses, identify, sell, buy, craft, store, train, take quests.
5. **Grow** — skills rise through use; stats rise through skill training; gear gets better; push back the blight.
6. **Defeat the big bad** in the final dungeon to win, then retire or continue into New Game+.
7. **On death or victory** — write the morgue, award meta-progression unlocks.

## 4. Roguelike Fundamentals

PRL follows conventional roguelike rules (Berlin Interpretation as a baseline):

- **Turn-based, tick-driven time.** Nothing happens until the player acts. See [time-and-turns.md](time-and-turns.md).
- **Grid-based** movement and combat; 8-directional movement.
- **Procedural generation** from a seed. See [world-generation.md](world-generation.md).
- **Permadeath** by default; one save per character, auto-saved on quit plus an overwritten crash-safety checkpoint, deleted on death (see §12).
- **Field of view** via recursive shadowcasting; remembered tiles shown dimmed (fog of war).
- **Resource management** — food (if hunger on), potions, scrolls, light, durability, gold.
- **Hack'n'slash + exploration**, non-modal play (no time passes in menus).
- **Unknown items** — unidentified gear and randomized potion/scroll appearances per seed.
- **Message log**, look/examine, auto-explore, travel-to, rest/search until interrupted (a hostile comes into view or something notable happens).

## 5. Character

### 5.1 Stats

Six classic stats: **STR, DEX, CON, INT, WIS, CHA**.

| Stat | Primary effects |
|---|---|
| STR | Melee damage, carry capacity, heavy weapon/armor requirements, stamina |
| DEX | Accuracy, evasion, ranged damage, light weapon speed, lockpicking/traps |
| CON | Max HP, HP regen, poison/disease resistance, stamina |
| INT | Max mana, spell power (Evocation, Mysticism, Summoning, Necromancy), crafting quality, Identification |
| WIS | Mana regen, spell power (Restoration, Abjuration), perception, magic resistance |
| CHA | Shop prices, quest rewards, pet/summon control slots, NPC reactions |

**Creation — Point-buy (27 points)**: every stat starts at 8; range 8–15 before racial modifiers.

| Score | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---|---|---|---|---|---|---|---|
| Cost | 0 | 1 | 2 | 3 | 4 | 5 | 7 | 9 |

Modifier = `floor((stat − 10) / 2)`.

**Creation modes (accepted, not implemented):** independently choose Default,
Customize or Random for stats, skills and gear, or Random Character for an
unlocked race/class and all character sections without changing world settings.
Customized sections survive a class change; provide explicit reset to class
defaults. Randomized sections cannot be edited but can be rerolled freely
before confirmation. Character-roll RNG is independent of the world seed.

Default/custom stats use 27 points and 8–15 before racial modifiers. Weighted
random stats use 24–30 points and can reach 18 before racial modifiers; scores
16, 17 and 18 cost 12, 15 and 18 points respectively. Default/custom skills
use 150 points, random skills 180; each starting skill is capped at 50. Custom
gear can use the full starter catalogue within the class kit value; random
gear is class-weighted with some off-theme results and rare over-budget
jackpots without debt. Values, distributions and class allocations remain open
(Q-033/I-037); free rerolls are not an irreversible risk.

**Growth** — stats train slowly through related skill use (e.g., heavy weapons train STR), with a logarithmic soft cap and no hard cap. See [skills-and-progression.md](skills-and-progression.md#stat-training).

### 5.2 Derived values (initial formulas, tunable)

| Value | Formula |
|---|---|
| Max HP | `20 + CON × 2 + Athletics / 2` |
| Max Mana | `INT × 2 + WIS + Meditation / 2` |
| Max Stamina | `STR + DEX + CON + Athletics / 4` |
| Base speed | 100 (see time system) |
| Carry capacity | `STR × 5` kg (encumbrance tiers reduce speed); space limited by the grid inventory — see [housing-and-logistics.md](housing-and-logistics.md#inventory) |
| Accuracy / Evasion / Armor | See [combat.md](combat.md#damage-pipeline) |
| Power Level (PL) | `floor(total trained skill points / 10)` — see [combat.md](combat.md#power-level--area-level) |

### 5.3 Races & Classes

5 starting races (Human, Elf, Dwarf, Halfling, Orc) and 6 starting classes (Warrior, Rogue, Mage, Cleric, Ranger, Paladin). **Class sets only starting skills and starting equipment** — nothing is restricted afterwards. See [races-and-classes.md](races-and-classes.md).

## 6. Progression

- **No character levels, no XP.**
- **Use-based skills** across Combat, Magic, Subterfuge, Crafting, Gathering and Companions; the revised catalogue remains under interview.
- **No skill cap** — neither per-skill nor total. Growth slows logarithmically but never stops.
- **Abilities unlock at skill thresholds** (25 / 50 / 75 / 100 / 150 / 200, then minor ranks every +50).
- **Synergy abilities** unlock from combinations of skill thresholds, forming the progression "tree".
- **Gear-gated** — most abilities require appropriate equipment.
- Essential survival and main-path actions must not be skill-gated. Every build
  has reliable baseline healing and non-skill identification routes. Basic
  weapon use requires no weapon skill, though gear may have stat requirements.
  Safe practice builds basics; meaningful field use develops advanced mastery.
  Gains must reflect opportunities to train each skill (Q-035).

See [skills-and-progression.md](skills-and-progression.md) and [abilities.md](abilities.md).

## 7. Items, Loot & Crafting

Diablo-style rarity (Normal, Magic, Rare, Set, Unique, Crafted), affixes, item level, sockets, durability, curses. Unidentified items can be identified by a **town Sage**, a **Scroll of Identify**, or the optional **Identification** skill. Full crafting and gathering. See [items-and-crafting.md](items-and-crafting.md). Gear comparison, an on-demand loadout optimizer, custom loot filters with preset defaults, custom storage tabs with auto-sort, inventory refill kits and crafting from storage are in [gear-and-storage.md](gear-and-storage.md).

## 8. Magic

- **Mana pool**, no Vancian memorization.
- Spells learned permanently by **studying spellbooks**; **scrolls** are single-use casts (Inscription scribes them).
- One **Magic skill** improves casting; spell schools are descriptive categories, not individual skills. Casting is not gated by Magic skill; casting gear may have stat requirements.
- Spell failure depends on Magic, spell difficulty, INT/WIS and armor hindrance; formulas remain open.

See [abilities.md](abilities.md#spells).

## 9. Combat

- **Always hit**: no miss rolls. Evasion → glancing blows (50%), shields → blocks (−75%), armor → flat + % physical reduction, resistances → elemental reduction.
- Damage types: **Physical** (slash/pierce/blunt), **Fire, Cold, Lightning, Poison**. Holy/unholy are bonus tags, not types.
- Resist cap 75% (90% max via rare affixes); difficulty and NG+ apply a resist penalty.
- Magic damage reduction and hostile magic-effect resistance chance each cap at
  90% total, including skill, stats and gear; higher Magic Resistance skill may
  substitute for gear contributions. Exact combination and interaction with the
  general resistance rule remain open.

See [combat.md](combat.md).

## 10. World

- Seeded overworld (biomes, roads, rivers), multiple towns, multiple themed dungeons, and one **final dungeon** (deepest) containing the **big bad**.
- **World knobs**: world size, dungeon count, dungeon depth, monster density, loot abundance, hunger on/off.
- **Difficulty**: presets (Story, Easy, Normal, Hard, Nightmare) plus **Custom**.
- **Overworld encounters**: visible and random, scaled by area level vs Power Level; weak monsters avoid you; repel/attraction consumables control spawns.
- **Instanced dungeons**: resettable at the entrance with **Dungeon Sigil** modifiers. Resetting the final dungeon grows the blight.
- **Player housing** (unlimited, Small → Castle), **pack animals** hauling loot home, Diablo-style **grid inventory**. See [housing-and-logistics.md](housing-and-logistics.md).
- **Victory → New Game+** (endless tiers). See [endgame.md](endgame.md).
- **Lore**: Malgareth the Undying, a betrayed champion turned Lich-Emperor; see [lore.md](lore.md).
- **Light**: dungeons are dark; torches, lanterns and light spells burn fuel; darkness aids stealth.
- **Quests**: seeded main quest line (blight, phylactery, Kaedric's secret) plus procedural side quests from town boards and NPCs.

See [world-generation.md](world-generation.md) and [difficulty.md](difficulty.md).

## 11. Monsters & AI

- ~10 families (beasts, vermin, goblinoids, orcs/ogres, undead, demons, elementals, constructs, giants, dragons/drakes) defined in data.
- **Monster level** comes from area level (dungeon depth, distance from start, blight) + difficulty offset + NG+ tier.
- **Bestiary v1:** 60–80 base monsters across ~10 families.
- Diablo-style **Champions** (1 modifier) and **Elites/Uniques** (2–4 modifiers: *Fast, Fire-Enchanted, Teleporting, Vampiric, Stone-Skinned, Summoner, Cursed, Multishot…*) with better loot.
- AI states: *Asleep → Wandering → Investigating (noise/scent) → Hunting → Fleeing*. Smart monsters open doors and use items; packs coordinate.
- Factions: some monsters fight each other; tamed pets, undead thralls and summons are allies.
- **Stables:** tamed pets beyond your control slots (or ones you choose to leave) are kept in town stables; only pets within your control slots travel with you. Travelling pets follow you across the overworld and dungeons.

## 12. Death, Saves & Meta-Progression

- **Permadeath (default)** — auto-save on quit plus a periodic crash-safety checkpoint (overwritten; never reloadable after death). A death barrier invalidates the living save before a fresh run can be continued.
- **Explorer mode** — optional, no permadeath (respawn at last visited town with penalties). Run is flagged *Explorer*.
- **Victory** — win screen, then Retire or continue into **New Game+** with the same character.
- **Meta-progression** — specific in-game feats unlock new races, classes and legacy special abilities; all feats can be earned in any mode, including Explorer. See [meta-progression.md](meta-progression.md).
- **Morgue file** — text summary of each finished run (seed, settings, skills, gear, kills, cause of death).

## 13. UI / UX

- Main view: tile map (64×64, integer zoom), side panel (HP/Mana/Stamina, hunger, light fuel, Power Level, status effects, minimap), message log, ability hotbar (1–0).
- Screens: grid inventory (with belt, bags, pack-animal views and bank/house storage tabs), gear comparison and optimizer preview, paper-doll equipment, skills (value, title, gain indicator), abilities/spellbook, crafting, character sheet, world map, morgue/history, options.
- Tooltips show full math for items, abilities and attacks; item tooltips compare against equipped gear using the same rules as combat ([gear-and-storage.md](gear-and-storage.md#gear-comparison)).
- Movement tweening is cosmetic only and never affects game time.
- Key rebinding; colour-blind-friendly rarity markers (shape + colour).

Inventory interaction contract and staged delivery: [inventory-ux.md](inventory-ux.md)
(I-033 through I-036, Q-030/Q-031). User interview 2026-11-10: keep inventory
open after safe actions, close after resolution on any visible hostile or player
damage, and allow free browsing in danger. Drop, unknown-consumable and threatened
Body-change warnings are individually configurable across runs/restarts.
The native equipment/grid/List/ground/details workspace and action flows are
implemented in I-034/I-035, with explicit known Identify sources (B-010 fixed).
General unknown-scroll Read and its discovery/skill/risk rules remain deferred;
starting unknown scrolls are not silently usable as known Identify sources.
Backpack rearrangement and 90-degree rotation are free; explicit Organize is
deterministic best effort and atomic, never silent on pickup. These accepted
organization rules are implemented with headless commands and persisted
orientation (snapshot v5, migrating v4 items as unrotated) in I-036. Placement
supports drag/drop, click-place and keyboard; cancellation/rejection preserves
the original layout. Free organization advances no action counter, clocks,
energy, AI or RNG and never closes inventory on danger. These commands do not
add the target belt/bag/storage systems or approve final art. The earlier browser
study uses synthetic actions and predates these interview decisions.

## 14. Art & Audio Direction

Production requirements, decision status, asset contracts and dependency-ordered
quality gates are defined in [Art Specification & Production Plan](art-production.md).
Current procedural atelier sprites are placeholders and are not approved release
art. A representative in-world reuse proof requires review before expansion.

**2026-11-11 sourcing change:** pursue reuse of CDDA art and separately audited
sound assets. Animation requirements now follow the selected tileset; the
[current direction override](art-production.md#current-direction-override-2026-11-11)
supersedes conflicting custom-animation gates. Pack selection, licensing and
full-game coverage review remain open (Q-036/I-039); the bounded UltiCa crypt
subset is audited and integrated as an opt-in candidate. This does not change gameplay or
approve a replacement UI style.

The [tileset integration plan](tileset-integration.md) pins the first UltiCa
technical candidate. Subset notices, offline conversion and opt-in current-crypt
rendering and bounded knowledge-safe sword/mail icons are implemented.
Consumable artwork and Windows acceptance remain open. This is not final art
selection or approval of full-game coverage.

- **Tone:** grimdark-hopeful high fantasy — a weathered, earthy world with saturated accents for magic and the blight. Applies to art, UI and writing.
- **Character baseline:** the earlier Gravebound brief calls for two unlabeled silhouettes and five looks per race/silhouette. Whether the selected tileset can support those targets and the earlier natural-proportion style remains under review (Q-036); no substitute appearance design is approved.
- **Equipment-driven appearance:** the earlier target displays race, silhouette and equipped gear, material and improvements independently of skills. Actual tileset coverage remains to be audited; classes impose no equipment restrictions after creation. Power Level remains skill-derived, not a gear tier or automatic aura.
- **Writing voice:** terse, archaic-lite prose; no modern slang.
- **64×64 logical world tiles**, shown with integer zoom so they stay crisp up to 4K. Source sprites may use different dimensions; larger sprites may span tiles.
- **World perspective:** three-quarter overhead on a hidden square grid, not isometric. Floor art has no visible grid overlay; cardinal and diagonal movement remains eight-directional. Use half-height foreground walls to reduce character occlusion while retaining depth.
- **Character animation:** use the selected tileset's supplied frames and facings; static sprites are acceptable. Independent left/right sets are not mandatory. Keep presentation extensible without changing movement rules.
- **Equipment animation:** detailed hand/grip attachments and temporary action props are optional where supported by the tileset. Main/off-hand equipment rules remain authoritative regardless of visual coverage; presentation never changes the equipped loadout.
- **Action coverage:** dedicated idle, walk, combat, work, consumable and casting poses are optional, not release requirements. Cosmetic movement, hit feedback, projectiles and UI outcomes must retain readability. Presentation never advances or gates game time.
- **World asset coverage:** audit the selected tileset for creatures, NPCs, environments, resource states, stations, items and effects; missing fantasy coverage needs explicit decisions. Visual studies do not establish gameplay rules.
- **Atmosphere/parallax:** subtle layered fog and drifting ash may use differential camera motion for depth. Terrain, characters, structures, resource nodes, and interactables stay grid-anchored. Preserve character/gear and targeting readability; provide reduced-motion and atmosphere-off controls. These effects are presentation-only, not a side-on or isometric camera change.
- **Palette (target):** a locked, ramp-based PRL palette (~24 hue ramps × 6–8 shades, ~150–190 colours), extended from a Lospec base and checked in CI. *Status:* not yet locked. The [P0 candidate](style/gravebound-p0.md#palette-candidate) is an unapproved original proposal; the proposed seed (Resurrect 64 by Kerrie Lake) has unverified reuse terms. Do not install a palette or CI check until approved (Q-014/Q-018).
- **Fonts (target):** Pixelify Sans (UI/body) and Silkscreen (small labels/numbers), both OFL; a fantasy display font for titles will be chosen later. *Status:* the placeholder UI uses both fonts from `assets/fonts/` (credited in `assets/CREDITS.md`), falling back to Bevy's bundled Fira Mono if they are missing.
- Hi-res UI elements (item icons, portraits); lighting/shader effects, particles.
- Idle frames where supplied by the tileset (static sprites acceptable); hit feedback; per-biome tilesets; light-radius rendering.
- **Audio:** basic SFX (combat, spells, UI, footsteps, ambience loops) in v1; music in a later milestone.
- **Art sourcing:** a pinned, audited UltiCa subset is imported as the opt-in `--tileset=ultica` candidate, with CC BY-SA 3.0 art notices, original selected PNGs and per-source provenance. New runs include an outdoor approach; use fantasy-appropriate assets, not modern designs. Candidate scenery and enemies are UltiCa; the earlier animated placeholder player is the default in this mode, with the fixed UltiCa player selectable. Neither depicts dynamic equipment. No final pack or full-game appearance coverage is approved. The default remains the repository-owned Gravebound placeholder atlas with shape fallback. Selected-candidate failures reject explicitly, not via silent fallback. No DCSS art or third-party audio is imported. Track licences in `assets/CREDITS.md`.
- **Placeholder rendering:** disposable gameplay placeholders must still respect the approved perspective rules where cheap to do so (for example, no visible floor-grid seams); they do not establish art direction.

## 15. Milestones

For the first playable slice, follow [playtest-plan.md](playtest-plan.md).
Its smaller P0-P5 gates select parts of these milestones without requiring every
v1 system before hands-on testing. Track unresolved decisions in
[open-questions.md](open-questions.md).

The art production phases P0–P5 in [art-production.md](art-production.md#4-dependency-ordered-production-phases)
run alongside these gameplay milestones. Visual prototypes do not complete an
art quality gate or a gameplay milestone.

Gameplay P1 ("Walk and see") and art P1 ("Finish one reference character") are
separate gates. Disposable gameplay placeholders do not approve production art.

**Current implementation:** gameplay P1 is implemented provisionally
(Windows hands-on acceptance pending); P2 combat core rules, enemy AI, death and
new run are implemented, with placeholder presentation awaiting a hands-on check.
P3 now boots a seeded two-floor crypt with persistent return travel, a guardian
endpoint, examine mode, a 10x6 first-fit backpack, Main hand/Body equipment,
Normal/Magic gear, healing potions and Identify scrolls. Combat and known-only
comparisons share one stat evaluator; ground labels use the Default loot filter.
The [P3 subset](items-and-crafting.md#current-p3-subset) documents reversible
limits: fixed supplies/affixes, no stacks or curses, a hard carry cap before
STR-derived encumbrance, frozen inactive floors and run-local IDs (Q-015/Q-019).
These do not replace the full rules above or mark M3/M5 complete. P4 now wires
the headless atomic snapshot storage API (I-029) into automatic
Continue, quit-save, provisional 20-action/floor checkpoints and death-barrier
recovery (I-030). I-031 adds text morgues for dead runs: seed, run, elapsed
time/actions, floor, cause, carried gear and recent messages survive retry and
startup recovery. I-032 now persists player-attributed lifetime kills **within
each run** (not profile totals), including across floor travel and log eviction,
and archives confirmed abandoned runs before replacement. Snapshot v4 migrates
v2/v3 with earlier kill totals explicitly unknown, never inferred from feedback.
Skills, character/difficulty settings, campaign victory records and the history
UI remain open portions of I-032.
The interval remains provisional under Q-012; hard crashes
can lose actions since the last checkpoint, and a crash between death resolution
and the barrier write can still expose the previous checkpoint. This is not a
power-loss or hands-on packaged Windows Continue acceptance claim; Windows CI
has built and packaged the executable, while user launch acceptance remains
pending. Skills, campaign victory and general content-pack manifests remain
later work.
P2/P3 Windows hands-on/audio acceptance is still pending.
I-023 adds conservative held movement/wait with adjustable session timings and
guided key remapping; it never repeats attacks or queues a command backlog.
B-009 adds cosmetic walking interpolation, with matching camera/HP-bar follow
and Fast/Off modes. Defaults and interruption policy remain provisional under
Q-007; neither feature changes core time, occupancy or RNG.
Actors and terrain draw from the disposable Gravebound placeholder atlas; this
does not complete art P1. Art P0 is awaiting review.
Track open work and defects in [backlog.md](backlog.md).

| # | Milestone | Scope |
|---|---|---|
| M0 | Skeleton | Bevy app builds; spec written ✅ |
| M1 | Core loop | Grid map, movement, energy/tick scheduler, FOV, message log |
| M2 | Combat | Monsters, AI states, melee/ranged, death, morgue |
| M3 | Items | Inventory, equipment, rarity/affixes, identification |
| M4 | Skills | Use-based gain, thresholds, ability unlocks, stat training |
| M5 | Dungeons | Seeded generators, instancing, resets, Dungeon Sigils, light & darkness |
| M6 | Overworld & towns | Biomes, encounters, towns, shops, Sage, bank, trainers, quests |
| M6b | Housing & logistics | Grid inventory polish, pack animals, houses, sieges |
| M7 | Magic | Mana, schools, spellbooks, scrolls |
| M8 | Crafting & gathering | All crafting/gathering skills, recipes, materials |
| M9 | Companions | Taming, Necromancy thralls, Summoning |
| M10 | Setup & difficulty | Seed + knobs UI, difficulty presets & custom |
| M11 | Meta-progression | Feats, unlocks, legacy perks |
| M12 | Endgame | Final dungeon, Malgareth, victory, New Game+, balance & polish |
