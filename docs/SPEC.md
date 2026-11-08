# PRL — Game Design Specification

> Status: **Draft v0.1**. This document is the source of truth for design. Sub-documents in `docs/` expand on individual systems. Items marked **[OPEN]** need a decision (see [open-questions.md](open-questions.md)).

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
| Art | Pixel art, **32×32** tiles, nearest-neighbour scaling |
| Input | Keyboard-first (arrows, numpad, vi-keys), mouse supported; no gamepad in v1 |
| Players | Single-player only |
| Content data | Data-driven (RON files under `assets/data/`) |

See [architecture.md](architecture.md).

## 3. Core Loop

1. **Create character** — pick race, class, point-buy stats, world seed, world knobs, difficulty, and (unlocked) legacy perks.
2. **Explore the overworld** — travel between towns and dungeon entrances; random encounters.
3. **Delve dungeons** — fight, loot, gather, find shrines and quests.
4. **Return to town** — identify, sell, buy, craft, bank, train, take quests.
5. **Grow** — skills rise through use; stats rise through skill training; gear gets better.
6. **Defeat the big bad** in the deepest dungeon to win.
7. **On death or victory** — record the run, award meta-progression unlocks.

## 4. Roguelike Fundamentals

PRL follows conventional roguelike rules (Berlin Interpretation as a baseline):

- **Turn-based, tick-driven time.** Nothing happens until the player acts. See [time-and-turns.md](time-and-turns.md).
- **Grid-based** movement and combat; 8-directional movement.
- **Procedural generation** from a seed. See [world-generation.md](world-generation.md).
- **Permadeath** by default; one save per character, saved only on quit, deleted on death.
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
| Carry capacity | `STR × 5` kg (encumbrance tiers reduce speed) |
| Accuracy | `DEX mod + weapon skill / 4 + Tactics / 8 + gear` |
| Evasion | `DEX mod + Athletics / 8 + gear − armor hindrance` |
| Armor | sum of armor pieces × armor-skill multiplier |

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

Diablo-style rarity (Normal, Magic, Rare, Set, Unique, Crafted), affixes, item level, sockets, durability, curses. Unidentified items are identified by a **town Sage**, a **Scroll of Identify**, or the **Item Lore** skill. Full crafting and gathering. See [items-and-crafting.md](items-and-crafting.md).

## 8. Magic

- **Mana pool**, no Vancian memorization.
- Spells learned permanently by **studying spellbooks**; **scrolls** are single-use casts (Inscription scribes them).
- **Spell schools are skills**: Evocation, Abjuration, Restoration, Mysticism, Necromancy, Summoning.
- Spell failure depends on school skill vs. spell circle, INT/WIS, and armor hindrance.

See [abilities.md](abilities.md#spells).

## 9. World

- Seeded overworld (biomes, roads, rivers), multiple towns, multiple themed dungeons, and one **final dungeon** (deepest) containing the **big bad**.
- **World knobs**: world size, dungeon count, dungeon depth, monster density, loot abundance, hunger on/off.
- **Difficulty**: presets (Story, Easy, Normal, Hard, Nightmare) plus **Custom**.

See [world-generation.md](world-generation.md) and [difficulty.md](difficulty.md).

## 10. Monsters & AI

- Families (beasts, undead, demons, goblinoids, giants, dragons, elementals, constructs…) defined in data.
- **Danger level** scales with dungeon depth and distance from the starting town.
- Diablo-style **Champions** (1 modifier) and **Elites/Uniques** (2–4 modifiers: *Fast, Fire-Enchanted, Teleporting, Vampiric, Stone-Skinned, Summoner, Cursed, Multishot…*) with better loot.
- AI states: *Asleep → Wandering → Investigating (noise/scent) → Hunting → Fleeing*. Smart monsters open doors and use items; packs coordinate.
- Factions: some monsters fight each other; tamed pets, undead thralls and summons are allies.
- **Stables:** tamed pets beyond your control slots (or ones you choose to leave) are kept in town stables; only pets within your control slots travel with you. Travelling pets follow you across the overworld and dungeons.

## 11. Death, Saves & Meta-Progression

- **Permadeath (default)** — save on quit only; save deleted on death.
- **Explorer mode** — optional, no permadeath (respawn at last visited town with penalties). Run is flagged *Explorer*.
- **Meta-progression** — specific in-game feats unlock new races, classes and legacy special abilities; all feats can be earned in any mode, including Explorer. See [meta-progression.md](meta-progression.md).
- **Morgue file** — text summary of each finished run (seed, settings, skills, gear, kills, cause of death).

## 12. UI / UX

- Main view: tile map (32×32, integer-scaled), side panel (HP/Mana/Stamina, hunger, status effects, minimap), message log, ability hotbar (1–0).
- Screens: inventory, paper-doll equipment, skills (value, title, gain indicator), abilities/spellbook, crafting, character sheet, world map, morgue/history, options.
- Tooltips show full math for items, abilities and attacks.
- Movement tweening is cosmetic only and never affects game time.
- Key rebinding; colour-blind-friendly rarity markers (shape + colour).

## 13. Art & Audio Direction

- 32×32 pixel art; restricted master palette (~64 colours) for cohesion.
- Idle animations 2–4 frames; hit flashes; per-biome tilesets; light-radius rendering.
- **Audio:** basic SFX (combat, spells, UI, footsteps, ambience loops) in v1; music in a later milestone.
- **Art sourcing:** start with an open-licensed 32×32 tileset as placeholder (licence tracked in `assets/CREDITS.md`), replaced by original art later.

## 14. Milestones

| # | Milestone | Scope |
|---|---|---|
| M0 | Skeleton | Bevy app builds; spec written ✅ |
| M1 | Core loop | Grid map, movement, energy/tick scheduler, FOV, message log |
| M2 | Combat | Monsters, AI states, melee/ranged, death, morgue |
| M3 | Items | Inventory, equipment, rarity/affixes, identification |
| M4 | Skills | Use-based gain, thresholds, ability unlocks, stat training |
| M5 | Dungeons | Seeded generators (caves, crypts, mines, ruins, towers) |
| M6 | Overworld & towns | Biomes, towns, shops, Sage, bank, trainers, quests |
| M7 | Magic | Mana, schools, spellbooks, scrolls |
| M8 | Crafting & gathering | All crafting/gathering skills, recipes, materials |
| M9 | Companions | Taming, Necromancy thralls, Summoning |
| M10 | Setup & difficulty | Seed + knobs UI, difficulty presets & custom |
| M11 | Meta-progression | Feats, unlocks, legacy perks |
| M12 | Endgame | Final dungeon, big bad, victory, balance & polish |
