# Art Specification & Production Plan

**Status:** CDDA asset-reuse direction accepted; an audited UltiCa crypt subset
is integrated as an opt-in candidate. Final pack/visual approval, broader
appearance/equipment coverage and Windows hands-on acceptance remain open.
This document expands [SPEC.md](SPEC.md#14-art--audio-direction).
Gameplay rules remain in their system specifications. The browser atelier is a
placeholder experiment, not the production renderer or a release-quality reference.

## Current direction override (2026-11-11)

The user approved reducing animation requirements to fit the selected Cataclysm:
Dark Days Ahead tileset. This section supersedes conflicting animation
requirements and custom-animation production gates below; those remain as
historical context, not prerequisites for adopting a tileset.

- Static actors and tiles are acceptable. Reuse supplied facings and frames;
  dedicated idle/walk/attack/work/consumable/casting poses and independently
  authored left/right sets are not mandatory.
- The user requested the earlier animated placeholder player in the candidate
  dungeon and an outdoor location. Candidate mode uses that player by default;
  the fixed UltiCa player remains selectable. Avoid visibly modern designs
  when selecting assets for these fantasy scenes.
- Detailed anatomical hand/grip animation and visible per-action props are
  optional where supported, not reasons to recreate the tileset. Preserve
  gameplay main/off-hand semantics regardless of visual coverage.
- Cosmetic movement, hit feedback, projectiles, targeting and outcome messages
  must make actions readable without waiting for bespoke animation. Reduced
  motion remains supported; presentation never changes rules or action timing.
- Do not infer native sprite resolution, equipment-layer coverage, available
  appearances or fantasy-content coverage before auditing the chosen pack.
  Logical grid size and gameplay movement remain unchanged by this decision.
- Select and pin a tileset, audit asset-specific licensing/credits, inventory
  coverage gaps and prove a representative in-game scene before bulk import.
  Audit sound assets separately; a tileset license does not license a soundpack.

UltiCa is a candidate, not an accepted pack selection. Gravebound-specific
appearance, palette and perspective requirements need reconciliation with the
selected assets; this animation decision does not approve replacements for all
of those choices or alter the separate inventory/UI UX work. See Q-036/I-039.

The [tileset integration plan](tileset-integration.md) records the pinned UltiCa
candidate audit, current-crypt coverage, renderer gaps and dependency-ordered
T0-T4 delivery checks. Subset rights/credits (I-040), the offline converter
(I-041) and opt-in renderer (I-042) are implemented. Original source PNGs,
provenance and CC BY-SA notices accompany the compact generated atlas.
Do not treat this bounded candidate as approval to expand the roster.

### Reuse and delivery checks

- The [CDDA tileset repository](https://github.com/I-am-Erk/CDDA-Tilesets)
  identifies UltiCa as CC BY-SA 3.0. Pin the version and verify each selected
  asset's provenance and exceptions before importing; include required
  attribution, licence text, source links and change notices with distributions.
  Adapted artwork must satisfy applicable ShareAlike terms. Independently
  authored MIT code remains separately licensed; review release packaging and
  mixed-asset terms before shipping.
- [CC-Sounds](https://github.com/Fris0uman/CDDA-Soundpacks) describes effects
  sourced from multiple authors with per-folder credits and music under
  separate permission. Its repository-level licence is not a substitute for
  verifying each recording's actual rights. Do not assume the music permission
  extends to PRL; obtain sufficient permission or omit it.
- Retain a per-asset inventory in `assets/CREDITS.md`, covering author, source,
  pinned revision, licence, used files and modifications. Package the notices
  alongside Windows artifacts. The selected UltiCa subset is imported with those
  records; no external audio is imported.
- Map validated assets to PRL content IDs in presentation code and consume
  committed outcomes for audio; never modify headless rules or gameplay timing
  to accommodate a visual. Verify a representative scene, missing-asset
  diagnostics, readability and packaged loading before expanding coverage.

## 1. Previous decision register (reconcile under Q-036)

The following legacy brief and phases document the earlier custom-art effort.
Animation, bespoke-art sourcing and associated phase prerequisites are
superseded by the current direction override above; remaining visual choices
must be reconciled against the selected tileset, not assumed approved.

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
| Palette & fonts | Target: a locked Lospec-derived ramp palette; Pixelify Sans and Silkscreen | Palette is **not yet locked** (P0 candidate unapproved, seed reuse terms unverified); final ramps, title font; prototype colours/fonts and the placeholder UI's font sizes are not the lock |

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
disposable **placeholder** that exercises independent left/right sets (player
and creatures, idle/walk/attack), frame rectangles, durations and per-tile draw offsets in Bevy. It does not implement
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

**Current path:** the CDDA reuse override and
[T0-T4 integration plan](tileset-integration.md) supersede the custom-production
next steps below. UltiCa's pinned release uses 32x32 base cells with larger
sprites/offsets. The audited candidate is runnable with `--tileset=ultica`;
the procedural atlas remains the default. The release ZIP omits licence/credit
files; the imported subset now includes legal text, original source metadata,
attribution and adaptation notices. Converter/runtime integration and Linux
packaged smoke are delivered, not final visual approval or Windows launch proof.
I-039 remains open for those gates and later coverage; I-043 item icons are
separate from this world-art proof.

**Historical custom-art position (not the current prerequisite):**

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

**Playtest feedback to carry into the final design:** movement must tween
between cells while the walk clip plays (B-009); idle loops are too fast and
too busy (I-021); skeleton arms are too short (I-022); confirm that 64×64 gives
recognisable sprites with hand-pixelled art (Q-028). Held-key movement (I-023)
must read as continuous walking. See `docs/backlog.md` and
`docs/open-questions.md`.

## 8. Artist-light pipeline investigation (2026-11-09)

**Recommendation, not an approved replacement for P0:** use rigged 3D sources
and an automated Blender render/export step to produce the existing 2D asset
contract. There will not be a dedicated asset artist. Prefer reusable rigs and
motions over independently generated animation frames. AI-generated geometry
is a candidate input, not a guarantee of correct anatomy, skin weights or style.
Do not install assets, change the tile grid, soften production edges or replace
the renderer on the strength of this investigation.

This is a documentation-based shortlist, not a visual benchmark. Official
documentation was located through web search; direct page retrieval failed
with DNS errors in this session. Conflicting comparison-site claims were not
used to establish prices or supported rigs. No vendor account, Azure subscription,
generation, export or animation was exercised. Exact prices, deployment access
and redistribution rights must be checked before expenditure or import.

### Fit against PRL

| Candidate | Relevant evidence | Assessment |
|---|---|---|
| Tripo generation + rigging/animation API | Developer docs describe humanoid and non-humanoid rigging, a compatibility check, animation retargeting and GLB/FBX outputs [1] | First AI candidate for the mixed creature roster. A supported rig type does not establish that rat anatomy, exposed skeleton bones, grips or required work animations will succeed. API billing is separate from the web product [2] |
| Meshy generation + rigging/animation API | Rigging API documents standard textured humanoid/bipedal inputs; animation follows rigging [3] | Humanoid comparison/fallback, not the default solution for rats and other animals. Do not infer API quadruped support from marketing for other product surfaces |
| Azure Foundry image models | Microsoft documents OpenAI image generation and FLUX deployment [4] | Useful for candidate reference images, props and appearance studies. These APIs produce images, not a rigged, interchangeable equipment system; independent frame generation is not the recommended animation path |
| Microsoft TRELLIS / Foundry TRELLIS NIM catalog entry | Research project generates 3D assets; a NIM entry is listed in the catalog [5] | Azure-hosted geometry experiment if eligible GPU capacity and terms permit. Does not itself solve skeletal rigging or animation. A catalog entry is not evidence of a cheap serverless endpoint or access in this subscription |
| Rodin / Hyper3D | Vendor describes animation-oriented, rigging-ready meshes [6] | Secondary geometry option. "Rigging-ready" is not a demonstrated automatic character-to-animation pipeline; not the first spend for this requirement |
| Existing CC0 rigs and motions | KayKit Skeletons and Quaternius Universal Animation Library offer animated assets/motions [7] | Best low-risk control and possible reusable animation source after verifying the exact downloads. Their stylised appearance is not an approved Gravebound replacement, and different skeletons are not automatically compatible |
| Three.js / Babylon.js or code-generated Blender geometry | Rendering, scene and skeletal-animation tools, not inherently trained asset generators | Suitable for preview or controlled construction. Coding agents can operate them, but the library alone does not supply convincing anatomy, weight transfer or topology. Another web runtime is unnecessary for the shipped Bevy game |

Blender can render frame sequences in background mode and import/export animated
glTF [8]. Use a fixed orthographic camera, screen-upper-left lighting, transparent
output and a fixed foot anchor. Render both facing poses with the correct
anatomical equipment assignments; never flip the finished composite. Do not
assume rotating the whole model preserves the intended near/far equipment
readability without review.

**Largest production risk: equipment, not generating one impressive monster.**
Generating an armoured character commonly bakes clothing and weapons into one
mesh. The production route must retain separate equipment and compatible rigs,
fit different bodies, support dual wield and two-handed grips, and handle
occlusion. Baking every complete loadout would multiply appearance combinations.
Layered exports need depth/occlusion information or bounded pose families; a
single flattened sheet cannot prove the art P2 attachment gate.

Mixamo is an optional humanoid motion source, not a universal creature rigger.
Adobe permits use in games [9], but that is not permission to relicense raw
animation files as MIT/CC0 or place them in a freely reusable asset repository.
Verify source and rendered-output distribution terms before choosing it.

### Azure credit route and limits

The reported $150 monthly Azure credit makes an **Azure-first experiment** worth
trying, not a reason to run a GPU endpoint continuously.

- Establish the actual offer type first. If these are Visual Studio credits,
  Microsoft's published benefit is for dev/test and excludes third-party branded
  and Marketplace products [10]. Do not assume asset production for release is
  covered merely because a visual experiment is covered.
- Models sold directly by Azure and partner Marketplace offers have different
  deployment/billing paths [11]. Start by checking an eligible Azure OpenAI image
  deployment for reference generation. FLUX's presence in Foundry does not, by
  itself, prove this credit offer covers it.
- TRELLIS self-hosting would consume GPU compute, storage and possibly other
  meters. Check region, GPU quota/capacity, model/dependency licences, NIM terms
  if used, and offer eligibility. Do not equate original TRELLIS, newer research
  releases and the NIM distribution: each has its own requirements.
- Tripo/Meshy external API balances are not Azure credits. No verified
  credit-eligible Foundry deployment of either was established here.
- Before a pilot, price the selected model/SKU, set a small explicit allowance
  within the monthly balance after other usage, retain any spending limit, and
  record metered usage. Budget alerts are not hard spending caps. Deallocate
  compute or delete managed endpoints after the job; residual storage can still
  cost money. Never remove the spending limit to make a deployment work.

### Bounded proof before choosing a vendor (I-024 / I-025)

1. Perform the Azure offer/quota/licence preflight (I-025). No keys belong in
   source control, and no existing repository artwork/code needs to be sent to a
   generation service. Use a generic, newly written test brief.
2. Compare one generated skeleton with one verified, already-animated control.
   Try Tripo first for the complete character path; use an eligible TRELLIS run
   as a geometry comparison only if the preflight permits it. Use one fixed
   camera/material treatment so renderer differences do not decide the result.
3. Produce idle, walk and melee, both facings, at 64 and 128 source pixels with
   the same world-space footprint. Compare crisp and selectively anti-aliased
   versions at the same actual gameplay size, not only enlarged contact sheets.
   A 128-pixel bake does not require changing the 64-pixel logical grid.
4. Require no clipped action frames, stable foot anchors, readable bone/limb
   silhouettes, correct hand identity and no obvious joint collapse or foot
   sliding. Check a quieter idle (I-021) and plausible arm reach (I-022). Then
   prove one Human with a separate weapon/shield and one coordinated two-handed
   action before approving the route for equipment-bearing actors.
5. Record attempts/rejections, total billed generation and GPU cost, retargeting
   and cleanup time, source licences, export sizes and reproducible settings.
   Compare cost per **accepted** asset, not per generated mesh. Repeated anatomy
   or skinning repair is a failure for the no-dedicated-artist constraint.

No whole-roster generation, cloud deployment or art-direction revision is
authorised by this research. B-009 movement interpolation is a separate
presentation defect; replacing sprites will not fix snap-then-walk behaviour.
Q-028/Q-029 remain review gates, not accepted changes to the SPEC.

### Sources

Located 2026-11-09; recheck current terms and billing before use.

[1]: https://developers.tripo3d.ai/en/models/rig
[2]: https://developers.tripo3d.ai/en/docs/billing
[3]: https://docs.meshy.ai/en/api/rigging
[4]: https://learn.microsoft.com/en-us/azure/foundry/openai/how-to/dall-e
[5]: https://ai.azure.com/catalog/models/Trellis-NIM-microservice
[6]: https://hyper3d.ai/use-cases/animation
[7]: https://kaylousberg.com/game-assets/characters-skeletons
[8]: https://docs.blender.org/manual/en/latest/advanced/command_line/render.html
[9]: https://helpx.adobe.com/creative-cloud/faq/mixamo-faq.html
[10]: https://learn.microsoft.com/en-us/visualstudio/subscriptions/faq/subscriber/azure/
[11]: https://learn.microsoft.com/en-us/azure/foundry/foundry-models/how-to/deploy-foundry-models

- [Tripo animation documentation](https://developers.tripo3d.ai/en/models/animation)
  and [API pricing](https://developers.tripo3d.ai/en/pricing).
- [Meshy animation documentation](https://docs.meshy.ai/en/api/animation),
  [API pricing](https://docs.meshy.ai/en/api/pricing) and
  [commercial-use guidance](https://help.meshy.ai/en/articles/9992001-can-i-use-meshy-assets-commercially-license-copyright-explained).
- [Tripo commercial-use guidance](https://www.tripo3d.ai/help/privacy-policy/how-to-use-tripo-models-commercially).
- [FLUX deployment guidance](https://learn.microsoft.com/en-us/azure/foundry/foundry-models/how-to/use-foundry-models-flux).
- [Microsoft TRELLIS research](https://microsoft.github.io/TRELLIS/).
- [Quaternius Universal Animation Library](https://opengameart.org/content/universal-animation-library).
- [Blender glTF import/export](https://docs.blender.org/manual/en/latest/addons/scene_gltf2.html).
