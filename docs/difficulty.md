# Difficulty

Difficulty is chosen at world creation and is separate from world knobs. Players choose a **preset** or **Custom**, which exposes every modifier. Presets fill the Custom fields; editing any field switches the label to *Custom*.

## Presets

| Modifier | Story | Easy | Normal | Hard | Nightmare |
|---|---|---|---|---|---|
| Monster HP | 60% | 80% | 100% | 130% | 170% |
| Monster damage | 50% | 75% | 100% | 130% | 175% |
| Monster accuracy | −20% | −10% | 0 | +10% | +20% |
| Monster danger offset | −2 | −1 | 0 | +1 | +3 |
| Champion/Elite chance | 50% | 75% | 100% | 150% | 250% |
| Overworld encounter rate | 50% | 75% | 100% | 125% | 175% |
| Resist penalty | 0 | 0 | −10% | −30% | −50% |
| Magic find (rarity) | +50% | +25% | 0 | +15% | +40% |
| Gold find | 150% | 125% | 100% | 90% | 75% |
| Healing potency | 150% | 125% | 100% | 85% | 70% |
| Natural regen | 200% | 150% | 100% | 75% | 50% |
| Hunger rate | 50% | 75% | 100% | 115% | 130% |
| Skill gain rate | 150% | 125% | 100% | **150%** | **250%** |
| Shop prices | 75% | 90% | 100% | 115% | 130% |
| Blight spread rate | 25% | 50% | 100% | 150% | 200% |
| Trap frequency | 50% | 75% | 100% | 125% | 150% |
| Sage identify cost | 50% | 75% | 100% | 125% | 150% |
| Permadeath | Off (Explorer) | On | On | On | On |
| Score multiplier | 0.25× | 0.5× | 1× | 1.5× | 2.5× |

Hard and Nightmare increase magic find because tougher monsters drop better loot (the Diablo convention). They also substantially raise **skill gain rate**, so mastering skills on a permadeath character is achievable rather than impossible.

**New Game+** tiers stack an extra difficulty step on top of the chosen preset; see [endgame.md](endgame.md#new-game).

## Custom mode

- Every row above is an individual control (slider or toggle) with the same ranges extended (e.g., Monster HP 25%–300%).
- Additional custom-only toggles:
  - **Ironman** — no quit-and-resume save (one sitting). 
  - **No town services** — no Sage/trainers/bank.
  - **Hardcore AI** — monsters always use optimal abilities/items.
  - **Fog of war reset** — levels forget explored areas on leaving.
- Custom runs show a *Custom* tag and their score multiplier is computed from the settings (capped, and 0 if Explorer).

## Explorer mode

Explorer (no permadeath) can be toggled with any preset. On death: respawn at the last visited town's temple, lose carried gold (dropped as a recoverable grave), −10% durability on gear. Explorer runs are tagged in the morgue/Hall of Records but can earn all feats — see [meta-progression.md](meta-progression.md).
