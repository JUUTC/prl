# Backlog: Open Items and Bugs

Running list of work that must not be lost between sessions. Design decisions
that need an answer belong in [open-questions.md](open-questions.md) (`Q-NNN`);
this file tracks **actionable work** (`I-NNN`) and **defects** (`B-NNN`).

## How to use this list

- Add an entry as soon as work is discovered; do not wait for a stage review.
- Keep IDs stable. Mark entries **Done** or **Fixed** with the commit instead of
  deleting them. Move closed rows to the closed tables at stage handoff.
- Link related `Q-NNN` IDs and stages from [playtest-plan.md](playtest-plan.md).
- Bugs record how to reproduce the problem and what was expected.

Status values: **Open**, **In progress**, **Blocked** (name the blocker),
**Done** / **Fixed** (name the commit), **Won't do** (give the reason).

## Open items

| ID | Item | Stage | Status | Notes / related |
|---|---|---|---|---|
| I-001 | Hands-on Windows launch of the packaged CI artifact | P0/P1 | Open | Needs the user on Windows; record build ID and hardware in playtest-plan.md. Q-013 |
| I-002 | 4K and multiple UI scales readability check | P1 | Open | Needs a 4K display; Q-013 |
| I-003 | Implement P2 combat pipeline, enemy AI, death and retry | P2 | In progress | Core rules done (combat, AI, death, new run, content-defined arena); combat presentation in UI/render pending. Q-009 blocks shield block/facing only |
| I-004 | P2 placeholder attack/UI sound effects with mute and volume | P2 | Open | Only self-generated or licence-verified audio |
| I-005 | Collect P2 hands-on findings (targeting, input, camera, readability, waiting) | P2 | Open | Record in playtest-plan.md before polishing |
| I-007 | Retune the P2 arena from hands-on results | P2 | Open | Provisional: holding the door wins ~78% of seeds 1-50, charging ~4% (ignored balance probe in prl_core). Q-004/Q-019 |
| I-006 | Decide the challenge model for combat skill gain | P4 | Blocked | Q-010 |

## Bugs

| ID | Bug | Status | Reproduce / expected | Found in |
|---|---|---|---|---|
| B-003 | HUD message-log panel covers the bottom-left of the map at the default window size | Open | Walk to the lower-left of the room at 1× zoom: tiles sit under the log panel. Expected: the map stays readable, e.g. the camera keeps the player clear of HUD panels or the panel is translucent/collapsible | P1 Linux screenshots |

## Closed

| ID | Entry | Resolution |
|---|---|---|
| B-001 | HUD, help and pause text showed a missing-glyph box instead of an em dash (Bevy's built-in font lacks the glyph) | Fixed in 753ea3d by using ASCII in placeholder text |
| B-002 | Message log showed internal sequence numbers that skipped because unseen NPC actions were counted | Fixed: the log shows message text only; sequence stays internal for ordering |
