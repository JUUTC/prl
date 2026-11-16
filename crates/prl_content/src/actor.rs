//! Actor definitions: scheduling, behaviour and placeholder combat numbers.

use serde::{Deserialize, Serialize};

use crate::ContentId;

/// Default critical multiplier, in percent (combat.md: base 150%).
pub const DEFAULT_CRIT_MULTIPLIER_PCT: u16 = 150;

/// Closed set of engine behaviours an actor definition can select.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ActorBehavior {
    /// Controlled by the local player's commands.
    Player,
    /// Hunts and attacks the player (P2 awareness/chase rules, Q-020).
    Hostile,
    /// Never attacks; only waits. Bumping into it is rejected as occupied.
    Passive,
}

/// One actor's definition.
///
/// Combat numbers are provisional placeholders until stats and skills exist
/// (Q-019): they feed the `docs/combat.md` pipeline directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorDefinition {
    /// Stable content identity.
    pub id: ContentId,
    /// Display name, e.g. `Skeleton`. The player definition uses `You`.
    pub name: String,
    /// Engine behaviour selected by this definition.
    pub behavior: ActorBehavior,
    /// Base scheduler speed, in energy gained per tick.
    pub speed: u16,
    /// Maximum hit points.
    pub max_hp: u16,
    /// Placeholder accuracy (raises crit chance, lowers target glance chance).
    pub accuracy: i16,
    /// Placeholder evasion (raises glance chance, lowers attacker crit chance).
    pub evasion: i16,
    /// Total physical armor (flat `armor / 10` HP, then armor%).
    pub armor: u16,
    /// Fixed base physical damage in hit points.
    pub damage: u16,
    /// Critical damage multiplier in percent (at least 100).
    #[serde(default = "default_crit_multiplier_pct")]
    pub crit_multiplier_pct: u16,
}

const fn default_crit_multiplier_pct() -> u16 {
    DEFAULT_CRIT_MULTIPLIER_PCT
}
