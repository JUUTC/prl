# PRL

A high-fantasy, pixel-art roguelike written in Rust with [Bevy](https://bevyengine.org/).

- Seeded worlds with an overworld, towns and many dungeons
- 5 races, 6 classes, classic six stats (point-buy)
- Ultima Online-style use-based skills with **no skill cap**
- Diablo-style loot, identification and full crafting
- Energy-based turns: the world ticks only when you act
- Instanced, resettable dungeons with Dungeon Sigil modifiers
- Player housing, pack animals and a Diablo-style grid inventory
- Permadeath (optional Explorer mode), meta-progression unlocks and endless New Game+

## Design

Start with the [Game Design Specification](docs/SPEC.md).

| Doc | Topic |
|---|---|
| [SPEC.md](docs/SPEC.md) | Overview & core rules |
| [playtest-plan.md](docs/playtest-plan.md) | Staged Windows playtest, acceptance gates and question workflow |
| [foundation-plan.md](docs/foundation-plan.md) | P0/P1 build order, core contracts and full-design compatibility |
| [races-and-classes.md](docs/races-and-classes.md) | Races, classes, starting kits |
| [skills-and-progression.md](docs/skills-and-progression.md) | Skills, gain rules, stat training |
| [abilities.md](docs/abilities.md) | Skill abilities, spells, synergies |
| [items-and-crafting.md](docs/items-and-crafting.md) | Loot, identification, crafting, gathering |
| [combat.md](docs/combat.md) | Always-hit combat, damage types, Power Level, monsters |
| [time-and-turns.md](docs/time-and-turns.md) | Energy scheduler, clock, light, hunger |
| [world-generation.md](docs/world-generation.md) | Seeds, overworld, towns, dungeons, world knobs |
| [housing-and-logistics.md](docs/housing-and-logistics.md) | Grid inventory, pack animals, player housing |
| [endgame.md](docs/endgame.md) | Dungeon instances & sigils, victory, New Game+ |
| [lore.md](docs/lore.md) | Malgareth, the blight, Kaedric's secret |
| [difficulty.md](docs/difficulty.md) | Presets & custom difficulty |
| [meta-progression.md](docs/meta-progression.md) | Feats & unlocks |
| [architecture.md](docs/architecture.md) | Technical architecture |
| [art-production.md](docs/art-production.md) | Art specification, production phases, quality gates and dependencies |
| [style/gravebound-p0.md](docs/style/gravebound-p0.md) | P0 visual brief, candidate palette, construction sheet and authoring proposal |
| [open-questions.md](docs/open-questions.md) | Pending decisions |
| [standards/](docs/standards/) | Coding standards, project structure, extensibility (mods/DLC) |

## Character art studies

**Placeholder experiments only; not release-quality artwork.** The approved
direction and gated production work are tracked in the
[Art Specification & Production Plan](docs/art-production.md).

The [character atelier](tools/character-study/README.md) is a standalone visual
prototype using the approved **Gravebound** baseline, two unlabeled silhouettes,
and five looks per race/silhouette. Starting class kits and per-slot equipment,
item-level, and material previews show how characters change as gear improves.
It includes a gridless three-quarter world preview and independently posed
left/right action animations with hand-aware equipment and two-handed tools.
Representative spell/arrow effects, creatures, townsfolk, harvesting and station
studies accompany optional fog/ash parallax with reduced-motion controls.
The [layered export pilot](tools/art-pipeline/README.md) checks real LibreSprite
source round-trips and per-layer atlas exports independently of placeholder art.

## Building

Requires a recent stable Rust toolchain.

```sh
cargo run -p prl
```

The P1 build uses temporary shapes to demonstrate movement, doors, turns and
fog of war. See [P1 playtest instructions](docs/p1-playtest.md) for controls and
scope. The Windows playtest workflow packages the executable, fixture content
and a diagnostic launcher; its downloadable build requires no Rust installation.

Linux needs: `pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`.

## License

MIT
