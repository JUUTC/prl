# Basic Playtest Plan

> Draft, 2026-11-08. This is a delivery plan, not a replacement for
> [SPEC.md](SPEC.md). Stage status is **planned**, not implemented, unless evidence
> below says otherwise. Decisions and unresolved questions live in
> [open-questions.md](open-questions.md).

## Goal and baseline

Deliver a Windows build that can be downloaded, extracted and played without
installing Rust. First prove that moving, exploring and fighting feel readable;
then build a repeatable 10-15 minute dungeon run with loot and save/resume.

The repository now has an initial P1 implementation: a fixed logical map,
validated tile/actor fixture content, command/energy scheduling, recursive
shadowcasting and memory, placeholder rendering, HUD/input and a versioned RON
snapshot proof. A Windows packaging workflow is included. There is no combat,
generation, production pack manifest or disk save yet; Windows hands-on
acceptance remains pending. See [p1-playtest.md](p1-playtest.md).

The recorded UX decisions are 64x64 world tiles, integer zoom, independent UI
scaling, `bevy_ui`, Pixelify Sans/Silkscreen, and cosmetic movement tweening.
The concurrent animation/UX session is now recorded in
[art-production.md](art-production.md) and [the Gravebound reference](style/gravebound-p0.md).
Q-006 records that integration. Gameplay P0-P5 and art P0-P5 are separate gates;
placeholder movement code does not approve final sprites, geometry or timings.
The defaults below remain proposals for the slice, not recovered timing answers.

## Scope of the first complete slice

| Include | Defer without changing the full-game spec |
|---|---|
| One prebuilt Human Warrior; visible seed and Normal difficulty | Full race/class selection, point-buy UI, other difficulty presets |
| Eight-direction movement, wait, doors, stairs, look/examine | Auto-explore, click-to-travel and long-distance travel |
| One two-floor seeded crypt; three enemy types including a final guardian | Overworld generation, towns, other dungeon themes, 60-80 monster bestiary |
| Physical melee, glance/block/crit/armor, simple enemy AI | Ranged combat, magic, elemental statuses and complex AI |
| Light radius, FOV and remembered terrain | Day/night, dynamic fire and shader polish |
| 10x6 grid inventory, weight, equipment, potion, Normal/Magic gear, Identify scroll | Bags, full affix catalogue, crafting, vendors, pack animals and houses |
| Swords/Tactics gains, skill display, Power Level and a tested unlock | All skills, synergies and long-run balance |
| Quit-save, overwritten checkpoint, permadeath, morgue and retry | Explorer respawn, profile feats, Malgareth, blight and NG+ |
| Minimal data-driven base pack, theme, licensed placeholders and basic SFX | Mod browser, complete patch operations, scripting, original art and music |

The guardian is a **playtest endpoint**, not Malgareth or a full-game victory.
Completing the slice shows a summary and allows a new run; it does not award
campaign victory feats. Hunger is off using the spec's existing world knob.
Features not implemented must be hidden or labelled unavailable, never presented
as functional controls.

## Stages and acceptance gates

The detailed implementation contracts, prerequisites and extension checks through
P1 are in [foundation-plan.md](foundation-plan.md). That plan starts a minimal
tile/actor content loader before P1; P3 expands it to the dungeon/loot definitions.

Stages are ordered by prerequisites, not estimated dates. Each stage ends in a
launchable build; do not accumulate several systems before testing interaction.

| Stage | Prerequisites | Main owners | Deliverable | Exit gate |
|---|---|---|---|---|
| P0 - Launch and distribute | None | `prl`, `prl_ui`, `prl_render` | Windows package with camera, test room and boot/error screen | Fresh extraction launches without Rust; assets resolve outside the repo; build identity and log location visible |
| P1 - Walk and see | P0 | `prl_core`, `prl_render`, `prl_ui` | Fixed test room, scheduler, movement, doors, FOV, light and HUD | Idle/menu time freezes rules; speed/action-cost tests pass; movement/zoom/input smoke test passes |
| P2 - Fight and retry | P1 | `prl_core`, `prl_content`, `prl_ui`, `prl_audio` | Small combat arena with three enemies, death and retry | At least three short runs completed; damage log agrees with rules; fast/normal animation produce identical outcomes |
| P3 - Explore and loot | P2 | `prl_gen`, `prl_content`, `prl_core`, `prl_ui` | Two-floor seeded crypt, persistent instance, inventory and endpoint | 100-seed connectivity check passes; same seed/content/commands reproduce state; loot/equip/identify loop works |
| P4 - Grow and resume | P3 | `prl_core`, `prl_save`, `prl_ui` | Skill gain, unlock, quit-save, checkpoint and morgue | Save/resume reproduces state; dead saves cannot reload; skill gain tests cover real challenge and no-gain cases |
| P5 - Basic playtest candidate | P4 | All existing crates | Packaged 10-15 minute run, help/options, regression fixes | Two end-to-end runs plus death/resume scenarios pass on Windows; no known progression or data-loss blocker |

**First external hands-on build: P2.** Loot, full art and progression must not delay
learning whether movement and combat feel good. **Complete basic slice: P5.**

### P0 - Launch and distribute

Build only **Windows x86_64** as the initial playable target, per the iteration
discussion. Linux/macOS remain supported-platform goals; their playable build
matrix is deferred, not removed from the spec.

Use a native Windows runner for distribution. Package `prl.exe`, assets, licences,
build/commit identifier and a short launch/readme file as a downloadable ZIP.
Test from a fresh folder whose working directory is not the repository. Surface
missing assets and unsupported graphics explicitly, with an actionable log.

Native Windows `cargo run -p prl` is the local playtest loop. Use debug builds
locally; measure a small playtest profile for downloadable builds. Benchmark
dynamic linking/linker changes before adopting them (**Q-011**); distributed
builds must not require a development DLL. Avoid repeated full release builds or
building all three platforms for each small change.

### P1 - Walk and see

Implement stable actor ordering and command validation before animation. Cover
speed 50/100/150/200 and 50/100/150-energy actions, including multiple actions per
tick and carried energy. Resolve diagonal corner rules through **Q-008**.
Rejected commands must explain why and use the documented no-time default.

Show HP, stamina, seed, message log and a turn/clock indicator. Browsing inventory,
look mode and settings takes no turns; equipping and consuming later do. Losing
window focus or opening help must not commit buffered actions.

Render 64x64 tiles with nearest filtering/integer world zoom and independent UI
scale. Use licensed DCSS CC0 placeholders at 2x when verified; repository-owned
temporary shapes can unblock P0/P1 without implying an art-direction change.
Do not wait for the final palette or titles to implement the shared theme.

### P2 - Fight and retry

Use one slow melee enemy, one normal melee enemy and one faster enemy to expose
scheduler errors. Add awareness/chase/attack states; complex noise, factions and
pack tactics are deferred. Use the physical pipeline in
[combat.md](combat.md), resolving facing/block details through **Q-009**.

Display attack, damage, glance, block, critical and death outcomes from resolved
core events. The player must be able to explain why HP changed. Retry creates a
new run rather than restoring a dead character. Basic attack/UI SFX need an
independent mute/volume control and verified licences.

Record playtest findings about targeting, input, camera, readability and perceived
waiting before adding polish.

### P3 - Explore and loot

Start with one rooms-and-corridors generator. Validate spawn, stairs, doors and
guardian reachability using the same movement rules as gameplay, not a looser
flood-fill. Guarantee a return route. Leaving/re-entering preserves killed enemies
and looted containers; automatic monster respawning is not introduced.

Load the slice's definitions from `assets/packs/prl` with `ContentId` references
and validation. Implement only the primitives required here; full content-pack
management follows later. Bad required content fails visibly rather than yielding
an apparently successful but unwinnable run.

Use the spec's pickup/equip/quaff/read costs. A full backpack or overweight action
has clear feedback; no item disappears on a failed transfer. Implement a small
Magic-affix subset and one Scroll of Identify, not six rarity systems at once.

### P4 - Grow and resume

Implement trained versus effective skill values and PL as specified. **Q-010**
blocks combat gain: the always-hit rule cannot simply supply `p = 1` to the gain
formula, which would yield zero progress. Settle and test a challenge model rather
than quietly changing hit chance or the gain curve.

Normal gameplay retains the spec's slow progression. Verify one threshold unlock
using a labelled developer fixture just below that threshold; do not multiply
production gains to make a 15-minute run reach Master. Record actions/gains for
later pacing simulations; a short playtest does not establish 20-hour balance.

Save seed/settings, content versions, RNG, stable actor state, clock, map/instance
state, inventory and skills. Write atomically at committed-action boundaries.
Implement quit-save and an overwritten checkpoint; surface load/write errors.
Test interruption during writing, incompatible saves, resume equivalence and
permadeath deletion/invalidation. Resolve checkpoint frequency via **Q-012**.

### P5 - Package and evaluate

Provide new run, continue (only for a valid living save), controls/help,
zoom/UI scale, audio settings, reduced-motion/fast presentation and quit.
Run the checklist below against the downloadable Windows package, not just the
developer checkout. Fix blockers before expanding to the next systems.

## Animation and UX contract

These constraints follow the spec; timing and input policy remain provisional:

| Concern | Required invariant | Initial proposal / question |
|---|---|---|
| Rules vs presentation | Animation never changes ticks, RNG, damage, visibility or command outcomes | Resolve rules first, emit ordered presentation events; Q-007 |
| Idle animation | Sprites can animate while waiting without advancing the world | Short idle loops; Q-007 |
| Movement | Logical occupancy/FOV come from core, not tween positions | Brief tween plus instant/fast mode; exact durations open in Q-007 |
| Input under motion | No accidental extra turn or stale-target action | No multi-command backlog; validate each accepted intent against current state; Q-007 |
| Combat feedback | Outcomes remain readable with motion/audio off | Text/icon feedback plus optional hit flash; Q-007 |
| Accessibility/display | UI and world zoom remain independent; no information depends only on colour or sound | Check 1080p and 4K at multiple UI scales; target hardware in Q-013 |

Measure input-to-feedback, frame time and visual backlog on the same replay in
normal and fast modes. Record hardware and resolution; choose numeric performance
budgets through **Q-013** rather than asserting an unmeasured FPS guarantee.

Do not introduce animation-complete waits into the energy scheduler. If animation
backlog builds up, shorten/skip presentation, not gameplay. Final animation
timings, interpolation style and repeat behaviour must incorporate Q-006's source.

## Working with unanswered questions

Use one canonical register in [open-questions.md](open-questions.md) for
decisions and [backlog.md](backlog.md) for open work items and bugs. When work
uncovers an ambiguity, add an ID immediately, with affected stage, status, working
default and next action. Refer to that ID from tasks and code reviews.

Only the dependent task is blocked. For example, Q-010 blocks skill-gain wiring,
not movement, combat damage, rendering or saves. Independent stages/tasks can
proceed when their own prerequisites and decisions are satisfied.

Reversible defaults can be implemented when clearly labelled and covered by tests.
Do not invent answers for high-impact rules or claim defaults came from an
interview. At each stage review, resolve ready questions, add discoveries and
record accepted decisions in the relevant system doc. Keep the question ID and
resolution/source in the register; do not delete the history.

## Validation and playtest record

Run the smallest affected tests during iteration; run workspace format, clippy,
tests and warning-free docs before stage handoff. Add the planned CI gates in P0;
they are not configured merely because standards describe them. Keep headless
tests in core/generation without presentation features; avoid workspace feature
unification turning those targeted tests into full renderer builds.

| Candidate scenario | Evidence to record |
|---|---|
| Launch extracted ZIP, create run, quit | Commit/build ID, OS/hardware, asset resolution and log location |
| Idle 60 seconds; browse help, settings and inventory | State/RNG/clock unchanged, excluding presentation and settings |
| Move, wait, open door, bump blocked tile | Intended command count, energy/clock changes and visible rejection feedback |
| Fight each speed class; replay fast vs normal | Equal core state hashes and damage log; no animation backlog/input loss |
| Explore both floors, loot, equip, identify, drink | Reachable endpoint/return route; correct costs and inventory contents |
| Save/exit/resume; interrupt checkpoint write | Equivalent next-command outcomes; valid previous save or explicit recovery error |
| Die and attempt continue; start new run | No resurrection via old save; morgue present and fresh run valid |
| Resize, change zoom/UI scale, mute/reduce motion | Legible HUD/log/tooltips; same gameplay with optional effects disabled |

Each stage handoff records status, commit/build ID, scenarios run, observed result,
known issues and related question IDs here. No stage is marked done just because
its code compiles.

| Stage | Status | Evidence / remaining work |
|---|---|---|
| P0 | Implementation present | Packaging workflow and boot errors added; extracted-package Linux software-rendering smoke passed; Windows launch acceptance pending |
| P1 | Implementation present | Commands/scheduler, shadowcasting, map memory, input/HUD/zoom and snapshot proof implemented; 36 workspace tests pass; Windows/4K acceptance pending |
| P2 | Planned | Combat/AI and presentation not implemented |
| P3 | Planned | Generation, pack loading and inventory not implemented |
| P4 | Planned | Progression and persistence not implemented |
| P5 | Planned | End-to-end candidate depends on earlier gates |

After P5, prioritize observed problems before breadth. The next slice can add a
minimal town/overworld link, dungeon resets, ranged/magic and character setup.
Housing, crafting, companions and NG+ remain later spec milestones. P0-P5 cover
selected parts of M1-M5 plus early setup/persistence; they do not mark those full
milestones complete.
