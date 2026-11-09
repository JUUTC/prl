//! Logical actors: identity, scheduling, health, combat numbers and AI state.

use prl_content::{ActorDefinition, ContentId};
use serde::{Deserialize, Serialize};

use crate::{ActorId, Position};

pub use prl_content::ActorBehavior;

/// Placeholder stamina shown by the P1/P2 HUD until stamina rules exist.
pub(crate) const PLACEHOLDER_STAMINA: u16 = 100;

/// Placeholder combat numbers copied from the actor definition (Q-019).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatStats {
    /// Raises crit chance and lowers the defender's glance chance.
    pub accuracy: i16,
    /// Raises glance chance and lowers the attacker's crit chance.
    pub evasion: i16,
    /// Total physical armor.
    pub armor: u16,
    /// Fixed base physical damage in hit points.
    pub damage: u16,
    /// Critical damage multiplier in percent.
    pub crit_multiplier_pct: u16,
}

impl CombatStats {
    /// Copies the combat numbers of a definition.
    #[must_use]
    pub const fn from_definition(definition: &ActorDefinition) -> Self {
        Self {
            accuracy: definition.accuracy,
            evasion: definition.evasion,
            armor: definition.armor,
            damage: definition.damage,
            crit_multiplier_pct: definition.crit_multiplier_pct,
        }
    }
}

/// Serializable hostile awareness (Q-020 provisional).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiState {
    /// Has not seen the player, or lost track at the last seen cell. Waits.
    #[default]
    Unaware,
    /// Saw the player and hunts toward the remembered cell.
    Chasing {
        /// Player cell at the most recent sighting.
        last_seen: Position,
    },
}

/// Logical actor state independent of ECS allocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub(crate) id: ActorId,
    pub(crate) definition: ContentId,
    pub(crate) name: String,
    pub(crate) behavior: ActorBehavior,
    pub(crate) position: Position,
    pub(crate) speed: u16,
    pub(crate) energy: u16,
    pub(crate) hp: u16,
    pub(crate) max_hp: u16,
    pub(crate) stamina: u16,
    pub(crate) max_stamina: u16,
    pub(crate) combat: CombatStats,
    pub(crate) ai: AiState,
}

impl Actor {
    /// Builds a fresh, full-health actor from its definition.
    pub(crate) fn spawn(
        id: ActorId,
        definition: &ActorDefinition,
        position: Position,
        energy: u16,
    ) -> Self {
        Self {
            id,
            definition: definition.id.clone(),
            name: definition.name.clone(),
            behavior: definition.behavior,
            position,
            speed: definition.speed,
            energy,
            hp: definition.max_hp,
            max_hp: definition.max_hp,
            stamina: PLACEHOLDER_STAMINA,
            max_stamina: PLACEHOLDER_STAMINA,
            combat: CombatStats::from_definition(definition),
            ai: AiState::Unaware,
        }
    }

    /// Stable actor identity.
    #[must_use]
    pub const fn id(&self) -> ActorId {
        self.id
    }

    /// Content definition identity.
    #[must_use]
    pub const fn definition(&self) -> &ContentId {
        &self.definition
    }

    /// Display name from content (the player is `You`).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Engine behaviour selected by content.
    #[must_use]
    pub const fn behavior(&self) -> ActorBehavior {
        self.behavior
    }

    /// Current logical cell.
    #[must_use]
    pub const fn position(&self) -> Position {
        self.position
    }

    /// Scheduler speed in energy gained per tick.
    #[must_use]
    pub const fn speed(&self) -> u16 {
        self.speed
    }

    /// Current carried energy.
    #[must_use]
    pub const fn energy(&self) -> u16 {
        self.energy
    }

    /// Current hit points; zero only for the dead player.
    #[must_use]
    pub const fn hp(&self) -> u16 {
        self.hp
    }

    /// Maximum hit points.
    #[must_use]
    pub const fn max_hp(&self) -> u16 {
        self.max_hp
    }

    /// Placeholder stamina.
    #[must_use]
    pub const fn stamina(&self) -> u16 {
        self.stamina
    }

    /// Maximum placeholder stamina.
    #[must_use]
    pub const fn max_stamina(&self) -> u16 {
        self.max_stamina
    }

    /// Placeholder combat numbers (Q-019).
    #[must_use]
    pub const fn combat(&self) -> CombatStats {
        self.combat
    }

    /// Hostile awareness state; always `Unaware` for non-hostiles.
    #[must_use]
    pub const fn ai_state(&self) -> AiState {
        self.ai
    }

    /// Returns whether the actor is controlled by the local player.
    #[must_use]
    pub const fn is_player(&self) -> bool {
        matches!(self.behavior, ActorBehavior::Player)
    }

    /// Returns whether the actor hunts the player.
    #[must_use]
    pub const fn is_hostile(&self) -> bool {
        matches!(self.behavior, ActorBehavior::Hostile)
    }

    /// Returns whether the actor has hit points left.
    #[must_use]
    pub const fn is_alive(&self) -> bool {
        self.hp > 0
    }
}
