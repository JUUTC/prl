# Skills & Progression

PRL has **no character levels and no XP**. Characters improve by using skills, Ultima Online style. There is **no per-skill cap and no total cap** — one character can, in principle, become an expert in every skill. Growth slows logarithmically so mastering everything is a very long-term goal. What a character *does* at any moment is defined by **equipped gear** (a mace uses Maces, a bow uses Archery, a holy symbol focuses Restoration…).

Skill values are fixed-point with one decimal (e.g., `47.3`).

## Interview decisions and revision status

The 2026-11-10 skills interview is ongoing (Q-034/Q-035, I-037). The
following accepted decisions supersede conflicting legacy tables and formulas
below; those tables are design history, not an implementation-ready catalogue.

- No skill may block essential survival or main-path progress. Baseline healing
  and non-skill identification are available to every build; required locked or
  trapped routes have a non-skill alternative, with keys expensive or rare.
- Keep weapon-family proficiencies, Shields, Light Armor, Heavy Armor, Athletics,
  Meditation and Magic Resistance. Basic weapon and equipment use does not
  require skill; some heavy weapons/armor and casting gear may require stats.
  Remove Tactics and Dual Wielding tracks; use weapon skills/stats instead.
- One Magic skill covers casting; schools are descriptive categories. Keep
  optional Identification and one Companion Handling skill for all companions.
  Basic follow/stay/attack/dismiss commands work reliably at zero skill.
- Combine locks and traps into Security, tracking and healing into Survival;
  use Equipment Crafting, Alchemy & Cooking, Arcane Crafting, Extraction
  (ore/wood) and Harvesting (herbs/animal materials). Remove Bartering;
  Charisma influences prices. Stealth remains undecided.
- Zero-skill use reaches basic competence quickly then slows. Safe practice
  can train basics, but meaningful field use is needed for advanced mastery.
  Balance gains against opportunities per skill; ID and Meditation may need
  different rates from attacks. The sample formula, pacing and safe-practice
  ceilings below need simulation rather than being accepted final rates.
- Magic Resistance combines skill, stats and gear: magic damage reduction
  and hostile-effect resistance chance each cap at 90%. Combination formulas
  and interaction with the general resistance cap remain open.

## Skill list

**Legacy list:** superseded where it conflicts with the interview decisions above.

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

Target pacing (Normal, focused use) — **slow, hardcore UO feel**: ~5 hours to Journeyman (50), **~20+ hours to Master (100)**, Legendary (150) and beyond are multi-run / very long-run goals. `G_base` and `K` are tuned to hit these targets (verified by a simulation test).

**Trainers** in towns can raise any skill to at most **30** for gold.

## Stat training

Each skill gain has a chance to train a linked stat:

- Chance `= 0.05 / (1 + (stat − 10) / 5)` per gain, split across linked stats.
- Stats rise +1 at a time; logarithmic soft cap, no hard cap.
- **No decay:** trained skills and stats never go down from disuse. (Temporary drains from monsters/curses are status effects and wear off or are cured.)

## Ability unlocks

Abilities unlock automatically at skill thresholds **25, 50, 75, 100, 150, 200**. Beyond 200, every +50 grants a **minor rank** to that skill's mastery passive. Full list: [abilities.md](abilities.md).

### Synergy abilities — the progression tree

**Legacy examples:** school and Healing thresholds below require redesign;
no direct mapping to new thresholds has been approved.

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
