# Combat

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

Spells use the school skill in place of the weapon skill and INT or WIS in place of DEX for accuracy. **Magic Resistance** gives targets glancing chance against spells and shortens hostile status effects.

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
