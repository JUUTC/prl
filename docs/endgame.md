# Dungeon Instances, Victory & New Game+

## Instanced dungeons

- Every dungeon is an **instance** generated from `hash(world seed, dungeon id, reset count)`.
- The instance's state persists while you come and go: killed monsters stay dead and looted chests stay empty. **Monsters do not respawn inside an instance.**
- **Reset:** at the dungeon entrance, while you are outside the dungeon, you can reset it at any time. The reset count increases and a fresh instance (new layout, monsters and loot) is generated on entry.
- Resetting is how you re-run dungeons for loot, materials, quest targets and skill training.

## Dungeon Sigils (reset modifiers)

When resetting, you may place a **Dungeon Sigil** at the entrance and socket modifiers into it. The sigil and its modifiers are **consumed** by the reset.

- **Sigil quality** (Normal / Magic / Rare / Unique) sets **slot count**: 1 / 2–3 / 4–5 / 6.
- **Modifiers** are runes, gems and special items. The **magnitude** of each effect is rolled within a range that scales with **item quality** and the **dungeon's level**.
- **Risk modifiers** (common) add danger and boost rewards. **Pure bonuses** (rare) only add rewards. A dungeon's total reward bonus is the sum of its modifiers, shown before you confirm.

| Kind | Examples |
|---|---|
| Risk | +Monster HP/damage, +Elite pack chance, Extra boss, Monster +resist, No HP regeneration, Pitch darkness, Blight infested (more undead), Hunted (a unique stalker spawns) |
| Reward | +Item quantity, +Item rarity (magic find), +Item level, +Gold, Extra vault, Family focus (more of a chosen monster family and its drops) |
| Pure bonus (rare) | *Pilgrim's Grace* (+rarity, no penalty), *Cartographer's Ink* (level fully mapped), *Gatherer's Bounty* (+gathering nodes) |

Sigils and modifiers drop from monsters, can be crafted (Inscription for sigils; Enchanting/Alchemy for modifiers) and can be bought in rare shop stock.

## The final dungeon

- The final dungeon is also an instance and can be reset. **Every reset of the final dungeon grows the blight by one step.**
- Defeating **Malgareth the Undying** on the last floor (or on the *Throne of Bone* if the phylactery survives) wins the run.

## Victory

On victory:
1. A **win screen** appears, the morgue is written, and victory feats are awarded.
2. You choose **Retire** (the run ends) or continue into **New Game+**.

## New Game+

- **Endless tiers:** NG+1, NG+2, and so on. Each tier applies:
  - **+1 difficulty step** to monsters (HP, damage, accuracy and Elite chance compound on top of the chosen difficulty).
  - An additional **−10% resistance penalty** (see [combat.md](combat.md#resistances)).
  - A **higher item-level cap** and **new affix tiers / base types** that drop only at that tier or above.
  - **Faster blight spread.**
- **World:** a **new seed world**. By default it is derived from the previous seed and the tier, and the player may enter a custom seed. World knobs can be adjusted.
- **Carried over:** the character (stats, skills, abilities, patron and piety), **all gear and inventory**, the **bank**, pets and pack animals. The contents of every house storage move into the bank as tabs, keeping their names and contents, and each house is refunded as a **deed of the same tier**.
- **Reset:** quests, the map, towns and houses as world objects.
- **Gear scaling:** old gear keeps its stats, so a previous Unique is strong early in the new tier and is gradually outclassed by new-tier drops.
  - **Retempering:** with **Enchanting 100 + the matching craft skill 100** (Blacksmithing, Tailoring or Carpentry), you can raise an item's item level to the current tier's floor. The affix types stay the same and their values are rerolled within the new tier's ranges. It costs tier-appropriate materials and gold.
- Victory and NG+ tier reached are recorded in the Hall of Records.
