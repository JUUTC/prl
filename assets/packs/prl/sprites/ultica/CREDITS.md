# UltiCa crypt candidate attribution

Artwork: **Ultimate Cataclysm (UltiCa), the UltiCa contributors**, distributed
under **Creative Commons Attribution-ShareAlike 3.0 Unported**.

Source: https://github.com/I-am-Erk/CDDA-Tilesets
Pinned revision: `5ee942f9e6601970c5da356200e7cf5061226262`
Release: `2026-11-08`, `UltimateCataclysm.zip`
Licence: https://creativecommons.org/licenses/by-sa/3.0/
Legal text: `CC-BY-SA-3.0.txt`. Original upstream notices are preserved in
`upstream-readme.md` and `upstream-LICENSE.txt`.

This is a small adapted art collection, **not MIT-licensed artwork**. PRL's
independently authored engine/tool code remains MIT. No endorsement by CDDA
or UltiCa contributors is implied.

## Files, authors and changes

`provenance.json` lists every used source PNG, its pinned source path and
SHA-256, compiled sheet/index/rectangle, selected variant, layers, PRL visual ID,
and file-history contributor names/revisions. Source links are formed as:
`https://github.com/I-am-Erk/CDDA-Tilesets/blob/5ee942f9e6601970c5da356200e7cf5061226262/gfx/UltimateCataclysm/<source>`.
Unmodified source PNGs (including embedded metadata) are preserved in `sources/`.

Credit the UltiCa contributors collectively. Recorded file-history contributors
include I-am-Erk, Serhiy Zahoriya, hoasd1, acepleiades, Isaac Freund and
barsoosayque. History entries can represent moves/reorganisation and are not
claims that the committer originally drew the work. The upstream aggregate
licence/attribution applies; no separate local/ancestor credit or licence files
were present for these selected PNGs in the complete pinned UltiCa subtree.
Other tiles' notices and filler-tree credits do not license unrelated artwork;
no filler tree or soundpack assets are used here.

PRL adaptations: selected fixed variants, composited source foregrounds,
backgrounds, shadows and a fixed player kit, repacked native pixels into
`ultica.png` with one-pixel extruded atlas borders, and authored PRL visual
mappings in `ultica.ron`. Door composites
include the selected rock floor. No palette recolour, mirroring or interpolated
resizing is baked into the PNG; the renderer applies nearest-neighbour 2x scale.
The adapted atlas is offered under CC BY-SA 3.0 Unported with these notices.

This is an opt-in technical candidate, not final visual approval. The optional
UltiCa player sword and mail are a fixed appearance study, not a depiction of
current equipment. A separate repository-owned animated placeholder player is
the candidate default. Stone walls, wooden stairs and an armoured skeleton
substitute replace the first fence, metal-stair and modern-clothed-zombie study;
wall perspective and foreground occlusion still need final visual approval.
Sword and mail icons reuse the pinned longsword and chainmail hauberk item
sprites; their native 32x32 rectangles are selected from the padded atlas without
recolouring. The longsword is on upstream's incomplete sheet and remains a
provisional visual, not quality approval. Potions and scrolls retain explicit
text/category placeholders; no modern bottle or effect-specific icon is imported.
