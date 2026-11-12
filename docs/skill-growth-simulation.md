# Experimental Skill-Growth Simulation

Status: **exploratory only**. This report does not select a final growth formula
or balance target. Q-010/I-006 remain open, and these curves are not connected
to `prl_core`, gameplay RNG, saves or the player skill state.

Run the deterministic fixed-point comparison and its tests with:

```sh
python3 tools/skill-growth/simulate.py
python3 -m unittest discover -s tools/skill-growth -v
```

## Candidate assumptions

Both candidates use expected values rather than random success rolls. One
opportunity can grant at most 0.1 skill, with a baseline 0.4 probability before
the other factors. Skill decay is `25 / (25 + skill)`. A target rated at or
above the trained skill receives full challenge weight. Below the skill, the
gap is tapered to zero across 20 skill points:

| Candidate | Challenge weight for a below-skill target |
|---|---|
| `linear-gap` | `max(0, 1 - gap / 20)` |
| `quadratic-gap` | `max(0, 1 - gap / 20) ^ 2` |

Both candidates also use a repeat multiplier of `max(0.2, 1 - 0.1 * repeat
index)`, where the first use has index zero. Safe practice runs at half rate
and stops at 25.0. These numerical choices are test parameters, not accepted
design decisions. The simulator uses integer fixed-point arithmetic and no
RNG; it approximates expected progress and does not model discrete player
outcomes.

## Results

The threshold run assumes every varied-field opportunity is against an
opponent rated exactly at the current skill. Same-target runs instead apply
the repeat penalty every opportunity. The practice scenario has the same
matched-rating assumption and the experimental 25.0 cap.

| Scenario | To 10 | To 25 | To 50 | To 75 | To 100 |
|---|---:|---:|---:|---:|---:|
| Varied field, either curve | 300 | 936 | 2,497 | 4,684 | 7,495 |
| Same target, either curve | 1,480 | 4,663 | 12,470 | 23,401 | 37,458 |
| Safe practice, either curve | 599 | 1,873 | Not reachable | Not reachable | Not reachable |

The challenge curves differ when targets are below current skill: at gaps of
5 and 10 points, the linear candidate weights are 75% and 50%, while the
quadratic candidate weights are 56% and 25%. Both reach zero at a 20-point
gap. Above-skill targets receive full weight in this candidate model.

For a fixed skill of 50.0, example expected gains per opportunity are:

| Candidate | Opponent 0 | Opponent 40 | Opponent 50 | Opponent 60 |
|---|---:|---:|---:|---:|
| `linear-gap` | 0.0000 | 0.0066 | 0.0133 | 0.0133 |
| `quadratic-gap` | 0.0000 | 0.0033 | 0.0133 | 0.0133 |

These are illustrative ratings chosen to exercise trivial, below-skill and
at/above-skill branches. They do not define how actual enemies are rated.

Opportunity frequency changes the effective pace substantially. At 0 skill,
the model yields 0.04 expected skill points per meaningful opportunity; at
50.0 it yields 0.0133. For example, 100 opportunities per 1,000 actions
correspond to 4.0 expected points at 0 skill and 1.333 at 50.0, while one
opportunity per 1,000 actions yields 0.04 and 0.0133 respectively. These are
action-count examples only; no actions-per-hour or real-time pacing is assumed.

The matched-rating threshold rows are identical for both candidates by
construction. The results show the effects of decay, repetition, practice
limits and opportunity frequency; they do not establish which challenge curve
is preferable or whether either curve fits real skill opportunities.

## Remaining decisions

- Define how combat opponents receive a rating and how that rating relates to
  skill, gear, stats and encounter context.
- Calibrate gains from real action opportunities, including infrequent skills
  such as Identification and Meditation.
- Decide repetition windows/target categories and safe-practice activities and
  ceilings.
- Compare against observed playtest action frequencies before setting any
  real-time pacing target.
- Revisit Stealth, racial aptitudes and ability thresholds separately; this
  simulation makes no decisions about them.
