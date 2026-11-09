# P2 placeholder audio

`PrlAudioPlugin` plays short placeholder sound effects generated in code at
startup (16-bit mono PCM WAV in memory; no audio files are shipped or
imported): hit, critical, glancing blow, death and a UI tick for doors and new
runs. Final audio is a separate effort.

Playback consumes core outcomes with its own last-seen sequence, so nothing is
replayed; at most four voices start per frame. Only perceivable events (the
player involved or a visible actor) make sound.

`AudioSettings` is independent of other volume controls: `M` toggles mute,
`9` / `0` lower/raise volume in 10% steps (0–100%, default 70%). Keys act only
in a focused window.

Requires Bevy's `bevy_audio` and `wav` features. Without an audio device (CI,
headless) Bevy logs a warning and the game runs silently; voices are expired
so they do not accumulate.
