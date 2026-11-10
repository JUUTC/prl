//! Read-only, deterministic text summaries of dead playtest runs.

use prl_core::{CoreState, EQUIPMENT, OutcomeKind};

pub(super) fn summary(state: &CoreState, fingerprint: &str) -> String {
    let cause = state.outcomes().iter().rev().find_map(|outcome| {
        if let OutcomeKind::ActorDied { actor, killer } = outcome.kind
            && actor == state.player_id()
        {
            Some(state.actor(killer).map_or_else(
                || format!("Actor {} (name no longer retained)", killer.get()),
                |actor| format!("{} [{}]", actor.name(), actor.definition()),
            ))
        } else {
            None
        }
    });
    let mut report = format!(
        "PRL morgue - playtest report format 1\n\
         Engine: {}\nSnapshot: {fingerprint}\n\
         Run: {}\nSeed: {}\nOutcome: Death\n\
         Cause: {}\n\
         Pack: {}\nMap: {} (logical ID {})\n\
         Floor: {}\n\
         Ticks: {}\nElapsed game seconds: {}\nCommitted player actions: {}\n\
         Crypt objective completed before death: {}\n\
         Settings: provisional playtest content; no character/difficulty selection yet\n\
         Skills: not implemented\n\
         Lifetime kills: not tracked; the recent log below is not a complete history\n\n\
         Carried equipment and backpack (known names only):\n",
        env!("CARGO_PKG_VERSION"),
        state.run_number(),
        state.seed(),
        cause.unwrap_or_else(|| "Unavailable (fatal outcome no longer retained)".into()),
        state.content().id(),
        state.map().definition(),
        state.map().id().get(),
        state
            .floor_number()
            .map_or_else(|| "Arena".into(), |floor| floor.to_string()),
        state.clock().ticks(),
        state.clock().elapsed_seconds(),
        state.clock().player_actions(),
        state.slice_completed(),
    );
    if let Some(inventory) = state.inventory() {
        for item in inventory
            .items()
            .filter(|item| item.container == prl_core::BACKPACK || item.container == EQUIPMENT)
        {
            let location = if item.container == EQUIPMENT {
                "Equipped"
            } else {
                "Backpack"
            };
            report.push_str(location);
            report.push_str(": ");
            report.push_str(&inventory.label(item));
            report.push('\n');
        }
    } else {
        report.push_str("No inventory system in this run.\n");
    }
    report.push_str("\nRecent messages (bounded, not a full replay):\n");
    for message in state.messages() {
        report.push_str(&message.text);
        report.push('\n');
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carried_gear_is_listed_without_disclosing_ground_items_or_unknown_supplies() {
        let config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
            "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
        ))
        .unwrap();
        let mut state = config
            .build(&prl_content::ContentPack::placeholder().unwrap(), 1)
            .unwrap();
        for _ in 0..3 {
            state.issue(state.pickup_command().unwrap());
        }
        let sword = state
            .inventory()
            .unwrap()
            .items()
            .find(|item| item.container == prl_core::BACKPACK)
            .unwrap()
            .id;
        state.issue(prl_core::Command::Transfer(prl_core::TransferItems {
            item: sword,
            destination: EQUIPMENT,
            cell: None,
        }));
        let report = summary(&state, "fixture");
        assert!(report.contains("Equipped: o Iron sword"));
        assert!(report.contains("Backpack: o Iron mail"));
        assert!(report.contains("potion (?)"));
        assert!(!report.contains("Healing potion"));
        assert!(!report.contains("of keen edges"));
        assert!(!report.contains("of warding"));
    }
}
