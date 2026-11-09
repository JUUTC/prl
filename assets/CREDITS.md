# Asset Credits

Track every third-party asset here: name, author, source URL, licence, files used.

Gameplay builds use repository-owned placeholders only: the generated
`packs/prl/sprites/placeholder/gravebound_placeholder.png` atlas (exported from
the project's own procedural studies in `tools/character-study/`, MIT like the
code) and fallback coloured shapes. No sprites are imported and no final palette
is installed. The UI temporarily uses Bevy's bundled default font (Fira Mono,
SIL Open Font License 1.1, supplied by `bevy_text`); no standalone font file is
distributed in the pack. Pixelify Sans/Silkscreen integration remains future UX
work. P2 placeholder sound effects are generated in code at startup
(`crates/prl_audio/src/synth.rs`); no third-party audio is used or shipped. Dependency/source licences remain with Bevy.

| Asset | Author | Source | Licence | Files |
|---|---|---|---|---|
| Fira Mono (bundled Bevy default subset) | Mozilla Corporation / Telefonica S.A. | https://github.com/google/fonts/tree/main/ofl/firamono | SIL OFL 1.1; `licenses/FiraMono-OFL.txt` | Embedded by `bevy_text` |
