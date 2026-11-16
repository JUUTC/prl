# Layered sprite export pilot

This is a **technical fixture**, not production character art. It exercises a
real LibreSprite batch editor; no procedural character renderer is involved.
Tested with the official LibreSprite 1.3 Linux AppImage, extracted to session
storage because AppImage/FUSE execution is not assumed. No editor binary or
generated fixture belongs in the repository.

Supply an installed LibreSprite executable (or extracted `AppRun`) and a **new**
output directory:

```sh
python3 tools/art-pipeline/check_roundtrip.py \
  --editor /absolute/path/to/libresprite \
  --output /absolute/path/to/new/technical_test
```

Python standard library only. The directory must not exist, to prevent overwriting
someone else's work. Editor configuration is isolated under that output directory.
Failures terminate with an explicit error; `report.json` is written only on success.
The test leaves its named artifacts for inspection rather than deleting them.

The fixture is a 64×64 RGBA source with four frames, two facing tags, and separate
`body`, `main_hand` and `off_hand` layers. Three colours come from the **candidate**
palette, not a production palette lock. Original rectangular marks test pixel
preservation; they are not an art-quality reference.

Checks: source dimensions, layers and durations after save/reopen; exported
facing tags and durations; untrimmed source rectangles; 256×64 atlas dimensions;
pixel-exact composited exports; transparent borders; opaque source pixels;
independent per-layer exports. This does not validate anatomical attachments,
Bevy metadata loading, licence compliance or finished visual quality.

## Pilot production export recipe

The tested CLI supports:

```sh
libresprite --batch --list-layers --list-tags human_reference.aseprite \
  --sheet-type horizontal --format json-array \
  --sheet human_reference.png --data human_reference.editor.json
```

Do not add `--trim`, `--ignore-empty` or automatic mirroring. Export separate
equipment layers using `--layer main_hand` before the source argument. Keep source
art beside exports in `art_src/` inside `assets/packs/prl/sprites/<category>/`,
as described by the upstream project-structure standard.

Editor JSON is **intermediate metadata**, not the future shipping RON contract.
The P2 adapter still needs pivots, grip anchors, effect origins, layer order and
validation. Editable art must be authored and reviewed before release exports.
Aseprite compatibility has not been tested; do not assume interchangeability
from the shared source-file extension.

## Verification record

On 2026-11-09, LibreSprite 1.3 passed the fixture round-trip and layer/atlas export.
The editor is GPL-2.0; installing/using it does not make this original fixture
third-party artwork. No editor code is vendored or distributed here. A real artist
still needs to author the P1 source; a passing pipeline does not approve P1 art.
