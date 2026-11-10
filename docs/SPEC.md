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
| Art | Pixel art, **64×64** tiles, nearest-neighbour **integer zoom** (4K-ready) |
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
| INT | Max mana, spell power (Evocation, Mysticism, Summoning, Necromancy), crafting quality, Item Lore |
| WIS | Mana regen, spell power (Restoration, Abjuration), perception, magic resistance |
| CHA | Shop prices, quest rewards, pet/summon control slots, NPC reactions |

**Creation — Point-buy (27 points)**: every stat starts at 8; range 8–15 before racial modifiers.

| Score | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---|---|---|---|---|---|---|---|
| Cost | 0 | 1 | 2 | 3 | 4 | 5 | 7 | 9 |

Modifier = `floor((stat − 10) / 2)`.

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
- **Use-based skills** — ~40 skills across Combat, Magic, Subterfuge, Crafting, Gathering, Companions.
- **No skill cap** — neither per-skill nor total. Growth slows logarithmically but never stops.
- **Abilities unlock at skill thresholds** (25 / 50 / 75 / 100 / 150 / 200, then minor ranks every +50).
- **Synergy abilities** unlock from combinations of skill thresholds, forming the progression "tree".
- **Gear-gated** — most abilities require appropriate equipment.

See [skills-and-progression.md](skills-and-progression.md) and [abilities.md](abilities.md).

## 7. Items, Loot & Crafting

Diablo-style rarity (Normal, Magic, Rare, Set, Unique, Crafted), affixes, item level, sockets, durability, curses. Unidentified items are identified by a **town Sage**, a **Scroll of Identify**, or the **Item Lore** skill. Full crafting and gathering. See [items-and-crafting.md](items-and-crafting.md). Gear comparison, an on-demand loadout optimizer, custom loot filters with preset defaults, custom storage tabs with auto-sort, inventory refill kits and crafting from storage are in [gear-and-storage.md](gear-and-storage.md).

## 8. Magic

- **Mana pool**, no Vancian memorization.
- Spells learned permanently by **studying spellbooks**; **scrolls** are single-use casts (Inscription scribes them).
- **Spell schools are skills**: Evocation, Abjuration, Restoration, Mysticism, Necromancy, Summoning.
- Spell failure depends on school skill vs. spell circle, INT/WIS, and armor hindrance.

See [abilities.md](abilities.md#spells).

## 9. Combat

- **Always hit**: no miss rolls. Evasion → glancing blows (50%), shields → blocks (−75%), armor → flat + % physical reduction, resistances → elemental reduction.
- Damage types: **Physical** (slash/pierce/blunt), **Fire, Cold, Lightning, Poison**. Holy/unholy are bonus tags, not types.
- Resist cap 75% (90% max via rare affixes); difficulty and NG+ apply a resist penalty.

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

## 14. Art & Audio Direction

Production requirements, decision status, asset contracts and dependency-ordered
quality gates are defined in [Art Specification & Production Plan](art-production.md).
**Gravebound is an approved direction, not approved artwork.** Current procedural
atelier sprites are placeholders and are not acceptable for release. Production
expansion requires explicit approval of a finished in-world reference.

- **Tone:** grimdark-hopeful high fantasy — a weathered, earthy world with saturated accents for magic and the blight. Applies to art, UI and writing.
- **Character baseline: Gravebound** — sunken faces, natural tapered proportions, ragged charcoal layers, and worn materials; avoid toy-like heads and squared limbs. Two visually selectable body silhouettes, without gender labels, with at least five looks per race and silhouette.
- **Equipment-driven appearance:** characters initially show their race, silhouette, and starting class equipment. Equipped items, material, and item-level improvements change the visible gear independently of the underlying appearance. Classes impose no visual equipment restrictions after creation. Power Level remains skill-derived, not a gear tier or automatic aura.
- **Writing voice:** terse, archaic-lite prose; no modern slang.
- **64×64** pixel-art world tiles, shown with integer zoom so they stay crisp up to 4K. Large monsters and bosses may use bigger multi-tile sprites.
- **World perspective:** three-quarter overhead on a hidden square grid, not isometric. Floor art has no visible grid overlay; cardinal and diagonal movement remains eight-directional. Use half-height foreground walls to reduce character occlusion while retaining depth.
- **Initial character animation:** two independently authored three-quarter facing sets (left/right), not mirrored composites. East/west and diagonal movement select facing from the horizontal component; north/south preserves last facing. Keep the asset/action schema extensible to additional facings without changing movement rules.
- **Hand-aware equipment animation:** main/off-hand items retain anatomical assignment across facings. Dual wielding has two attachments; two-handed weapons and tools coordinate both grips and reserve the off hand. Action props temporarily stow held gear without modifying the equipped loadout.
- **Action coverage:** idle, walk, melee attack, archery draw/release, crafting, mining, tree chopping, gathering, eating, drinking, healing, and casting require dedicated action poses. Different crafting disciplines and weapon-specific actions can extend those families. Presentation never advances or gates game time.
- **World asset coverage:** spell cast/travel/impact effects, distinct arrow appearances, monsters/enemies, townsfolk, animals (including pack animals), standing/falling/felled trees, intact/depleted ore, crafting stations, and food/drink props share the Gravebound direction. Visual arrow studies do not establish new damage or affix rules.
- **Atmosphere/parallax:** subtle layered fog and drifting ash may use differential camera motion for depth. Terrain, characters, structures, resource nodes, and interactables stay grid-anchored. Preserve character/gear and targeting readability; provide reduced-motion and atmosphere-off controls. These effects are presentation-only, not a side-on or isometric camera change.
- **Palette (target):** a locked, ramp-based PRL palette (~24 hue ramps × 6–8 shades, ~150–190 colours), extended from a Lospec base and checked in CI. *Status:* not yet locked. The [P0 candidate](style/gravebound-p0.md#palette-candidate) is an unapproved original proposal; the proposed seed (Resurrect 64 by Kerrie Lake) has unverified reuse terms. Do not install a palette or CI check until approved (Q-014/Q-018).
- **Fonts (target):** Pixelify Sans (UI/body) and Silkscreen (small labels/numbers), both OFL; a fantasy display font for titles will be chosen later. *Status:* the placeholder UI uses both fonts from `assets/fonts/` (credited in `assets/CREDITS.md`), falling back to Bevy's bundled Fira Mono if they are missing.
- Hi-res UI elements (item icons, portraits); lighting/shader effects, particles.
- Idle animations 2–4 frames; hit flashes; per-biome tilesets; light-radius rendering.
- **Audio:** basic SFX (combat, spells, UI, footsteps, ambience loops) in v1; music in a later milestone.
- **Art sourcing:** placeholder art may use the CC0 subset of the Dungeon Crawl Stone Soup 32×32 tiles, shown at 2× until original 64×64 art replaces it, once the subset's licence is verified. *Status:* no DCSS tiles are imported; gameplay builds draw a generated atlas of the repository-owned Gravebound procedural studies (disposable placeholders, `assets/packs/prl/sprites/placeholder/`) and fall back to coloured shapes if it is missing. Every third-party asset's licence is tracked in `assets/CREDITS.md`.
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
startup recovery. Lifetime kill totals, skills, selectable settings, abandoned
run records and the history UI are not yet implemented (I-032).
The interval remains provisional under Q-012; hard crashes
can lose actions since the last checkpoint, and a crash between death resolution
and the barrier write can still expose the previous checkpoint. This is not a
power-loss or hands-on packaged Windows Continue acceptance claim; Windows CI
has built and packaged the executable, while user launch acceptance remains
pending. Skills, campaign victory and general content-pack manifests remain
later work.
P2/P3 Windows hands-on/audio acceptance is still pending.
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
