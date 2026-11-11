# Combat

**Skill revision:** the general Tactics skill has been removed; weapon-family
skills and stats replace its benefits. The full-game formulas below are legacy
proposals needing recalibration (Q-034/Q-035/I-037); the P2 implementation
described separately remains unchanged. One Magic skill replaces school skills.
Total magic damage reduction and hostile magic-effect resistance chance each
cap at 90%; their combination and interaction with the general resistance cap
require design and simulation.

## Resolution model: always hit

Attacks and offensive spells **always connect**. There is no miss roll. Defence works by **reducing** damage:

1. **Evasion** gives a chance to turn a hit into a **glancing blow** (50% damage).
2. **Blocking** with a shield gives a chance to reduce damage by 75%; Shields abilities can raise this.
3. **Armor** subtracts a flat amount, then a percentage. It applies to physical damage only.
4. **Resistances** reduce elemental damage by a percentage.

Accuracy stays meaningful: it lowers the defender's glancing chance and raises the attacker's critical chance.

## Damage pipeline

```
raw     = weapon_or_spell_base × (1 + mastery% + Tactics% + stat% + affix%)
crit?   = roll(crit_chance)            → raw × crit_multiplier (base 150%)
glance? = roll(glance_chance)          → × 0.5   (crits cannot glance)
block?  = roll(block_chance)           → × 0.25  (shield, frontal arc)
physical: − flat_armor, then × (1 − armor% )
element : × (1 − effective_resist)
final   = max(1, round(result))
```

| Value | Formula (initial, tunable) |
|---|---|
| Accuracy | `DEX mod × 2 + weapon skill / 4 + Tactics / 8 + gear` |
| Evasion | `DEX mod × 2 + Athletics / 8 + gear − armor hindrance` |
| Glance chance | `clamp( (Evasion − attacker Accuracy) × 2% + 10%, 0%, 60% )` |
| Crit chance | `clamp( 5% + (Accuracy − defender Evasion) × 0.5% + gear, 1%, 75% )` |
| Block chance | `shield base + Shields / 4 %`, capped at 75% |
| Flat armor | `Σ armor pieces / 10` |
| Armor % | `armor / (armor + 10 × raw)`, capped at 85% |

Spells use Magic in place of the weapon skill and INT or WIS in place of DEX for accuracy. Magic Resistance combines with stats and equipment; the precise defensive formulas remain open.

### P2 provisional implementation

The P2 "Fight and retry" arena implements this pipeline in `prl_core` with
integers only, in hundredths of HP ("centi-HP") and basis points (bp, 1/100 of a
percent). All values are provisional: stats come straight from actor definitions
in `assets/packs/prl/base.ron` (Q-019, Q-004), the armor order and the missing
block step follow Q-009, and RNG use follows Q-016.

1. `raw_c = damage × 100` (no mastery, skill or gear bonuses until P4).
2. `crit_bp = clamp(500 + (acc − eva) × 50, 100, 7500)`; one roll. On a crit,
   `raw_c = raw_c × crit_multiplier_pct / 100` (default 150) and **no glance roll
   is made**.
3. Otherwise `glance_bp = clamp(1000 + (eva − acc) × 200, 0, 6000)`; one roll; a
   glance halves `raw_c` (integer division).
4. No block step: no shields exist in P2 (Q-009). The code marks where it goes.
5. Armor: `flat_c = armor × 10`; `armor_bp = min(8500, armor × 1_000_000 /
   (armor × 100 + 10 × raw_c))` using `raw_c` **after** crit/glance and **before**
   flat armor (armor 0 → 0%); `after_c = max(0, raw_c − flat_c) × (10000 −
   armor_bp) / 10000`.
6. `final = max(1, (after_c + 50) / 100)` (round half up). HP saturates at 0.

A roll succeeds when `(next_u32 × 10000) >> 32 < chance_bp` on the run's
`ChaCha8` combat stream. Each attack consumes one word on a crit and two
otherwise; rejected commands consume none. The `Attacked` outcome reports `raw`
(post crit/glance, pre-armor, rounded), `mitigated` (`raw − damage`, floored at
0) and `damage`.

The worked examples are formula cases using the first-draft stats in brackets
(they match the unit tests). The shipped arena numbers are tuned separately in
`base.ron`. Current tuning: player 40 HP, Shambler 12, Skeleton 8, Grave rat 5
damage. Holding the door wins about 78% of seeds 1–50; charging into the hall
wins about 4% (see the ignored `report_naive_door_fight_win_rate` probe).

| Example | Working (centi-HP) | Final |
|---|---|---|
| Skeleton (5) hits player (armor 10) | 500 → armor 1666 bp; (500 − 100) × 0.8334 = 333 | 3 |
| Skeleton crit on player | 750 → 1176 bp; 650 × 0.8824 = 573 | 6 |
| Glancing Skeleton on player | 250 → 2857 bp; 150 × 0.7143 = 107 | 1 |
| Player (7) hits Skeleton (armor 6) | 700 → 789 bp; 640 × 0.9211 = 589 | 6 |
| Player crit on Grave rat (armor 0) | 1050 → 0 bp; 1050 | 11 |
| Glancing Shambler (8) on player | 400 → 2000 bp; 300 × 0.8 = 240 | 2 |
| Shambler crit on player | 1200 → 769 bp (1111 bp if pre-crit raw were used); 1100 × 0.9231 = 1015 | 10 |
| Grave rat (3) on player | 300 → 2500 bp; 200 × 0.75 = 150 | 2 |
| 1 damage vs armor 10 | 100 − 100 = 0 | 1 (minimum) |
| Armor 100 vs 100 centi | 1_000_000 / 11_000 = 9090 bp → capped | 85% cap |

Chance examples: player (acc 10) vs Skeleton (eva 4) crits on 800 bp and never
glances; Skeleton (acc 6) vs player (eva 8) crits on 400 bp and glances on 1400 bp.

Melee is bump-to-attack: moving into a Hostile actor attacks it, moving into a
Passive actor is rejected as occupied, and the Q-008 corner rule rejects
diagonal attacks exactly like diagonal moves. A non-player reduced to 0 HP is
removed; the player stays on the map at 0 HP and the run ends (Q-021).

## Damage types

| Type | Secondary effect (chance scales with damage) |
|---|---|
| **Physical** (slash / pierce / blunt subtypes) | Bleed (slash), Armor pierce (pierce), Stun (blunt) |
| **Fire** | Burn (damage over time; spreads to flammable terrain) |
| **Cold** | Chill (slow) → Freeze (lose turns) |
| **Lightning** | Shock (target takes extra damage for a few turns); chains in water |
| **Poison** | Poisoned (stacking damage over time; blocks HP regeneration) |

"Holy" and "unholy" are not damage types. They are **bonus tags** (e.g., Smite deals +100% to targets tagged *undead* or *demon*). Physical subtypes matter for monster weaknesses (skeletons take extra blunt damage, for example).

## Resistances

- Range is −100% to the cap. Base cap is **75%**. Rare *+max resist* affixes and some uniques can raise the cap to **90%**.
- **Difficulty resist penalty** is subtracted from your resistances:

| Story | Easy | Normal | Hard | Nightmare | Per NG+ tier |
|---|---|---|---|---|---|
| 0 | 0 | −10% | −30% | −50% | additional −10% |

- Physical damage is mitigated by armor, not resistance (some rare affixes give *% physical reduction*, capped at 50%).

## Status effects

Bleed, Burn, Chill, Freeze, Shock, Poisoned, Stun, Root, Slow, Fear, Blind, Silence, Cursed, Diseased. Each has duration in ticks, may stack (data-defined), and can be cured by consumables, spells or skills.

## Power Level & area level

PRL has no character levels. To compare the player with monsters and areas, the game computes a **Power Level (PL)**:

```
PL = floor( total trained skill points / 10 )
```

All trained skills count (crafting and gathering included). Effective bonuses from gear do not count. A new character starts at about **PL 15**.

- Every overworld region, dungeon level and monster has an **area level / monster level** on the same scale.
- `Δ = area level − PL` drives encounter rates, monster avoidance and threat colouring in the UI (grey, green, white, yellow, red, purple).
- Exact area-level ranges per region and depth will be tuned in the balance milestone.

## Monsters

- **Bestiary target for v1:** 60–80 base monsters across ~10 families (beasts, vermin, goblinoids, orcs/ogres, undead, demons, elementals, constructs, giants, dragons/drakes), plus Champion/Elite/Unique modifiers.
- **Monster level** comes from area level plus difficulty offset plus NG+ tier.
- Much weaker monsters (`Δ ≤ −15`) try to avoid or flee from the player unless cornered or commanded by a stronger leader.
