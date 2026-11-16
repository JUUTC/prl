# Asset Credits

Track every third-party asset here: name, author, source URL, licence, files used.

By default, gameplay builds use repository-owned placeholder sprites: the generated
`packs/prl/sprites/placeholder/gravebound_placeholder.png` atlas (exported from
the project's own procedural studies in `tools/character-study/`, MIT like the
code) and fallback coloured shapes. An **opt-in UltiCa crypt candidate**
(`--tileset=ultica`) uses the adapted atlas in `packs/prl/sprites/ultica/`.
Its CC BY-SA 3.0 attribution, legal text, per-source provenance and unmodified
selected source PNGs ship in that directory. It is not final art approval;
player equipment is a fixed visual study and item icons remain placeholders.
No final palette is installed. The placeholder UI uses Pixelify Sans (body text) and Silkscreen
(short labels) from `fonts/`, each with its OFL licence beside it; if they are
missing it falls back to Bevy's bundled default font (Fira Mono, SIL Open Font
License 1.1, supplied by `bevy_text`). P2 placeholder sound effects are generated in code at startup
(`crates/prl_audio/src/synth.rs`); no third-party audio is used or shipped. Dependency/source licences remain with Bevy.

| Asset | Author | Source | Licence | Files |
|---|---|---|---|---|
| Fira Mono (bundled Bevy default subset) | Mozilla Corporation / Telefonica S.A. | https://github.com/google/fonts/tree/main/ofl/firamono | SIL OFL 1.1; `licenses/FiraMono-OFL.txt` | Embedded by `bevy_text` |
| Pixelify Sans (variable weight) | Stefie Justprince, The Pixelify Sans Project Authors | https://github.com/google/fonts/tree/main/ofl/pixelifysans (upstream https://github.com/eifetx/Pixelify-Sans) | SIL OFL 1.1; `fonts/pixelify_sans/OFL.txt` | `fonts/pixelify_sans/pixelify_sans.ttf` (renamed from `PixelifySans[wght].ttf`, unmodified) |
| Silkscreen Regular | Jason Kottke, The Silkscreen Project Authors | https://github.com/google/fonts/tree/main/ofl/silkscreen (upstream https://github.com/googlefonts/silkscreen) | SIL OFL 1.1; `fonts/silkscreen/OFL.txt` | `fonts/silkscreen/silkscreen_regular.ttf` (renamed from `Silkscreen-Regular.ttf`, unmodified) |
| UltiCa crypt candidate | UltiCa contributors; detailed file-history attribution and original sources in `packs/prl/sprites/ultica/provenance.json` | I-am-Erk/CDDA-Tilesets, revision `5ee942f9e6601970c5da356200e7cf5061226262`, release `2026-11-08` | CC BY-SA 3.0 Unported; `packs/prl/sprites/ultica/CC-BY-SA-3.0.txt` and `CREDITS.md` | Adapted `ultica.png`, original selected PNGs under `sources/`; PRL mappings in `ultica.ron`. Selected fixed variants/layer composition/repacking; no third-party sounds |
