# Open Questions

Canonical running decision register for design and implementation. Delivery stages
are defined in [playtest-plan.md](playtest-plan.md). Review this register at each
stage handoff; add discoveries as they arise, not just at the next interview.

## How to use the register

Keep stable `Q-NNN` IDs. **Open** needs a decision; **Provisional** has a reversible
working default, not an accepted design answer; **Blocked** prevents only the
named dependent task; **Deferred** is outside the current slice; **Resolved**
records an accepted answer and its source. Preserve resolved rows so the rationale
is not lost. New questions must name their impact and next action.

Working defaults below are planning proposals dated 2026-11-08. They are not
claims about answers in an unrecovered interview. When a decision is accepted,
update the owning system doc and retain the ID, resolution and source here.

## Active register

| ID | Question | Status | Impact / needed by | Working default and next action |
|---|---|---|---|---|
| Q-001 | World name, pre-war history and invading host? | Deferred | Campaign lore; after P5 | Keep existing Malgareth history; decide in [lore.md](lore.md) before campaign writing |
| Q-002 | Final game title? | Deferred | Branding; not a playtest blocker | Use PRL; revisit after systems/playtest feedback |
| Q-003 | Names/personalities of the other five gods? | Deferred | Cleric/Paladin content; after P5 | Keep drafts; validate before those character templates are playable |
| Q-004 | Area-level ranges, NG+ multipliers and house costs? | Open | Slice encounter tuning at P3; wider balance later | Use explicitly provisional crypt encounter data; record deaths/damage/run length; defer NG+/house values |
| Q-005 | Does counting crafting/gathering in PL over-match crafters? | Open | Multi-skill/overworld slice after P5 | Preserve all trained skills in PL; later compare combat-only and crafter-heavy characters in identical areas |
| Q-006 | Where are the latest animation/UX session decisions, and what changed? | Resolved; animation superseded by Q-036 | Gameplay/art boundary | Integrated the concurrent art session's [art-production.md](art-production.md) and [Gravebound reference](style/gravebound-p0.md); 2026-11-11 reduces bespoke animation requirements for CDDA reuse. Preserve separate inventory/UI UX work; final pack and visual reconciliation remain Q-036 |
| Q-007 | Movement/attack timing, key repeat, input buffering and motion options? | Provisional | Input/presentation integration at P1/P2 | I-023/B-009 implement reversible defaults: enabled held movement/wait after 500 ms, then 180 ms; F2/F3/F4 adjust, F5/F6 remap/reset (session-only). Stop at visible hostiles, damage, HP <=25%, notable cells and modal/focus/outcome interruptions; never repeat attacks or catch up missed intervals. Walk tween 150 ms Normal / 60 ms Fast / instant Off. Confirm comfort/timing and future persisted options through hands-on feedback; no final art/UX approval |
| Q-008 | Can diagonal movement squeeze past blocked corners; do invalid actions cost time? | Provisional | Movement/pathfinding tests at P1 | No corner squeezing when either orthogonal neighbour blocks passage; invalid intents explain rejection and consume no energy; confirm and document in time-and-turns.md |
| Q-009 | How are facing/frontal shield blocks and physical mitigation ordering defined? | Provisional | Block/armor implementation at P2 | P2 working default: no shields in P2 content, so the block step and facing are not implemented; armor% uses `raw` after crit/glance and before flat armor; single-type physical only. Still needed: facing/arc rules and mixed-damage handling before shields ship; numeric cases in combat.md |
| Q-010 | What challenge probability drives combat skill gains when attacks always hit? | Model direction resolved; tuning open | Combat skill-gain integration at P4 | 2026-11-10 interview: derive challenge from the opponent, not hit/miss; trivial/easy targets give little or no gain. Calibrate opponent rating, repeat penalty and gains by simulation/playtest before implementation |
| Q-011 | Which Windows build profile/linker gives the fastest useful iteration? | Open | Distribution at P0 | Native Windows local builds and one Windows artifact target; benchmark incremental/link times and package size before enabling dynamic linking or changing linker |
| Q-012 | Checkpoint interval and crash recovery contract? | Provisional | Persistence at P4 | Working default implemented in I-030: every 20 committed player actions and floor transition, plus quit-save; confirm interval through playtest/write-cost evidence. Death uses a durable tombstone before removing the living slot, but the post-command write leaves a small hard-crash window; checkpoints may roll back progress since the last save. I-029/I-030 cover atomic storage and runtime lifecycle, not write-ahead death atomicity. Same-directory file sync then atomic replacement; Unix directory sync follows. Windows sudden-power-loss metadata durability remains unproven; do not call interrupted-temp-file tests a power-cut guarantee |
| Q-013 | Minimum Windows version/GPU and performance/readability budgets? | Open | Launch compatibility at P0; acceptance at P5 | Target Windows x86_64; test 1080p/4K and multiple UI scales; obtain tester hardware, measure latency/frame time, then set supported minimums and budgets |
| Q-014 | Final palette base, placeholder asset subset and title font? | Open | Asset integration at P0/P1; visual sign-off later | Existing 64x64/ramp palette/font decisions stand; temporary owned shapes unblock launch; verify CC0 subset/OFL files and credits before importing; record palette exceptions for placeholders |
| Q-015 | Persistent IDs and character/world/map ownership across reset and NG+? | Provisional | P3 local instance implemented; review before overworld/resets | P3 stores one run-local instance: MapIds 1/2 are floor ordinals, actor IDs allocate across both floors, item/container allocators are separate. Inactive floors persist but freeze without catch-up/respawn. This is not a world/reset descriptor; review scope and migrate before resets/NG+ ([foundation-plan.md](foundation-plan.md)) |
| Q-016 | Stable seed/domain encoding, RNG state format and canonical snapshot hash? | Provisional | Review before further domains/formats | Combat retains ChaCha8 seed/stream/word-position. Snapshot v4 adds per-run kill counters/completeness; v2 arenas/v3 crypts migrate with past kills unknown. Canonical BLAKE3 compact-RON context remains `PRL core snapshot RON hash v2` ([architecture.md](architecture.md#current-snapshot-proof-i-008-q-016)); v4 golden reviewed. Crypt layout seed v1 remains unchanged. Consumable appearance v1 hashes seed LE-u64 plus definition ID; multi-type appearance permutation remains open. These are golden-tested reversible defaults, not interview answers |
| Q-017 | How do render, audio, logs and later feats consume ordered outcomes without loss? | Provisional | Foundation F3/F5 | Sequenced core outcomes, independent consumer fan-out and bounded presentation retention; snapshot contains state, not animations; define reset, overflow and lag handling before integrating consumers |
| Q-018 | Production art lock: palette, anatomy/perspective, authoring tools, frame/timing and improvement variants? | Open | Art P0-P2, not gameplay placeholder P1 | Follow [art-production.md](art-production.md); [P0 reference package](style/gravebound-p0.md) is ready for review but not approved. Original palette is not locked/verified Lospec-derived; LibreSprite export feasibility passed; palette reuse terms and author/tool selection remain open |
| Q-019 | Where do combat numbers come from before stats and skills exist (P4)? | Provisional | P2 base/P3 equipment; replace base derivation at P4 | Actor definitions supply placeholder HP/accuracy/evasion/armor/damage/crit multiplier (150); pure `derive_stats` now adds P3 gear bonuses. Actual combat includes unknown affixes; known-only comparisons do not. No damage variance roll. Fixed item bonuses, heal 24 and temporary 60 kg hard pickup cap are slice tuning, not approved balance; replace base/carry derivation with stats/skills at P4 and tune with Q-004 |
| Q-020 | P2 enemy awareness, chase and door rules? | Provisional | P2 AI | An enemy is aware while it stands in a cell the player can see (shared FOV), remembers the player's last seen cell, chases by deterministic 8-way shortest path with the Q-008 corner rule, attacks when adjacent, waits when unaware or path-blocked, and cannot open doors. Revisit with noise/factions after P2 playtest |
| Q-021 | What does retry/new run mean before saves and character creation? | Provisional | P2/P3 death and retry | Death ends the run; only new run is accepted. Runtime RunFactory regenerates both crypt floors/supplies at previous seed + 1 and increments run number, preserving outcome sequence continuity. Failed construction rejects explicitly and preserves the old state. Guardian completion is a one-time slice outcome, not campaign victory. Replace setup/death handling with character creation/morgue at P4 |
| Q-022 | Optimizer objectives, constraints and comparison reference target? | Provisional | Gear tools after P5; `derive_stats` shape at P3 | Presets (balanced, offence, defence, resist caps, carry weight) plus custom weights; constraints for locked slots, unknown/cursed items, requirements, encumbrance and set bonuses; reference target = training dummy at current area level. Confirm presets and weights with the user before building the optimizer |
| Q-023 | How are storage tabs obtained, priced and capped, and which tab types exist? | Provisional | First town/housing slice | In-game gold or play rewards only, never real money; types General, Material, Consumable, Gear, Favorites; houses grant free tabs by tier (2/4/8/16); a per-location cap and prices to be set in balance. Confirm types, cap and costs |
| Q-024 | Do storage transfers and refills cost game time? | Provisional | First town/housing slice | Free at storage when no hostiles are in view; disallowed in hostile range; any time cost would follow time-and-turns.md |
| Q-025 | How many kits per character, and do kits pin specific items or only definitions? | Provisional | First town/housing slice | Several named kits; definition + quantity by default, with optional pinned item IDs; kit cells act as locks for Deposit all |
| Q-026 | Which storage can a crafting station draw from? | Provisional | First crafting slice | Carried inventory plus storage at the same site (house storage at a house, the bank at a town station) and an adjacent pack animal; carried first, then Material tabs, then General; materials reserved at start, consumed on completion |
| Q-027 | Loot filter presets, protections, scope and auto-pickup defaults? | Provisional | `ItemPredicate` + Default preset at P3; custom filters after P5 | Built-in read-only presets (Default, Strict, Very strict, Crafter, Show all, Manual) as content data; custom filters copied from presets, stored in the profile, chosen per character, plain-text import/export; Unique/Set/quest/wanted items cannot be hidden; auto-pickup uses normal pickup commands and time, stops when a hostile is in view or before over-encumbrance; gold auto-pickup on by default. Confirm preset list and default auto-pickup set |
| Q-028 | Which source resolutions and scaling are readable for selected tiles? | Open | CDDA asset proof I-039; distinguish logical grid from texture resolution | Keep 64×64 logical tiles and integer zoom. Audit chosen sprites, scaling and readability in-world; source cell need not be 64×64. Earlier rigged-3D comparison is not a prerequisite for CDDA reuse |
| Q-029 | How should art gaps and edge treatment be addressed? | Open | CDDA coverage review I-039 | Select and audit reusable assets first. The earlier rigged-3D/Blender pipeline and vendor research remain optional background, not prerequisites; decide missing-asset production and edge treatment after coverage review |
| Q-030 | Inventory commitment, targeted reviews and unknown-scroll reading policy? | Inventory UX implemented; scroll-use contract resolved, tuning open | I-034/I-035 delivered; scroll read in I-038 | Safe actions stay open; after resolution close on visible hostile or damage; free browsing in danger. Persistent Drop/unknown-consumable/threatened-Body warnings remain independently configurable. Unknown scroll: choose scroll first, then any ordinary target (including self); do not filter targets by hidden effect. A mismatch commits as a bounded fizzle/weaken/backfire variant, costs 100 energy and consumes the scroll. Magic reduces misfire probability/severity; Expert (75) removes misfires. Reading reveals only the observed outcome, never the type automatically. Separate 100-energy ID attempt uses Identification skill, scales difficulty with level/rarity, and failure neither consumes nor harms. Exact spell+level identification reveals matching copies for this run only. Existing unknown-consumable warning applies. Probability, effect-specific variants and ID formula need simulation/playtest |
| Q-031 | What are same-bag packing/rotation, drag, sorting and stack-split command/cost semantics? | Organization and stack direction resolved; bounded core stack foundation delivered, richer content open | I-036 delivered; stack foundation in I-028 | Organization is free, atomic, explicit best-effort; pickup never silently repacks. Compatible consumables/materials/ammo auto-merge on pickup/transfer up to content limits; gear, uniques and scrolls never stack; unknown consumables remain separate. Implemented foundation uses exact definition identity, identified Healing items, optional limits (default one), one/half/custom free split actions, whole-stack Drop, per-unit use and snapshot v6 singleton migration. No shipped definition currently opts in. Material/ammo kinds, other compatibility fields and automatic merging immediately after identification remain open |
| Q-032 | What Magic Book/Codex stores and how it exposes spells/scrolls? | Interaction contract resolved; implementation in I-038 | Magic/scroll slice after I-037 | One carried book stores physical scrolls (effectively unlimited slots, contents weight still counts) and is acquired by purchase, loot or crafting. Magic-class or Magic>=10 starts with one, charged against kit value; no-book pickup falls back to backpack. Auto-file existing/future scrolls when a book is acquired. Scrolls can be extracted/used directly; learned spells are separate permanent knowledge and cannot be extracted. One interface has separate Learned Spells/Carried Scrolls, unlimited favorites and eight click/1-8 quick slots. Known slot binds family+preferred level; no silent level substitution. Unknown slot pins one physical instance and empties on use, never refills automatically. Scroll index: Unknown, then school/category > family > level. Unextracted scrolls travel with a dropped/stored book. Quick-slot and taxonomy implementation details follow playtest |
| Q-033 | How should character creation offer class defaults, customization and random rolls? | Mode contract resolved; budget/economy tuning open | Character creation I-037; I-046 foundation; Q-010 and skills system | Class is a starting template, never an equipment restriction. Independent Default/Customize/Random paths for stats, skills and gear, plus Random Character (unlocked race/class and all character sections, not world settings). Custom class switching preserves edits; explicit Reset to Class Defaults. Custom stats: 27 points, 8–15 pre-race. Random class-weighted stats: 24–30 points, max18 pre-race; cost 16=12,17=15,18=18; race modifiers follow. Default class stats use 27. Skills: class/default and custom use 150 points, any skill, max50 each; random class-weighted uses 180, same cap. Random gear is class-weighted with off-theme chance and a rare above-budget jackpot, no debt; custom gear uses full starter catalogue within the class kit's value budget. Randomized section cannot be manually edited, but can be rerolled without limit before final confirmation; other sections remain independent. Character-roll RNG is independent of world seed. I-046 validates stat/skill allocations and draft transitions with caller-supplied rolls, but does not generate rolls, enforce gear budgets or validate unlocked classes/races. Exact class defaults, distributions, prices and jackpot odds need balance testing |
| Q-034 | What is the starting Magic/Identification skill model? | General Magic and optional ID resolved; details open | Skills/class creation I-037; scroll system I-038 | One Magic skill improves all casting; schools are descriptive categories. Stats may gate casting gear, not Magic skill. Optional Identification has non-skill fallbacks. High skill has one chance per identifiable physical item to add a property, not merely reveal one; service/scroll ID preserve a later personal chance, including on identified store items. Failure spends the action and chance but leaves the item unharmed. Bonus strengths, costs, odds, per-type properties and ability prerequisites remain open |
| Q-035 | What actions may skills gate, given solo play, slow advancement and permadeath? | Core principle and action policies resolved; skill taxonomy open | Before revising Q-034, ability prerequisites, class allocations or skill catalog | Essential survival and main-path actions cannot be skill-gated. Reliable baseline healing and non-skill identification serve all builds; required routes have non-skill alternatives, with keys expensive or hard to find. Zero-skill weapon use remains possible subject to gear stat requirements. Zero-skill use should reach a basic level quickly, then slow; practice and skill-specific gain rates need tuning. Accepted skill consolidations and training decisions are in [skills-and-progression.md](skills-and-progression.md#interview-decisions-and-revision-status). The next unanswered interview question is whether Stealth remains separate, merges with Survival, or uses stats/equipment instead |
| Q-036 | Which CDDA tileset/audio assets and remaining visual requirements fit PRL? | Animation reduction resolved; subset audited/integrated; final selection/visual coverage open | Presentation reuse I-039; I-040-I-045 | [Integration plan](tileset-integration.md): UltiCa 2026-11-08 is pinned and imported as an opt-in candidate with 18 original PNGs, collective/file-history provenance, original metadata and CC BY-SA notices. New runs add a grassy approach, animated placeholder player option and fantasy-appropriate sprite substitutions. Sword/mail category icons are integrated; modern bottle art is excluded and potion/scroll artwork stays placeholder pending suitable candidates. These are not final visual approval. Resolve foreground occlusion/perspective, fixed-kit versus dynamic equipment, race/appearance coverage, consumable art and whether to select UltiCa as the default. Do not impose the old unapproved palette. Windows hands-on acceptance and separate sound rights remain open |

## Resolution history

Skill-growth simulation prototype (2026-11-12): I-047 now has isolated
deterministic fixed-point tooling comparing two opponent-rating gap curves,
repetition, safe practice and opportunity-frequency sensitivity. Candidate
numbers and results are exploratory only; Q-010 remains unresolved until
opponent rating and real opportunity data are calibrated, and Q-035 practice
activities/ceilings remain open. No gameplay, save or RNG behavior changed.
See [the simulation report](skill-growth-simulation.md).

CDDA presentation pivot (2026-11-11): user approved reducing animation
requirements to fit reused tilesets. Static sprites and supplied facings/frames
are acceptable; dedicated action poses, independent left/right sets and
anatomical grip animation are no longer mandatory. The art-production current
direction override supersedes older custom-animation gates. Pack selection and
remaining visual-contract reconciliation are Q-036/I-039.

Identification bonus clarification (2026-11-10): high Identification can add a
new bonus property (affix/prefix), not merely reveal an existing hidden one.
There is one bonus chance per physical item; repeated attempts cannot farm
properties. Follow-up supersedes the initial "first successful identification"
timing: town-service identification does not grant the chance or forfeit the
player's later skill-based attempt. Identification scrolls likewise reveal
identity without spending the personal bonus attempt. Already identified store
items are also eligible for that one personal attempt. Any item that can be
identified is eligible, not just equipment. A failed bonus attempt spends the
action and the item's one chance, but never damages, consumes or weakens the
item. Bonus strength, odds, action cost and compatible properties for each item
type remain open.

Meditation and training frequency (2026-11-10): retain Meditation as a separate
skill, naturally trained through mana recovery. Provide a practical way to
train it; the specific activity remains open. Skill gains need not be uniform:
balance by realistic opportunities to use each skill. Identification and other
infrequent tasks may need different gains from frequent casting or attacks.
Safe practice (such as meditation, training dummies or study) may build basic
competence; meaningful field use is required for advanced mastery. Exact
per-skill rates, practice ceilings and training activities remain to be designed.

Magic Resistance (2026-11-10): retain a separate defensive skill with
frequency-adjusted training, alongside stat/equipment contributions. Total
magic damage reduction and hostile-effect resistance chance are each capped
at 90% regardless of skill or gear; higher skill can replace resistance
otherwise needed on gear, never raise either ceiling. Exact combination
formulas and training rates remain open.

Companion skills (2026-11-10): use one Companion Handling skill for all
companions rather than separate living-beast and magical-summon handling
tracks. Basic follow/stay/attack/dismiss commands work reliably at zero
Companion Handling; higher skill improves capacity and advanced coordination,
not basic obedience. Spell schools remain categories under Magic; the division between
casting a summon and controlling it, and migration of Animal Taming ability
requirements, remain to be specified.

Crafting skill review (2026-11-10): consolidate the seven professions into
three tracks: Equipment Crafting (smithing, tailoring, woodwork), Alchemy &
Cooking (potions, food, poisons), and Arcane Crafting (scrolls, books,
enchantments). Recipes retain their appropriate materials and tools.
Consolidate gathering into Extraction (ore/wood) and Harvesting
(herbs/animal materials), retaining appropriate tools and resource locations.
Combine Lockpicking and Traps into Security, covering locks and mechanical
traps; required routes still need a viable non-skill alternative.
Combine Tracking and Healing into Survival for tracking, field medicine and
wilderness knowledge; baseline healing remains available to every build.
Remove Bartering as a skill; shop-price influence uses Charisma only. The
interview's reputation suggestion was not accepted and does not introduce a
reputation system.
Remove Tactics as a skill; its general weapon damage/critical benefits instead
belong to weapon-family skills and stats.
Keep Shields as a separate skill. Remove Dual Wielding as a separate track;
its benefits belong to weapon-family skills and stats. Basic equipment use
does not require a skill threshold.
Keep Light Armor and Heavy Armor as separate skills; heavy armor may still
have stat requirements.
Keep Athletics as a separate skill improving exertion/movement efficiency;
ordinary movement and required traversal remain possible at zero skill.

Inventory interview, 2026-11-10 (user): keep inventory open after actions and
close automatically if danger appears. This supersedes Q-030's proposed
one-action/world-return default. After an action resolves, any visible hostile
or damage taken closes inventory; opening/browsing while threatened is allowed
and free. Remaining review/read policies are still being interviewed; update
the owning UX contract together once those answers are captured.

Inventory warning policy (same interview): confirmations must be configurable.
Warn on the first and subsequent occurrences unless the player unchecks a
checked "Warn me next time" box. Separate settings for dropping, unknown
consumables and slow Body changes in danger persist across runs and restarts;
Options can re-enable each. Do not turn this into a one-time-only warning.

Unknown-scroll reading (initial interview; resolved in the follow-up below):
the starting direction was target-first, but the user clarified that hidden
effects make that flow inappropriate. Preserve this history; the later
scroll-first, ordinary-target contract is authoritative.

Packing (same interview): rearranging within the backpack is free organization,
including while enemies are visible. Equip/use/pickup/drop retain their normal
costs. Allow 90-degree rotation during placement, also free. Support drag/drop,
click-to-pick-up/click-to-place and keyboard placement/rotation through the same
rules. Escape cancels; invalid placement leaves the item at its original
position. Automatic packing may move existing items only through an explicit
Organize button; never silently on pickup. Organize is free, deterministic
best-effort packing and may rotate items. Commit only if every item fits;
otherwise preserve the entire original layout and explain the failure. No
guarantee to find every mathematically possible arrangement is required.

Q-006 resolved by integrating the concurrent art session's committed documents.
Q-007/Q-014/Q-018 remain open/provisional: prototype timing, colours and procedural
shapes are not production approvals. Diagonal collision remains Q-008.

Q-012 follow-up: confirmed abandonment now archives the report before committing
an `.abandoned` barrier; after that barrier, the old run cannot Continue.
A crash or failed barrier publication before commitment can leave a report of
the confirmed request while the old checkpoint remains continuable. This is
not an atomic history/write-ahead guarantee. Both death and abandonment barriers
are recovered before Continue; reports from both are retained if a replacement
run dies before its first successful save.

Inventory, magic and character-creation follow-up, 2026-11-10 (user interview):

- Unknown scroll: choose the scroll first; then the normal target picker must
  permit self, visible actors and other ordinarily targetable cells/objects
  without consulting the hidden effect. A mismatch is not rejected for free:
  it commits and resolves as a bounded fizzle/weaken/backfire variant of that
  scroll. Every committed read consumes the scroll and costs 100 energy,
  including misfires; chooser/target setup and cancellation are free. Magic
  lowers misfire chance and severity; Expert Magic removes it. Reading reveals
  only the observed outcome, never automatically identifies the type. The
  existing configurable unknown-consumable warning applies.
- Deliberate ID is a separate 100-energy Identification-skill attempt on one
  scroll. Difficulty scales with its level/rarity; failure is harmless and
  leaves it unconsumed. Success identifies that exact spell+level family for
  this run, including present and future matching copies, but not other levels
  or spells. Magic trains from magic actions; ID trains from ID attempts with
  per-target diminishing returns/one meaningful award. No automatic discovery
  from reading, repetition or high skill. Curves and effect-specific variants
  require simulation and playtests.
- Magic Book/Codex: one real carried container for physical scroll items with
  effectively unlimited slot capacity; stored scroll weight still counts.
  Buy/find/craft books; a magic-focused class or at least 10 starting Magic
  receives a basic book and small starter kit, charged to the kit budget.
  Without a book, scroll pickup falls back to the backpack; acquiring the first
  book auto-files existing and future scrolls. Scrolls remain extractable and
  usable directly; learned spells are a separate section and permanent
  knowledge, not extractable items. Scrolls remain individual instances and
  never stack. One interface has separate Learned Spells/Carried Scrolls,
  unlimited favorites and eight click/1-8 quick slots. Identified scrolls are
  indexed school/category -> family -> level; unknowns share an Unknown section.
  Known quick slots select a family and preferred level, never silently
  substitute another; unknown quick slots bind an exact instance and never
  auto-refill after use. Unextracted contents travel with a dropped/stored book.
- Starting builds: class defaults are templates, not restrictions. Independently
  choose Default, Customize or Random for stats/skills/gear, or Random Character
  for an unlocked race/class plus all character sections; world settings/seed
  stay untouched. Switching class preserves customized sections and offers
  explicit Reset to Class Defaults. Custom stats keep 27 points and 8-15
  pre-race; random class-weighted stats roll 24-30 and may reach 18 pre-race,
  with costs 16=12, 17=15, 18=18; race modifiers apply afterward. Class/default
  and custom skills use 150 points; random skills use 180; no starting skill
  exceeds 50. Custom gear uses the full starter catalogue within class kit
  value, with no class gear locks. Random gear is class-weighted, sometimes
  off-theme, can be worse or rarely jackpot above budget without debt. A random
  section cannot be manually edited, but may be rerolled repeatedly before final
  confirmation; separate character-roll RNG never alters the world seed.
- Stacks: compatible consumables/materials/ammo auto-merge on pickup/transfer
  under per-definition limits. Unknown consumables remain separate until
  identified; gear, unique items and scrolls remain separate. Quantity split
  offers one/half/custom-quantity shortcuts; transferring or dropping a stack
  acts on all its units. Exact post-identification merge timing and
  compatibility keys remain implementation questions.
- Magic skill replaces separate trained spell-school skills; schools remain
  descriptive categories. Meditation and Magic Resistance stay trained skills.
  Identification (ID) replaces Item Lore. Starting skill totals and per-skill
  creation cap are 150/default or custom, 180/random, and 50 respectively.
  Class allocations must be rebuilt around the revised catalogue while
  preserving totals; exact distributions are open.

Skills-design interview (2026-11-10): user emphasized that slowly progressing
skills can make players unable to perform needed actions, and PRL is a solo
permadeath game without an MMO guild/friend safety net. Resolved principle:
essential survival and main-path actions must never be blocked by a skill;
skills may improve those actions and may gate optional specialist capabilities.
The inventory of essential actions and classification of existing skill gates
remain to be interviewed. This principle supersedes any earlier implication
that a school skill or Identification check may be required to access a
necessary action. In particular, every build must have reliable baseline
healing without Healing/Restoration investment; those skills may improve
recovery or add optional stronger methods. Basic item/scroll identification
also has a fallback for every build, such as a paid town service or an
identification scroll; Identification skill must not be required.
Required routes cannot be blocked by Lockpicking or Traps: these skills may
grant safer, quieter or more rewarding optional access, while keys provide an
alternate route but are expensive or hard to find as loot.
Every character can use any weapon at a basic level; weapons, heavy armor and
magic-casting items may impose stat requirements instead of weapon-skill gates.
Likewise, stats may gate access to casting gear, while Magic skill improves
casting and is not a hard prerequisite to use it.
Zero-skill use should grow to a basic level quickly, then slow as the existing
use-based progression design intends. Low skill can mean failures or lower
quality/quantity rather than an inability to attempt the action.
Keep weapon proficiencies split by family (for example Swords, Maces,
Archery); these separate progression tracks must not prevent basic weapon use.

## Decided

- Big bad: Lich-Emperor with spreading undead blight.
- Blight spreads over time (difficulty-scaled), can be pushed back.
- Pantheon: required only for Clerics/Paladins.
- Art: 64×64 logical tiles with integer zoom (4K-ready); CDDA tileset reuse under review, with source resolution determined by the chosen assets (Q-036).
- Palette: locked ramp-based PRL palette (~24 ramps × 6–8 shades, ~150–190 colours), CI-checked. This is the target; no palette is locked yet (Q-014/Q-018).
- Fonts: Pixelify Sans + Silkscreen (OFL); title font later. Integrated in the placeholder UI (I-010); final type scale belongs to the UX effort.
- UI: bevy_ui for the game; bevy_egui only for dev tools behind a feature flag.
- Tone: grimdark-hopeful high fantasy; terse, archaic-lite prose.
- Character art baseline: Gravebound; two unlabeled silhouettes, five looks per race/silhouette, actual equipped items and gear improvements visible independently of skill-derived PL.
- Production quality: existing procedural sprites are placeholders, not shippable art. A finished character in a representative world scene must be approved before batch expansion; follow the gated [art production plan](art-production.md).
- World perspective: three-quarter overhead, hidden square grid, no visible floor grid, eight-direction movement, half-height foreground walls for character readability.
- Animation: fit the chosen tileset; static sprites and supplied facings are acceptable. Dedicated action poses and anatomical grip animation are not mandatory; gameplay hand assignments remain intact (Q-036).
- Visual coverage: archery draw/release, arrow variants, spell effects, monsters/enemies, townsfolk, animals, tree-felling and ore-depletion states, crafting benches, eating and drinking.
- Atmosphere: optional subtle fog/ash parallax, no shifting interactables or terrain; preserve actor/target readability and offer reduced-motion/off controls.
- Mods/DLC: v1 data-only content packs (namespaced IDs, load order); DLC = first-party packs; WASM scripting deferred.
- Code: Cargo workspace (prl, prl_core, prl_content, prl_gen, prl_render, prl_ui, prl_audio, prl_save); strict lints (clippy all + pedantic as errors, no unsafe, no unwrap/expect outside tests, documented pub items).
- Overworld: separate scaled-down map (ADOM style).
- Meta-progression: everything earnable in Explorer mode.
- Pets: stabled in towns; only control-slot pets travel.
- Audio: basic SFX in v1, music later.
- Skill pacing: ~20+ hours of focused use to reach Master (100).
- No reputation system.
- No skill/stat decay.
- Keyboard + mouse only for v1.
- Malgareth the Undying: betrayed champion; Kaedric demanded a sacrifice he refused (secret quest line).
- Combat: always hit; evasion → glancing blows; armor flat + %; resist cap 75% (90% max) with difficulty penalty.
- Damage types: Physical + Fire, Cold, Lightning, Poison.
- Bestiary: 60–80 base monsters, ~10 families.
- Dungeons: instanced, resettable at entrance; Dungeon Sigil modifiers (risk/reward, rare pure bonus).
- Final dungeon resets grow the blight; victory → win screen → New Game+ (endless tiers, keep character/gear/bank, retempering).
- Skill gain: Hard 150%, Nightmare 250%.
- Overworld encounters: visible + random, area level vs Power Level, weak monsters avoid, repel/attraction consumables, evade prompt.
- Power Level = total trained skill points / 10.
- Inventory: Diablo grid + weight, bags; pack animals (ambush risk shown, trainable, barding).
- Housing: unlimited houses (Small/Medium/Large/Castle), clear area first, separate storage, fast travel, blight sieges.
- Quests: seeded main line + procedural side quests; light narrative.
- Light: dark dungeons, fuelled light sources, darkness aids stealth.
- Saves: auto-save on quit + overwritten crash checkpoint.
