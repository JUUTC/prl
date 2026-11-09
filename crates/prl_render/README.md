# P1 placeholder view

`PrlRenderPlugin` requires Bevy's 2D renderer and `PrlCorePlugin`. Startup must
insert a successfully loaded `CoreState`; the renderer never constructs rules.
The existing binary composes these plugins.

All visuals are repository-owned coloured rectangles, not final PRL assets or
palette. Map cells occupy 64 logical world units. The camera follows the player;
`+`/`-` (including numpad) select integer 1–4× world zoom. UI scaling is independent.
Terrain uses core's remembered definition/door state and current visibility;
actors come exclusively from `visible_actors()`. Unknown cells stay black.
Door bars and differently shaped actors supplement colour.

Refresh runs in `CoreSet::Presentation`, after command resolution, only when
`CoreState` changes. There are no simulation writes or animation-completion waits.
Final textures, font/style assets and cosmetic tweening remain deferred.
