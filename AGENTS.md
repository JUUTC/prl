# Contributor and coding-agent rules

Read `docs/SPEC.md` and the affected system document before changing game rules.
Follow `docs/standards/coding.md`, `docs/standards/project-structure.md` and
`docs/standards/extensibility.md`.

For current delivery scope, read `docs/foundation-plan.md` and
`docs/playtest-plan.md`. Track unresolved choices in `docs/open-questions.md`.
Do not treat proposed defaults as accepted interview answers.

Core/generation rules are headless and deterministic. UI and rendering submit
commands and read outcomes; they never write gameplay positions, clocks or RNG.
Use logical persistent IDs, not Bevy Entity IDs, in snapshots.

The animation/UX work has its own gates in `docs/art-production.md` and
`docs/style/gravebound-p0.md`. Gameplay P1 visuals are disposable procedural
placeholders, not completion of art P1. Do not establish a new art style, import
unverified assets, or override those design decisions.

Run targeted tests while iterating. Before handoff:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Build Windows as the initial downloadable playtest target. Never claim a packaged
Windows launch was verified merely because the Linux build or headless tests pass.
