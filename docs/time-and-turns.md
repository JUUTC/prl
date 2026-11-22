# Time & Turns

PRL uses an **energy-based scheduler**. The world only advances when the player
commits a costed action. Menus, inventory browsing and look mode consume no time.
Same-backpack placement, 90-degree rotation and explicit Organize are also free,
including in danger (Q-031/I-036).

## Energy model

- Every actor has **speed** (base 100) and an **energy** pool.
- Each **tick**, every active actor gains `speed` energy.
- An actor with `energy ≥ 100` may act; the action's **cost** is subtracted.
- When it is the player's turn, the game **waits for input**. Once the player commits a costed action, ticks advance until the player can act again; all other actors act as their energy allows.
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
| Same-backpack placement / rotation / explicit Organize | 0 |
| Equip / unequip | 100 per item (armor body 300) |
| Swap loadouts | 100 |
| Quaff / read | 100 |
| Open/close door | 100 |
| Pick lock / disarm trap | 100–1000 (multi-turn, interruptible) |
| Rest / wait | 100 |
| Gather / craft | multi-turn, recipe-dependent, interruptible |

P2/P3 provisional (Q-020, Q-021): move, bump melee attack, open door,
wait and stairs cost 100 energy. P3 adds pickup 50, equip/unequip 100
(Body 300), quaff/read 100 and drop 100 (removing worn Body 300).
Costed item actions take effect on commitment, then scheduling runs until the next
player input boundary; body changes can expose the player to multiple enemy
actions. The scheduler accounts for the complete cost even if current carried
energy is below it; no negative-energy debt remains at the committed snapshot
boundary. Monsters still use 100-energy actions.
Accepted free organization commands change only the bag layout/orientation and
record their outcome/message. They do not increment the player action counter,
subtract energy, refresh visibility or run clocks, AI or RNG. Gameplay transfers
(pickup/equip/unequip/drop) retain their normal costs; pickup never implicitly
organizes the bag.
Rejected commands cost nothing. Once the player dies the scheduler stops at once
and every command except starting a new run is rejected without spending
energy, advancing the clock or consuming RNG.

## World clock

- 1 standard turn (100 energy at speed 100) = **6 seconds** of game time in dungeons/towns.
- On the **overworld map**, each step = **10 minutes** of game time; the local tick system resumes when entering a location or on random encounters.
- **Day/night cycle** (24h): affects light, monster spawns, shop hours, herb availability.
- Per-tick world systems: HP/Mana/Stamina regen, hunger, status effect durations, cooldowns, poison/bleed, fire spread, light sources burning down, node regrowth, spawn timers.
- Off-screen levels are **frozen** while the player is elsewhere, then "caught up" with a coarse simulation on return (regeneration, node regrowth). Dungeon instances never respawn monsters; only resets refresh them. Overworld encounters respawn over time.
- P3 subset: inactive crypt floors remain entirely frozen, including actor
  energy/AI and terrain memory; coarse catch-up is not implemented yet.
- Pack-animal trips and house sieges advance with the world clock.

## Interruption rules

Multi-turn actions (rest, auto-explore, travel, gather, craft, pick lock) are interrupted when:
- a hostile comes into view,
- the player takes damage,
- a significant message occurs (status change, hunger stage change, item breaks).

### Current held movement/wait (I-023)

Held input resubmits ordinary commands; it is not rest, travel or auto-explore.
The first press acts immediately. Reversible Q-007 defaults are a 500 ms delay
then one action per 180 ms, enabled initially. F2 toggles repetition, F3 cycles
300/500/800 ms delays, and F4 cycles 100/180/300 ms intervals. Settings and
F5 guided movement/wait remapping are session-only; F6 restores default bindings.

Release, any fresh key, focus changes and modal transitions cancel the hold.
Repetition also stops on a visible hostile, damage/attack against the player,
HP at or below 25%, items/doors/stairs underfoot, a blocked or occupied next
cell, rejection, floor/new-run changes, endpoint completion or lost outcome
history. It never repeats a bump attack. After an interruption a fresh press
is required; manual single actions remain available even in danger. Slow frames
produce at most one command, never catch-up bursts or a queue backlog.
Future status/hunger/item-break systems must extend the interruption check when
their outcomes exist.

## Light

- **Dungeons are dark.** Visibility = light radius, except races with darkvision see unlit tiles within 6 (desaturated). Races with **low-light vision** (Elves) see +2 tiles beyond any light radius but nothing in total darkness.
- Light sources: **torch** (radius 4, ~1,500 turns), **lantern** (radius 5, refuelled with oil), **Light** spell (radius 5, duration by Mysticism), glowing affixes, braziers and magical terrain.
- Towns and the overworld use daylight; night reduces sight on the overworld.
- **Stealth:** light and darkness affect ordinary enemy visibility; passive movement detection is separate from the Stealth skill. Stealth supports deliberate Hide/Ambush actions, whose exact checks remain open (Q-035). Carrying a light makes you visible to monsters from farther away, so you trade vision against being noticed.

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
