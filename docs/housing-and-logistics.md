# Inventory, Pack Animals & Housing

## Inventory

**Diablo-style grid inventory with weight.**

- Backpack grid: **10 × 6** cells. Items have shapes (potion 1×1, dagger 1×2, sword 1×3, bow 2×4, body armor 2×3, shield 2×2…).
- **Weight** applies on top of the grid. Exceeding carry capacity (`STR × 5` kg, modified by Athletics) reduces speed in encumbrance tiers.
- **Bag options:**
  - **Belt:** 2–4 quick slots for potions and scrolls, usable from the hotbar.
  - **Containers:** pouches, packs and satchels (crafted with Tailoring or bought) add extra grid pages. Larger bags add more cells but weigh more.
  - **Quivers:** separate ammo stacks.
- Stackables (ammo, potions of one type, materials) stack inside a cell up to a per-item limit.

## Pack animals

Pack animals (mule, pack horse, ox, plus tamed large beasts with Animal Taming) carry loot. They **cannot enter dungeons**.

- **At the entrance:** a pack animal waits outside the dungeon. When you step outside, you can transfer items between your inventory and its pack grid.
- **Sending it home:** send the animal to a chosen **town bank** or **player house**. The trip takes game time based on distance and terrain.
  - Before you confirm, the UI shows the **ambush risk %**. Risk is based on the area levels along the route compared with the animal's power, blight proximity, night time, and escorts (pets assigned to it).
  - **On ambush**, a roll decides the outcome: the animal arrives safely (it fought off or evaded the attack), **loses some cargo**, or **is killed**. If it is killed, its cargo is dropped at the ambush site on the overworld and guarded by the ambushers, so it can be recovered.
- **Leading it yourself:** it follows you on the overworld and joins overworld encounters.
- **Training & barding:**
  - Pack animals have HP, armor, speed, **Endurance** (capacity) and **Wariness** (ambush evasion). These improve with completed trips and with paid training at stables.
  - **Barding** (animal armor) is crafted with Tailoring or Blacksmithing and reduces both ambush risk and the chance of the animal being killed.
- Pack animals use **pet control slots** only while travelling with you. Animals at the entrance, en route or stabled don't use slots.

## Player housing

- **Unlimited houses.** Each house occupies **one overworld tile**.
- **Acquiring a house:**
  1. Buy a **house deed** in a town.
  2. Choose a free buildable overworld tile (not water, a town, a dungeon or a point of interest).
  3. **Clear the area:** a local map is generated with monsters based on the area level. It is more dangerous near dungeons, scaling with the nearby dungeon's level. Once it is cleared, the house is built.
- **Tiers (v1):** Small house → Medium house → Large house → Castle. You upgrade in place with gold plus materials (wood, stone, metal; higher tiers need higher-tier materials).

| Tier | Storage (grid pages of 10×10) | Crafting station slots | Extras |
|---|---|---|---|
| Small | 2 | 1 | Bed (full rest) |
| Medium | 4 | 3 | Stable for 1 pack animal / pet |
| Large | 8 | 5 | Stable ×3, herb garden plot |
| Castle | 16 | 8 | Stable ×6, ward stone socket ×2, walls (siege bonus) |

- **Storage is separate per house.** It is not shared with other houses or the bank.
- **Fast travel:** houses act as fast-travel points between each other and towns. Each jump costs a Recall rune or scroll, or gold scaled by distance.
- **Sieges:**
  - Houses are safe **except** in or near the blight. When the blight reaches a house's tile or an adjacent tile, the house is **besieged** and the player is notified.
  - You can defend in person (a local wave-defence map), or rely on **ward stones** crafted with Abjuration/Enchanting, which hold the siege for a duration based on their quality.
  - A house that falls becomes **Ruined**: part of its storage is lost, stations are disabled, and repair needs gold and materials.
  - Pushing the blight back ends the siege.
