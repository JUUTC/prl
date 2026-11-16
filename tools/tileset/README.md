# UltiCa crypt candidate tooling

The renderer's opt-in candidate is generated offline from release `2026-11-08`.
It is a bounded technical proof, not art approval. See
[`docs/tileset-integration.md`](../../docs/tileset-integration.md).

Run without changing the default art:

```sh
cargo run -p prl -- --dev-assets --tileset=ultica
```

The candidate uses static, unmirrored enemies and can show either the animated
repository-owned placeholder player or the fixed UltiCa sword/mail study.
It does not depict equipment changes. Audited sword/mail item icons decorate
existing inventory controls and visible ground loot; potion/scroll art remains
text/category placeholders. No inventory redesign or audio import is included.
Omitting `--tileset=ultica` keeps the
repository-owned procedural atlas.

## Checks and reproducible export

Tests/export require Python 3 and Pillow (`requirements.txt`); package validation
uses only Python's standard library and does not download anything:

```sh
python3 -m unittest discover -s tools/tileset -v
python3 tools/tileset/convert.py --verify
```

Obtain `UltimateCataclysm.zip` from the fixed upstream release (not `latest`).
The converter verifies its SHA-256 against `ultica.json` before parsing.
The package already preserves all selected original pixels and their audited
history, notices and legal text, so a checkout can regenerate without fetching
the whole upstream source tree:

```sh
python3 tools/tileset/convert.py \
  --archive /path/to/UltimateCataclysm.zip \
  --source-dir assets/packs/prl/sprites/ultica/sources \
  --notices-dir assets/packs/prl/sprites/ultica \
  --check
```

Omit `--check` to regenerate. The committed output was generated with Pillow
10.2.0; a different PNG encoder version may require reviewing byte differences
even when pixels match. There is no network access in the converter or renderer.
Use `--output PATH` / `--verify --output PATH` to validate a copied package.
Notice text is normalized to LF before hashing, matching the repository's Git
attributes on both Windows and Linux; original source PNG bytes are untouched.

`ultica.json` selects layers explicitly; indices are resolved from the upstream
configuration, not guessed. Every selected compiled rectangle is compared with
the audited original source pixels. The converter rejects ambiguous/missing IDs,
unsupported directional/animated encodings, mismatched source pixels, clipped
layers, unsafe paths and oversized inputs. It composes on a common 32x64 native
canvas with origin `(0, -32)`; the manifest's 2x scale and `(0, 32)` world offset
place the source cell correctly on PRL's 64-unit grid.
Each frame has a one-pixel extruded border outside its manifest rectangle to
prevent adjacent sprites bleeding into moving-camera tile edges.
An optional integer `crop: [x, y, width, height]` selects an in-canvas native
rectangle for item markers without resizing source pixels; invalid bounds fail
explicitly. The two item crops exclude the unused top half of the common canvas.

The atlas is CC BY-SA 3.0 artwork, separately from MIT tool code.
`CREDITS.md`, `provenance.json`, original source PNGs and upstream notices live
beside the generated atlas and are included in `checksums.json`. When expanding
the selection, audit source and inherited notices/history first; do not assume
that adjacent tiles or sounds share the same rights.
