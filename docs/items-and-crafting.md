# Items, Loot & Crafting

## Rarity (Diablo-style)

| Rarity | Colour / marker | Affixes | Notes |
|---|---|---|---|
| Normal | White ○ | 0 | May have sockets; may be Superior (+quality) or Damaged |
| Magic | Blue ◆ | 1–2 (≤1 prefix, ≤1 suffix) | |
| Rare | Yellow ★ | 3–6 (≤3 prefix, ≤3 suffix) | Random generated name |
| Set | Green ⬢ | Fixed + set bonuses | Bonuses for wearing 2+ pieces |
| Unique | Gold ✦ | Fixed, hand-designed, some rolled ranges | Lore text |
| Crafted | Orange ⚒ | Crafter-influenced | Maker's mark |

## Item level & affixes

- **Item level (ilvl)** = danger level of the source (monster, chest, dungeon depth).
- Affixes have **tiers**; higher tiers require higher ilvl. Each affix rolls within its tier range.
- Affix families: offense (+damage, +elemental damage, +attack speed, +crit, life/mana leech), defense (+armor, +resists, +HP, +block), utility (+skill, +stat, +light radius, +magic find, +gold find, +movement speed), on-hit/on-struck procs.
- **Sockets** (0–6 by base type and ilvl): gems (stat bonuses that differ by slot type) and **runes** (runewords — specific rune sequences in specific bases create powerful fixed effects).
- **Durability**: items degrade; at 0 they break (non-functional until repaired). Repair via Blacksmithing/Tailoring/Carpentry skill or town smith.
- **Requirements**: stat requirements (e.g., STR 18 for a greatsword); using gear below requirements is allowed but with heavy penalties.

## Equipment slots

Head, Amulet, Body, Cloak, Hands, Belt, Feet, Ring ×2, Main hand, Off hand, Ammo (quiver), plus **2 quick-swap weapon loadouts**. Gear can be compared against what is equipped, and an on-demand optimizer can propose a loadout; see [gear-and-storage.md](gear-and-storage.md). Ground loot is shown and auto-picked up through custom **loot filters** with built-in presets; see [gear-and-storage.md](gear-and-storage.md#loot-filters).

## Identification

- **Gear:** Normal items are always known. Magic/Rare/Set/Unique/Crafted-by-others items drop **unidentified**: base type and rarity colour are visible, affixes hidden, curse status hidden. Unidentified items cannot be enchanted or socketed.
- **Consumables:** potions, scrolls, wands and rings use **randomized appearances per seed** ("murky green potion", "scroll labelled XOTH"). Using one identifies its type for the rest of the run.
- **Ways to identify:**
  1. **Sage NPC** in towns — costs gold scaled by ilvl and rarity (and Bartering/CHA).
  2. **Scroll of Identify** — one item (blessed: all carried items).
  3. **Item Lore skill** — inspect (takes time) or automatic on pickup at higher skill (see [abilities.md](abilities.md)).
  4. **Equipping** — reveals affixes after some turns but risks curses.
- **Curses:** some items carry negative affixes and/or are *sticky* (can't be unequipped). Removed by Remove Curse scroll, Abjuration *Dispel Magic*, a town temple, or Enchanting 150.
- **Blessed / Uncursed / Cursed** status for consumables (standard roguelike); detectable via Item Lore 75 or altars.

**Retempering** (NG+): raise an old item to the current NG+ tier; see [endgame.md](endgame.md#new-game).

## Consumables

Potions (healing, mana, stamina, cure, resist, speed, strength, invisibility, and some harmful), scrolls (identify, teleport, recall, enchant, remove curse, magic mapping, spell scrolls), wands (charges, rechargeable by Mysticism/Enchanting), food (rations, cooked meals, raw meat), bandages, reagents, gems, runes, keys, lockpicks, ammo, bombs.

## Gold & economy

- Shops in each town with seeded stock that refreshes over time. Prices affected by CHA and Bartering.
- **Bank** shared across all towns. Each player house has its **own separate storage**; see [housing-and-logistics.md](housing-and-logistics.md). Both use named custom **tabs** with auto-sort rules and refill kits; see [gear-and-storage.md](gear-and-storage.md#storage).
- Inventory is a Diablo-style **grid + weight**; see [housing-and-logistics.md](housing-and-logistics.md#inventory).
- Selling unidentified items yields reduced prices.

## Crafting

### General rules

- Crafting requires a **station** (forge, anvil, loom, workbench, alchemy table, scribe's desk, cooking fire, enchanting altar) — found in towns, some dungeons, and portable kits (lower quality cap).
- **Recipes** are known by default for basic items; advanced recipes come from books, trainers, quest rewards, or experimentation.
- Success chance and quality scale with `skill − recipe_difficulty`, INT/DEX and material tier. Failure may destroy some materials.
- Crafting time costs game time (ticks) — usable only when no hostiles are in view.
- Materials can come from the carried inventory **and the storage at the same site** (house storage at a house station, the bank at a town station); see [gear-and-storage.md](gear-and-storage.md#crafting-from-storage).
- Output quality: **Poor → Normal → Superior → Exceptional → Masterwork**. Higher quality adds base stats, durability, sockets and (at skill thresholds) affixes.
- Crafted items are auto-identified for the crafter and carry a maker's mark.

### Material tiers

| Tier | Metal | Wood | Leather / Cloth | Herb examples |
|---|---|---|---|---|
| 1 | Copper / Iron | Pine | Rawhide / Linen | Mandrake, Garlic |
| 2 | Steel | Oak | Leather / Wool | Nightshade, Ginseng |
| 3 | Silver (vs undead) | Ash | Hardened leather / Silk | Bloodmoss, Ashroot |
| 4 | Mithril | Yew | Wyvern hide / Spidersilk | Moonpetal |
| 5 | Adamantine | Ironwood | Drake scale / Shadowweave | Starbloom |
| 6 | Starmetal / Dragonbone | Dragonwood / Elderheart | Dragon scale / Celestial silk | Phoenix feather, Void lotus |

Materials can themselves carry properties (silver: +damage vs undead; mithril: light, low hindrance).

### Crafting skills

| Skill | Station | Products |
|---|---|---|
| Blacksmithing | Forge + anvil | Metal weapons, heavy armor, shields, tools; repair metal |
| Tailoring | Loom / sewing kit | Cloth & leather armor, cloaks, bags (increase capacity), bandages |
| Carpentry / Bowcraft | Workbench | Bows, crossbows, arrows/bolts, staves, wooden shields, traps' frames |
| Alchemy | Alchemy table | Potions, poisons, oils (weapon coatings), bombs |
| Inscription | Scribe's desk | Scrolls (any spell you know), spellbooks (copy spells), treasure maps, rune tablets |
| Cooking | Fire / oven | Meals: satiation + timed buffs (stat, regen, resists) |
| Enchanting | Enchanting altar | Socket/unsocket, add/reroll/remove affixes, upgrade rarity, remove curses |

### Gathering skills

| Skill | Tool | Sources |
|---|---|---|
| Mining | Pickaxe | Ore veins & gem deposits in mines/caves/mountains |
| Lumberjacking | Hatchet / axe | Trees in overworld forests; magical trees in special areas |
| Herbalism | Sickle (optional) | Plants in biomes; dungeon fungi; rare plants by biome & time of day |
| Skinning | Skinning knife | Beast & monster corpses: hides, bones, scales, organs (reagents) |

Gathering nodes are **seeded** (same seed → same node positions) and **regrow** over game time.
