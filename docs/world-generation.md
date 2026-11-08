# World Generation

## Seeds & determinism

- A world is defined by **(seed, world knobs, difficulty settings, game version)**.
- Seeds are `u64`, displayed/entered as a shareable string (e.g., `EMBER-WOLF-7731`) or raw number. Blank → random.
- RNG: `rand_chacha::ChaCha8Rng`. Each subsystem derives an **independent stream** by hashing `(seed, domain, coordinates)` — e.g., `hash(seed, "dungeon", dungeon_id, depth)`. Consequences:
  - Same seed + settings ⇒ identical overworld, towns, dungeon layouts, item appearances, uniques placement.
  - Player actions never change *what* a level is, only its state.
  - Levels are generated **lazily** on first visit, yet deterministically.
- Combat/loot rolls use a separate gameplay stream saved with the game (prevents save-scum rerolling by design; still deterministic given same inputs → enables replays and tests).

## Overworld

1. **Heightmap + moisture + temperature** via layered noise (`noise` crate / fBm).
2. **Biomes**: ocean, coast, plains, forest, deep forest, hills, mountains, swamp, desert, tundra, volcanic wastes, blighted lands (near the final dungeon).
3. **Rivers** flow downhill from mountains; **roads** connect towns (A* weighted by terrain).
4. **Towns** placed on habitable biomes, spaced apart; one is the **starting town**.
5. **Dungeon entrances** placed by biome affinity (crypts near plains/forests, mines in mountains, ruins in deserts, etc.).
6. **Final dungeon** placed far from start inside the blighted lands.
7. **Points of interest**: shrines, camps, lone towers, caravans, groves, ruins (small one-level sites).
8. **Danger gradient**: danger level rises with distance from the starting town and toward the blight.

Overworld travel is tile-by-tile on a **separate, scaled-down overworld map** (ADOM style; 1 tile ≈ a local area). Entering a town, dungeon or point of interest switches to the full-scale local map; encounters drop you into a small generated local map.

### Overworld encounters

- **Visible groups:** monster groups appear on the overworld map within your sight range, which Tracking extends. They wander and may approach or avoid you.
- **Random ambushes:** each overworld step also rolls a hidden encounter chance based on biome, area level, blight and night time.
- **Level comparison:** `Δ = area level − Power Level` (see [combat.md](combat.md#power-level--area-level)) scales how often and how strong encounters are. Much weaker monsters **avoid** you.
- **Avoiding encounters:** when an encounter triggers you get a prompt to engage or try to evade. Evasion is a skill check using Stealth, Athletics and speed.
- **Control consumables:**
  - **Repel** potions and scrolls (e.g., *Ward of Passage*) reduce or block encounters for a duration. Their strength depends on item quality versus area level.
  - **Attraction** potions and lures **increase** spawns, optionally for a chosen **family** (e.g., *Wolf Musk* draws beasts). Use them to farm quest targets or specific monster drops in an area.
- Overworld monsters respawn naturally over time. Monsters inside dungeon instances do not (see [endgame.md](endgame.md#instanced-dungeons)).

## Towns

Each town is seeded with:
- **Shops**: general store, smith (weapons/armor/repair), alchemist, magic shop, fletcher/tailor (varies by town size).
- **Sage** (identification), **temple** (remove curse, healing, resurrection of pets), **bank/stash** (shared across towns), **stables** (store tamed pets and pack animals, buy and train pack animals; fee per day), **deed office** (house deeds), **trainers** (skills up to 30), **inn** (rest, rumours), **crafting stations**.
- **Quests:**
  - **Main quest line** (seeded): the blight, Malgareth's history, finding and destroying the phylactery, and the final dungeon.
  - **Secret quest line:** *Kaedric's secret* (see [lore.md](lore.md#kaedrics-secret)).
  - **Side quests** (procedural) from town boards and NPCs: bounties, monster-part fetches, exploration, escorts, craft-and-deliver, blight cleansing, rescues.
  - Rewards are gold, items, recipes, sigils and unique keys. There is no reputation system.
  - Narrative is light: rumours, lore books and flavour text, with no branching dialogue.
- Town size (village / town / city) scales services. Towns are safe zones (no hostile spawns; guards).

## Dungeons

Each dungeon has: theme, generator type, depth (levels), danger range, monster table, boss, unique loot table, special features.

| Theme | Generator | Typical features |
|---|---|---|
| Cave | Cellular automata | Underground lakes, fungi, beasts |
| Crypt / Catacomb | Rooms & corridors (BSP) | Undead, sarcophagi, traps, holy relics |
| Mine | Drunkard's-walk tunnels + chambers | Ore veins, kobolds/dwarven ghosts, minecarts |
| Ruins | Prefab vaults + rooms | Mixed monsters, puzzles, secret rooms |
| Wizard Tower | Vertical small floors (ascend) | Constructs, elementals, spellbooks |
| Lair | Single large cave + boss | Dragon/giant/beast boss |
| **Final Dungeon** | Mixed generators per floor, deepest | Escalating bosses, the big bad at the bottom |

Common level features: stairs up/down (≥1 each), doors (locked/secret), traps, chests, altars, fountains, shops (rare), vaults (hand-designed prefabs with seeded variation), gathering nodes, light sources.

Dungeons are **instanced and resettable** with optional **Dungeon Sigil** modifiers; see [endgame.md](endgame.md#instanced-dungeons). Dungeons are **dark**; see [time-and-turns.md](time-and-turns.md#light).

Connectivity guarantee: every level is validated so stairs are reachable (flood-fill), and placement avoids unwinnable states.

## The big bad

The big bad is **Malgareth the Undying**, the Lich-Emperor, a betrayed champion (see [lore.md](lore.md)) whose **undead blight** spreads from the final dungeon.

- The blight is a visible overworld region around the final dungeon; it **grows slowly over game time** (rate scaled by difficulty), converting biomes to blighted lands, raising undead encounter rates, and eventually threatening outlying villages.
- Quests, rumours and cleansing shrines let the player slow or push back the blight.
- The Lich-Emperor's **phylactery** must be found and destroyed (hidden in another dungeon, revealed via quests/lore) — otherwise he reforms once after defeat on a deeper "Throne of Bone" floor.
- Final floor: multi-phase boss fight (necromantic summons, life drain, curses, blight terrain).
- Resetting the final dungeon grows the blight by one step.
- Defeating him wins the game and offers **New Game+** (see [endgame.md](endgame.md)).

## World knobs (player-tweakable at start)

| Knob | Options | Effect |
|---|---|---|
| World size | Small / Medium / Large / Huge | Overworld 96² / 160² / 256² / 384² tiles; town count 3 / 5 / 7 / 9 |
| Dungeon count | Few / Normal / Many | ×0.6 / ×1 / ×1.6 dungeons per world size (final dungeon always present) |
| Dungeon depth | Shallow / Normal / Deep | Levels per dungeon ×0.6 / ×1 / ×1.5; final dungeon 15 / 25 / 40 |
| Monster density | 0.5× – 2.0× (slider, step 0.25) | Spawn count per level & encounter rate |
| Loot abundance | 0.5× – 2.0× (slider, step 0.25) | Drop count & chest frequency (not rarity — that's difficulty) |
| Hunger | On / Off | Enables food clock |

Knobs are recorded in the save and morgue; they affect the generated world deterministically with the seed.
