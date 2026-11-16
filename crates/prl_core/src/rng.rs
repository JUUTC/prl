//! Seeded, serializable gameplay randomness (Q-016 provisional).

use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

/// Fixed `ChaCha8` stream number for combat rolls within a run seed.
pub const COMBAT_STREAM: u64 = 1;

/// Basis points in 100%.
pub const BASIS_POINTS: u16 = 10_000;

/// Portable, inspectable RNG state: seed, stream and 32-bit word position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "RngPositionRecord", into = "RngPositionRecord")]
pub struct RngPosition {
    /// Run seed.
    pub seed: u64,
    /// `ChaCha` stream number.
    pub stream: u64,
    /// Number of 32-bit words consumed from the stream.
    pub word_pos: u128,
}

/// Snapshot shape: `u128` is split so every serde format can store it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct RngPositionRecord {
    seed: u64,
    stream: u64,
    word_pos_high: u64,
    word_pos_low: u64,
}

impl From<RngPositionRecord> for RngPosition {
    fn from(record: RngPositionRecord) -> Self {
        Self {
            seed: record.seed,
            stream: record.stream,
            word_pos: (u128::from(record.word_pos_high) << 64) | u128::from(record.word_pos_low),
        }
    }
}

impl From<RngPosition> for RngPositionRecord {
    fn from(position: RngPosition) -> Self {
        let high = u64::try_from(position.word_pos >> 64).unwrap_or(u64::MAX);
        let low = u64::try_from(position.word_pos & u128::from(u64::MAX)).unwrap_or(u64::MAX);
        Self {
            seed: position.seed,
            stream: position.stream,
            word_pos_high: high,
            word_pos_low: low,
        }
    }
}

/// The run's combat roll stream.
#[derive(Debug, Clone)]
pub(crate) struct CombatRng {
    seed: u64,
    rng: ChaCha8Rng,
}

impl CombatRng {
    pub(crate) fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        rng.set_stream(COMBAT_STREAM);
        Self { seed, rng }
    }

    pub(crate) fn restore(position: RngPosition) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(position.seed);
        rng.set_stream(position.stream);
        rng.set_word_pos(position.word_pos);
        Self {
            seed: position.seed,
            rng,
        }
    }

    pub(crate) fn position(&self) -> RngPosition {
        RngPosition {
            seed: self.seed,
            stream: self.rng.get_stream(),
            word_pos: self.rng.get_word_pos(),
        }
    }

    /// Consumes exactly one 32-bit word and succeeds with `chance_bp / 10000`.
    ///
    /// Uses an integer multiply-shift so the mapping is portable and unbiased
    /// enough for gameplay (bias below 2^-18).
    pub(crate) fn roll_bp(&mut self, chance_bp: u16) -> bool {
        let word = u64::from(self.rng.next_u32());
        ((word * u64::from(BASIS_POINTS)) >> 32) < u64::from(chance_bp)
    }
}

impl PartialEq for CombatRng {
    fn eq(&self, other: &Self) -> bool {
        self.position() == other.position()
    }
}

impl Eq for CombatRng {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_same_rolls_and_restore_continues_stream() {
        let mut first = CombatRng::new(7);
        let mut second = CombatRng::new(7);
        let rolls: Vec<bool> = (0..50).map(|_| first.roll_bp(5_000)).collect();
        let again: Vec<bool> = (0..50).map(|_| second.roll_bp(5_000)).collect();
        assert_eq!(rolls, again);
        assert_eq!(first.position().word_pos, 50);
        assert_eq!(first.position().stream, COMBAT_STREAM);
        let mut restored = CombatRng::restore(first.position());
        let continued: Vec<bool> = (0..20).map(|_| first.roll_bp(5_000)).collect();
        let restored_rolls: Vec<bool> = (0..20).map(|_| restored.roll_bp(5_000)).collect();
        assert_eq!(continued, restored_rolls);
    }

    #[test]
    fn roll_bounds_are_never_and_always() {
        let mut rng = CombatRng::new(3);
        assert!((0..200).all(|_| !rng.roll_bp(0)));
        assert!((0..200).all(|_| rng.roll_bp(BASIS_POINTS)));
    }

    #[test]
    fn rng_position_round_trips_through_ron_including_high_word_bits() {
        let position = RngPosition {
            seed: 9,
            stream: COMBAT_STREAM,
            word_pos: (u128::from(3_u8) << 64) | 0x11,
        };
        let encoded = ron::to_string(&position).unwrap();
        assert!(encoded.contains("word_pos_high:3"));
        assert_eq!(ron::from_str::<RngPosition>(&encoded).unwrap(), position);
    }

    #[test]
    fn different_seeds_produce_different_streams() {
        let mut first = CombatRng::new(1);
        let mut second = CombatRng::new(2);
        let a: Vec<bool> = (0..64).map(|_| first.roll_bp(5_000)).collect();
        let b: Vec<bool> = (0..64).map(|_| second.roll_bp(5_000)).collect();
        assert_ne!(a, b);
    }
}
