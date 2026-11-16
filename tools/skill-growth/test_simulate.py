"""Regression tests for the experimental, deterministic growth simulator."""

import unittest

import simulate


class SkillGrowthTests(unittest.TestCase):
    def test_candidate_curves_are_equal_at_or_above_rating_and_diverge_below_it(self):
        linear, quadratic = simulate.CANDIDATES
        self.assertEqual(simulate.challenge_ppm(linear, 500, 600), simulate.PPM)
        self.assertEqual(simulate.challenge_ppm(quadratic, 500, 600), simulate.PPM)
        self.assertGreater(
            simulate.challenge_ppm(linear, 600, 500),
            simulate.challenge_ppm(quadratic, 600, 500),
        )

    def test_trivial_targets_stop_producing_gain(self):
        for candidate in simulate.CANDIDATES:
            self.assertEqual(simulate.challenge_ppm(candidate, 500, 300), 0)
            self.assertEqual(
                simulate.expected_gain_microtenths(candidate, 500, 300), 0
            )

    def test_candidate_curves_compare_easy_meaningful_and_above_skill_opponents(self):
        linear, quadratic = simulate.CANDIDATES
        linear_easy = simulate.expected_gain_microtenths(linear, 500, 400)
        quadratic_easy = simulate.expected_gain_microtenths(quadratic, 500, 400)
        meaningful = simulate.expected_gain_microtenths(linear, 500, 500)
        above_skill = simulate.expected_gain_microtenths(linear, 500, 600)
        self.assertGreater(linear_easy, quadratic_easy)
        self.assertGreater(meaningful, linear_easy)
        self.assertEqual(above_skill, meaningful)

    def test_repetition_reduces_gain_and_has_a_bounded_floor(self):
        for candidate in simulate.CANDIDATES:
            first = simulate.expected_gain_microtenths(candidate, 100, 100)
            repeated = simulate.expected_gain_microtenths(
                candidate, 100, 100, repeat_index=5
            )
            exhausted = simulate.expected_gain_microtenths(
                candidate, 100, 100, repeat_index=100
            )
            self.assertLess(repeated, first)
            self.assertGreater(exhausted, 0)
            self.assertEqual(
                exhausted,
                simulate.expected_gain_microtenths(
                    candidate, 100, 100, repeat_index=8
                ),
            )

    def test_safe_practice_is_half_rate_and_stops_at_its_cap(self):
        candidate = simulate.CANDIDATES[0]
        field = simulate.expected_gain_microtenths(candidate, 0, 0)
        practice = simulate.expected_gain_microtenths(
            candidate, 0, 0, practice=True
        )
        self.assertEqual(practice, field // 2)
        self.assertGreater(
            simulate.expected_gain_microtenths(
                candidate, simulate.PRACTICE_CAP_TENTHS - 1, 10_000, practice=True
            ),
            0,
        )
        self.assertEqual(
            simulate.expected_gain_microtenths(
                candidate, simulate.PRACTICE_CAP_TENTHS, 10_000, practice=True
            ),
            0,
        )

    def test_threshold_counts_are_deterministic_and_slow_at_higher_skill(self):
        for candidate in simulate.CANDIDATES:
            first = [
                simulate.opportunities_to_threshold(candidate, threshold)
                for threshold in simulate.THRESHOLDS_TENTHS
            ]
            second = [
                simulate.opportunities_to_threshold(candidate, threshold)
                for threshold in simulate.THRESHOLDS_TENTHS
            ]
            self.assertEqual(first, second)
            self.assertEqual(first, sorted(first))
            self.assertGreater(first[-1] - first[-2], first[1] - first[0])

    def test_repeated_targets_slow_growth_and_practice_stops_at_its_cap(self):
        candidate = simulate.CANDIDATES[0]
        varied = simulate.opportunities_to_threshold(candidate, 250)
        repeated = simulate.opportunities_to_threshold(
            candidate, 250, repeated_target=True
        )
        self.assertIsNotNone(varied)
        self.assertIsNotNone(repeated)
        self.assertGreater(repeated, varied)
        practice_25 = simulate.opportunities_to_threshold(
            candidate, 250, practice=True
        )
        self.assertIsNotNone(practice_25)
        self.assertIsNone(
            simulate.opportunities_to_threshold(candidate, 251, practice=True)
        )

    def test_action_frequency_scales_expected_gain_without_changing_per_opportunity_rate(self):
        candidate = simulate.CANDIDATES[0]
        one = simulate.expected_gain_per_thousand_actions(candidate, 0, 0, 1)
        ten = simulate.expected_gain_per_thousand_actions(candidate, 0, 0, 10)
        hundred = simulate.expected_gain_per_thousand_actions(candidate, 0, 0, 100)
        self.assertEqual(ten, one * 10)
        self.assertEqual(hundred, one * 100)
        with self.assertRaises(ValueError):
            simulate.expected_gain_per_thousand_actions(candidate, 0, 0, -1)

    def test_invalid_inputs_are_rejected(self):
        with self.assertRaises(ValueError):
            simulate.expected_gain_microtenths(
                simulate.CANDIDATES[0], -1, 0
            )
        with self.assertRaises(ValueError):
            simulate.expected_gain_microtenths(
                simulate.CANDIDATES[0], 0, 0, repeat_index=-1
            )
        with self.assertRaises(ValueError):
            simulate.opportunities_to_threshold(simulate.CANDIDATES[0], -1)


if __name__ == "__main__":
    unittest.main()
