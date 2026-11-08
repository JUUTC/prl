# Time & Turns

PRL uses an **energy-based scheduler**. The world only advances when the player commits an action. Menus, inventory and look mode consume no time.

## Energy model

- Every actor has **speed** (base 100) and an **energy** pool.
- Each **tick**, every active actor gains `speed` energy.
- An actor with `energy ≥ 100` may act; the action's **cost** is subtracted.
- When it is the player's turn, the game **waits for input**. Once the player acts, ticks advance until the player can act again; all other actors act as their energy allows.
- Ties are broken deterministically (highest energy, then stable entity order) to keep runs reproducible.

### Speed examples

| Speed | Meaning |
|---|---|
| 50 | Very slow (zombie) — acts every 2 player turns |
| 100 | Normal |
| 150 | Fast (wolf) |
| 200 | Very fast (hasted, bat) — acts twice per normal turn |

Speed modifiers: encumbrance, armor, haste/slow, terrain (swamp, deep snow), racial traits, affixes.

### Action costs (initial)

| Action | Energy |
|---|---|
| Move (orthogonal or diagonal) | 100 |
| Melee attack | weapon speed: dagger 70–80, sword 100, axe/mace 110, great weapon 130–150 |
| Ranged attack | bow 100, crossbow 150 (higher damage) |
| Cast spell | 100 × spell cost factor (circle-dependent, 100–200) |
| Pick up item | 50 |
| Equip / unequip | 100 per item (armor body 300) |
| Swap loadouts | 100 |
| Quaff / read | 100 |
| Open/close door | 100 |
| Pick lock / disarm trap | 100–1000 (multi-turn, interruptible) |
| Rest / wait | 100 |
| Gather / craft | multi-turn, recipe-dependent, interruptible |

## World clock

- 1 standard turn (100 energy at speed 100) = **6 seconds** of game time in dungeons/towns.
- On the **overworld map**, each step = **10 minutes** of game time; the local tick system resumes when entering a location or on random encounters.
- **Day/night cycle** (24h): affects light, monster spawns, shop hours, herb availability.
- Per-tick world systems: HP/Mana/Stamina regen, hunger, status effect durations, cooldowns, poison/bleed, fire spread, light sources burning down, node regrowth, spawn timers.
- Off-screen levels are **frozen** while the player is elsewhere, then "caught up" with a coarse simulation on return (regen, respawn, node regrowth) — standard roguelike behaviour.

## Interruption rules

Multi-turn actions (rest, auto-explore, travel, gather, craft, pick lock) are interrupted when:
- a hostile comes into view,
- the player takes damage,
- a significant message occurs (status change, hunger stage change, item breaks).

## Hunger (when enabled)

| Stage | Effect |
|---|---|
| Satiated | Slight regen bonus; eating more may cause vomiting |
| Normal | — |
| Hungry | — (warning) |
| Weak | −2 STR/DEX, slower regen |
| Fainting | Random fainting (lose turns) |
| Starving | HP loss each turn |

When hunger is **off**, food still provides Cooking buffs but no satiation clock exists.
