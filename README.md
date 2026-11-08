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
| [open-questions.md](docs/open-questions.md) | Pending decisions |
| [standards/](docs/standards/) | Coding standards, project structure, extensibility (mods/DLC) |

## Building

Requires a recent stable Rust toolchain.

```sh
cargo run
```

Linux needs: `pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`.

## License

MIT
