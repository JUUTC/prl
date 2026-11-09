# Playtest Guide (P2 "Fight and retry")

This build tests one question: is a fight in the small arena readable and fair,
and is it quick to try again after dying? It is not the full game. There is no
loot, inventory, procedural dungeon, disk save or campaign. The sprites are a
generated placeholder atlas of the project's own Gravebound studies; if it fails
to load, the game draws coloured shapes instead. Sprites, shapes, colours, the
font and the sounds are all temporary placeholders. The separate
art and animation effort still decides the final look.

## Running

On Windows, extract the whole downloaded artifact into a writable folder and
run `Launch.cmd`. Do not run the executable from inside the ZIP. The launcher
writes diagnostic output to `playtest.log`. Include that file and `BUILD.txt`
when you report a launch failure.

For local development:

```sh
cargo run -p prl
```

Linux development also needs the system packages listed in the repository README.
Windows is the first downloadable target. A Linux build working does not prove
the Windows build works.

## Goal

You start in a small room. A closed door is to the east. Behind it are three
hostile enemies, each moving at a different speed:

- Shambler: slow (fallback shape: a green wide block).
- Skeleton: normal speed (fallback: an orange tall block).
- Grave rat: fast (fallback: a pink diamond).

You are the figure with a shield and coif. Your character faces left or right
from the horizontal part of each move or attack; straight up or down keeps the
last facing. Creatures have only one placeholder facing so far.

Survive and kill them. If you die, start a new run straight away. Each new run
uses the next seed.

## Controls

| Control | Action |
|---|---|
| Arrows, numpad 1-9, vi `ykuhlbjn` | Move; bump an enemy to attack it (numpad and vi keys include diagonals) |
| `.` / numpad 5 / Space | Wait one turn |
| `O`, then a direction | Open an adjacent door |
| `R` twice within 2 s, or `R` then `Y` | Start a new run while alive (any other key cancels) |
| `R` once | Start a new run after death |
| `T` | Motion speed: Normal, Fast or Off (visual only) |
| `M` / `9` / `0` | Mute sound, or turn the volume down/up in 10% steps |
| F1 / `?` | Help, including the art legend |
| Esc | Pause menu, or close an overlay |
| `+` / `-` | World zoom, 1-4x in whole steps |
| `[` / `]` | UI scale, separate from world zoom |

Each key press does at most one thing. Holding a key does not repeat it. Time
only passes when you act. Help, the pause menu and an unfocused window all stop
the clock. After you die, only new run, help, menu, zoom and settings keys work.

## What you see

- The top-left panel shows the run number, seed, HP, turns, the clock, and the
  current motion and sound settings.
- The top-right panel lists the enemies you can see, with their HP and speed
  (slow, normal or fast).
- The bottom-left log keeps the last five messages. Each hit lists its raw
  damage and how much armor removed, so you can see why HP changed.
- Attackers lunge toward their target, and the target flashes (red on sprites). A number floats
  up from the target: `-6` for a normal hit, `CRIT -11` for a critical hit,
  `glance -2` for a glancing blow. Hits on you show in red.
- A red bar under an actor shows its HP once it is hurt. When you die, your
  figure (or square) turns grey and a message tells you how to start a new run.

Lunges, flashes and numbers never hold up input. A new key press cuts short any
effect that is still playing. With motion set to Off, numbers appear briefly
without moving and sprites hold a still frame. Fast, Normal and Off must play out identically; only the visual
timing differs.

## What to try

1. Hold the door. Open it, then step back and wait so enemies reach you one at a
   time. Then try charging through the doorway. Which works better, and why?
2. Fight each speed of enemy. Can you tell them apart at a glance? Does the
   fast rat feel fair, or does it get free hits you could not see coming?
3. Die on purpose. Is it clear that you died and what killed you? Is it clear
   how to start again? Does a stray `R` mid-fight cost you a run? It should not.
4. Change motion speed (`T`) and sound (`M`, `9`, `0`) during a fight. Does Fast
   or Off feel better? Are the sounds useful or irritating?

## What to report

Record your findings for backlog item I-005 in `docs/playtest-plan.md`, with
the build ID, Windows version, display resolution and zoom/UI scale, and the
steps to reproduce. We are especially interested in:

- Targeting and input: wrong-direction attacks, bumps you did not intend,
  missed or doubled key presses.
- Camera and HUD: the player or enemies hidden under a panel, text you cannot
  read, numbers that overlap.
- Readability: whether you understood each HP change from the numbers and the
  log.
- Waiting and pacing: whether holding the door felt like the right choice, and
  whether the speeds felt different.
- Feedback: whether any effect or sound felt too slow, too fast or too loud.
- Launch errors or crashes, with `playtest.log` attached.

## Implementation boundary

`assets/packs/prl/base.ron` holds the validated tile and actor definitions for
the arena. It is a small content schema, not the full mod/DLC manifest described
in `docs/standards/extensibility.md`. Unknown or invalid required content must
fail with an error; there is no silent fallback when a packaged file is broken.

Combat numbers are provisional (`docs/combat.md`). Shield block and facing wait
on Q-009. Gear-derived stats come later (Q-019). Sounds are short waveforms
generated in code at startup; with no audio device the game runs silently.
Snapshots store logical state, independent of Bevy Entity allocations. Full
save recovery, world instances and NG+ are later work.
