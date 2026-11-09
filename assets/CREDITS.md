# Asset Credits

Track every third-party asset here: name, author, source URL, licence, files used.

Gameplay builds use repository-owned placeholders only: the generated
`packs/prl/sprites/placeholder/gravebound_placeholder.png` atlas (exported from
the project's own procedural studies in `tools/character-study/`, MIT like the
code) and fallback coloured shapes. No sprites are imported and no final palette
is installed. The placeholder UI uses Pixelify Sans (body text) and Silkscreen
(short labels) from `fonts/`, each with its OFL licence beside it; if they are
missing it falls back to Bevy's bundled default font (Fira Mono, SIL Open Font
License 1.1, supplied by `bevy_text`). P2 placeholder sound effects are generated in code at startup
(`crates/prl_audio/src/synth.rs`); no third-party audio is used or shipped. Dependency/source licences remain with Bevy.

| Asset | Author | Source | Licence | Files |
|---|---|---|---|---|
| Fira Mono (bundled Bevy default subset) | Mozilla Corporation / Telefonica S.A. | https://github.com/google/fonts/tree/main/ofl/firamono | SIL OFL 1.1; `licenses/FiraMono-OFL.txt` | Embedded by `bevy_text` |
| Pixelify Sans (variable weight) | Stefie Justprince, The Pixelify Sans Project Authors | https://github.com/google/fonts/tree/main/ofl/pixelifysans (upstream https://github.com/eifetx/Pixelify-Sans) | SIL OFL 1.1; `fonts/pixelify_sans/OFL.txt` | `fonts/pixelify_sans/pixelify_sans.ttf` (renamed from `PixelifySans[wght].ttf`, unmodified) |
| Silkscreen Regular | Jason Kottke, The Silkscreen Project Authors | https://github.com/google/fonts/tree/main/ofl/silkscreen (upstream https://github.com/googlefonts/silkscreen) | SIL OFL 1.1; `fonts/silkscreen/OFL.txt` | `fonts/silkscreen/silkscreen_regular.ttf` (renamed from `Silkscreen-Regular.ttf`, unmodified) |
