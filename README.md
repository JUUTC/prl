# PRL

A high-fantasy, pixel-art roguelike written in Rust with [Bevy](https://bevyengine.org/).

- Seeded worlds with an overworld, towns and many dungeons
- 5 races, 6 classes, classic six stats (point-buy)
- Ultima Online-style use-based skills with **no skill cap**
- Diablo-style loot, identification and full crafting
- Energy-based turns: the world ticks only when you act
- Permadeath (optional Explorer mode) with meta-progression unlocks

## Design

Start with the [Game Design Specification](docs/SPEC.md).

| Doc | Topic |
|---|---|
| [SPEC.md](docs/SPEC.md) | Overview & core rules |
| [races-and-classes.md](docs/races-and-classes.md) | Races, classes, starting kits |
| [skills-and-progression.md](docs/skills-and-progression.md) | Skills, gain rules, stat training |
| [abilities.md](docs/abilities.md) | Skill abilities, spells, synergies |
| [items-and-crafting.md](docs/items-and-crafting.md) | Loot, identification, crafting, gathering |
| [time-and-turns.md](docs/time-and-turns.md) | Energy scheduler, clock, hunger |
| [world-generation.md](docs/world-generation.md) | Seeds, overworld, towns, dungeons, world knobs |
| [difficulty.md](docs/difficulty.md) | Presets & custom difficulty |
| [meta-progression.md](docs/meta-progression.md) | Feats & unlocks |
| [architecture.md](docs/architecture.md) | Technical architecture |
| [open-questions.md](docs/open-questions.md) | Pending decisions |

## Building

Requires a recent stable Rust toolchain.

```sh
cargo run
```

Linux needs: `pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`.

## License

MIT
