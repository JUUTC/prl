# Coding Standards

These rules are **mandatory** for humans and AI coding agents. CI enforces most of them. When a rule and convenience conflict, the rule wins; to change a rule, open a PR that edits this file.

## 1. Baseline

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) and the [Rust Style Guide](https://doc.rust-lang.org/style-guide/) (enforced by `rustfmt`).
- Rust **stable**, edition **2024**, MSRV in `Cargo.toml` (`rust-version`).
- Bevy conventions follow [bevy_best_practices](https://github.com/tbillington/bevy_best_practices) and the [bevy_new_2d design patterns](https://github.com/TheBevyFlock/bevy_new_2d/blob/main/docs/design.md), as adapted below.

## 2. Quality gates (CI fails on any of these)

| Gate | Command |
|---|---|
| Format | `cargo fmt --all --check` |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests | `cargo test --workspace` |
| Docs | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |

Lint levels live in `[workspace.lints]` (root `Cargo.toml`): clippy `all` + `pedantic`, `unsafe_code = forbid`, `missing_docs`, and bans on `unwrap`/`expect`/`panic!`/`todo!`/`dbg!`/`print!` outside tests. Locally they are warnings; CI turns them into errors.

**Suppressing a lint** needs `#[expect(clippy::lint_name, reason = "…")]` on the smallest possible item. Never use `#[allow]` without a reason, and never disable a lint crate-wide without changing this document.

## 3. Error handling

- No `unwrap()`, `expect()`, `panic!()`, indexing that can panic on untrusted data, or `unreachable!()` on reachable paths in non-test code.
- Library crates return `Result<T, E>` with **typed errors** (an enum per module that implements `std::error::Error` + `Display`). We will add `thiserror` when the boilerplate grows.
- Bevy systems that can fail return `Result` (Bevy's `BevyError`), or log with `warn!`/`error!` and degrade gracefully. The game must not crash because of bad content data.
- **Content errors** (bad RON, unknown IDs) are reported with the file, pack and ID, and the offending definition is skipped. They are fatal only in `--strict-content` / CI validation.

## 4. Naming

| Item | Convention | Example |
|---|---|---|
| Crates | `prl_` + `snake_case` | `prl_core` |
| Modules / files | `snake_case`, one concept per file | `scheduler.rs` |
| Types, traits, enums | `UpperCamelCase` | `EnergyPool`, `DamageType` |
| Functions, vars | `snake_case`, verbs for functions | `apply_damage` |
| Constants | `SCREAMING_SNAKE_CASE` | `BASE_SPEED` |
| Bevy plugins | `Prl<Area>Plugin` (crate root) or `pub(super) fn plugin(app: &mut App)` (inside crates) | `PrlCorePlugin` |
| Components | Nouns, no `Component` suffix | `Health`, `Speed` |
| Marker components | Nouns or adjectives | `Player`, `Hostile` |
| Events / messages | Past tense | `DamageDealt`, `SkillGained` |
| Resources | Nouns | `WorldSeed`, `TurnClock` |
| System sets | `<Area>Set` enum | `TurnSet::Resolve` |
| Content IDs | `namespace:snake_case` | `prl:goblin_archer` |
| Asset files | `snake_case`, no spaces | `goblin_archer.png` |

Use game-domain terms from the spec consistently (`skill`, `ability`, `synergy`, `area_level`, `power_level`, `tick`, `energy`). Don't invent synonyms.

## 5. Architecture rules

1. **Determinism:** game logic (`prl_core`, `prl_gen`) must give identical results for identical seed + inputs.
   - All randomness comes from seeded RNG streams. Never use `rand::thread_rng`, `HashMap` iteration order, system time, frame time or `f32` accumulation for gameplay decisions.
   - Use `BTreeMap`/`IndexMap` or sorted iteration where order matters.
   - Prefer integer or fixed-point maths for gameplay values. Skills are stored ×10 as integers (`47.3` → `473`).
2. **Logic and presentation are separate.** `prl_core` and `prl_gen` never depend on rendering, UI, audio or input crates. Presentation reads state and sends **commands**; it never mutates rules directly.
3. **Data over code.** Content (monsters, items, skills, abilities, spells, recipes, affixes, feats, difficulty presets) is defined in content packs. Code implements generic **effect primitives** composed by data. A `match` on specific content IDs in Rust is a code smell; use data-driven tags or flags.
4. **Strong IDs.** Persistent references use `ContentId` (definitions) or typed instance IDs (e.g., `ActorId(u64)`), **never** `Entity`, which is session-only.
5. **Plugins everywhere.** Each feature is a plugin and each file registers its own systems, resources and events. The binary only composes plugins.
6. **Bounded systems:** every `Update` system has run conditions (state and/or system set). Turn logic runs only in the turn-resolution state, not per frame.
7. **Entities:** spawn top-level entities with `Name` first and a state-scoped cleanup (`DespawnOnExit`). Use bundle functions (`fn goblin(...) -> impl Bundle`) as templates.
8. **No global mutable state** besides Bevy resources. No `static mut` and no `lazy_static` singletons for game state.
9. **Saves are versioned.** Every serialized type has a stable shape. Changing one requires a migration and a test.
10. **Public API is minimal.** Default to private, then `pub(crate)`; mark `pub` only what other crates need. Re-export through the crate root.

## 6. Documentation

- Every crate has a `//!` crate doc explaining its responsibility and boundaries.
- Every `pub` item has a `///` doc. Docs on fallible functions include an `# Errors` section, and on functions that can panic, `# Panics`.
- Comments explain **why**, not what. Link the governing spec section for game rules (e.g., `// Spec: docs/combat.md#damage-pipeline`).
- Formulas in code must match the spec. If you change a formula, update the spec in the same PR.

## 7. Testing

- Unit tests go next to the code (`#[cfg(test)] mod tests`). Integration tests go in `crates/<crate>/tests/`.
- **Required** for: every formula (gain rules, damage, failure chances), every generator (connectivity and reachability across many seeds), every serialization format (round trip), and every content validator.
- **Determinism tests:** same seed + command list ⇒ same state hash.
- Headless Bevy tests use `MinimalPlugins`. Never open a window in tests.
- Name tests as behaviour sentences: `glancing_blow_halves_damage`.

## 8. Dependencies

- Add workspace-wide dependencies in root `[workspace.dependencies]`; crates use `dep.workspace = true`.
- Before adding a crate, check maintenance, licence (MIT/Apache/BSD/Zlib/CC0/OFL for assets) and compile-time cost. Note new dependencies in the PR description.
- Library crates use Bevy with `default-features = false` and enable only the features they need.
- **CDDA art exception (2026-11-11):** the accepted reuse direction permits a
  reviewed UltiCa CC BY-SA 3.0 subset, with per-file provenance, preserved source
  notices, legal text and adapted-art change notices packaged with the artwork.
  This is not permission to relicense art as MIT, import unreviewed tilesets or
  treat soundpacks as covered. See [the integration plan](../tileset-integration.md).

## 9. Performance

- Avoid per-frame allocation in hot systems. Use change detection (`Changed<T>`, `Ref<T>`).
- FOV, pathfinding and generation operate on dense grids (`Vec` indexed by `y * width + x`), not hash maps.
- Profile before optimizing; add a benchmark when optimizing.

## 10. Git & PRs

- Branch per change. Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat(core): energy scheduler`, `fix(gen): unreachable stairs`, `docs(spec): …`).
- Keep PRs small and focused. Fill in the PR template checklist.
- Commits made by AI agents include the `Co-authored-by` trailer required by the agent's platform.
