//! Knowledge-safe item workspace; all gameplay goes through core commands.

use bevy::prelude::*;
use prl_content::{EquipSlot, ItemKind};
use prl_core::{
    BACKPACK, Command, CoreCommandQueue, CoreSet, CoreState, EQUIPMENT, Item, ItemId, OutcomeKind,
    RejectReason, TransferItems,
};

use crate::input::{Overlay, UiMode};

mod details;
mod packing;
#[cfg(test)]
mod tests;
mod view;

/// Optional reviewed item images, supplied by the composition root.
///
/// UI does not depend on world rendering. Missing categories keep the existing
/// text placeholders; these images never select by hidden definition or affix.
#[derive(Resource, Default)]
pub struct ItemIconImages(pub std::collections::BTreeMap<prl_content::ItemIcon, ImageNode>);

/// Session view of independently persisted inventory warning preferences.
///
/// The application bridges these to `prl_save`; UI never writes character saves.
#[derive(Resource, Clone, PartialEq, Eq)]
pub struct InventoryWarningSettings {
    /// Drop, unknown-consumable and threatened-Body warnings, respectively.
    pub enabled: [bool; 3],
    /// Visible persistence result; errors must not masquerade as saved options.
    pub status: String,
}

impl Default for InventoryWarningSettings {
    fn default() -> Self {
        Self {
            enabled: [true; 3],
            status: "Session-only warning settings".into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Intent {
    Select(ItemId),
    List,
    Close,
    Options,
    ToggleWarning(usize),
    Act(Verb),
    Source(ItemId),
    Confirm,
    Cancel,
    BeginPlacement,
    PlaceCell(prl_core::GridCell),
    Rotate,
    Place,
    Organize,
    DragPlace(prl_core::PlaceItem),
    CancelDrag,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verb {
    Pickup,
    Equip,
    Drink,
    Drop,
    Identify,
}

#[derive(Clone)]
struct Action {
    verb: Verb,
    title: &'static str,
    command: Option<Command>,
    cost: u16,
    reason: Option<String>,
    warning: [bool; 3],
}

#[derive(Clone, PartialEq, Eq)]
enum Stage {
    Browse,
    Sources,
    Review {
        command: Command,
        cost: u16,
        warning: [bool; 3],
    },
    Options,
    Placement(prl_core::PlaceItem),
}

#[derive(Clone)]
struct Flight {
    command: Command,
    sequence: u64,
    hp: u16,
    region: Vec<ItemId>,
    index: usize,
    source: prl_core::ContainerId,
}

#[derive(Resource)]
struct Workspace {
    selected: Option<ItemId>,
    list: bool,
    stage: Stage,
    focus: Option<Intent>,
    feedback: String,
    flight: Option<Flight>,
    run: u32,
    initialized: bool,
    scroll: [f32; 5],
}

#[derive(Resource, Default)]
struct PointerIntent(Option<Intent>);

fn capture(
    event: On<'_, '_, bevy::ui_widgets::Activate>,
    controls: Query<'_, '_, &view::Control>,
    mut pointer: ResMut<'_, PointerIntent>,
    drag: Res<'_, packing::Drag>,
) {
    if drag.suppress_activation() {
        return;
    }
    if let Ok(control) = controls.get(event.entity) {
        pointer.0.get_or_insert(control.0);
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            selected: None,
            list: false,
            stage: Stage::Browse,
            focus: None,
            feedback: "Select to inspect. Browsing is free.".into(),
            flight: None,
            run: 0,
            initialized: false,
            scroll: [0.0; 5],
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    packing::plugin(app);
    app.init_resource::<Workspace>()
        .init_resource::<PointerIntent>()
        .init_resource::<InventoryWarningSettings>()
        .init_resource::<ItemIconImages>()
        .add_observer(capture)
        .add_systems(
            PostUpdate,
            view::reveal_focus
                .after(bevy::ui::UiSystems::Layout)
                .run_if(resource_exists::<CoreState>),
        )
        .add_systems(
            Update,
            input
                .in_set(CoreSet::Input)
                .after(crate::input::handle_input)
                .run_if(resource_exists::<ButtonInput<KeyCode>>)
                .run_if(resource_exists::<CoreState>)
                .run_if(resource_exists::<CoreCommandQueue>),
        )
        .add_systems(
            Update,
            (settle, view::refresh, view::decorate_icons)
                .chain()
                .in_set(CoreSet::Presentation)
                .run_if(resource_exists::<CoreState>),
        )
        .add_systems(
            Update,
            view::scroll
                .in_set(CoreSet::Presentation)
                .before(view::refresh)
                .run_if(resource_exists::<Messages<bevy::input::mouse::MouseWheel>>),
        );
}

fn items(core: &CoreState) -> Vec<&Item> {
    let Some(inventory) = core.inventory() else {
        return Vec::new();
    };
    let mut bag: Vec<_> = inventory
        .items()
        .filter(|item| item.container == BACKPACK)
        .collect();
    bag.sort_by_key(|item| item.cell.map(|cell| (cell.y, cell.x, item.id)));
    let equipment = inventory.items().filter(|item| item.container == EQUIPMENT);
    let ground = core
        .player()
        .into_iter()
        .flat_map(|player| core.ground_items(player.position()));
    equipment.chain(bag).chain(ground).collect()
}

fn selected<'a>(core: &'a CoreState, workspace: &Workspace) -> Option<&'a Item> {
    items(core)
        .into_iter()
        .find(|item| Some(item.id) == workspace.selected)
}

fn sources(core: &CoreState) -> Vec<ItemId> {
    core.inventory()
        .into_iter()
        .flat_map(prl_core::Inventory::items)
        .filter(|item| item.container == BACKPACK && item.identified)
        .filter(|item| {
            core.inventory()
                .and_then(|bag| bag.definition(item))
                .is_some_and(|definition| definition.kind == ItemKind::Identify)
        })
        .map(|item| item.id)
        .collect()
}

fn threatened(core: &CoreState) -> bool {
    core.visible_actors().any(prl_core::Actor::is_hostile)
}

fn rejection(reason: RejectReason, core: &CoreState, target: &Item, command: Command) -> String {
    match reason {
        RejectReason::BackpackFull => {
            let Some(bag) = core.inventory() else {
                return "No backpack available.".into();
            };
            let displaced = bag
                .definition(target)
                .and_then(|definition| {
                    (target.container == BACKPACK)
                        .then(|| {
                            bag.items().find(|other| {
                                other.container == EQUIPMENT
                                    && bag
                                        .definition(other)
                                        .is_some_and(|old| old.kind == definition.kind)
                            })
                        })
                        .flatten()
                })
                .unwrap_or(target);
            bag.definition(displaced).map_or_else(
                || "No room in the backpack. Nothing moved.".into(),
                |definition| {
                    format!(
                        "No contiguous {}x{} space for {}. Nothing moved.",
                        bag.footprint(displaced)
                            .map_or(definition.width, |shape| shape.0),
                        bag.footprint(displaced)
                            .map_or(definition.height, |shape| shape.1),
                        bag.label(displaced)
                    )
                },
            )
        }
        RejectReason::TooHeavy => core.inventory().map_or_else(
            || "Pickup exceeds carry limit.".into(),
            |bag| {
                format!(
                    "Pickup exceeds limit: {} + {} / {} g. Leave something behind.",
                    bag.carried_weight(),
                    bag.definition(target).map_or(0, |item| item.weight),
                    bag.carry_capacity()
                )
            },
        ),
        RejectReason::NoEffect if matches!(command, Command::Quaff(_)) => {
            "No effect: already at full HP. Potion kept; nothing spent.".into()
        }
        RejectReason::NoEffect => "Target is already known. Source kept; nothing spent.".into(),
        RejectReason::PlayerDead => "The run has ended.".into(),
        RejectReason::ItemInaccessible => "Item is no longer reachable. Nothing spent.".into(),
        _ => format!("Item action rejected: {reason:?}. Nothing spent."),
    }
}

fn actions(core: &CoreState, workspace: &Workspace) -> Vec<Action> {
    let Some(item) = selected(core, workspace) else {
        return Vec::new();
    };
    let Some(bag) = core.inventory() else {
        return Vec::new();
    };
    let Some(definition) = bag.definition(item) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut add = |verb, title, command, cost, warning, hidden| {
        let preview = if hidden {
            Ok(cost)
        } else {
            core.item_action_cost(command)
        };
        result.push(Action {
            verb,
            title,
            command: Some(command),
            cost: preview.as_ref().copied().unwrap_or(cost),
            reason: preview
                .err()
                .map(|reason| rejection(reason, core, item, command)),
            warning,
        });
    };
    if !matches!(item.container, BACKPACK | EQUIPMENT) {
        add(
            Verb::Pickup,
            "Pick up [G]",
            Command::Transfer(TransferItems {
                item: item.id,
                destination: BACKPACK,
                cell: None,
            }),
            50,
            [false; 3],
            false,
        );
        return result;
    }
    if let ItemKind::Gear(slot) = definition.kind {
        let body = slot == EquipSlot::Body;
        add(
            Verb::Equip,
            if item.container == EQUIPMENT {
                "Unequip [E]"
            } else {
                "Equip [E]"
            },
            Command::Transfer(TransferItems {
                item: item.id,
                destination: if item.container == EQUIPMENT {
                    BACKPACK
                } else {
                    EQUIPMENT
                },
                cell: None,
            }),
            if body { 300 } else { 100 },
            [false, false, body && threatened(core)],
            false,
        );
    }
    if matches!(definition.kind, ItemKind::Healing { .. }) {
        add(
            Verb::Drink,
            "Drink [Q]",
            Command::Quaff(item.id),
            100,
            [false, !item.identified, false],
            !item.identified,
        );
    }
    let worn_body =
        item.container == EQUIPMENT && definition.kind == ItemKind::Gear(EquipSlot::Body);
    add(
        Verb::Drop,
        "Drop [D]",
        Command::Drop(item.id),
        if worn_body { 300 } else { 100 },
        [true, false, worn_body && threatened(core)],
        false,
    );
    if !item.identified {
        result.push(Action {
            verb: Verb::Identify,
            title: "Identify [C]",
            command: None,
            cost: 100,
            warning: [false; 3],
            reason: sources(core)
                .is_empty()
                .then(|| "No known Identify scroll. Unknown-scroll reading is deferred.".into()),
        });
    }
    result
}

fn controls(core: &CoreState, workspace: &Workspace) -> Vec<Intent> {
    match workspace.stage {
        Stage::Browse => items(core)
            .iter()
            .map(|item| Intent::Select(item.id))
            .chain([
                Intent::List,
                Intent::BeginPlacement,
                Intent::Organize,
                Intent::Options,
                Intent::Close,
            ])
            .chain(
                actions(core, workspace)
                    .iter()
                    .map(|action| Intent::Act(action.verb)),
            )
            .collect(),
        Stage::Sources => sources(core)
            .into_iter()
            .map(Intent::Source)
            .chain([Intent::Cancel])
            .collect(),
        Stage::Review { warning, .. } => [Intent::Cancel, Intent::Confirm]
            .into_iter()
            .chain(
                warning
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, enabled)| enabled.then_some(Intent::ToggleWarning(index))),
            )
            .collect(),
        Stage::Options => (0..3)
            .map(Intent::ToggleWarning)
            .chain([Intent::Cancel])
            .collect(),
        Stage::Placement(_) => vec![Intent::Rotate, Intent::Place, Intent::Cancel],
    }
}

fn input(
    keys: Res<'_, ButtonInput<KeyCode>>,
    mut mode: ResMut<'_, UiMode>,
    core: Res<'_, CoreState>,
    mut workspace: ResMut<'_, Workspace>,
    mut warnings: ResMut<'_, InventoryWarningSettings>,
    mut queue: ResMut<'_, CoreCommandQueue>,
    mut pointer: ResMut<'_, PointerIntent>,
    mut drag: ResMut<'_, packing::Drag>,
) {
    let clicked = pointer.0.take();
    if mode.overlay != Overlay::Inventory
        || !mode.window_focused()
        || workspace.flight.is_some()
        || !queue.is_empty()
    {
        drag.cancel();
        return;
    }
    if drag.keyboard(&keys) {
        return;
    }
    let intent = if keys.just_pressed(KeyCode::Escape) {
        Some(if workspace.stage == Stage::Browse {
            Intent::Close
        } else {
            Intent::Cancel
        })
    } else {
        clicked.or_else(|| {
            keys.get_just_pressed()
                .next()
                .and_then(|_| keyboard_intent(&keys, &core, &mut workspace))
        })
    };
    if let Some(intent) = intent {
        activate(
            intent,
            &core,
            &mut workspace,
            &mut mode,
            &mut warnings,
            &mut queue,
        );
    }
}

fn keyboard_intent(
    keys: &ButtonInput<KeyCode>,
    core: &CoreState,
    workspace: &mut Workspace,
) -> Option<Intent> {
    let choices = controls(core, workspace);
    if keys.just_pressed(KeyCode::Tab) {
        let backward = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        let index = workspace
            .focus
            .and_then(|focus| choices.iter().position(|choice| *choice == focus));
        let next = if backward {
            index
                .filter(|index| *index > 0)
                .map_or(choices.len().saturating_sub(1), |index| index - 1)
        } else {
            index.map_or(0, |index| (index + 1) % choices.len().max(1))
        };
        workspace.focus = choices.get(next).copied();
        return None;
    }
    if keys.just_pressed(KeyCode::Enter) {
        return workspace.focus;
    }
    if matches!(workspace.stage, Stage::Placement(_)) {
        return packing::keyboard(keys, workspace);
    }
    if workspace.stage != Stage::Browse {
        return None;
    }
    let direction =
        if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::ArrowRight) {
            1
        } else if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowLeft) {
            -1
        } else {
            0
        };
    if direction != 0 {
        let list = items(core);
        let index = list
            .iter()
            .position(|item| Some(item.id) == workspace.selected)
            .unwrap_or(0);
        let index = if direction > 0 {
            (index + 1).min(list.len().saturating_sub(1))
        } else {
            index.saturating_sub(1)
        };
        return list.get(index).map(|item| Intent::Select(item.id));
    }
    for (key, verb) in [
        (KeyCode::KeyE, Verb::Equip),
        (KeyCode::KeyQ, Verb::Drink),
        (KeyCode::KeyD, Verb::Drop),
        (KeyCode::KeyC, Verb::Identify),
        (KeyCode::KeyG, Verb::Pickup),
    ] {
        if keys.just_pressed(key) {
            return Some(Intent::Act(verb));
        }
        if keys.just_pressed(KeyCode::KeyP) {
            return Some(Intent::BeginPlacement);
        }
        if keys.just_pressed(KeyCode::KeyO) {
            return Some(Intent::Organize);
        }
    }
    None
}

fn activate(
    intent: Intent,
    core: &CoreState,
    workspace: &mut Workspace,
    mode: &mut UiMode,
    warnings: &mut InventoryWarningSettings,
    queue: &mut CoreCommandQueue,
) {
    if packing::activate(intent, core, workspace, queue) {
        return;
    }
    workspace.focus = Some(intent);
    match intent {
        Intent::Select(id) if workspace.stage == Stage::Browse => {
            if items(core).iter().any(|item| item.id == id) {
                workspace.selected = Some(id);
                workspace.scroll[2] = 0.0;
                workspace.feedback = "Inspecting only. No time passes.".into();
            }
        }
        Intent::List if workspace.stage == Stage::Browse => workspace.list = !workspace.list,
        Intent::Close => {
            mode.close_inventory();
            workspace.stage = Stage::Browse;
        }
        Intent::Options if workspace.stage == Stage::Browse => {
            workspace.stage = Stage::Options;
            workspace.focus = Some(Intent::Cancel);
        }
        Intent::ToggleWarning(index) => {
            if let Some(enabled) = warnings.enabled.get_mut(index) {
                *enabled = !*enabled;
            }
        }
        Intent::Cancel => {
            workspace.stage = Stage::Browse;
            workspace.focus = workspace.selected.map(Intent::Select);
        }
        Intent::Act(verb) if workspace.stage == Stage::Browse => {
            let Some(action) = actions(core, workspace)
                .into_iter()
                .find(|action| action.verb == verb)
            else {
                workspace.feedback = "That action is not available for the selected item.".into();
                return;
            };
            if let Some(reason) = action.reason {
                workspace.feedback = reason;
                return;
            }
            if verb == Verb::Identify {
                workspace.stage = Stage::Sources;
                workspace.focus = Some(Intent::Cancel);
            } else if let Some(command) = action.command {
                let warning =
                    std::array::from_fn(|index| action.warning[index] && warnings.enabled[index]);
                if warning.into_iter().any(std::convert::identity) {
                    workspace.stage = Stage::Review {
                        command,
                        cost: action.cost,
                        warning,
                    };
                    workspace.focus = Some(Intent::Cancel);
                } else {
                    submit(command, core, workspace, queue);
                }
            }
        }
        Intent::Source(scroll) if workspace.stage == Stage::Sources => {
            if let Some(target) = workspace.selected
                && sources(core).contains(&scroll)
            {
                let command = Command::Identify { scroll, target };
                match core.item_action_cost(command) {
                    Ok(cost) => {
                        workspace.stage = Stage::Review {
                            command,
                            cost,
                            warning: [false; 3],
                        };
                        workspace.focus = Some(Intent::Cancel);
                    }
                    Err(reason) => {
                        workspace.feedback = format!("Source rejected: {reason:?}. Nothing spent.");
                    }
                }
            }
        }
        Intent::Confirm => {
            if let Stage::Review { command, .. } = workspace.stage {
                submit(command, core, workspace, queue);
            }
        }
        _ => {}
    }
}

fn submit(
    command: Command,
    core: &CoreState,
    workspace: &mut Workspace,
    queue: &mut CoreCommandQueue,
) {
    if workspace.flight.is_some() || !queue.is_empty() {
        return;
    }
    let Some(target) = selected(core, workspace) else {
        workspace.feedback = "Item is no longer reachable. Nothing spent.".into();
        workspace.stage = Stage::Browse;
        return;
    };
    let region: Vec<_> = items(core)
        .iter()
        .filter(|item| item.container == target.container)
        .map(|item| item.id)
        .collect();
    workspace.flight = Some(Flight {
        command,
        sequence: core.outcomes().last().map_or(0, |outcome| outcome.sequence),
        hp: core.player().map_or(0, prl_core::Actor::hp),
        index: region.iter().position(|id| *id == target.id).unwrap_or(0),
        region,
        source: target.container,
    });
    workspace.stage = Stage::Browse;
    queue.push(command);
}

fn settle(
    core: Res<'_, CoreState>,
    mut workspace: ResMut<'_, Workspace>,
    mut mode: ResMut<'_, UiMode>,
) {
    if workspace.run != core.run_number() {
        *workspace = Workspace {
            run: core.run_number(),
            ..default()
        };
    }
    if core.is_player_dead() {
        if mode.overlay == Overlay::Inventory {
            mode.close_inventory();
        }
        workspace.flight = None;
        workspace.stage = Stage::Browse;
        return;
    }
    if let Some(flight) = workspace.flight.as_ref()
        && let Some(outcome) = core.outcomes().iter().find(|outcome|
            outcome.sequence > flight.sequence && matches!(outcome.kind,
                OutcomeKind::CommandAccepted { command, .. } | OutcomeKind::CommandRejected { command, .. }
                if command == flight.command))
    {
        let flight = flight.clone();
        let failed = matches!(outcome.kind, OutcomeKind::CommandRejected { .. });
        workspace.feedback = core.messages().iter().find(|message| message.sequence >= outcome.sequence)
            .map_or_else(|| "Item action resolved.".into(), |message| message.text.clone());
        if let OutcomeKind::CommandAccepted { cost, .. } = outcome.kind {
            workspace.feedback = format!("{} ({cost} energy)", workspace.feedback);
        }
        if let OutcomeKind::CommandRejected { reason, .. } = outcome.kind
            && let Some(target) = selected(&core, &workspace)
        {
            workspace.feedback = rejection(reason, &core, target, flight.command);
        }
        if !failed {
            let moved_region = matches!(flight.command, Command::Drop(_))
                || matches!(flight.command, Command::Transfer(TransferItems { destination: BACKPACK, .. }))
                    && !matches!(flight.source, BACKPACK | EQUIPMENT);
            if moved_region || selected(&core, &workspace).is_none() {
                let surviving: Vec<_> = flight.region.iter().filter(|id|
                    items(&core).iter().any(|item| item.id == **id && Some(item.id) != workspace.selected)).copied().collect();
                workspace.selected = surviving.get(flight.index.min(surviving.len().saturating_sub(1))).copied();
            }
            let damaged = core.outcomes().iter().any(|event| event.sequence > flight.sequence
                && matches!(event.kind, OutcomeKind::Attacked { target, .. }
                    if core.player().is_some_and(|player| player.id() == target)))
                || core.player().is_some_and(|player| player.hp() < flight.hp);
            if !matches!(flight.command, Command::PlaceItem(_) | Command::Organize)
                && (threatened(&core) || damaged) { mode.close_inventory(); }
        }
        workspace.focus = workspace.selected.map(Intent::Select);
        workspace.flight = None;
    }
    if mode.overlay == Overlay::Inventory
        && !workspace.initialized
        && let Some(first) = items(&core).first()
    {
        workspace.selected = Some(first.id);
        workspace.focus = Some(Intent::Select(first.id));
        workspace.initialized = true;
    }
}
