# P1 Placeholder Playtest

This build exercises the movement foundation, not the full game. There is no
combat, loot, procedural dungeon, disk save or campaign. Visuals are temporary
repository-owned shapes; the separate animation/UX work remains authoritative
for the eventual style.

## Running

On Windows, extract the entire downloaded artifact into a writable folder and
run `Launch.cmd`. Do not run the executable from inside the ZIP. The launcher
writes diagnostic output to `playtest.log`; include that file and `BUILD.txt`
when reporting a launch failure.

For local development:

```sh
cargo run -p prl
```

Linux development also needs the system packages listed in the repository README.
Windows is the initial downloadable target; a successful Linux build does not
establish Windows playtest compatibility.

## What to check

Move around the fixture, inspect doors, wait, explore the fog of war, and try the
HUD/help and independent world/UI scaling controls shown in the running build.
The orange test actor only waits; it is not an enemy yet.

| Control | Action |
|---|---|
| Arrows, numpad 1-9, vi `ykuhlbjn` | Move (numpad/vi include diagonals) |
| `.` / numpad 5 / Space | Wait |
| `O`, then a direction | Open an adjacent door |
| `R` | Start a new run with the next seed (abandons the current one) |
| F1 / `?` | Help |
| Esc | Pause/menu, or dismiss an overlay |
| `+` / `-` | Integer world zoom, 1-4x |
| `[` / `]` | Independent UI scale |

Each key press submits at most one action; holding keys does not auto-repeat.

The clock must stop while idle, reading help or unfocused. Rejected movement must
not advance the clock. Diagonal movement cannot squeeze past a blocked corner.
Opening a door should change both movement and visibility.

Report control confusion, unreadable text, hidden actors shown through fog,
incorrect turn counts, or launch errors. Record build ID, Windows version,
display resolution and zoom/UI scale with the reproduction steps.

The snapshot proof is a headless test; this build does not offer a disk save or
continue button.

## Implementation boundary

`assets/packs/prl/base.ron` contains the validated tile and actor definitions for
this fixture. It is a small P1 content schema, not the full mod/DLC manifest
described in `docs/standards/extensibility.md`. Unknown or invalid required content
must fail explicitly; there is no silent fallback when a packaged file is broken.

The map is fixed, the seed is a fixture identifier, and no random gameplay is
implemented yet. Generation/RNG stream selection remains tracked in Q-016.
P1's snapshot stores logical state independently of Bevy Entity allocations.
Full save recovery, world instances and NG+ are still later work.
