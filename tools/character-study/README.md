# Character atelier

An isolated, dependency-free browser prototype using the approved **Gravebound** baseline.
It does not replace the planned `bevy_ui` implementation or change game rules.

**Quality status: placeholders only.** These procedural sprites are not acceptable
shipping artwork and are not approved production references. Further roster
expansion is gated on the [art production plan](../../docs/art-production.md),
starting with P0's reference package and a finished in-world character approval.
Successful browser checks establish mechanical correctness, not visual quality.

From the repository root:

```sh
python3 -m http.server 8080 --bind 127.0.0.1
```

Open `http://127.0.0.1:8080/tools/character-study/`. Serving is needed for the
JavaScript module; opening the HTML with `file://` is not supported.

## Review

The approved world view is **three-quarter overhead**, with continuous floor
texture, no drawn grid, and half-height walls (18px of raised height in this study).
Movement still uses hidden 64×64 square cells in eight directions. Focus the
scene and use arrows, numpad, or vi keys (`ykuhlbjn`), or use the movement buttons.
Toggle the foreground pillar to inspect lower-body occlusion. Diagonal corner
cutting is prevented in the preview; final gameplay collision rules remain open.
This is an isolated positioning study, not an implemented Bevy gameplay camera.
The world figure uses shortened proportions and independent left/right attachment
poses. Successful horizontal or diagonal movement chooses facing; vertical movement
preserves facing. The world plays a cosmetic walk cycle after each accepted step,
then returns to idle. No game time or inventory is changed.

## Animation proof

Compare both facing sets side by side, play/pause, step through six frames, or
export a 384×128 PNG: six 64×64 frames left-to-right, left-facing row first and
right-facing row second, at 140ms per frame. Playback starts paused when the
browser requests reduced motion. Animation settings are not part of the saved
appearance/gear study.

Independent per-facing poses locate anatomical main/off hands and determine which
arm/item is in front; the renderer never mirrors a composite. The torso reuses
the material study; head, arms, legs, and held gear are posed separately.
Proof buttons load sword/shield, dual daggers, two-handed sword, mining tool,
or casting equipment. They do not change the selected starting class.

Actions: idle, walk, melee attack, crafting/hammering, mining, chopping,
gathering, eating, drinking, bandaging, casting, archery draw/release. Work actions temporarily use demonstration
tools, and consumable/casting actions stow held weapons visually without changing
the equipped configuration. Greatsword, pickaxe, wood axe, and bow reserve the
off hand; selecting them explicitly unequips any off-hand item with a message.
A staff may be held one-handed with a book in this study.

This is an **animation proof**, not production sprite art:
weapon-specific combat, cloth physics, work-contact refinement, other crafting
disciplines, and full directional assets beyond left/right remain future work.

For browser regression checks, run this in the served atelier's developer console:

```js
await (await import("./checks.js")).runChecks()
```

It checks 2,880 frames across races, silhouettes, actions and facings for clipping,
opaque pixel edges, action variation, coordinated grips, anatomical hand identity,
facing retention, non-mirrored art, and two-handed loadout validation, plus 156
world/effects showcase frames for pixel edges, bounds and variation.

## Playtest placeholder atlas

`export.html`/`export.js` render the Human Gravebound baseline in the Warrior
kit (idle, walk and attack; independent left and right rows), skeleton,
shambler and grave-rat placeholders from `creatures.js` (each facing painted
separately, never mirrored, with idle, walk and attack), a seamless floor, a half-height wall
and closed/open doors into one atlas. Regenerate the committed playtest assets
with headless Chrome or Chromium (localhost only, no network):

```sh
python3 tools/character-study/export_placeholder_atlas.py          # write
python3 tools/character-study/export_placeholder_atlas.py --check  # verify
```

Outputs go to `assets/packs/prl/sprites/placeholder/`: the PNG plus a RON
manifest keyed by content ID, which `prl_render` loads. They are disposable
placeholders, not candidates for art P1; do not hand-edit them. PNG bytes may
change with the Chrome encoder version, so `--check` failing after a browser
update only means the files should be regenerated and reviewed.
Pure sprite construction lives in `sprite.js` so it imports without the
atelier UI.

## World & effects showcase

Twenty-six representative six-frame studies cover six spell effects, five arrow
appearances, skeleton/goblin/wraith enemies, smith/merchant townsfolk,
wolf/deer/pack mule animals, tree felling, ore depletion, forge/alchemy/loom
stations, and meal/drink props. These are original procedural studies, not a
complete bestiary, finalized spell roster, or approved arrow gameplay.
Harvest sequences loop only for visual review; they do not replenish game resources.
Use the shared animation Play/Pause and Step frame controls.
Export downloads a 576×1664 transparent PNG plus a JSON row manifest; each row
contains six 96×64 study frames at 140ms. Work props can exceed one world tile.

The gridless world preview has Off / Subtle fog / Fog & drifting ash controls.
Two low-opacity layers use different camera-offset factors as the player moves.
This is a **simulated camera-offset study**, not a scrolling Bevy camera.
Terrain and interactables do not shift; a clear region protects the traveller's
silhouette. Play/Pause controls all drift. Reduced-motion preference starts
playback paused and atmosphere off. Final in-game effects must also protect
targeting cues and honour the game's accessibility settings.

Choose a race,
one of the two visually presented silhouettes, and one of five looks. Skin and
hair colours are independent cosmetic choices. Both silhouettes have the same
options. Choose one of six starting class kits, then change each visible equipment
slot independently. The full-company view shows all 100 looks wearing the current
equipment; selecting a thumbnail opens it in the editor.

Class kits follow `docs/races-and-classes.md`: Warrior chainmail/sword/kite shield,
Rogue leather/two daggers, Mage robe/hood/staff/book, Cleric scale/mace/buckler/symbol,
Ranger leather/bow/quiver, and Paladin banded mail/sword/heater shield/symbol.
Backpack items, the Ranger's sheathed secondary sword, and quick-swap loadouts are
not rendered in this front-view prototype. A shroud is optional: no cloak is
silently equipped by a class kit.

Each visible slot has its own item level and material tier. The six material
families follow `docs/items-and-crafting.md`; they are not inferred from item level.
Item-level **1/25/50/100** samples progressively add fittings, reinforcement,
edge work, and restrained ornament. These are provisional **visual sample bands**,
not gameplay thresholds, stat increases, rarity assignments, or affix rules.
The prototype accepts ilvl 1–200; detailing saturates at 100, without implying a
game item-level cap. PL is not an editor control: it remains derived from trained
skills, and equipment bonuses do not count toward it.

**Keep this study** stores the reference in this browser and downloads JSON settings.
**Export sprite** downloads a transparent native 64×64 PNG including equipped gear.
Settings contain the full per-slot equipment configuration and the approved
Gravebound baseline. Baseline approval is separate from production-asset approval.
Older art-direction studies are rejected with a visible version message.

## Scope and constraints

- Aligned with `docs/SPEC.md` §14: 64×64 native sprites, earthy grimdark-hopeful
  fantasy, integer nearest-neighbour world previews.
- Five starting races and five unlockable races, each with two silhouettes and
  five distinct hair/feature presets: 100 base looks with freely mixed equipment.
  Unlockable anatomy is exploratory, not an additional lore decision.
- Revised after visual feedback: smaller heads, tapered limbs, longer legs,
  asymmetric frayed cloaks, worn leather and iron, and deeply shadowed faces.
  The first toy-like, chunky treatments are superseded.
- Original scanline-rasterized pixel constructions, not generated third-party
  artwork. No external assets, fonts, libraries, or network calls.
- Palette ramps are **provisional**. The locked Lospec-derived PRL master palette
  has not been established; these are not production palette-compliant assets.
- Portraits are enlarged face crops for comparison, **not** the final high-resolution
  portraits. Front-view customization and two-facing action studies are implemented;
  production animation polish and further facing sets remain future work.
- Browser typography is a system-font approximation, not the game's approved
  Pixelify Sans/Silkscreen font setup.
- Keyboard access uses native buttons, Tab, Enter/Space, visible focus, pressed
  states and descriptive accessible names. Silhouettes have no gender labels.

The eventual in-game creation screen should retain race unlock gates, store
appearance independently of gameplay attributes, and use the shared Bevy theme.
Equipment layers must derive from actual equipped item definitions rather than PL
or class. Small accessories such as rings, belt inventory, and hidden backpack
items belong on the paper doll; not every slot needs visible pixels at world scale.
