# Meta-Progression

**Skill prerequisites need revision:** the skills interview removed Bartering
and consolidated other tracks. Feat examples below that name retired skills are
legacy proposals; replacements require explicit design, not automatic mapping
(Q-034/Q-035, I-037).

Meta-progression persists **across runs** in a profile file (separate from character saves). It never grants raw power directly in a new run; it **unlocks options**: races, classes, and legacy special abilities (perks) selectable at character creation.

## Feats

Feats are specific in-game accomplishments, tracked by an event system (kills, skill thresholds, items crafted, places visited, deaths, conditions). Feats are data-defined (RON) so new ones are easy to add.

### Race unlocks
| Unlock | Feat |
|---|---|
| Gnome | Reach Master (100) in any two crafting skills in one run |
| Half-Elf | Have both a Human and an Elf character reach Expert (75) in any skill |
| Dark Elf | Clear a dungeon without being seen (never detected while in Stealth on its deepest level) |
| Half-Giant | Kill a Giant boss in melee without armor heavier than Light |
| Revenant | Die to the big bad (or any Lich), *or* craft a Phylactery |

### Class unlocks
| Unlock | Feat |
|---|---|
| Necromancer | Reach Necromancy 75 |
| Summoner | Have 3 summoned creatures active at once |
| Druid | Tame 10 different beast species in one run |
| Bard | Reach Bartering 100 and complete 15 quests in one run |
| Monk | Kill a dungeon boss using only Unarmed |
| Beastmaster | Have a pet survive from start of a dungeon to its final boss kill |
| Artificer | Craft a Masterwork item with a runeword |

### Legacy perks (special abilities)
Selected at character creation. Each run has **2 legacy slots** (+1 slot after first victory). Examples:

| Perk | Feat | Effect |
|---|---|---|
| Heirloom | Find a Unique item | Start with a random Normal item with 2 sockets |
| Old Soldier | Kill 1,000 monsters (cumulative) | +5 Tactics at start |
| Scholar | Learn 25 spells (cumulative) | Start knowing one random circle-1 spell |
| Lucky Coin | Die with 10,000+ gold | +10% gold find |
| Cartographer | Explore 100 dungeon levels (cumulative) | Overworld map starts revealed |
| Survivor | Survive at < 5% HP 10 times | +5% max HP |
| Tamer's Legacy | Tame a dragon | Start with a random tamed small beast |
| Iron Will | Win on Hard or Nightmare | +10 Magic Resistance at start |
| Bane of the Undying | Defeat Malgareth | New Game+ option: "Blight Ascendant" world modifier |
| Oathbreaker's Truth | Uncover Kaedric's secret | Start with the *Testament of the Companions* lore book (reveals one secret-quest location) |
| Ascendant | Reach NG+3 | +1 legacy slot |

## Rules

- **All feats can be earned in any mode**, including Explorer runs.
- Unlocks are global to the profile; a "reset profile" option exists.
- The **Hall of Records** screen lists runs (morgues), feats, progress toward locked feats (hidden feats show `???`).
