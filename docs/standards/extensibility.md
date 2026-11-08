# Extensibility: Content Packs, DLC & Mods

**Goal:** add races, classes, skills, monsters, items, dungeons, quests, sprites and sounds **without changing engine code**. The base game, first-party DLC and community mods all use one mechanism: **content packs**.

## Decisions

| Topic | Decision |
|---|---|
| v1 modding | **Data-only content packs**: RON definitions plus assets |
| DLC | First-party content packs (same format), possibly gated by ownership on the distribution platform |
| Scripting | **Deferred.** The architecture leaves room for sandboxed **WASM** behaviour hooks later; no Lua |
| IDs | Namespaced `ContentId`: `namespace:name` (`prl:skeleton_warrior`) |

## Pack layout

```
<pack root>/                     # assets/packs/<namespace>/ or <user data>/mods/<namespace>/
├── pack.ron                     # manifest
├── data/
│   ├── races/*.ron  classes/*.ron  skills/*.ron  abilities/*.ron  spells/*.ron
│   ├── monsters/*.ron  items/*.ron  affixes/*.ron  recipes/*.ron
│   ├── dungeons/*.ron  biomes/*.ron  quests/*.ron  feats/*.ron  difficulty/*.ron
│   └── patches/*.ron            # modifications to other packs' definitions
├── sprites/  audio/  locale/
└── LICENSE
```

### Manifest (`pack.ron`)

```ron
(
    namespace: "prl",                // unique, [a-z][a-z0-9_]*
    name: "PRL Base Game",
    version: "0.1.0",                // semver of the pack
    engine: ">=0.1, <0.2",           // compatible game versions (semver req)
    format: 1,                       // content schema version
    kind: Base,                      // Base | Dlc | Mod
    depends: [],                     // [("prl", ">=0.1")]
    load_after: [],                  // soft ordering hints
    authors: ["…"],
    license: "MIT / CC-BY-4.0 …",
)
```

## Loading rules

1. Discover packs: base (`assets/packs/prl`), then installed DLC, then enabled mods in the user's mod folder.
2. Resolve **load order**: dependencies first, then `load_after` hints, then alphabetical by namespace (deterministic).
3. Each pack may **define** new IDs only inside its own namespace.
4. Packs modify other namespaces only through **patches** (`data/patches/`), declared operations on existing IDs (`Override`, `Merge`, `Remove`, `AppendTo(list)`). A patch on an ID that doesn't exist is a warning, not a crash.
5. After loading, a **validator** checks references (every `ContentId` resolves), ranges and schema. Errors name the pack, file and ID.
6. The active pack list (namespace + version) is **stored in every save and in the world seed descriptor**. Loading a save with missing or changed packs shows a clear warning, and the same seed with different packs is a different world.

## Designing for extension

- **Registries, not enums,** for content: `Registry<MonsterDef>` keyed by `ContentId`. Rust enums are only for closed engine concepts (e.g., `DamageType`, `EquipSlot`).
- **Effect primitives:** abilities, items, spells and affixes are compositions of engine-provided effects (`Damage`, `Heal`, `ApplyStatus`, `Summon`, `Teleport`, `ModifyStat`, `SpawnTerrain`, …) with conditions and scaling expressions defined in data. Adding a new primitive is an engine change; adding content is not.
- **Tags over special cases:** `tags: ["undead", "blunt_weak"]`. Logic checks tags, never specific IDs.
- **Generator hooks:** dungeon themes, biomes, prefab vaults and sigil modifiers are data-registered, so packs add new ones.
- **Localization-ready:** player-facing text goes in locale files keyed by ID (Fluent `.ftl` planned). Definitions reference keys, not literal prose.
- **Future WASM hooks:** events (`on_hit`, `on_turn`, `on_enter_level`…) and effect primitives are designed as a stable, serializable interface so a sandboxed WASM runtime can implement custom effects later without changing call sites.

## Versioning & compatibility

- `format` is bumped on breaking schema changes. The loader supports at least the current and previous formats, with automatic upgrade.
- Base-game `ContentId`s are **never renamed or removed** after release. Use `aliases.ron` to map old IDs.
