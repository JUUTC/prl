//! Physical melee damage pipeline (P2 provisional implementation).
//!
//! Spec: `docs/combat.md#damage-pipeline`. All values are integers; damage is
//! carried in hundredths of a hit point ("centi-HP") until the final rounding.
//! Numbers come straight from actor definitions until stats/skills exist
//! (Q-019); mitigation ordering follows the Q-009 working default.

use crate::CombatStats;
use crate::rng::{BASIS_POINTS, CombatRng};

/// Base crit chance: 5%.
const CRIT_BASE_BP: i32 = 500;
/// Crit chance per point of accuracy above evasion: 0.5%.
const CRIT_PER_POINT_BP: i32 = 50;
/// Crit chance bounds: 1%..75%.
const CRIT_MIN_BP: i32 = 100;
const CRIT_MAX_BP: i32 = 7_500;
/// Base glance chance: 10%.
const GLANCE_BASE_BP: i32 = 1_000;
/// Glance chance per point of evasion above accuracy: 2%.
const GLANCE_PER_POINT_BP: i32 = 200;
/// Glance chance bounds: 0%..60%.
const GLANCE_MAX_BP: i32 = 6_000;
/// Armor percentage cap: 85%.
const ARMOR_PERCENT_CAP_BP: u64 = 8_500;
/// Centi-HP per hit point.
const CENTI: u32 = 100;

/// Result of resolving one attack, before it is applied to the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AttackResolution {
    /// Rounded HP after crit/glance and before armor.
    pub(crate) raw: u16,
    /// Rounded HP removed by armor (`raw - damage`, saturating).
    pub(crate) mitigated: u16,
    /// Final HP damage, at least 1.
    pub(crate) damage: u16,
    pub(crate) critical: bool,
    pub(crate) glancing: bool,
}

/// `clamp(5% + (accuracy − evasion) × 0.5%, 1%, 75%)` in basis points.
pub(crate) fn crit_chance_bp(accuracy: i16, evasion: i16) -> u16 {
    let difference = i32::from(accuracy) - i32::from(evasion);
    let chance = (CRIT_BASE_BP + difference * CRIT_PER_POINT_BP).clamp(CRIT_MIN_BP, CRIT_MAX_BP);
    u16::try_from(chance).unwrap_or(0)
}

/// `clamp(10% + (evasion − accuracy) × 2%, 0%, 60%)` in basis points.
pub(crate) fn glance_chance_bp(accuracy: i16, evasion: i16) -> u16 {
    let difference = i32::from(evasion) - i32::from(accuracy);
    let chance = (GLANCE_BASE_BP + difference * GLANCE_PER_POINT_BP).clamp(0, GLANCE_MAX_BP);
    u16::try_from(chance).unwrap_or(0)
}

/// Critical hit: `raw × multiplier%`.
pub(crate) fn apply_crit(raw_centi: u32, multiplier_pct: u16) -> u32 {
    let scaled = u64::from(raw_centi) * u64::from(multiplier_pct) / 100;
    u32::try_from(scaled).unwrap_or(u32::MAX)
}

/// Glancing blow: half damage (integer division).
pub(crate) const fn apply_glance(raw_centi: u32) -> u32 {
    raw_centi / 2
}

/// `min(85%, armor / (armor + 10 × raw))` in basis points, with `raw` in HP.
///
/// Q-009: `raw` is the value after crit/glance and before flat armor.
pub(crate) fn armor_percent_bp(armor: u16, raw_centi: u32) -> u16 {
    if armor == 0 {
        return 0;
    }
    let armor_centi = u64::from(armor) * u64::from(CENTI);
    let denominator = armor_centi + 10 * u64::from(raw_centi);
    let percent = (armor_centi * u64::from(BASIS_POINTS) / denominator).min(ARMOR_PERCENT_CAP_BP);
    u16::try_from(percent).unwrap_or(0)
}

/// Flat armor (`armor / 10` HP), then armor%, both in centi-HP.
pub(crate) fn mitigate_physical(raw_centi: u32, armor: u16) -> u32 {
    let percent = armor_percent_bp(armor, raw_centi);
    let flat_centi = u32::from(armor) * 10;
    let after_flat = u64::from(raw_centi.saturating_sub(flat_centi));
    let kept = after_flat * u64::from(BASIS_POINTS - percent) / u64::from(BASIS_POINTS);
    u32::try_from(kept).unwrap_or(u32::MAX)
}

/// Rounds centi-HP half up to whole HP, saturating.
pub(crate) fn round_centi(centi: u32) -> u16 {
    u16::try_from(centi.saturating_add(CENTI / 2) / CENTI).unwrap_or(u16::MAX)
}

/// `max(1, round(after))`: every connecting attack deals at least 1.
pub(crate) fn final_damage(after_centi: u32) -> u16 {
    round_centi(after_centi).max(1)
}

/// Resolves one physical melee attack. Consumes one crit roll and, unless it
/// crits, one glance roll, so RNG consumption is deterministic per outcome.
pub(crate) fn resolve_attack(
    attacker: &CombatStats,
    defender: &CombatStats,
    rng: &mut CombatRng,
) -> AttackResolution {
    let mut raw_centi = u32::from(attacker.damage) * CENTI;
    let critical = rng.roll_bp(crit_chance_bp(attacker.accuracy, defender.evasion));
    let mut glancing = false;
    if critical {
        raw_centi = apply_crit(raw_centi, attacker.crit_multiplier_pct);
    } else if rng.roll_bp(glance_chance_bp(attacker.accuracy, defender.evasion)) {
        glancing = true;
        raw_centi = apply_glance(raw_centi);
    }
    // Q-009: shield block roll (×0.25, frontal arc) goes here once shields exist.
    let after_centi = mitigate_physical(raw_centi, defender.armor);
    let damage = final_damage(after_centi);
    let raw = round_centi(raw_centi);
    AttackResolution {
        raw,
        mitigated: raw.saturating_sub(damage),
        damage,
        critical,
        glancing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(accuracy: i16, evasion: i16, armor: u16, damage: u16) -> CombatStats {
        CombatStats {
            accuracy,
            evasion,
            armor,
            damage,
            crit_multiplier_pct: 150,
        }
    }

    #[test]
    fn crit_chance_scales_half_percent_per_point_and_clamps() {
        assert_eq!(crit_chance_bp(10, 4), 800);
        assert_eq!(crit_chance_bp(6, 8), 400);
        assert_eq!(crit_chance_bp(0, 200), 100);
        assert_eq!(crit_chance_bp(200, 0), 7_500);
    }

    #[test]
    fn glance_chance_scales_two_percent_per_point_and_clamps() {
        assert_eq!(glance_chance_bp(6, 8), 1_400);
        assert_eq!(glance_chance_bp(10, 10), 1_000);
        assert_eq!(glance_chance_bp(20, 0), 0);
        assert_eq!(glance_chance_bp(0, 30), 6_000);
    }

    #[test]
    fn critical_hit_multiplies_raw_by_percent() {
        assert_eq!(apply_crit(700, 150), 1_050);
        assert_eq!(apply_crit(500, 200), 1_000);
    }

    #[test]
    fn glancing_blow_halves_damage() {
        assert_eq!(apply_glance(800), 400);
        assert_eq!(apply_glance(500), 250);
        assert_eq!(apply_glance(1), 0);
    }

    #[test]
    fn armor_percent_uses_post_crit_raw() {
        // Armor 10 vs 8 damage: 1111 bp before crit, 769 bp after a 150% crit.
        assert_eq!(armor_percent_bp(10, 800), 1_111);
        assert_eq!(armor_percent_bp(10, apply_crit(800, 150)), 769);
        // Shambler crit on the player: (1200 − 100) × (1 − 7.69%) = 1015 → 10.
        assert_eq!(mitigate_physical(1_200, 10), 1_015);
        assert_eq!(final_damage(1_015), 10);
    }

    #[test]
    fn armor_percent_caps_at_85() {
        assert_eq!(armor_percent_bp(100, 100), 8_500);
        assert_eq!(armor_percent_bp(u16::MAX, 1), 8_500);
    }

    #[test]
    fn zero_armor_mitigates_nothing() {
        assert_eq!(armor_percent_bp(0, 700), 0);
        assert_eq!(mitigate_physical(1_050, 0), 1_050);
        assert_eq!(final_damage(1_050), 11);
    }

    #[test]
    fn minimum_damage_is_one() {
        // 1 damage vs armor 10: flat armor removes all 100 centi-HP.
        assert_eq!(mitigate_physical(100, 10), 0);
        assert_eq!(final_damage(0), 1);
        assert_eq!(final_damage(49), 1);
    }

    #[test]
    fn final_damage_rounds_half_up() {
        assert_eq!(final_damage(149), 1);
        assert_eq!(final_damage(150), 2);
        assert_eq!(final_damage(589), 6);
    }

    #[test]
    fn worked_examples_match_combat_doc() {
        // Skeleton (5 dmg) hits the player (armor 10): 500 → flat 400 → ×(1 − 16.66%).
        assert_eq!(armor_percent_bp(10, 500), 1_666);
        assert_eq!(mitigate_physical(500, 10), 333);
        assert_eq!(final_damage(333), 3);
        // Player (7 dmg) hits the Skeleton (armor 6): 700 → 640 × (1 − 7.89%) = 589 → 6.
        assert_eq!(armor_percent_bp(6, 700), 789);
        assert_eq!(mitigate_physical(700, 6), 589);
        // Glancing Shambler (8 dmg) vs player: 400 → 300 × (1 − 20%) = 240 → 2.
        assert_eq!(armor_percent_bp(10, apply_glance(800)), 2_000);
        assert_eq!(mitigate_physical(400, 10), 240);
        assert_eq!(final_damage(240), 2);
        // Grave rat (3 dmg) vs player: 300 → 200 × (1 − 25%) = 150 → 2.
        assert_eq!(mitigate_physical(300, 10), 150);
        assert_eq!(final_damage(150), 2);
    }

    #[test]
    fn crit_cannot_glance() {
        // 1% crit and 50% glance: a crit consumes only its own roll and never glances.
        let attacker = stats(0, 0, 0, 7);
        let defender = stats(0, 20, 0, 1);
        let mut rng = CombatRng::new(11);
        let mut crits = 0;
        for _ in 0..3_000 {
            let before = rng.position().word_pos;
            let result = resolve_attack(&attacker, &defender, &mut rng);
            let consumed = rng.position().word_pos - before;
            if result.critical {
                crits += 1;
                assert!(!result.glancing);
                assert_eq!(consumed, 1);
                assert_eq!(result.damage, 11);
            } else {
                assert_eq!(consumed, 2);
            }
        }
        assert!(crits > 0);
    }

    #[test]
    fn resolution_reports_raw_mitigated_and_final_hp() {
        let skeleton = stats(6, 4, 6, 5);
        let player = stats(10, 8, 10, 7);
        let mut rng = CombatRng::new(1);
        let mut seen = [false; 3];
        for _ in 0..500 {
            let result = resolve_attack(&skeleton, &player, &mut rng);
            // Crit: 750 → 650 × (1 − 11.76%) = 573 → 6. Glance: 250 → 150 × (1 − 28.57%) = 107 → 1.
            let (index, expected) = match (result.critical, result.glancing) {
                (true, _) => (0, (8, 2, 6)),
                (false, true) => (1, (3, 2, 1)),
                (false, false) => (2, (5, 2, 3)),
            };
            seen[index] = true;
            assert_eq!((result.raw, result.mitigated, result.damage), expected);
        }
        assert_eq!(seen, [true; 3]);
    }
}
