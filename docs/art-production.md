# Art Specification & Production Plan

**Status:** approved direction; production artwork not yet approved.
This document expands [SPEC.md](SPEC.md#14-art--audio-direction).
Gameplay rules remain in their system specifications. The browser atelier is a
placeholder experiment, not the production renderer or a release-quality reference.

## 1. Decision register

| Area | Approved requirement | Not approved / still to resolve |
|---|---|---|
| Art direction | Gravebound: grimdark-hopeful, weathered, earthy, sunken faces, natural proportions, ragged layers, restrained magic accents | A finished visual reference; existing procedural sprites are too crude to ship |
| Resolution | Original 64×64 world tiles; nearest-neighbour integer zoom; larger assets may span tiles | Final character footprint within the tile, portrait/icon sizes |
| Perspective | Three-quarter overhead; hidden square grid; no drawn floor grid; half-height foreground walls | Exact camera elevation, wall dimensions, lighting and occlusion treatment in the production reference |
| Movement | Eight directions; independent left/right facing sets initially; vertical movement retains last facing | Diagonal corner collision; additional facings only if the proof exposes a readability problem |
| Bodies & looks | Two unlabeled visually distinct silhouettes, one masculine-appearing and one feminine-appearing; at least five looks per race/silhouette; equal cosmetic and equipment access | Finished anatomy sheets and unlockable-race detail |
| Equipment | Actual starting class kit; subsequent gear changes visible; main/off-hand identity preserved; dual wield and coordinated two-handed grips; no mirrored equipped composite | Exact visual mapping for item level, quality, rarity and material |
| Progression | Gear appearance comes from items/materials/improvements, not class restrictions or skill-derived PL | Prototype ilvl 1/25/50/100 bands and ilvl 200 cap are not game rules |
| Actions | Idle, walk, melee, archery, crafting, mining, chopping, gathering, eating, drinking, healing, casting | Production frame counts, timing, weapon/action variants |
| World assets | Monsters/enemies, townsfolk, animals, harvest states, stations, food/drink, spell/projectile effects | Final roster, asset counts, work order |
| Atmosphere | Optional subtle fog/ash parallax; terrain and interactables remain anchored; protect characters and targeting; reduced-motion/off settings | Shipping performance budget, layer strengths and shader implementation |
| Palette & fonts | Target: a locked Lospec-derived ramp palette; Pixelify Sans and Silkscreen | Palette is **not yet locked** (P0 candidate unapproved, seed reuse terms unverified); final ramps, title font; prototype colours/fonts and the placeholder UI's Fira Mono are not the lock |

The atelier's six-frame cycles, 140ms timing, hardcoded gear presets, enlarged
face crops, sample projectile types, simulated parallax, and procedural actors
are **demonstrations only**. They must not silently become production standards.

## 2. Production quality specification

The target is detailed, deliberately authored pixel art, not primitive block
figures or enlarged 32×32 placeholders. More colours or more frames alone do not
establish quality.

| Surface | Required result | Review evidence |
|---|---|---|
| Character | Convincing anatomy and weight; tapered limbs; distinct race/body silhouettes; readable face where visible | Native-size and 2×/4× views, both facings, neutral background and world scene |
| Materials | Cloth, leather, metal, bone and wood distinguishable through shape, texture and highlight structure | Same silhouette wearing contrasting materials under identical lighting |
| Equipment | Clear silhouette changes; stable anatomical grips; correct far/near layering; plausible two-handed support | Sword/shield, dual wield, two-handed weapon/tool and bow poses |
| Animation | Anticipation, contact/release, follow-through, weight transfer; stable foot anchor; no unintended popping | Frame strips and real-time playback, both facings; slow/paused inspection |
| World | Continuous floor art without grid seams; depth consistent with characters; half walls do not hide identification or targeting | Standing/moving character behind a wall and near props in a representative scene |
| Effects | Distinct shapes and timing, not recolours alone; cast/travel/impact readable; bounded visual noise | Fire, frost, lightning, poison, ward/healing comparison; stacked-effects scene |
| UI | Portraits/icons designed for their actual display size, not a cropped sprite promoted to finished art | Character creation and paper-doll mockups using the shared font/theme direction |
| Accessibility | Gear/targets remain legible; effects can be reduced/disabled; effects not dependent on colour alone | Default, atmosphere off and reduced-motion comparisons |

Every production candidate must pass palette, dimensions, transparency,
attachment/depth, loop/one-shot, licensing and in-world review. Pixel-art source
edges are crisp; intentional shader transparency is separate from source sprites.
Review at intended gameplay zoom, not only an enlarged contact sheet.

**Stop gate:** do not multiply an unapproved reference across races or asset
families. Reject or revise the reference first. User approval of Gravebound is
approval of the direction, not of the existing sprites.

## 3. Asset and animation contract

Define and validate the contract before batch production:

- Keep editable sources, runtime sprite atlases and definition manifests linked
  by stable namespaced IDs in content packs. Record provenance/licence for every
  externally sourced asset in `assets/CREDITS.md`. Folder/authoring format is
  selected in P0; do not create a competing pack structure in the atelier.
- Each animation declares facing, action, frame rectangles, durations, looping or
  one-shot behaviour, foot/pivot anchor, main/off grip anchors, far/near draw order,
  effect origins and visual bounds. Additional facing sets remain extensible.
- Main and off hand are anatomical identities, not screen-left/screen-right.
  Two-handed grips share one object; dual wield retains two distinct objects.
  Art must support item properties rather than a universal hardcoded type list.
- Compose body, appearance and equipment layers where feasible; do not bake every
  skin/hair/gear combination into a separate sheet. Share compatible pose families,
  but author race-specific fit where anatomy changes. Never mirror the equipped
  composite to produce the other facing.
- The selected class supplies only starting gear. Item definitions supply visible
  models/materials and improvement variants. Separate ilvl, material, rarity,
  quality and durability; do not infer gameplay values from ornament.
- Work/consumable actions display tools or props from the resolved action context.
  Temporarily stowing a weapon is presentation state, not an inventory mutation.
- Rules resolve attacks, consumption, resource depletion and tree felling through
  explicit commands/events. Contact/release markers trigger only presentation.
  Skipping animation must leave the same game state.
- Separate character facing from aim direction: arrows/spells must depict targets
  in all eight movement directions despite only two character facing sets.
- Harvest art distinguishes standing/intact, action transition, depleted/felled,
  and collected states. Review loops must not imply in-game resource regeneration.
- Atmospheric layers alone use differential camera offsets. No parallax for
  actors, walls, trees, benches, ore or other interactables. Establish draw-order
  and visibility rules for large props and effects.

## 4. Dependency-ordered production phases

These phases accompany, not replace, the gameplay milestones in SPEC §15.
No calendar or effort estimate is committed until P2 reveals real production cost.

| Phase | Prerequisites | Bounded deliverables | Exit gate |
|---|---|---|---|
| **P0 — Lock the brief and pipeline** | Approved decisions above | Original/reference mood sheet with provenance; palette candidate; anatomy/perspective sheet; proposed authoring tool and source/export format; draft animation/asset contract; inventory of required art | User approves quality target, palette and reference scope; technical export feasibility established |
| **P1 — Finish one reference character** | P0 | One Human silhouette, both facings, sword/shield, idle and walk; one matching floor/wall sample; native-size and world-scene presentation | User explicitly approves production quality, anatomy, materials, perspective and movement; revise rather than expand if rejected |
| **P2 — Prove modular actions and renderer** | P1; asset contract from P0 | Second Human silhouette; dual wield, two-handed weapon/tool, bow draw/release, cast and bandage proof; one upgraded-material kit; Bevy inspection scene with actual anchors, depth order and cosmetic playback | Both silhouettes retain grips and gear identity; vertical travel acceptable; layered assets load; skipping effects preserves rules; measured authoring effort recorded |
| **P3 — Build a representative vertical slice** | P2 | One dungeon scene: skeleton enemy, wolf, smith, tree/ore states, forge/bench, meal/drink; representative elemental/protective effects and physical/coated arrows; all required action families; optional fog/ash | User approves cohesive in-game scene; harvesting, projectiles and consumption use rule events; accessibility and performance budgets measured |
| **P4a — Expand player coverage** | P3 | Five starting races × two silhouettes × at least five looks; all six starting class kits; required visible item families and compatible action sets | Full appearance/gear matrix, unlock behaviour, persistence and in-world readability pass |
| **P4b — Expand world coverage** | P3; roster/work order approved | Bestiary toward the existing 60–80 target, NPC roles, animals, biome/resource variants, stations, effects and consumables | Every required gameplay definition maps to accepted art; large assets/occlusion and effect stacking pass |
| **P5 — Release integration** | P4a, P4b; relevant gameplay milestones | Unlockable races/classes as required by their gameplay milestone, portraits/icons, pack exports, licence/palette checks, performance/accessibility polish | Release content inventory has no accidental placeholder dependencies; user signs off final visual quality |

P4a and P4b may proceed independently after P3. Art can be inspected before a
system is playable, but that is not gameplay acceptance. P2's inspection scene
must use the planned Bevy renderer/content boundaries rather than make the
browser prototype a shipping dependency.

### Gameplay milestone alignment

| Gameplay milestone | Art required for its playable scope |
|---|---|
| M1–M2: movement/combat | World tiles, two-facing locomotion, enemies, melee/ranged reaction and projectile art |
| M3–M4: items/skills | Layered equipment, item icons, paper doll, improvement variants; skill/PL UI without automatic gear changes |
| M5–M6/M6b: dungeons/towns/housing | Biomes, props, townsfolk, stations, housing/storage, pack animals and readable occlusion |
| M7–M9: magic/crafting/companions | Spell phases, work/consumable actions, resource states, pets and summon families |
| M10–M12: setup/unlocks/endgame | Final creation UX, unlockable appearances, bosses/multi-tile art, effects and release polish |

## 5. Scope and estimation rules

The starting appearance floor is **50 looks** (5 races × 2 silhouettes × 5),
with another **50** for the five planned unlockable races. Those are selectable
looks, not 100 independently baked full animation libraries.

Estimate work using:

`body/pose families × facings × action variants × frames`

plus independently fitted equipment layers, appearance layers, effects, props,
portraits and integration. Material recolours are not a substitute for distinct
equipment construction or finished shading. Two facings reduce directional art,
but asymmetric gear and action variants still require validation.

After P2, measure time for a finished base, a fitted gear layer, an action and a
revision. Use those measurements for estimates and batch sizing; do not quote
completion dates based on procedural-placeholder speed.

The gameplay playtest currently draws a generated atlas of the Gravebound
procedural studies (`tools/character-study/export_placeholder_atlas.py`, manifest
`assets/packs/prl/sprites/placeholder/gravebound_placeholder.ron`). It is a
disposable **placeholder** that exercises independent left/right sets, frame
rectangles, durations and per-tile draw offsets in Bevy. It does not implement
attachment poses, pivots/visual bounds or palette revisions, is not the P2
attachment gate, and does not complete art P1.

Maintain an approved asset inventory before P4. Each entry records ID, gameplay
definition, family, required facings/actions, layer/variant dependencies, author,
source/export paths, licence, review evidence and status:
**placeholder → candidate → approved → integrated**. Assign an owner when a
production batch starts. Approval and integration are separate.

## 6. Review and change control

Work from a phase-scoped brief with named deliverables and acceptance criteria.
At each exit gate, record the accepted reference and unresolved issues here or
in the asset inventory. User review chooses whether to accept, revise or change
scope. Do not treat tool-generated examples or a successful export as art approval.

A decision change updates this document, SPEC and affected system docs before
dependent production continues. Palette, anatomy, perspective and animation
contract changes require re-review of fitted layers and the world reference.
New asset families need a gameplay purpose, inventory entry, dependencies and
acceptance criteria; avoid unrelated showcase expansion.

## 7. Current position and next work

**P0 reference package prepared; approval and feasibility checks pending.**
Review [the P0 package](style/gravebound-p0.md), including its original construction
sheet, palette proposal, authoring/export recommendation and bounded P1 work order.
No production phase has passed its exit gate.
The atelier demonstrates customization, layering, facing, action and effects
behaviour. Its sprites are explicitly rejected as shipping-quality references.
The palette is not locked, the export contract is not implemented in Bevy, and
no production character or scene is approved.

Before P1, review the P0 brief, geometry and reference scope; resolve palette
reuse terms and actual seed-extension work, approve final ramps, and confirm the
authoring tool with the art author. LibreSprite's real layered save/reopen/export
pilot has passed; palette values and corrected attribution were verified through
public sources, but palette-specific reuse terms remain unresolved.
See [the export workflow](../tools/art-pipeline/README.md).
Do not expand the roster while the gate is open.
