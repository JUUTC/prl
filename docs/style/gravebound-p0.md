# Gravebound P0 Reference Package

**Status: candidate for review, not approved production art.**
Scope: the P0 gate in [art-production.md](../art-production.md).
This package does not authorize expanding the placeholder roster or starting P1.

## Visual brief

Gravebound depicts capable people surviving a worn, hostile world. Dread comes
from age, material and light, not from oversized heads, uniformly black sprites,
or glowing equipment. Hope appears in warm skin, lamplight and restrained magic.

| Element | Aim | Avoid |
|---|---|---|
| Anatomy | Natural head/body ratio, tapered limbs, visible joints, believable stance | Toy-like square bodies, oversized heads, identical limbs |
| Cloth | Uneven hems, compression folds, repaired seams, layered charcoal values | Flat rectangles or random single-pixel noise |
| Leather | Thick edges, straps under tension, warm worn highlights | Treating leather as brown metal |
| Metal | Cold planes, narrow reflected edges, dark joints, irregular wear | Full bright outlines, plastic highlights, automatic rarity glow |
| Face | Recessed eyes, nose/cheek planes, one readable expression | Flat mask, oversized white eyes, relying on a portrait crop |
| Environment | Connected weathered floor texture, coherent wall thickness, quiet ground | Visible cell borders, clutter around feet, full foreground walls hiding gear |
| Magic | Distinct silhouette and phase per effect; brief saturated accent | Recolouring the same particle pattern for every element |

### Original mood and proportion board

[Open the original P0 construction sheet](gravebound-p0.svg).
It combines material marks, anatomy measurements and a perspective/occlusion
diagram. **The figures are wireframes, not a sample of finished visual quality.**
Use it to review proportions and space allocation; do not promote these shapes
into runtime sprites.

The board is original schematic work created for this repository. No external
artwork, game screenshots, textures or character designs were copied into it.
Darkest Dungeon was discussed as an atmosphere/parallax example, not as a
licence to reproduce its assets or replace the approved overhead perspective.

## Proposed P1 reference geometry

All numbers here are candidates to be checked against finished art.

| Property | Candidate | Review condition |
|---|---|---|
| Source cell | 64×64, transparent background | Preserve pixel edges at native, 2× and 4× |
| Foot pivot | `(32, 58)` in top-left-origin source coordinates | Foot contact stable between facings/frames |
| Human projected height | 42–46 px; head height approximately 7 px | Natural proportions still readable overhead |
| Unarmoured torso width | Approximately 12–16 px | Two silhouettes remain distinct without equipment restrictions |
| Equipped visual bounds | Approximately x=8–56, y=8–60 | Sword/shield not clipped; larger action bounds declared separately |
| Floor cell | Hidden 64×64 square | Continuous neighbouring texture; no grid overlay |
| Foreground wall rise | Approximately 16–20 px | Test a figure immediately behind the wall, including small races later |
| Light direction | Upper-left screen key light, soft reflected earth tones | Same material reads consistently in both facings |

These are projected 2D pixel dimensions, not physical body measurements. A
precise camera-elevation angle is not being invented from the placeholder.
The finished reference must establish the perspective visually.

The two silhouettes use separate anatomy constructions and identical equipment
access. Neither gets a gender label in player-facing UI. P1 proves one Human
silhouette; P2 must validate the other before family expansion.

**Facing construction:** author the right-facing and left-facing three-quarter
poses independently. Keep anatomical right hand as the main-hand reference and
left hand as off hand for this first reference; do not imply that a handedness
selector is implemented. Far/near order changes between views. Lighting does
not flip with the figure. Eight-direction aim is separate from facing.

## Palette candidate

[gravebound-palette-candidate.json](gravebound-palette-candidate.json) proposes
**24 named ramps × 7 shades = 168 unique colours**, dark-to-light.
This is an original review palette, **not the approved master palette** and
**not yet verified as a Lospec-derived extension**.

Proposed external seed for evaluation: **Resurrect 64 by Kerrie Lake**, at
`https://lospec.com/palette-list/resurrect-64`. The earlier ENDESGA attribution
was incorrect and is superseded.

The website remains unreachable, but on 2026-11-09 the 64 values were verified
in Lospec's own public editor source at commit
`e22aba45af672844636180ae06846fbde119fd36`, file `js/data/palettes.js`:
`https://github.com/lospec/pixel-editor/blob/e22aba45af672844636180ae06846fbde119fd36/js/data/palettes.js`.
Author attribution is explicitly recorded in
`https://github.com/kirinokirino/Palettes/blob/master/palettes/bit64/Resurrect%2064.json`,
whose colour list matches Lospec's. The latter is corroboration, not an
authoritative palette licence.

No external palette values have been imported into repository assets. Lospec's
editor repository licence does not establish palette-specific reuse terms.
Verify those terms before adoption, then compare/adjust the original proposal
against the seed. The existing 168-colour proposal is still **not** an established
extension. Record any imported resource in `assets/CREDITS.md` before merging assets.

The file stays in `docs/style/`; it must not be installed as `assets/palette/`
or wired into CI as the locked palette before approval.

| Ramp groups | Use |
|---|---|
| Ink, charcoal, slate, iron, steel | Depth, stone and controlled metal reflections |
| Bone, linen, wool, leather, wood, rust | Worn surfaces, restrained warm highlights |
| Skin light/ochre/deep; orc olive; elf ash; revenant pallor | Skin plane separation rather than one skin recolour |
| Ember, frost, lightning, venom, ward, blight | Effect identity; brightest values limited to focal accents |

Shade indices describe ramp order, not a requirement to use all seven on every
sprite. Begin P1 with a small working subset. Test silhouette separation against
both stone and woodland backgrounds without solving readability by universal
bright outlines. Material construction must change with gear, not just its hue.
No new item-level, rarity, skin/lore, or elemental gameplay rules are introduced.

## Authoring and export recommendation

**Recommend the now-tested LibreSprite workflow for the P1 pilot.** It supports
layered pixel sources, facing/action tags, unequal frame durations and PNG/JSON
atlas export. The official GPL-2.0 LibreSprite 1.3 Linux AppImage was downloaded
and extracted into session storage, not committed or installed system-wide.
Its CLI and a real save/reopen/export fixture were exercised successfully.
See [the reproducible export pilot](../../tools/art-pipeline/README.md).

Aseprite remains an alternative for artist preference, but its export/source
compatibility has not been tested here. No purchase is required or authorized
by this recommendation. Tool preference for the actual art author remains open.

Align production paths with the latest upstream
`docs/standards/project-structure.md` and `docs/standards/extensibility.md`:

```text
assets/packs/prl/
  sprites/characters/human/
    art_src/human_reference.aseprite
    human_reference.png
  data/animations/human_reference.ron
assets/palette/                 # only after palette approval
```

Namespaced reference ID: `prl:human_reference`. This is a proposed animation
definition category inside the existing pack, not a separate pack format or
an implemented registry. `art_src/` sits beside exports; editable source and
runtime exports are not confused with browser screenshots.
Git LFS remains a later repository decision, not enabled by this proposal.

### Proposed metadata requirements

| Field | Meaning / invariant |
|---|---|
| Schema version and content ID | Explicit version, stable `prl:` namespace |
| Atlas and source provenance | Pack-relative runtime path, linked editable source, author/licence record |
| Facing and action | Explicit named sets; no auto-mirrored equipped composite |
| Frames and duration | Integer source rectangles, per-frame duration, loop/one-shot flag |
| Pivot and visual bounds | Feet anchored; declare any frame extending beyond one tile |
| Attachment poses | Anatomical main/off grips, two-hand constraints, effect origins and per-frame draw order |
| Item fit | Compatible body/pose families; data-driven item properties, not name-specific engine exceptions |
| Presentation markers | Contact/release effects only; never determine damage, inventory or resource state |
| Palette revision | Approved revision required at production validation |

No RON schema or loader is implemented in P0. Validate these requirements in a
small P2 renderer prototype rather than make the atelier JSON a shipping format.

### Technical feasibility gate: editor round-trip passed

LibreSprite 1.3 passed a 64×64, three-layer, four-frame technical fixture with
`idle_left` / `idle_right` tags and durations of 100/180/120/220ms:

1. Save/reopen preserved dimensions, layer names and durations.
2. Export preserved facing tags and untrimmed 64×64 source rectangles.
3. Atlas dimensions, transparent borders, candidate colours and source alpha
   passed; composited pixels matched before/after reopening.
4. Independent `body`, `main_hand` and `off_hand` exports passed.

An editor round-trip/export is P0 feasibility evidence; the Bevy attachment
integration is the P2 gate and remains unimplemented. This fixture has no artwork
approval or anatomical proof. Run `tools/art-pipeline/check_roundtrip.py` against
the chosen artist's editor before changing tools or versions.

## P1 work order, proposed for approval

| ID / family | Deliverable | Dependencies | Current status |
|---|---|---|---|
| `prl:human_reference` / body | One Human silhouette, independent left/right; idle and walk | P0 brief, geometry, palette and tool approval | Planned; no production candidate |
| `prl:human_reference_appearance` / appearance | One face/hair/skin selection | Body reference | Planned |
| `prl:reference_warrior_kit` / equipment | Chainmail, coif, gloves/boots, longsword and kite shield matching Warrior kit | Body anchors and material brief | Planned |
| `prl:reference_floor_wall` / environment | Connected floor sample and half-height wall | Palette and projected scale | Planned |
| `prl:reference_review` / evidence | Native, 2×/4×, both facings, animation strip, in-world wall case | All above | Planned |

IDs are reserved proposals, not registered runtime definitions. Asset author
is unassigned until the production batch starts. Art ownership/licensing must
be recorded before an export becomes a candidate.

P1 scope does **not** include the entire roster, a second silhouette, additional
weapon sets, spell effects, portraits or new crafting systems. Use only enough
frames to establish convincing idle/walk motion; final counts/durations are
proposed by the artist and reviewed, not copied from the six-frame atelier.

## Approval and blockers

P0 deliverables are prepared for review: brief, original construction board,
168-colour proposal, authoring/export recommendation, metadata contract and
bounded P1 work order. **P0 has not passed its gate.**

Remaining gates: user review of the brief/geometry and P1 scope; palette-specific
reuse terms, actual seed-extension work and palette approval; actual author's
tool choice. Editor save/reopen/export feasibility has passed with LibreSprite.
No production-quality character has been authored or approved.
