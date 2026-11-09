# Contributor and coding-agent rules

Read `docs/SPEC.md` and the affected system document before changing game rules.
Follow `docs/standards/coding.md`, `docs/standards/project-structure.md` and
`docs/standards/extensibility.md`.

For current delivery scope, read `docs/foundation-plan.md` and
`docs/playtest-plan.md`. Track unresolved choices in `docs/open-questions.md` and actionable work
and defects in `docs/backlog.md` (open items `I-NNN`, bugs `B-NNN`).
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

## Agent environment disk

Bevy builds need several GB; agent machines have small disks and a full disk
breaks the session itself. Before the first Rust build in a session:

- Run `tools/agent-env/free-disk.sh` to remove preinstalled tools PRL never
  uses (CodeQL, .NET, Android, Haskell, Swift, Julia, Java, ...). It is safe
  to re-run and keeps Rust, Python, Node, git, Chrome and the Bevy libraries.
  `.github/workflows/copilot-setup-steps.yml` runs it for Copilot cloud agent
  sessions automatically.
- Check `df -h /` before the full gates; run `cargo clean` if less than 3 GB is
  free. Do not add removals for tools a workflow or test still needs.

## Windows target

Build Windows as the initial downloadable playtest target. Never claim a packaged
Windows launch was verified merely because the Linux build or headless tests pass.
