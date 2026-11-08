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

Overworld travel is tile-by-tile on a scaled-down map; **random encounters** (biome- and danger-based) drop you into a small generated local map.

## Towns

Each town is seeded with:
- **Shops**: general store, smith (weapons/armor/repair), alchemist, magic shop, fletcher/tailor (varies by town size).
- **Sage** (identification), **temple** (remove curse, healing, resurrection of pets), **bank/stash** (shared across towns), **trainers** (skills up to 30), **inn** (rest, rumours), **crafting stations**.
- **Quest givers**: kill/fetch/escort/explore/craft quests; rewards = gold, items, recipes, reputation, unique keys.
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

Connectivity guarantee: every level is validated so stairs are reachable (flood-fill), and placement avoids unwinnable states.

## The big bad

The final dungeon's last floor hosts the big bad — working name **[OPEN]** (e.g., *Malgareth the Undying, the Lich-Emperor*). Defeating it wins the game. It is telegraphed through rumours, quests and the blight's spread.

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
