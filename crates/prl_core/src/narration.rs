//! Plain-English message text for outcomes (localized later by presentation).

use crate::combat::AttackResolution;
use crate::{Command, RejectReason};

/// Who took part in an attack, for second-person phrasing of the player.
pub(crate) struct Participant<'a> {
    pub(crate) name: &'a str,
    pub(crate) is_player: bool,
}

/// e.g. "The Skeleton hits you for 3 (5 raw, armor -2)." or
/// "Glancing blow: you hit the Shambler for 2 (4 raw, armor -2)."
pub(crate) fn attack(
    attacker: &Participant<'_>,
    target: &Participant<'_>,
    result: AttackResolution,
) -> String {
    let subject = if attacker.is_player {
        "you".to_owned()
    } else {
        format!("the {}", attacker.name)
    };
    let verb = if attacker.is_player { "hit" } else { "hits" };
    let object = if target.is_player {
        "you".to_owned()
    } else {
        format!("the {}", target.name)
    };
    let manner = if result.critical { "critically " } else { "" };
    let breakdown = if result.mitigated == 0 {
        String::new()
    } else {
        format!(" ({} raw, armor -{})", result.raw, result.mitigated)
    };
    let sentence = format!(
        "{subject} {manner}{verb} {object} for {}{breakdown}.",
        result.damage
    );
    if result.glancing {
        format!("Glancing blow: {sentence}")
    } else {
        capitalize(&sentence)
    }
}

pub(crate) fn death(actor: &Participant<'_>) -> String {
    if actor.is_player {
        "You die. Start a new run to try again.".to_owned()
    } else {
        format!("The {} is slain.", actor.name)
    }
}

pub(crate) fn run_started(run: u32, seed: u64) -> String {
    format!("Run {run} begins (seed {seed}).")
}

pub(crate) fn accepted(command: Command) -> Option<String> {
    match command {
        Command::Move(_) => Some("You move.".into()),
        Command::Wait => Some("You wait.".into()),
        Command::OpenDoor(_) => Some("You open the door.".into()),
        Command::NewRun
        | Command::Stairs(_)
        | Command::Transfer(_)
        | Command::SplitStack { .. }
        | Command::Quaff(_)
        | Command::Identify { .. }
        | Command::Drop(_)
        | Command::PlaceItem(_)
        | Command::Organize => None,
    }
}

pub(crate) fn rejected(reason: RejectReason) -> String {
    match reason {
        RejectReason::OutOfBounds => "You cannot move beyond the map.".into(),
        RejectReason::BlockedTerrain => "The way is blocked.".into(),
        RejectReason::Occupied => "The destination is occupied.".into(),
        RejectReason::CornerBlocked => "You cannot squeeze past the blocked corner.".into(),
        RejectReason::NoClosedDoor => "There is no closed door there.".into(),
        RejectReason::PlayerDead => "You are dead. Start a new run.".into(),
        RejectReason::RunUnavailable => "A new run could not be built from content.".into(),
        RejectReason::NoStairs => "There are no usable stairs in that direction here.".into(),
        RejectReason::GuardianAlive => "Defeat the crypt guardian before leaving.".into(),
        RejectReason::AlreadyCompleted => {
            "This crypt is already cleared. Start a new run to try again.".into()
        }
        RejectReason::InvalidItem => "That item cannot be used for this action.".into(),
        RejectReason::ItemInaccessible => "That item or container is not accessible here.".into(),
        RejectReason::BackpackFull => "There is no room in the backpack for that shape.".into(),
        RejectReason::TooHeavy => "That pickup would exceed your carrying limit.".into(),
        RejectReason::NoEffect => "That would have no effect; no item or time spent.".into(),
    }
}

fn capitalize(text: &str) -> String {
    let mut characters = text.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(characters).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const YOU: Participant<'static> = Participant {
        name: "You",
        is_player: true,
    };

    fn result(raw: u16, damage: u16, critical: bool, glancing: bool) -> AttackResolution {
        AttackResolution {
            raw,
            mitigated: raw - damage,
            damage,
            critical,
            glancing,
        }
    }

    #[test]
    fn attack_messages_use_second_person_for_the_player() {
        let skeleton = Participant {
            name: "Skeleton",
            is_player: false,
        };
        assert_eq!(
            attack(&skeleton, &YOU, result(5, 3, false, false)),
            "The Skeleton hits you for 3 (5 raw, armor -2)."
        );
        let rat = Participant {
            name: "Grave rat",
            is_player: false,
        };
        assert_eq!(
            attack(&YOU, &rat, result(11, 11, true, false)),
            "You critically hit the Grave rat for 11."
        );
        let shambler = Participant {
            name: "Shambler",
            is_player: false,
        };
        assert_eq!(
            attack(&shambler, &YOU, result(4, 2, false, true)),
            "Glancing blow: the Shambler hits you for 2 (4 raw, armor -2)."
        );
        assert_eq!(death(&skeleton), "The Skeleton is slain.");
        assert!(death(&YOU).starts_with("You die."));
    }
}
