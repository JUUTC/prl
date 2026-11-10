//! Player intents, validation reasons and the queued command resource.

use std::collections::VecDeque;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::{Direction, StairDirection};

/// Runtime intent accepted by the core command path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Move one adjacent cell; moving into a hostile actor attacks it.
    Move(Direction),
    /// Spend one normal action waiting.
    Wait,
    /// Open a closed adjacent door.
    OpenDoor(Direction),
    /// Abandon the current run (alive or dead) and start the next one
    /// (Q-021 provisional). Costs no game time.
    NewRun,
    /// Use stairs or the final playtest endpoint under the player.
    Stairs(StairDirection),
    /// Atomically move one item between accessible containers.
    Transfer(crate::TransferItems),
    /// Drink a carried potion.
    Quaff(crate::ItemId),
    /// Put a carried/equipped item on the current cell.
    Drop(crate::ItemId),
    /// Read a carried Identify scroll, targeting a carried item.
    Identify {
        /// Scroll instance.
        scroll: crate::ItemId,
        /// Target instance.
        target: crate::ItemId,
    },
}

/// Why an intent was rejected. Rejections do not consume energy, advance time
/// or touch the RNG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectReason {
    /// Destination is outside the map.
    OutOfBounds,
    /// Destination blocks movement.
    BlockedTerrain,
    /// Destination is occupied by a non-hostile actor.
    Occupied,
    /// Diagonal move or attack would squeeze past a blocked orthogonal neighbour.
    CornerBlocked,
    /// No closed door is adjacent in the requested direction.
    NoClosedDoor,
    /// The player is dead; only [`Command::NewRun`] is accepted.
    PlayerDead,
    /// A new run could not be built from the loaded content.
    RunUnavailable,
    /// No usable stairs in this direction under the player.
    NoStairs,
    /// The final guardian must be defeated before completing the slice.
    GuardianAlive,
    /// The playtest endpoint was already completed.
    AlreadyCompleted,
    /// Item is absent or not suitable for this action.
    InvalidItem,
    /// Item/container is not reachable here.
    ItemInaccessible,
    /// Grid shape cannot fit without overlap.
    BackpackFull,
    /// Pickup would exceed the current slice carry limit.
    TooHeavy,
    /// Healing at full health or identifying a known item does not spend time.
    NoEffect,
}

/// Public result returned from synchronous command resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    /// Command was resolved.
    Accepted {
        /// Sequence assigned to the accepted command outcome.
        sequence: u64,
    },
    /// Command was rejected without consuming game time.
    Rejected {
        /// Sequence assigned to the rejection outcome.
        sequence: u64,
        /// Validation reason.
        reason: RejectReason,
    },
}

/// Commands submitted by input and resolved on the next core update.
#[derive(Debug, Default, Resource)]
pub struct CoreCommandQueue {
    commands: VecDeque<Command>,
}

impl CoreCommandQueue {
    /// Queues an intent for the next `CoreSet::Commands` update.
    pub fn push(&mut self, command: Command) {
        self.commands.push_back(command);
    }

    /// Number of pending intents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether there are pending intents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub(crate) fn pop(&mut self) -> Option<Command> {
        self.commands.pop_front()
    }
}
