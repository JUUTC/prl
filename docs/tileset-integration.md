# CDDA Tileset Integration

**Status (2026-11-11):** the audited crypt subset, offline converter and opt-in
world renderer are implemented. Final art selection/visual approval, broader
coverage, item-icon work and Windows hands-on acceptance remain open.
Q-036 / I-039 govern the work. This plan implements the accepted reuse direction
without treating UltiCa as the approved final pack or replacing inventory UX.
The [art-production override](art-production.md#current-direction-override-2026-11-11)
governs animation scope.

Run `cargo run -p prl -- --dev-assets --tileset=ultica`, or use
`Launch-UltiCa.cmd` in a future Windows package built from this work.
Omit the flag for the unchanged procedural-art default.

### Delivered candidate, not final visual acceptance

- The compact 320x64 atlas covers four actors, floor/wall, both door states and
  both stair markers. Static single-facing sprites scale 2x on the existing grid.
  The player is a fixed sword/mail study; equipment changes are not depicted.
- The converter compares every selected compiled rectangle against 15 original
  source pixels. The selected source histories and complete-subtree ancestor
  notice check found no additional per-folder licence/credit files for this
  subset. Original PNG metadata, upstream notices, CC BY-SA 3.0 legal text,
  collective attribution and adaptation notices are preserved in
  `assets/packs/prl/sprites/ultica/`; this does not clear unrelated tiles/audio.
- `tools/tileset/convert.py` emits deterministic art/mappings, selection,
  provenance and checksums. `--verify` checks the complete packaged inventory
  without Pillow/network; `--check` regenerates from the pinned ZIP and preserved
  source PNGs. Synthetic fixtures cover invalid inputs and selected encodings.
- Schema 2 adds scale, two-axis offsets and stair markers; schema-1 procedural
  art still loads. Explicit candidate selection validates required mappings and
  image decoding before play. Missing/corrupt candidate data is a visible boot
  error, not shape or checkout fallback.
- Linux packaged launch outside the checkout, movement, 1x/2x zoom, inventory
  browsing and resize were exercised with native Bevy rendering. The first
  fence-like half-wall/metal-stair study was revised to solid stone walls and
  wooden up-stairs after visual inspection. Final scene/style approval and
  Windows launch are not inferred from that smoke check.
- Windows packaging includes a candidate launcher and validates original/copied
  image, provenance and licence inventories. Workflow 42957259134 passed all
  gates and uploaded the artifact for d12ed5f. Hands-on Windows launch remains
  separate; successful packaging does not claim user-machine verification.

The six item definitions retain existing category/text placeholders; ground loot
still uses `*`. No inventory layout/widget code, UI theme, audio, rules or save
schema changed. Dynamic equipment appearance, additional races/looks, connected
wall variants and a final occlusion/perspective treatment remain later coverage.

## 1. Candidate and reproducible input

Use **non-isometric UltiCa** as the first technical candidate. Its base grid
fits the existing renderer without changing movement or logical positions.
Do not combine other tilesets or import the entire CDDA game/content database.

The pinned input below supplies only the selected subset, not a bulk import:

| Field | Verified value |
|---|---|
| Upstream | [I-am-Erk/CDDA-Tilesets](https://github.com/I-am-Erk/CDDA-Tilesets) |
| Fixed release | `2026-11-08`, `UltimateCataclysm.zip` |
| Source commit | `5ee942f9e6601970c5da356200e7cf5061226262` (release tag resolves to this commit) |
| Archive SHA-256 | `da1edc5b2ecc6390b143dbea5bd3e5302faae473b939631e1593e9b86b0f120d` |
| Archive size | 4,744,728 bytes |
| Source subtree | `gfx/UltimateCataclysm/` |
| Release metadata | `tile_config.json`, `layering.json`, `tileset.txt` |
| Base cell | 32x32; `pixelscale: 1`, `iso: false` |
| Other source cells | Includes 20x20, 32x36, 32x48, 32x64, 64x64, 64x96 and 96x96 with explicit offsets |
| Release sheets | 21 PNGs including fallback art; not a single uniform atlas |
| Rights evidence | Pinned root `readme.md` / `LICENSE.txt` state CC BY-SA 3.0; source subtree has 432 credit/readme/licence/author-named records |
| Packaging gap | ZIP has no licence or credit files; `tileset.txt` points to source-folder credits and names Yar / I-am-Erk for filler trees |

The original metadata findings do not clear all assets in the archive; the
selected subset audit is recorded above and in the packaged provenance.
The complete UltiCa subtree was queried without tree
truncation; the repository-wide recursive tree was truncated and must not be
used to claim a complete audit.

Pinned evidence: [source readme](https://github.com/I-am-Erk/CDDA-Tilesets/blob/5ee942f9e6601970c5da356200e7cf5061226262/readme.md),
[sprite metrics](https://github.com/I-am-Erk/CDDA-Tilesets/blob/5ee942f9e6601970c5da356200e7cf5061226262/gfx/UltimateCataclysm/tile_info.json),
and [pack notices](https://github.com/I-am-Erk/CDDA-Tilesets/blob/5ee942f9e6601970c5da356200e7cf5061226262/gfx/UltimateCataclysm/tileset.txt).
Resolve sources by the full commit above, not `master` or `latest`.

## 2. First playable proof, not full-game coverage

Replace only the existing two-floor crypt's world art first. Keep current
controls, camera, inventory layout, fonts, rules, audio and save data unchanged.
Use static actors, existing cosmetic traversal/lunges and resolved hit feedback.
Do not wait for custom animations, a race roster or a full equipment compositor.

The IDs below exist in the inspected release. They are **mapping candidates**,
not reviewed visual matches or permission to ship:

| PRL surface | Upstream candidate | Remaining check |
|---|---|---|
| `prl:floor` | `t_rock_floor` | Select connected/center variants; continuous floor without accidental seams |
| `prl:wall` | Fixed `t_rock/center` | Solid stone selected after fence-like half-wall review; final perspective/foreground occlusion remains open |
| `prl:door`, closed/open | `t_door_c` / `t_door_o` | Include foreground/background; visible state and passage remain clear |
| Entrance/exit markers | `t_wood_stairs_up` / `t_stairs_down` | Implemented as existing floor-plan endpoint overlays; final fit/readability review remains |
| `prl:player` | `player_male` with mail/sword layers | Fixed kit study only; no implication that equipment/races/looks are covered |
| `prl:shambler` | `mon_zombie` | Choose a suitable non-modern-looking variant; alternatives are weighted variants, not automatically animation frames |
| `prl:skeleton` (guardian) | `mon_skeleton` | 32x48 sprite, shadow and anchor; readable at gameplay zoom |
| `prl:grave_rat` | `mon_sewer_rat`, `mon_black_rat` or `mon_big_rat` | Size/readability and visual fit; do not infer availability from an exact `mon_rat` lookup |
| Sword/mail icons and optional player layers | `longsword`, `overlay_wielded_longsword`, chainmail families | Item sword is on an `incomplete` sheet; review quality and actual layer compatibility before choosing |
| Potion/scroll icons | Not selected | Generic appearance-safe placeholders are allowed until reviewed candidates exist |

Stairs now use candidate art in `prl_render/src/map.rs`; ground loot retains the
text marker. Item icons are the next bounded pass, not a
reason to redesign the inventory workspace.

Fantasy races, five looks per silhouette, full equipment/material coverage,
creatures outside this roster, crafting/resource states and spell effects remain
unproven. Record gaps before promising the rest of the game can use this pack.

## 3. Dependency-ordered delivery

| Step | Work / owners | Prerequisite | Exit evidence |
|---|---|---|---|
| T0 / I-040 | Audited selected subset and preserved rights notices; done for candidate | Candidate pin above | Source pixel matches, per-file history, original metadata and notices; Q-036 visual exceptions remain unapproved |
| T1 / I-041 | Offline subset converter; implemented | T0 for distributable art | Reproducible bounded output, checksums/provenance, invalid-input fixtures and complete packaged inventory check |
| T2 / I-042 | Opt-in presentation manifest/loading, scaled terrain/actors/stairs; implemented | T1 | Native crypt scene, strict diagnostics and unchanged core outcomes at all animation speeds; final art approval separate |
| T3 / I-043 | Knowledge-safe item icons on existing inventory/details/equipment/ground surfaces, without layout changes | T1/T2 shared visual lookup; item source clearance | Six current item definitions covered or explicitly marked placeholders; hidden-effect/affix invariance, rotation and drag/selection preserved |
| T4 / I-044 | Release notices, automated package checks and Windows hands-on proof | T2; include T3 if shipped | Fresh extracted package loads without checkout/network; all referenced images and notices present; Windows launch evidence and visual review recorded |

Do not remove the existing placeholder path until the candidate proof is accepted.
No new gameplay milestone or full mod/pack loader is required for this work.
Keep T3 separate so world-art review does not override the in-flight UX effort.

### T0: rights and selection before import

- Review source-folder notices, inherited credits and history for every selected
  image, including shared shadows, backgrounds and equipment layers. A missing
  local credit file does not establish that there are no attribution requirements.
- Record PRL visual ID, upstream tile ID, source PNG/JSON, source commit, release
  sheet/index, author, licence, notices, modifications and review status.
  Link this inventory from `assets/CREDITS.md` when assets are imported.
- Include CC BY-SA 3.0 legal text, required attribution/source links and change
  notices with the adapted artwork. Repacked/composited/cropped art must retain
  applicable ShareAlike terms; do not label it MIT because PRL code is MIT.
  Resolve uncertain rights before distribution, not through a silent fallback.
- Reconcile the asset-licence shortlist in `standards/coding.md` with the reviewed
  CC BY-SA asset exception before import. Keep independently authored engine code
  separate; do not copy CDDA gameplay code or definitions for the renderer.
- Record whether any wall treatment, perspective or appearance compromise needs
  Q-036 approval. Do not recolour the pack to the old unapproved palette by default.
- Exclude audio from this import. Keep generated SFX; soundpack provenance and
  music permissions are a separate audit and do not block the tileset proof.

### T1: convert once, do not implement CDDA's renderer

Use the pinned compiled release as the pixel/index input and pinned source tree
for provenance. Prefer a small offline converter over a runtime CDDA dependency
or rebuilding all upstream art. Runtime consumes a PRL manifest keyed by
`ContentId`, never hardcoded upstream numeric indices.

- Verify archive hash before processing. Allowlist selected files; reject unsafe
  archive/output paths and missing dependencies. Normal play/build/package steps
  must not fetch `latest` or require network access.
- Normalize ID strings/aliases, global sprite indices, per-sheet cell sizes and
  offsets, foreground/background pairs and selected connectivity variants.
  Validate index ranges against decoded image dimensions; never assume index
  zero or a sheet's first tile is the requested sprite.
- Distinguish weighted alternatives, directional/rotation arrays, `animated`
  entries and `additional_tiles`. Do not play all arrays as frame loops.
  Select fixed approved variants for the first proof; reject unsupported selected
  encodings instead of dropping layers or pretending conversion succeeded.
- Bake approved static foreground/background and fixed player-kit combinations
  where sufficient for the proof. Carry all source dependencies into the credits.
  A fixed kit is a labelled limitation, not dynamic equipment coverage.
- Preserve native pixels and emit display scale/anchors separately. Pack only
  selected rectangles and required layers into bounded PNG output, not the
  512x14272 `normal.png` sheet or the whole source tree. No smoothing or forced
  32x32 crop of tall bodies/stairs.
- Sort output deterministically; emit a versioned manifest, input/output hashes
  and coverage report. Repeating conversion must reproduce bytes. Use synthetic
  fixtures for index boundaries, aliases, offsets, composition, unsupported data
  and malformed input; keep third-party test pixels within the same audit.

### T2: adapt the existing presentation seam

`prl_render/src/art.rs` already handles optional walk/attack clips and a missing
left-facing set without mirroring. Reuse that behavior; do not author animations
merely to satisfy the existing placeholder coverage test.

The implemented boundary and remaining extension checks are:

- Generalize the former placeholder-specific source/atlas and manifest selection
  so a reviewed candidate can be loaded explicitly. `ArtSource` selects the source;
  schema 2 accepts `status: "candidate"` without mislabelling reused art as
  procedural placeholders. Schema 1 and the old manifest remain supported.
- Add explicit sprite display scale and two-axis anchors/offsets. Current sprites
  previously drew source dimensions directly with only `offset_y`. A 32x32 source cell
  must occupy 64x64 logical units at base zoom (2x nearest sampling); 32x48 bodies
  need proportional 64x96 bounds, not a square stretch. Convert source top-left,
  downward-Y offsets to Bevy centered, upward-Y coordinates with fixture tests.
- Preserve floor coverage beneath transparent walls/doors, row depth and visible
  actor/target readability. Review a small connected-wall room before generated
  crypts. If connectivity variants are used, derive them from **remembered**
  terrain, not unseen live neighbours; decoration must not reveal hidden doors.
- Update terrain, actor and stair call sites and coverage tests together.
  Keep gameplay positions, eight-direction movement, FOV/memory, camera cell
  math, hit-test geometry, timing, RNG and snapshots unchanged.
- Log missing mappings once with pack/file/content ID. Placeholder mode can retain
  explicit shape fallback; selected-candidate validation and package gates must
  fail on missing required art rather than pass with unnoticed placeholders.
  Corrupt packaged assets must never trigger a source-checkout search.

### T3: item identity safety and UX boundary

Use the shared known-only item presentation contract, not hidden effect/affix
IDs, for visual selection. Unknown potions/scrolls must depict their run-specific
appearance or a generic category icon. Revealed gear may show physical type,
but unknown bonuses cannot change icons, glow, borders or tooltips.
Ground art follows the same knowledge and visibility/loot-label rules as text.

Keep the current grid footprints, explicit rotation, stable `ItemId` selection,
free placement and drag lifecycle. Rotating a backpack footprint need not rotate
the art. Do not rebuild widgets every frame to animate icons. If UI and rendering
need shared visual definitions, keep their data headless (in `prl_content`);
do not make UI depend on render systems or put Bevy handles into core snapshots.

### T4: acceptance, packaging and rollback

Use the actual crypt and a small diagnostic room: player beside/behind a wall,
both door states, up/down stairs, three enemies, ground loot, attacks/death,
remembered cells and a floor transition. Review native size and 1x/2x world zoom,
1280x720/900, UI-scale independence and Normal/Fast/Off motion.

Prove identical core snapshots/outcomes for the same commands with old/new art
and all motion settings. Exercise invalid manifests, missing image/ID, bad bounds
and unsupported data. Run repository Rust gates for runtime changes and converter
tests for tooling changes.

The Windows workflow already copies the entire `assets/` directory, so place
credits/licences under that root and assert they and every manifest dependency
are in the artifact. Verify hashes and coverage during packaging; launch a fresh
extraction outside the checkout with no network. A successful Linux build or
headless test is not a Windows launch verification.

After review, record the accepted pin, covered surfaces, remaining placeholders,
build ID and hands-on findings in `art-production.md` / `playtest-plan.md`.
Only then consider switching the default and expanding beyond the crypt.
Retain an explicit development placeholder option for comparison, not an
automatic success-shaped fallback from a broken selected pack.
