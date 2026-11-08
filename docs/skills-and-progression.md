# Skills & Progression

PRL has **no character levels and no XP**. Characters improve by using skills, Ultima Online style. There is **no per-skill cap and no total cap** — one character can, in principle, become an expert in every skill. Growth slows logarithmically so mastering everything is a very long-term goal. What a character *does* at any moment is defined by **equipped gear** (a mace uses Maces, a bow uses Archery, a holy symbol focuses Restoration…).

Skill values are fixed-point with one decimal (e.g., `47.3`).

## Skill list

### Combat
| Skill | Trained by | Trains stat |
|---|---|---|
| Swords | Attacking with swords | STR / DEX |
| Axes | Attacking with axes | STR |
| Maces | Maces, hammers, flails | STR |
| Polearms | Spears, halberds, glaives | STR / DEX |
| Daggers | Daggers, short blades | DEX |
| Staves | Quarterstaves, battle staves | DEX / INT |
| Unarmed | Fists, claws, kicks | STR / DEX |
| Archery | Bows and crossbows | DEX |
| Throwing | Knives, axes, javelins, bombs | DEX |
| Tactics | Any weapon attack (damage, crit) | STR |
| Shields | Blocking with a shield | STR / CON |
| Dual Wielding | Fighting with two weapons | DEX |
| Light Armor | Taking hits in light armor | DEX |
| Heavy Armor | Taking hits in heavy armor | CON / STR |
| Athletics | Moving encumbered, running, climbing, swimming | CON |

### Magic
| Skill | Trained by | Trains stat |
|---|---|---|
| Evocation | Damage spells (fire, frost, lightning, force) | INT |
| Abjuration | Wards, dispels, resistances | WIS |
| Restoration | Healing, holy, cure, bless | WIS |
| Mysticism | Utility: teleport, light, detect, telekinesis | INT |
| Meditation | Regenerating mana | WIS |
| Magic Resistance | Resisting hostile spells | WIS |

### Companions & Dark Arts
| Skill | Trained by | Trains stat |
|---|---|---|
| Animal Taming | Taming and commanding beasts | CHA |
| Necromancy | Raising undead, life drain, curses | INT |
| Summoning | Conjuring elementals, spirits, bound weapons | INT / CHA |

### Subterfuge & Utility
| Skill | Trained by | Trains stat |
|---|---|---|
| Stealth | Moving unseen near enemies | DEX |
| Lockpicking | Picking locks | DEX |
| Traps | Detecting, disarming, setting traps | DEX / INT |
| Item Lore | Inspecting/identifying items | INT |
| Tracking | Reading tracks; revealing monsters | WIS |
| Healing | Bandages, cure kits | WIS / DEX |
| Bartering | Trading with shops | CHA |

### Crafting
| Skill | Produces | Trains stat |
|---|---|---|
| Blacksmithing | Metal weapons & armor; repair | STR |
| Tailoring | Cloth & leather armor, bags | DEX |
| Carpentry / Bowcraft | Bows, arrows, staves, wooden shields | DEX |
| Alchemy | Potions, poisons, bombs | INT |
| Inscription | Scrolls, spellbooks, maps | INT |
| Cooking | Food (buffs, satiation) | WIS |
| Enchanting | Add/reroll/remove affixes; socket gems & runes | INT |

### Gathering
| Skill | Gathers | Trains stat |
|---|---|---|
| Mining | Ore, gems, stone | STR |
| Lumberjacking | Wood, bark, sap | STR |
| Herbalism | Herbs, reagents, mushrooms | WIS |
| Skinning | Hides, bones, monster parts | DEX |

## Skill titles

| Value | Title |
|---|---|
| 0 | Untrained |
| 10 | Neophyte |
| 25 | Apprentice |
| 50 | Journeyman |
| 75 | Expert |
| 100 | Master |
| 125 | Grandmaster |
| 150 | Legendary |
| 200 | Mythic |
| 300+ | Ascendant I, II, III… (every +100) |

## Gain rules

When an action uses skill `S` (value `s`) against difficulty `D`:

1. Compute success probability `p` for the action.
2. **Challenge factor** `c(p) = 4·p·(1−p)` (peaks at `p = 0.5`). If `p > 0.97` (trivial) or `p < 0.03` (hopeless), `c = 0` → **no gains from trivial or impossible actions**.
3. **Decay factor** `d(s) = 1 / (1 + s / K)`, `K = 25` (tunable) — slows logarithmically, never stops.
4. Gain chance `g = G_base × c(p) × d(s) × race_mult × difficulty_mult`, `G_base = 0.5`.
5. On gain, `s += 0.1`.
6. Failed attempts can also gain (at 50% weight).
7. **Repeat penalty:** repeating the same action on the same target type within a short window reduces `g` (anti-macro).
8. Gear affixes `+X to [Skill]` raise the *effective* value but not the *trained* value; gains are based on trained value.

Target pacing (Normal, focused use): Journeyman in a few hours of play; Master by mid-game; Legendary late game; 200+ is long-run territory.

**Trainers** in towns can raise any skill to at most **30** for gold.

## Stat training

Each skill gain has a chance to train a linked stat:

- Chance `= 0.05 / (1 + (stat − 10) / 5)` per gain, split across linked stats.
- Stats rise +1 at a time; logarithmic soft cap, no hard cap.
- Stat decay for unused stats: **none** (default).

## Ability unlocks

Abilities unlock automatically at skill thresholds **25, 50, 75, 100, 150, 200**. Beyond 200, every +50 grants a **minor rank** to that skill's mastery passive. Full list: [abilities.md](abilities.md).

### Synergy abilities — the progression tree

**Synergy abilities** unlock when several skill thresholds are met together. Single-skill unlocks plus synergies form a web connecting skill families:

```
 Swords|Maces 75 ─────┐
                      ├──► Smite
 Restoration 75 ──────┘
 Any weapon 75 ───┬───► Spellblade   ◄── Evocation 75
                  └───► Battle Medic ◄── Healing 75
 Stealth 100 ─────┬───► Assassinate  ◄── Daggers 100
                  └───► Shadowstep   ◄── Mysticism 50
```

Full list: [abilities.md](abilities.md#synergy-abilities).

## Gear dependency

- Weapon-skill abilities require an equipped weapon of that type.
- Spells require a free hand **or** a focus (staff, wand, holy symbol, tome, orb).
- Armor hindrance raises spell failure (Heavy > Light > Cloth) unless mitigated (e.g., *Armored Faith*).
- Switching gear sets is a core part of play; a quick-swap of two loadouts costs one action.
