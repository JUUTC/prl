//! Ordered core outcomes and correlated player-facing messages.

use serde::{Deserialize, Serialize};

use crate::{ActorId, Command, MapId, Position, RejectReason};

/// Newest outcomes retained for presentation consumers.
pub(crate) const OUTCOME_LIMIT: usize = 256;
/// Newest messages retained for the log.
pub(crate) const MESSAGE_LIMIT: usize = 64;

/// Ordered result of one command or scheduler event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreOutcome {
    /// Monotonic sequence; continues across new runs so consumers never replay.
    pub sequence: u64,
    /// Resolved event data.
    pub kind: OutcomeKind,
}

/// Core-resolved event data suitable for independent presentation consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeKind {
    /// Active map changed; presentation must discard old floor effects.
    FloorChanged {
        /// Previous map identity.
        from: MapId,
        /// New map identity.
        to: MapId,
    },
    /// Final playtest endpoint reached after defeating its guardian.
    SliceCompleted,
    /// Item ownership, knowledge or consumable state changed.
    ItemsChanged,
    /// A player action changed authoritative state and consumed energy.
    CommandAccepted {
        /// Accepted player intent.
        command: Command,
        /// Energy charged for the intent.
        cost: u16,
    },
    /// A player command was rejected before simulation mutation.
    CommandRejected {
        /// Rejected player intent.
        command: Command,
        /// Validation reason.
        reason: RejectReason,
    },
    /// One local world tick advanced.
    TickAdvanced {
        /// New absolute tick count.
        tick: u64,
    },
    /// A non-player actor spent an action waiting.
    ActorWaited {
        /// Stable identity of the waiting actor.
        actor: ActorId,
    },
    /// An actor (player or AI) stepped to an adjacent cell.
    ActorMoved {
        /// Moving actor.
        actor: ActorId,
        /// Cell before the step.
        from: Position,
        /// Cell after the step.
        to: Position,
    },
    /// A melee attack connected (attacks always hit; see `docs/combat.md`).
    Attacked {
        /// Attacking actor.
        attacker: ActorId,
        /// Defending actor.
        target: ActorId,
        /// HP after crit/glance and before armor, rounded.
        raw: u16,
        /// HP removed by armor (`raw - damage`).
        mitigated: u16,
        /// Final HP lost by the target (at least 1).
        damage: u16,
        /// Whether the attack was a critical hit.
        critical: bool,
        /// Whether the attack was a glancing blow (never with `critical`).
        glancing: bool,
        /// Target HP after the attack.
        target_hp: u16,
    },
    /// An actor reached 0 HP. Non-players are removed; the player's death ends the run.
    ActorDied {
        /// Actor that died.
        actor: ActorId,
        /// Actor whose attack killed it.
        killer: ActorId,
    },
    /// A run began (initial construction or [`Command::NewRun`]).
    RunStarted {
        /// One-based run number.
        run: u32,
        /// Seed of the new run.
        seed: u64,
    },
}

/// Human-readable message correlated with its ordered core outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreMessage {
    /// Sequence of the outcome that produced this message.
    pub sequence: u64,
    /// Localized later by presentation; currently plain English.
    pub text: String,
}

/// Bounded, sequenced outcome and message retention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OutcomeLog {
    pub(crate) next_sequence: u64,
    pub(crate) outcomes: Vec<CoreOutcome>,
    pub(crate) messages: Vec<CoreMessage>,
}

impl Default for OutcomeLog {
    fn default() -> Self {
        Self {
            next_sequence: 1,
            outcomes: Vec::new(),
            messages: Vec::new(),
        }
    }
}

impl OutcomeLog {
    pub(crate) fn record(&mut self, kind: OutcomeKind, message: Option<String>) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.outcomes.push(CoreOutcome { sequence, kind });
        if self.outcomes.len() > OUTCOME_LIMIT {
            self.outcomes.remove(0);
        }
        if let Some(text) = message {
            self.messages.push(CoreMessage { sequence, text });
        }
        // Messages must reference retained outcomes, so trim both together.
        let oldest = self
            .outcomes
            .first()
            .map_or(sequence, |outcome| outcome.sequence);
        self.messages.retain(|message| message.sequence >= oldest);
        if self.messages.len() > MESSAGE_LIMIT {
            self.messages.remove(0);
        }
        sequence
    }

    /// Whether sequences are monotonic, bounded and messages reference outcomes.
    pub(crate) fn is_consistent(&self) -> bool {
        let outcomes_ok = self.outcomes.len() <= OUTCOME_LIMIT
            && self.messages.len() <= MESSAGE_LIMIT
            && self.next_sequence != 0
            && self
                .outcomes
                .iter()
                .all(|outcome| outcome.sequence != 0 && outcome.sequence < self.next_sequence)
            && self
                .outcomes
                .windows(2)
                .all(|pair| matches!(pair, [left, right] if left.sequence < right.sequence));
        let messages_ok = self.messages.iter().all(|message| {
            self.outcomes
                .binary_search_by_key(&message.sequence, |outcome| outcome.sequence)
                .is_ok()
        }) && self
            .messages
            .windows(2)
            .all(|pair| matches!(pair, [left, right] if left.sequence < right.sequence));
        outcomes_ok && messages_ok
    }
}
