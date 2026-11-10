# Playtest Guide (P3 "Explore and loot")

This build tests a seeded two-floor crypt: exploring, fighting, looting, equipping,
using consumables, returning upstairs and retrying after death. It is not the full
game: there are no skills, profile, overworld, towns or campaign. The
sprites are a generated placeholder atlas of the project's own Gravebound
studies; if it fails
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
cargo run -p prl -- --dev-assets
```

The current provisional save slot continues automatically on launch. It saves
after 20 committed player actions, on floor changes and when quitting. A death
barrier prevents the prior living checkpoint from being continued; launch then
creates the next run. On Windows the save is under `%LOCALAPPDATA%\PRL`, on
Linux under `$XDG_DATA_HOME/PRL` or `~/.local/share/PRL`, and on macOS under
`~/Library/Application Support/PRL`. Keep the save file if startup reports
corruption or content incompatibility; it is not silently replaced.

The 20-action interval is still provisional (Q-012). A hard crash can lose
actions since the last checkpoint, and a crash immediately after death resolves
but before its barrier is written can still leave the previous checkpoint.

Death also writes a text report under `character.morgues` in the same PRL
data directory. It records the run/seed, cause of death, floor, elapsed time,
carried equipment, lifetime kills within that run and recent messages, and
remains after retry. Confirming a new run while alive also archives the abandoned
run; a sibling `.abandoned` barrier blocks Continue of that run until replacement
is saved. Legacy saves label kills before migration as unknown. Skills and
profile-wide totals are not implemented; there is no in-game history browser
yet. Archive failures reject retry visibly rather than discard the old run.
If an existing report conflicts
with recovery, preserve that file and the `.dead` barrier and report the error;
the game will not silently overwrite it or resume the dead character.

Linux development also needs the system packages listed in the repository README.
Windows is the first downloadable target. A Linux build working does not prove
the Windows build works.
The development flag opts into checkout assets only when no `assets` directory
exists next to the executable. Packaged launches omit it and require adjacent
assets, regardless of build profile or current working directory. Missing or
malformed adjacent content produces the boot-error screen, never a checkout
fallback.

## Goal

You start at `<` on the upper floor with supplies at your feet. Press `G`
seven times to pick them up, or open `I` and use the ground panel's Pick up.
Select and equip sword/mail with `E` or the labelled action button. Inventory
stays open after safe actions; it closes after an action if a hostile is visible
or you take damage. Supplies include three healing potions and
two Identify scrolls; their descriptions are unknown until used.
The crypt has enemies with different speeds:

- Shambler: slow (fallback shape: a green wide block).
- Skeleton: normal speed (fallback: an orange tall block).
- Grave rat: fast (fallback: a pink diamond).

You are the figure with a shield and coif. Your character faces left or right
from the horizontal part of each move or attack; straight up or down keeps the
last facing. Creatures have independent left/right idle, walk and attack clips.

Explore toward `>`; some corridors have closed doors (`O`, then direction).
Both exits contain Magic gear. In inventory, `C` offers an explicit known
Identify source, followed by review. Starting scrolls are unknown; the UI
deliberately does not guess their hidden type. General unknown-scroll Read is
not yet available, so fresh runs currently have no usable known source.
Enter at the upper `>` descends; comma at the lower
`<` returns, preserving kills, loot and doors. Floor one's `<` has no outside
destination. Defeat the lower-floor skeleton guardian, reach its `>` and press
Enter to clear the crypt. This is a playtest endpoint, not campaign victory.
If you die, `R` starts a fresh crypt at the next seed.
The depicted shield is placeholder art, not a working shield/block mechanic.

## Controls

| Control | Action |
|---|---|
| Arrows, numpad 1-9, vi `ykuhlbjn` | Move; bump an enemy to attack it (numpad and vi keys include diagonals) |
| `.` / numpad 5 / Space | Wait one turn |
| F2 / F3 / F4 | Toggle held movement/wait / cycle delay / cycle repeat interval (shown in help/menu) |
| F5 / F6 | Guided movement/wait remapping / restore default bindings (session only) |
| `O`, then a direction | Open an adjacent door |
| `G` | Pick up one item at your feet (50 energy) |
| `I`; arrows or click | Open inventory workspace; select an item to inspect (free) |
| Tab / Shift+Tab; Enter | Focus previous/next inventory control; activate it |
| `E` / `D` in inventory | Equip/unequip / drop selected item; normal energy costs |
| `Q` / `C` in inventory | Drink selected potion / choose a known Identify source |
| `G` in inventory | Pick up the selected ground item; does not equip |
| Grid/List; Options | Free view switch; persistent inventory warning controls |
| `X`; arrows/vi | Examine a selected cell without taking a turn; Esc closes |
| Enter at `>` / comma at `<` | Descend or finish / ascend (100 energy) |
| `V` | Show all ground labels, bypassing the Default filter (free) |
| `R` twice within 2 s, or `R` then `Y` | Start a new run while alive (any other key cancels) |
| `R` once | Start a new run after death |
| `T` | Motion speed: Normal, Fast or Off (visual only) |
| `M` / `9` / `0` | Mute sound, or turn the volume down/up in 10% steps |
| F1 / `?` | Help, including the art legend |
| Esc | Cancel the inventory review/chooser first, then close; otherwise pause/close overlay |
| PgUp / PgDn in inventory | Scroll details (Shift: backpack); narrow layouts scroll stacked content |
| Mouse wheel | Scroll the inventory pane under the pointer |
| PgUp / PgDn in help/examine | Scroll long text (free) |
| `+` / `-` | World zoom, 1-4x in whole steps |
| `[` / `]` | UI scale, separate from world zoom |

Each key press does at most one thing. Holding movement/wait repeats after
500 ms, then every 180 ms by default. It stops at danger, HP at or below 25%,
loot, doors, stairs, rejection, focus loss or any new key. It never repeats
attacks; release and press again after a stop. F5 asks for eight directions
clockwise from North, then Wait. Use unique arrows, numpad, Space, period or
unreserved letters; reserved/duplicate keys show an error. Esc cancels without
changing the bindings. Timings and remapping reset on relaunch.
Time only passes when you act. Help, the pause menu and an unfocused window stop
the clock. After you die, only new run, help, menu, zoom and settings keys work.

Inventory reviews default on for dropping, unknown consumables and slow Body
changes in danger. Uncheck **Warn me next time** to disable just that category;
Inventory **Options** can re-enable it. These flags survive runs/restarts in
`character.warnings.ron`, separate from the character save. A visible **NOT
SAVED** or options error means persistence failed; keep the file and report it.
Backpack placement is currently read-only: free repacking, rotation and explicit
Organize are approved follow-up work, not functional controls yet.

## What you see

- The top-left panel shows the run number, seed, HP, turns, the clock, and the
  current motion/sound settings, floor and carried weight.
- The top-right panel lists the enemies you can see, with their HP and speed
  (slow, normal or fast), followed by up to six visible ground labels.
- `<`/`>` mark stairs; `*` marks visible loot cells. Inventory shows real 10x6
  footprints, a List alternative, equipment, current-cell ground and persistent
  details/comparison. Placeholder glyphs are SW/AR/P/S; `?` means unidentified.
  Unknown affixes remain hidden in names and comparisons.
- The bottom-left log keeps the last five messages. Each hit lists its raw
  damage and how much armor removed, so you can see why HP changed.
- Attackers lunge toward their target, and the target flashes (red on sprites). A number floats
  up from the target: `-6` for a normal hit, `CRIT -11` for a critical hit,
  `glance -2` for a glancing blow. Hits on you show in red.
- A red bar under an actor shows its HP once it is hurt. When you die, your
  figure (or square) turns grey and a message tells you how to start a new run.

Walking slides between cells for 150 ms (Fast: 60 ms), with its walk clip,
camera and HP bar following the same cosmetic base. Floor changes and new runs
snap. Lunges, flashes, walking and numbers never hold up input; new actions
replace rather than queue walking. With motion set to Off, numbers appear briefly
without moving and sprites hold a still frame. Fast, Normal and Off must play out identically; only the visual
timing differs.

## What to try

1. Equip entrance supplies, explore both floors, loot/compare Magic gear and
   return upstairs. Are kills, doors and looted cells still as you left them?
   Can you find the guardian and understand the endpoint message?
2. Fight each speed of enemy. Can you tell them apart at a glance? Does the
   fast rat feel fair, or does it get free hits you could not see coming?
3. Die on purpose. Is it clear that you died and what killed you? Is it clear
   how to start again? Does a stray `R` mid-fight cost you a run? It should not.
4. Drink an unknown potion when hurt (`I`, select it, `Q`); its type becomes known.
   At full HP it should reject without consuming it. Body equip/unequip costs
   three standard turns, so avoid changing armor while surrounded.
5. Change motion speed (`T`) and sound (`M`, `9`, `0`) during a fight. Does Fast
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
the actors/terrain. Required `data/dungeons/crypt_playtest.ron` supplies two floor
settings, encounters, items and Default filter. This is not the full mod/DLC manifest described
in `docs/standards/extensibility.md`. Unknown or invalid required content must
fail with an error; there is no silent fallback when a packaged file is broken.

Combat numbers are provisional (`docs/combat.md`). Shield block and facing wait
on Q-009. Gear bonuses now derive through one core function; actor base numbers
remain provisional until P4 stats/skills (Q-019). Sounds are short waveforms
generated in code at startup; with no audio device the game runs silently.
Snapshots store logical state, independent of Bevy Entity allocations. Text
death/abandonment morgues and per-run kill counters are implemented; skills,
settings/victory history, the history browser, profile, general
world/reset identities and NG+ are later work.
Inactive crypt floors are frozen with no catch-up or respawning. The slice uses
a temporary 60 kg hard pickup cap, no stacking/repacking, and only Main hand/Body
slots; failed transfers never lose items. The repository's
`docs/items-and-crafting.md` records the full subset limits.
Windows launch/audio and 4K/UI-scale hands-on acceptance remain unverified.
