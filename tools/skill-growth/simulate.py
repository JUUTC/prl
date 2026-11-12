#!/usr/bin/env python3
"""Compare experimental skill-growth curves without gameplay RNG or runtime hooks."""

from __future__ import annotations

from dataclasses import dataclass

PPM = 1_000_000
MICRO_TENTHS = 1_000_000
BASE_GAIN_PPM = 400_000
DECAY_K_TENTHS = 250
TRIVIAL_GAP_TENTHS = 200
PRACTICE_FACTOR_PPM = 500_000
PRACTICE_CAP_TENTHS = 250
THRESHOLDS_TENTHS = (100, 250, 500, 750, 1_000)


@dataclass(frozen=True)
class Candidate:
    name: str
    quadratic_gap: bool


CANDIDATES = (
    Candidate("linear-gap", False),
    Candidate("quadratic-gap", True),
)


def challenge_ppm(candidate: Candidate, skill_tenths: int, opponent_tenths: int) -> int:
    """Return candidate challenge weight; only opponents below skill are tapered."""
    gap = max(0, skill_tenths - opponent_tenths)
    if gap >= TRIVIAL_GAP_TENTHS:
        return 0
    linear = (TRIVIAL_GAP_TENTHS - gap) * PPM // TRIVIAL_GAP_TENTHS
    if candidate.quadratic_gap:
        return linear * linear // PPM
    return linear


def expected_gain_microtenths(
    candidate: Candidate,
    skill_tenths: int,
    opponent_tenths: int,
    *,
    repeat_index: int = 0,
    practice: bool = False,
) -> int:
    """Expected gain per opportunity in millionths of one 0.1 skill point."""
    if skill_tenths < 0 or opponent_tenths < 0 or repeat_index < 0:
        raise ValueError("skill, opponent rating, and repeat index must be non-negative")
    if practice and skill_tenths >= PRACTICE_CAP_TENTHS:
        return 0

    rate = BASE_GAIN_PPM
    rate = rate * challenge_ppm(candidate, skill_tenths, opponent_tenths) // PPM
    decay_ppm = DECAY_K_TENTHS * PPM // (DECAY_K_TENTHS + skill_tenths)
    rate = rate * decay_ppm // PPM
    repeat_ppm = max(200_000, PPM - repeat_index * 100_000)
    rate = rate * repeat_ppm // PPM
    if practice:
        rate = rate * PRACTICE_FACTOR_PPM // PPM
    return rate


def expected_gain_per_thousand_actions(
    candidate: Candidate,
    skill_tenths: int,
    opponent_tenths: int,
    opportunities_per_thousand: int,
) -> int:
    if opportunities_per_thousand < 0:
        raise ValueError("opportunity frequency must be non-negative")
    return (
        expected_gain_microtenths(candidate, skill_tenths, opponent_tenths)
        * opportunities_per_thousand
    )


def opportunities_to_threshold(
    candidate: Candidate,
    threshold_tenths: int,
    *,
    repeated_target: bool = False,
    practice: bool = False,
) -> int | None:
    """Simulate matched-rated opportunities to a threshold under one scenario."""
    if threshold_tenths < 0:
        raise ValueError("threshold must be non-negative")
    if threshold_tenths == 0:
        return 0
    if practice and threshold_tenths > PRACTICE_CAP_TENTHS:
        return None
    skill_microtenths = 0
    threshold_microtenths = threshold_tenths * MICRO_TENTHS
    for opportunities in range(1, 10_000_001):
        skill_tenths = skill_microtenths // MICRO_TENTHS
        gain = expected_gain_microtenths(
            candidate,
            skill_tenths,
            skill_tenths,
            repeat_index=opportunities - 1 if repeated_target else 0,
            practice=practice,
        )
        if gain == 0:
            return None
        skill_microtenths += gain
        if practice:
            skill_microtenths = min(
                skill_microtenths, PRACTICE_CAP_TENTHS * MICRO_TENTHS
            )
        if skill_microtenths >= threshold_microtenths:
            return opportunities
    raise RuntimeError("threshold was not reached within the simulation limit")


def format_points(microtenths: int) -> str:
    ten_thousandths = microtenths // 1_000
    return f"{ten_thousandths // 10_000}.{ten_thousandths % 10_000:04d}"


def format_skill_tenths(skill_tenths: int) -> str:
    return f"{skill_tenths // 10}.{skill_tenths % 10}"


def main() -> None:
    print("EXPERIMENTAL ONLY - expected values, not gameplay rules or accepted balance")
    print(
        "Shared assumptions: base gain 0.4, decay K=25.0, trivial cutoff=20.0 "
        "skill points below, repeat penalty floor=0.2, practice half-rate/cap=25.0."
    )
    print("\nMatched-rated opportunities to reach each skill:")
    print(
        "candidate       "
        + " ".join(
            f"{format_skill_tenths(threshold):>8}" for threshold in THRESHOLDS_TENTHS
        )
    )
    for candidate in CANDIDATES:
        for label, kwargs in (
            ("varied field", {}),
            ("same target", {"repeated_target": True}),
            ("safe practice", {"practice": True}),
        ):
            counts = [
                opportunities_to_threshold(candidate, threshold, **kwargs)
                for threshold in THRESHOLDS_TENTHS
            ]
            rendered = [str(count) if count is not None else "N/A" for count in counts]
            print(f"{candidate.name + ' ' + label:<25} " + " ".join(
                f"{count:>8}" for count in rendered
            ))

    print("\nChallenge weight against opponents below the current skill:")
    print("candidate       gap=0  gap=5  gap=10  gap=20 (skill points)")
    for candidate in CANDIDATES:
        weights = [
            challenge_ppm(candidate, 500, 500 - gap * 10) // 10_000
            for gap in (0, 5, 10, 20)
        ]
        print(f"{candidate.name:<15} " + " ".join(f"{weight:>5}%" for weight in weights))

    print("\nExpected gain per opportunity at skill 50.0:")
    print("candidate       opponent=0  opponent=40  opponent=50  opponent=60")
    for candidate in CANDIDATES:
        gains = [
            format_points(
                expected_gain_microtenths(candidate, 500, opponent_tenths)
            )
            for opponent_tenths in (0, 400, 500, 600)
        ]
        print(f"{candidate.name:<15} " + " ".join(f"{gain:>11}" for gain in gains))

    print("\nExpected skill-point gain per 1,000 actions under an explicit opportunity rate:")
    print("candidate       skill  opportunities/1000  expected_gain")
    for candidate in CANDIDATES:
        for skill_tenths in (0, 500):
            for opportunities in (1, 10, 100):
                gain = expected_gain_per_thousand_actions(
                    candidate, skill_tenths, skill_tenths, opportunities
                )
                print(
                    f"{candidate.name:<15} {format_skill_tenths(skill_tenths):>5}"
                    f" {opportunities:>19} {format_points(gain):>14}"
                )


if __name__ == "__main__":
    main()
