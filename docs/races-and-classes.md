# Races & Classes

**Revision required:** the 2026-11-10 interview changed the skill catalogue and
creation modes (Q-033–Q-035/I-037). The aptitude and class allocation tables
below are legacy examples, not valid final builds: school and removed skills
must be reconciled, capped at 50 per starting skill and redistributed to retain
150 default points. Exact revised class values and race-aptitude treatment are
open; do not treat these lists as approved new allocations. Class is a starting
template, never an equipment restriction.

The headless I-046 creation draft checks pre-racial point-buy and starting
skill budgets using caller-supplied class defaults and a caller-supplied
skill catalogue. It stores opaque gear section values without validating
prices, allowed gear or jackpot rules. Class availability, racial application,
random distributions and the actual playable character are not implemented.

## Races

Racial stat modifiers apply **after** point-buy. Racial traits are permanent special abilities. Racial **aptitudes** give +5 starting points and a +20% gain-rate multiplier on listed skills.

| Race | Stat mods | Vision | Size | Hunger |
|---|---|---|---|---|
| Human | +1 all | Normal | Medium | 1.0× |
| Elf | DEX +2, INT +1, WIS +1, CON −1 | Low-light | Medium | 0.9× |
| Dwarf | CON +2, STR +1, WIS +1, DEX −1, CHA −1 | Darkvision (6) | Medium | 1.0× |
| Halfling | DEX +2, CHA +1, CON +1, STR −2 | Normal | Small | 0.8× |
| Orc | STR +2, CON +2, INT −1, CHA −2 | Darkvision (6) | Medium | 1.2× |

### Human — *the adaptable*
- **Versatile:** +10% gain rate on **all** skills.
- **Ambitious:** choose 2 skills at creation for +5 starting points each.
- **Diplomat:** +10% gold and item-quality rewards from quests.
- **Active — Second Wind** (once per rest): restore 25% HP and Stamina.

### Elf — *the ancient*
- **Aptitudes:** Archery, Mysticism, Herbalism, Stealth.
- **Keen Senses:** +2 FOV radius; passive search for hidden doors/traps.
- **Fey Blood:** immune to sleep; resistant to charm.
- **Active — Glamour** (cooldown): briefly invisible to non-adjacent enemies.

### Dwarf — *the stoneborn*
- **Aptitudes:** Axes, Maces, Mining, Blacksmithing.
- **Stoneborn:** 25% poison resistance; immune to petrification.
- **Stonecunning:** detect ore veins and secret passages within 3 tiles.
- **Steady:** cannot be knocked back; −10% movement speed.
- **Active — Stand Firm:** 10 turns of +50% armor, cannot move.

### Halfling — *the lucky*
- **Aptitudes:** Stealth, Throwing, Cooking, Lockpicking.
- **Lucky:** once per 100 turns an incoming critical becomes a normal hit; 5% chance to re-roll any failed check.
- **Small:** +2 evasion and stealth; "great" two-handed weapons −20% accuracy; reduced carry capacity.
- **Nimble:** may swap places with allied or smaller creatures.
- **Active — Slip Away:** next move costs no energy and provokes no attacks.

### Orc — *the relentless*
- **Aptitudes:** Axes, Tactics, Athletics, Unarmed.
- **Active — Bloodrage:** 8 turns of +30% melee damage, +20% speed, −30% defense, no casting; costs Stamina.
- **Relentless:** once per rest, survive a lethal blow at 1 HP.
- **Intimidating:** weak monsters may flee; town prices +15%.
- **Thick-skinned:** +2 natural armor.

## Classes

A class is a **starting template only**: starting skills, starting kit and a title. Every class starts with **150 skill points** across its skills. After creation there are no class restrictions.

### Warrior
Swords 30 · Tactics 30 · Heavy Armor 30 · Shields 20 · Athletics 20 · Healing 10 · Blacksmithing 10

**Kit:** longsword, kite shield, chainmail set, 5 bandages, 3 rations, 50 gold.

### Rogue
Daggers 30 · Stealth 30 · Lockpicking 25 · Traps 25 · Light Armor 20 · Throwing 15 · Item Lore 5

**Kit:** 2 daggers, leather armor set, lockpicks ×10, throwing knives ×10, smoke bombs ×2, 3 rations, 80 gold.

### Mage
Evocation 30 · Meditation 30 · Mysticism 20 · Item Lore 20 · Inscription 15 · Abjuration 15 · Alchemy 10 · Magic Resistance 10

**Kit:** quarterstaff, robe & hood, spellbook (*Magic Missile, Light, Minor Ward, Detect Magic*), 2 mana potions, 5 blank scrolls, ink, 3 rations, 60 gold.

### Cleric
Restoration 30 · Maces 25 · Abjuration 25 · Shields 20 · Heavy Armor 15 · Meditation 15 · Healing 10 · Magic Resistance 10

**Kit:** mace, buckler, scale mail, holy symbol (focus), prayer book (*Minor Heal, Bless, Turn Undead*), 5 bandages, 3 rations, 50 gold.

### Ranger
Archery 30 · Tracking 25 · Light Armor 20 · Animal Taming 20 · Stealth 15 · Skinning 15 · Swords 15 · Cooking 10

**Kit:** shortbow, 60 arrows, short sword, leather armor set, skinning knife, flint & tinder, 3 rations, 50 gold.

### Paladin
Swords 25 · Heavy Armor 25 · Restoration 20 · Shields 20 · Tactics 20 · Magic Resistance 15 · Healing 15 · Abjuration 10

**Kit:** longsword, heater shield, banded mail, holy symbol, prayer book (*Lay on Hands, Bless*), 3 rations, 40 gold.

## Unlockable races & classes

Unlocked via meta-progression feats — see [meta-progression.md](meta-progression.md).

- **Races:** Gnome, Half-Elf, Dark Elf, Half-Giant, Revenant (undead).
- **Classes:** Necromancer, Summoner, Druid, Bard, Monk, Beastmaster, Artificer.

## Pantheon (Clerics & Paladins)

Clerics and Paladins **must choose a patron deity** at character creation. Other classes have no patron. Draft pantheon (names are placeholders):

| Deity | Domain | Boon | Taboo (loses piety) |
|---|---|---|---|
| Solenne | Sun, healing | Restoration spells +15% potency | Using Necromancy |
| Kaedric* | War, honour | +10% melee damage while at full HP | Attacking fleeing or sleeping foes |
| Verdana | Nature, beasts | Pets regenerate; Herbalism +20% gain | Killing tamed or neutral animals |
| Thalos | Knowledge, magic | −10% spell failure; Item Lore +20% gain | Destroying spellbooks or scrolls unread |
| Morrow | Death's passage (anti-undead) | Turn Undead/Smite +25% vs undead | Raising or commanding undead |
| Iridel | Luck, travel | +5% magic find; cheaper Sage/temple services | Refusing quests, breaking deals |

\* Kaedric betrayed Malgareth; a secret quest line reveals the truth and forces his followers to choose. See [lore.md](lore.md#kaedrics-secret).

**Piety** (0–200) rises with deeds the deity favours (e.g., destroying undead, healing allies, tithing) and falls with taboos.

- **Pray** (action, long cooldown): at sufficient piety, the deity heals, cures, or removes curses; low piety risks wrath (curse, summoned punisher).
- Piety thresholds (50/100/150/200) grant the deity's boon and later a unique **divine favour** ability.
- Switching patron is possible at a temple of the new god, but resets piety and angers the old god (temporary curse).
- If a Cleric/Paladin drops to 0 piety, Restoration spells suffer +20% failure until piety recovers.
