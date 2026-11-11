use bevy::prelude::*;
use prl_content::{EquipSlot, ItemKind};
use prl_core::{BACKPACK, CoreState, EQUIPMENT, Item};

use crate::PlaceholderTheme;

use super::{Workspace, selected, view::text};

pub(super) fn render(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    workspace: &Workspace,
    theme: &PlaceholderTheme,
    images: &super::ItemIconImages,
) {
    text(parent, "ITEM DETAILS", theme);
    let Some(item) = selected(core, workspace) else {
        text(parent, "Select an item.", theme);
        return;
    };
    let Some(bag) = core.inventory() else { return };
    let Some(definition) = bag.definition(item) else {
        return;
    };
    super::view::detail_icon(parent, core, item, images);
    text(parent, bag.label(item), theme);
    text(
        parent,
        format!(
            "{:?} / {}\n{} / {} x {} / {} g",
            definition.rarity,
            if item.identified {
                "Identified"
            } else {
                "Unidentified"
            },
            if item.container == BACKPACK {
                "Backpack"
            } else if item.container == EQUIPMENT {
                "Equipped"
            } else {
                "Ground"
            },
            bag.footprint(item)
                .map_or(definition.width, |shape| shape.0),
            bag.footprint(item)
                .map_or(definition.height, |shape| shape.1),
            definition.weight
        ),
        theme,
    );
    match definition.kind {
        ItemKind::Healing { amount } if item.identified => text(
            parent,
            format!(
                "Restore up to {amount} HP.\nAt current HP: {} restored.",
                core.player()
                    .map_or(0, |player| amount.min(player.max_hp() - player.hp()))
            ),
            theme,
        ),
        ItemKind::Identify if item.identified => text(
            parent,
            "Identify one carried target.\nChoose this source explicitly.",
            theme,
        ),
        ItemKind::Healing { .. } | ItemKind::Identify => text(
            parent,
            "Unknown effect.\nNo effect preview or hidden source hints.",
            theme,
        ),
        ItemKind::Gear(slot) => gear(parent, core, item, slot, theme),
    }
    text(
        parent,
        "Armor is raw points, not a percent.\nAccuracy/evasion are ratings, not hit chances.",
        theme,
    );
}

fn gear(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    item: &Item,
    slot: EquipSlot,
    theme: &PlaceholderTheme,
) {
    let Some(bag) = core.inventory() else { return };
    let Some(definition) = bag.definition(item) else {
        return;
    };
    if item.container == EQUIPMENT {
        let bonus = if item.identified {
            definition.affix
        } else {
            prl_content::ItemBonus::default()
        };
        text(
            parent,
            format!(
                "KNOWN ITEM CONTRIBUTION\nDamage {:+} / armor {:+}\nAccuracy {:+} / evasion {:+}",
                i32::from(definition.base.damage) + i32::from(bonus.damage),
                i32::from(definition.base.armor) + i32::from(bonus.armor),
                i32::from(definition.base.accuracy) + i32::from(bonus.accuracy),
                i32::from(definition.base.evasion) + i32::from(bonus.evasion)
            ),
            theme,
        );
    } else {
        let incumbent = bag.items().find(|other| {
            other.container == EQUIPMENT
                && bag
                    .definition(other)
                    .is_some_and(|old| old.kind == ItemKind::Gear(slot))
        });
        text(
            parent,
            format!(
                "Replaces: {}",
                incumbent.map_or_else(|| "Empty slot".into(), |old| bag.label(old))
            ),
            theme,
        );
        if let Some((before, after)) = core
            .player()
            .and_then(|player| bag.compare(player, item.id))
        {
            text(parent, "KNOWN TOTALS: Now -> If equipped", theme);
            for (label, before, after) in [
                ("Damage", i32::from(before.damage), i32::from(after.damage)),
                (
                    "Armor points",
                    i32::from(before.armor),
                    i32::from(after.armor),
                ),
                (
                    "Accuracy",
                    i32::from(before.accuracy),
                    i32::from(after.accuracy),
                ),
                (
                    "Evasion",
                    i32::from(before.evasion),
                    i32::from(after.evasion),
                ),
            ] {
                text(
                    parent,
                    format!("{label}: {before} -> {after} ({:+})", after - before),
                    theme,
                );
            }
        }
        text(
            parent,
            "Unknown affixes excluded on both sides;\nactual result may differ.",
            theme,
        );
    }
    if !item.identified {
        text(
            parent,
            "Affixes and curse state unknown.\nNo safety claim.",
            theme,
        );
    }
}
