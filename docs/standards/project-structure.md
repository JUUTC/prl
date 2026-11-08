# Project Structure

PRL is a **Cargo workspace** with a thin binary and focused library crates, following standard Rust layout (`src/lib.rs`, `src/main.rs`, `tests/`, `benches/`, `examples/`).

```
prl/
├── Cargo.toml                 # virtual workspace: members, shared deps, lints, profiles
├── rust-toolchain.toml        # stable + rustfmt + clippy
├── rustfmt.toml  clippy.toml  .editorconfig  .gitattributes
├── AGENTS.md                  # rules for AI coding agents (canonical)
├── .github/
│   ├── copilot-instructions.md
│   ├── instructions/          # path-specific agent rules (*.instructions.md)
│   ├── skills/                # agent skills (<name>/SKILL.md)
│   ├── workflows/ci.yml
│   └── pull_request_template.md
├── crates/
│   ├── prl/                   # binary: composes plugins only
│   ├── prl_content/           # content packs, ContentId, definitions, loading, validation
│   ├── prl_core/              # headless deterministic rules
│   ├── prl_gen/               # seeded procedural generation
│   ├── prl_save/              # saves, checkpoints, profile, morgue
│   ├── prl_render/            # tilemap, sprites, lighting, camera, animation
│   ├── prl_ui/                # theme, widgets, HUD, menus, screens (bevy_ui)
│   └── prl_audio/             # SFX (v1), music (later)
├── assets/                    # Bevy asset root
│   ├── packs/
│   │   └── prl/               # base game content pack (namespace `prl`)
│   │       ├── pack.ron
│   │       ├── data/          # RON definitions by kind (monsters/, items/, skills/ …)
│   │       ├── sprites/       # 64×64 PNGs by category
│   │       ├── audio/
│   │       └── locale/        # en.ftl (later)
│   ├── fonts/                 # OFL fonts + their licences
│   ├── palette/               # master palette (.hex/.gpl/.png)
│   ├── shaders/
│   └── CREDITS.md             # every third-party asset, licence, source
├── docs/
│   ├── SPEC.md …              # game design
│   ├── standards/             # coding, structure, extensibility
│   └── style/                 # art bible, UX guide, writing guide
└── tools/                     # dev scripts (palette check, content validator) — later
```

## Crate dependency rules

```
prl_content  ◄── prl_core ◄── prl_gen
     ▲              ▲  ▲
     │              │  └── prl_save
     ├── prl_render ┤
     ├── prl_ui ────┤
     │              └── prl_audio
     └──────────── prl (binary) depends on all
```

- Arrows point at dependencies. **No cycles. No upward dependencies.**
- `prl_core` and `prl_gen` must **never** depend on `prl_render`, `prl_ui`, `prl_audio`, or Bevy's render/window/audio features.
- Presentation crates talk to core through **commands** (intents in) and **events/messages** (outcomes out).
- New crates need a reason (a clear boundary, compile-time win, or optional feature) and an update to this file.

## Inside a crate

```
crates/prl_core/src/
├── lib.rs            # crate docs, PrlCorePlugin, pub use re-exports
├── scheduler/        # a feature folder when it has >1 file
│   ├── mod.rs        # pub(super) fn plugin(app: &mut App)
│   ├── energy.rs
│   └── clock.rs
├── combat.rs         # a single-file feature
└── skills.rs
```

- Use one feature per module, and register the module's plugin from its parent.
- Use `mod.rs` for feature folders; don't mix the `foo.rs` + `foo/` style within a crate.
- Test helpers go in a `#[cfg(test)]` module or the `tests/common/mod.rs` of integration tests.

## Assets

- Asset paths are `snake_case` and grouped by category: `sprites/monsters/undead/skeleton_warrior.png`.
- Source art files (`.aseprite`) live next to exports in `art_src/` (later; possibly Git LFS).
- Every third-party asset is listed in `assets/CREDITS.md` before it is merged.
