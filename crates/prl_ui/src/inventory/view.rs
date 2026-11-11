use super::{
    Intent, InventoryWarningSettings, Stage, Workspace, actions, details, items, selected, sources,
    threatened,
};
use crate::{
    PlaceholderTheme, SettingsStatus,
    input::{Overlay, UiMode},
};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    ui::RelativeCursorPosition,
};
use prl_content::{EquipSlot, ItemIcon, ItemKind};
use prl_core::{BACKPACK, CoreState, EQUIPMENT, Item};

#[derive(Component)]
pub(super) struct Root;

#[derive(Component)]
pub(super) struct Control(pub(super) Intent);

#[derive(Component)]
pub(super) struct Pane(usize);

#[derive(Component)]
pub(super) struct IconTarget(ItemIcon, bool);

/// Decorate newly built controls only; idle frames retain the same widget tree.
pub(super) fn decorate_icons(
    mut commands: Commands<'_, '_>,
    images: Res<'_, super::ItemIconImages>,
    mut targets: Query<'_, '_, (Entity, &IconTarget, &mut Node), Added<IconTarget>>,
) {
    for (entity, target, mut node) in &mut targets {
        let Some(image) = images.0.get(&target.0) else {
            continue;
        };
        if !target.1 {
            node.padding.left = px(38);
        }
        commands.entity(entity).with_child((
            Name::new("Reviewed item icon"),
            image.clone(),
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                width: px(28),
                height: px(28),
                left: if target.1 { Val::Auto } else { px(4) },
                right: if target.1 { px(2) } else { Val::Auto },
                bottom: px(2),
                ..default()
            },
        ));
    }
}

pub(super) fn detail_icon(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    item: &Item,
    images: &super::ItemIconImages,
) {
    if let Some(image) = core
        .inventory()
        .and_then(|bag| bag.icon(item))
        .and_then(|icon| images.0.get(&icon))
    {
        parent.spawn((
            Name::new("Reviewed item icon"),
            image.clone(),
            Pickable::IGNORE,
            Node {
                width: px(32),
                height: px(32),
                flex_shrink: 0.0,
                ..default()
            },
        ));
    }
}

pub(super) fn text(
    parent: &mut ChildSpawnerCommands<'_>,
    value: impl Into<String>,
    theme: &PlaceholderTheme,
) {
    parent.spawn((
        Text::new(value),
        theme.font(),
        TextColor(theme.text),
        Node {
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

fn button(
    parent: &mut ChildSpawnerCommands<'_>,
    label: String,
    intent: Intent,
    workspace: &Workspace,
    theme: &PlaceholderTheme,
    mut node: Node,
    disabled: bool,
) -> Entity {
    let compact = node.height != Val::Auto;
    node.min_height = px(if compact { 0 } else { 44 });
    node.padding = UiRect::all(px(if compact { 2 } else { 6 }));
    node.border = UiRect::all(px(if workspace.focus == Some(intent) {
        3
    } else {
        1
    }));
    let active = matches!(intent, Intent::Select(id) if workspace.selected == Some(id));
    parent
        .spawn((
            Name::new(label.clone()),
            bevy::ui_widgets::Button,
            node,
            Control(intent),
            BackgroundColor(if active {
                Color::srgb(0.20, 0.28, 0.32)
            } else {
                Color::srgb(0.10, 0.13, 0.15)
            }),
            BorderColor::all(if workspace.focus == Some(intent) {
                Color::WHITE
            } else if active {
                Color::srgb(1.0, 0.83, 0.5)
            } else {
                Color::srgb(0.40, 0.45, 0.47)
            }),
        ))
        .with_child((
            Text::new(label),
            theme.font(),
            TextColor(if disabled {
                Color::srgb(0.75, 0.75, 0.75)
            } else {
                theme.text
            }),
        ))
        .id()
}

fn row() -> Node {
    Node {
        display: Display::Flex,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(8),
        row_gap: px(6),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        min_width: px(0),
        flex_shrink: 0.0,
        ..default()
    }
}

fn item_label(core: &CoreState, item: &Item) -> String {
    core.inventory()
        .map_or_else(|| "Item unavailable".into(), |bag| bag.label(item))
}

fn item_button(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    item: &Item,
    workspace: &Workspace,
    theme: &PlaceholderTheme,
    node: Node,
    grid: bool,
) {
    let label = if grid {
        core.inventory()
            .and_then(|bag| bag.definition(item))
            .map_or_else(
                || "?".into(),
                |definition| {
                    format!(
                        "{}{}",
                        match definition.kind {
                            ItemKind::Gear(EquipSlot::Body) => "AR",
                            ItemKind::Gear(_) => "SW",
                            ItemKind::Healing { .. } => "P",
                            ItemKind::Identify => "S",
                        },
                        if item.identified { "" } else { " ?" }
                    )
                },
            )
    } else {
        item_label(core, item)
    };
    let entity = button(
        parent,
        label,
        Intent::Select(item.id),
        workspace,
        theme,
        node,
        false,
    );
    if let Some(icon) = core.inventory().and_then(|bag| bag.icon(item)) {
        parent
            .commands()
            .entity(entity)
            .insert(IconTarget(icon, grid));
    }
}

fn equipment(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    workspace: &Workspace,
    theme: &PlaceholderTheme,
) {
    text(parent, "EQUIPMENT", theme);
    for (slot, name) in [
        (EquipSlot::MainHand, "Main hand"),
        (EquipSlot::Body, "Body"),
    ] {
        text(parent, name, theme);
        if let Some(item) = core
            .inventory()
            .into_iter()
            .flat_map(prl_core::Inventory::items)
            .find(|item| {
                item.container == EQUIPMENT
                    && core
                        .inventory()
                        .and_then(|bag| bag.definition(item))
                        .is_some_and(|definition| definition.kind == ItemKind::Gear(slot))
            })
        {
            item_button(
                parent,
                core,
                item,
                workspace,
                theme,
                Node {
                    width: percent(100),
                    ..default()
                },
                false,
            );
        } else {
            text(parent, "Empty", theme);
        }
    }
    text(
        parent,
        "Two functional slice slots.\nNo shield/block slot yet.",
        theme,
    );
}

fn bag(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    workspace: &Workspace,
    theme: &PlaceholderTheme,
) {
    text(parent, "BACKPACK / 10 x 6", theme);
    if workspace.stage == Stage::Browse {
        button(
            parent,
            if workspace.list {
                "Show Grid"
            } else {
                "Show List"
            }
            .into(),
            Intent::List,
            workspace,
            theme,
            Node::default(),
            false,
        );
    }
    let bag_items: Vec<_> = items(core)
        .into_iter()
        .filter(|item| item.container == BACKPACK)
        .collect();
    if bag_items.is_empty() {
        text(parent, "Backpack empty", theme);
    } else if workspace.list && !matches!(workspace.stage, Stage::Placement(_)) {
        for item in bag_items {
            item_button(
                parent,
                core,
                item,
                workspace,
                theme,
                Node {
                    width: percent(100),
                    ..default()
                },
                false,
            );
        }
    } else {
        grid(parent, core, &bag_items, workspace, theme);
    }
    if let Some(bag) = core.inventory() {
        text(
            parent,
            format!(
                "Carry {} / {} g\nPlaytest pickup limit (not STR-derived)",
                bag.carried_weight(),
                bag.carry_capacity()
            ),
            theme,
        );
    }
    if workspace.stage == Stage::Browse {
        parent.spawn(row()).with_children(|buttons| {
            button(
                buttons,
                "Move / rotate [P]".into(),
                Intent::BeginPlacement,
                workspace,
                theme,
                Node::default(),
                false,
            );
            button(
                buttons,
                "Organize [O] / free".into(),
                Intent::Organize,
                workspace,
                theme,
                Node::default(),
                false,
            );
        });
    }
    text(
        parent,
        "Select: inspect only. Move: click source, then cell.\nDrag: release at top-left destination; R rotates; Esc cancels.\nPickup never rearranges existing items.",
        theme,
    );
}

fn grid(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    bag_items: &[&Item],
    workspace: &Workspace,
    theme: &PlaceholderTheme,
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                max_width: px(480),
                aspect_ratio: Some(10.0 / 6.0),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgb(0.03, 0.04, 0.05)),
            super::packing::Grid,
        ))
        .with_children(|grid| {
            for y in 0_u8..6 {
                for x in 0_u8..10 {
                    grid.spawn((
                        bevy::ui_widgets::Button,
                        Control(Intent::PlaceCell(prl_core::GridCell { x, y })),
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(f32::from(x) * 10.0),
                            top: percent(f32::from(y) * 100.0 / 6.0),
                            width: percent(10),
                            height: percent(100.0 / 6.0),
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.18, 0.22, 0.24)),
                    ));
                }
            }
            for item in bag_items {
                if let Some((cell, (width, height))) = item
                    .cell
                    .zip(core.inventory().and_then(|bag| bag.footprint(item)))
                {
                    item_button(
                        grid,
                        core,
                        item,
                        workspace,
                        theme,
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(f32::from(cell.x) * 10.0),
                            top: percent(f32::from(cell.y) * 100.0 / 6.0),
                            width: percent(f32::from(width) * 10.0),
                            height: percent(f32::from(height) * 100.0 / 6.0),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        true,
                    );
                }
            }
            if let Stage::Placement(placement) = workspace.stage {
                for y in 0..6 {
                    for x in 0..10 {
                        grid.spawn((
                            bevy::ui_widgets::Button,
                            Control(Intent::PlaceCell(prl_core::GridCell { x, y })),
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent(f32::from(x) * 10.0),
                                top: percent(f32::from(y) * 100.0 / 6.0),
                                width: percent(10),
                                height: percent(100.0 / 6.0),
                                ..default()
                            },
                        ));
                    }
                }
                super::packing::preview(grid, core, placement, theme);
            } else {
                super::packing::drag_preview_node(grid, theme);
            }
        });
}

fn review(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    workspace: &Workspace,
    warnings: &InventoryWarningSettings,
    theme: &PlaceholderTheme,
) {
    match workspace.stage {
        Stage::Sources => {
            text(parent, "IDENTIFY / choose a known carried source", theme);
            text(
                parent,
                "One scroll / 100 energy. Unknown types never auto-selected.",
                theme,
            );
            for id in sources(core) {
                if let Some(item) = items(core).into_iter().find(|item| item.id == id) {
                    button(
                        parent,
                        format!("{} (item {})", item_label(core, item), id.0),
                        Intent::Source(id),
                        workspace,
                        theme,
                        Node::default(),
                        false,
                    );
                }
            }
        }
        Stage::Review {
            command,
            cost,
            warning,
        } => {
            text(
                parent,
                format!("REVIEW / {cost} energy\nNo time passes until Commit."),
                theme,
            );
            if let Some(item) = selected(core, workspace) {
                text(parent, item_label(core, item), theme);
            }
            if let prl_core::Command::Identify { scroll, .. } = command
                && let Some(source) = items(core).into_iter().find(|item| item.id == scroll)
            {
                text(
                    parent,
                    format!("Consumes: {} (item {})", item_label(core, source), scroll.0),
                    theme,
                );
            }
            if matches!(command, prl_core::Command::Identify { .. }) {
                text(
                    parent,
                    "One source is consumed to identify this target.",
                    theme,
                );
            }
            warning_checkboxes(parent, warning, workspace, warnings, theme);
            button(
                parent,
                "Commit one action".into(),
                Intent::Confirm,
                workspace,
                theme,
                Node::default(),
                false,
            );
        }
        Stage::Options => warning_options(parent, workspace, warnings, theme),
        Stage::Browse | Stage::Placement(_) => {}
    }
    if workspace.stage != Stage::Browse {
        text(parent, warnings.status.clone(), theme);
        button(
            parent,
            "Cancel / Back [Esc]".into(),
            Intent::Cancel,
            workspace,
            theme,
            Node::default(),
            false,
        );
    }
}

fn warning_checkboxes(
    parent: &mut ChildSpawnerCommands<'_>,
    warning: [bool; 3],
    workspace: &Workspace,
    warnings: &InventoryWarningSettings,
    theme: &PlaceholderTheme,
) {
    for (index, _) in warning
        .into_iter()
        .enumerate()
        .filter(|(_, enabled)| *enabled)
    {
        text(
            parent,
            [
                "Drop leaves the item on this cell. No undo of the turn.",
                "Unknown effect. This commits to use, not inspect.",
                "Body change: 3 standard actions. Enemies may act multiple times.",
            ][index],
            theme,
        );
        button(
            parent,
            format!(
                "[{}] Warn me next time: {}",
                if warnings.enabled[index] { "x" } else { " " },
                ["dropping", "unknown use", "Body changes in danger"][index]
            ),
            Intent::ToggleWarning(index),
            workspace,
            theme,
            Node::default(),
            false,
        );
    }
}

fn warning_options(
    parent: &mut ChildSpawnerCommands<'_>,
    workspace: &Workspace,
    warnings: &InventoryWarningSettings,
    theme: &PlaceholderTheme,
) {
    text(parent, "OPTIONS / Inventory warnings", theme);
    for (index, label) in [
        "Dropping items",
        "Unknown consumables",
        "Body changes in danger",
    ]
    .iter()
    .enumerate()
    {
        button(
            parent,
            format!(
                "[{}] {label}",
                if warnings.enabled[index] { "x" } else { " " }
            ),
            Intent::ToggleWarning(index),
            workspace,
            theme,
            Node::default(),
            false,
        );
    }
    text(parent, "Settings apply across runs and restarts.", theme);
}

pub(super) fn refresh(
    mut commands: Commands<'_, '_>,
    core: Res<'_, CoreState>,
    workspace: Res<'_, Workspace>,
    mode: Res<'_, UiMode>,
    warnings: Res<'_, InventoryWarningSettings>,
    theme: Res<'_, PlaceholderTheme>,
    images: Res<'_, super::ItemIconImages>,
    settings: Res<'_, SettingsStatus>,
    scale: Option<Res<'_, UiScale>>,
    windows: Query<'_, '_, &Window, With<bevy::window::PrimaryWindow>>,
    roots: Query<'_, '_, Entity, With<Root>>,
    mut last_view: Local<'_, (Vec2, Option<Overlay>)>,
) {
    let size = windows
        .iter()
        .next()
        .map_or(Vec2::new(1280.0, 900.0), |window| {
            Vec2::new(window.width(), window.height())
        })
        / scale.as_ref().map_or(1.0, |scale| scale.0);
    if !core.is_changed()
        && !workspace.is_changed()
        && !warnings.is_changed()
        && !settings.is_changed()
        && last_view.0 == size
        && last_view.1 == Some(mode.overlay)
    {
        return;
    }
    *last_view = (size, Some(mode.overlay));
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    if mode.overlay != Overlay::Inventory {
        return;
    }
    commands
        .spawn((
            Name::new("Inventory workspace"),
            Root,
            ZIndex(20),
            BackgroundColor(theme.panel.with_alpha(1.0)),
            Node {
                position_type: PositionType::Absolute,
                left: percent(1),
                top: percent(1),
                width: percent(98),
                height: percent(98),
                padding: UiRect::all(px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|root| {
            header(root, &core, &workspace, &theme);
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    flex_basis: px(0),
                    min_height: px(0),
                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    ..default()
                },
                ScrollPosition(Vec2::new(0.0, workspace.scroll[3])),
                RelativeCursorPosition::default(),
                Pane(3),
            ))
            .with_children(|content| {
                if matches!(workspace.stage, Stage::Browse | Stage::Placement(_)) {
                    browse(content, &core, &workspace, &theme, &images, size);
                    if matches!(workspace.stage, Stage::Placement(_)) {
                        content.spawn(row()).with_children(|buttons| {
                            button(
                                buttons,
                                "Rotate [R] / free".into(),
                                Intent::Rotate,
                                &workspace,
                                &theme,
                                Node::default(),
                                false,
                            );
                            button(
                                buttons,
                                "Place [Enter] / free".into(),
                                Intent::Place,
                                &workspace,
                                &theme,
                                Node::default(),
                                false,
                            );
                            button(
                                buttons,
                                "Cancel [Esc]".into(),
                                Intent::Cancel,
                                &workspace,
                                &theme,
                                Node::default(),
                                false,
                            );
                        });
                    }
                } else {
                    review(content, &core, &workspace, &warnings, &theme);
                }
            });
            footer(
                root, &core, &workspace, &warnings, &theme, &settings, size.y,
            );
        });
}

fn header(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    workspace: &Workspace,
    theme: &PlaceholderTheme,
) {
    parent.spawn(row()).with_children(|header| {
        text(
            header,
            core.player().map_or_else(
                || "INVENTORY".into(),
                |player| {
                    format!(
                        "INVENTORY / HP {}/{} / Browsing: free",
                        player.hp(),
                        player.max_hp()
                    )
                },
            ),
            theme,
        );
        if workspace.stage == Stage::Browse {
            button(
                header,
                "Options".into(),
                Intent::Options,
                workspace,
                theme,
                Node::default(),
                false,
            );
        }
        button(
            header,
            "Close [Esc]".into(),
            Intent::Close,
            workspace,
            theme,
            Node::default(),
            false,
        );
    });
    text(
        parent,
        if threatened(core) {
            "HOSTILE VISIBLE: costed actions advance enemies and close this view; packing is free."
        } else {
            "Costed actions close on danger/damage. Browsing and packing remain free."
        },
        theme,
    );
}

fn browse(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    workspace: &Workspace,
    theme: &PlaceholderTheme,
    images: &super::ItemIconImages,
    size: Vec2,
) {
    let narrow = size.x < 1000.0;
    parent
        .spawn(Node {
            flex_direction: if narrow {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            },
            column_gap: px(12),
            row_gap: px(12),
            min_height: px(0),
            flex_grow: if narrow { 0.0 } else { 1.0 },
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|columns| {
            for index in 0..3 {
                columns
                    .spawn((
                        Node {
                            width: if narrow { percent(100) } else { px(0) },
                            flex_grow: if narrow {
                                0.0
                            } else {
                                [20.0, 43.0, 35.0][index]
                            },
                            flex_shrink: if narrow { 0.0 } else { 1.0 },
                            max_height: if narrow {
                                px(10000)
                            } else {
                                px((size.y - 340.0).max(130.0))
                            },
                            overflow: if narrow {
                                Overflow::visible()
                            } else {
                                Overflow::scroll_y()
                            },
                            padding: UiRect::all(px(6)),
                            ..column()
                        },
                        ScrollPosition(Vec2::new(0.0, workspace.scroll[index])),
                        RelativeCursorPosition::default(),
                        Pane(index),
                    ))
                    .with_children(|pane| match index {
                        0 => equipment(pane, core, workspace, theme),
                        1 => bag(pane, core, workspace, theme),
                        _ => details::render(pane, core, workspace, theme, images),
                    });
            }
        });
    ground(parent, core, workspace, theme);
}

fn ground(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    workspace: &Workspace,
    theme: &PlaceholderTheme,
) {
    text(
        parent,
        "AT YOUR FEET / current cell only / manual pickup",
        theme,
    );
    parent.spawn(row()).with_children(|ground| {
        let mut count = 0;
        if let Some(player) = core.player() {
            for item in core.ground_items(player.position()) {
                item_button(
                    ground,
                    core,
                    item,
                    workspace,
                    theme,
                    Node {
                        max_width: px(240),
                        ..default()
                    },
                    false,
                );
                count += 1;
            }
        }
        if count == 0 {
            text(ground, "No items at your feet.", theme);
        }
    });
}

fn footer(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    workspace: &Workspace,
    warnings: &InventoryWarningSettings,
    theme: &PlaceholderTheme,
    settings: &SettingsStatus,
    height: f32,
) {
    parent.spawn((Node { max_height: px((height * 0.42).max(80.0)),
        overflow: Overflow::scroll_y(), ..column() },
        ScrollPosition(Vec2::new(0.0, workspace.scroll[4])),
        RelativeCursorPosition::default(), Pane(4),
        BackgroundColor(Color::srgb(0.07, 0.09, 0.11))))
        .with_children(|footer| {
            if workspace.stage == Stage::Browse {
                text(footer, selected(core, workspace).map_or_else(|| "No item selected".into(),
                    |item| format!("Selected: {}", item_label(core, item))), theme);
                footer.spawn(row()).with_children(|buttons| {
                    for action in actions(core, workspace) {
                        buttons.spawn(Node { max_width: px(300), ..column() }).with_children(|option| {
                            button(option, format!("{} / {} energy", action.title, action.cost),
                                Intent::Act(action.verb), workspace, theme, Node::default(), action.reason.is_some());
                            if let Some(reason) = action.reason { text(option, reason, theme); }
                        });
                    }
                });
            }
            text(footer, workspace.feedback.clone(), theme);
            text(footer, format!("Save: {} | Warnings: {}", settings.persistence, warnings.status), theme);
            text(footer, "Tab/Shift+Tab: focus; Enter: activate; arrows: items; PgUp/PgDn: details (Shift: bag); Esc: cancel/close.", theme);
        });
}

pub(super) fn scroll(
    mut wheel: MessageReader<'_, '_, MouseWheel>,
    keys: Option<Res<'_, ButtonInput<KeyCode>>>,
    mode: Res<'_, UiMode>,
    mut workspace: ResMut<'_, Workspace>,
    windows: Query<'_, '_, &Window, With<bevy::window::PrimaryWindow>>,
    scale: Option<Res<'_, UiScale>>,
    mut panes: Query<'_, '_, (&Pane, &Node, &RelativeCursorPosition, &mut ScrollPosition)>,
) {
    let delta: f32 = wheel
        .read()
        .map(|event| {
            -event.y
                * if event.unit == MouseScrollUnit::Line {
                    36.0
                } else {
                    1.0
                }
        })
        .sum();
    if mode.overlay != Overlay::Inventory {
        return;
    }
    let key_delta = keys.as_ref().map_or(0.0, |keys| {
        if keys.just_pressed(KeyCode::PageDown) {
            120.0
        } else if keys.just_pressed(KeyCode::PageUp) {
            -120.0
        } else {
            0.0
        }
    });
    let hovered = panes
        .iter()
        .filter(|(_, node, cursor, _)| {
            node.overflow.y == OverflowAxis::Scroll && cursor.cursor_over()
        })
        .map(|(pane, _, _, _)| pane.0)
        .min();
    let narrow = windows.iter().next().is_some_and(|window| {
        window.width() / scale.as_ref().map_or(1.0, |scale| scale.0) < 1000.0
    });
    let key_target = if narrow || workspace.stage != Stage::Browse {
        3
    } else if keys
        .as_ref()
        .is_some_and(|keys| keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight))
    {
        1
    } else {
        2
    };
    let target = hovered.unwrap_or(3);
    for (pane, _, _, mut scroll) in &mut panes {
        if (pane.0 == target && delta != 0.0) || (pane.0 == key_target && key_delta != 0.0) {
            scroll.0.y = (scroll.0.y
                + if pane.0 == target { delta } else { 0.0 }
                + if pane.0 == key_target { key_delta } else { 0.0 })
            .max(0.0);
            workspace.scroll[pane.0] = scroll.0.y;
        }
    }
}

pub(super) fn reveal_focus(
    mode: Res<'_, UiMode>,
    mut workspace: ResMut<'_, Workspace>,
    mut previous: Local<'_, Option<Intent>>,
    controls: Query<'_, '_, (Entity, &Control, &ComputedNode, &UiGlobalTransform)>,
    parents: Query<'_, '_, &ChildOf>,
    mut panes: Query<
        '_,
        '_,
        (
            &Pane,
            &Node,
            &ComputedNode,
            &UiGlobalTransform,
            &mut ScrollPosition,
        ),
    >,
) {
    if mode.overlay != Overlay::Inventory || workspace.focus == *previous {
        return;
    }
    let Some((entity, _, node, transform)) = controls
        .iter()
        .find(|(_, control, _, _)| Some(control.0) == workspace.focus)
    else {
        return;
    };
    if node.size().y <= 0.0 {
        return;
    }
    *previous = workspace.focus;
    let mut top = transform.translation.y - node.size().y * 0.5;
    let height = node.size().y;
    let mut ancestor = entity;
    while let Ok(parent) = parents.get(ancestor) {
        ancestor = parent.parent();
        if let Ok((pane, style, node, transform, mut scroll)) = panes.get_mut(ancestor) {
            if style.overflow.y != OverflowAxis::Scroll {
                continue;
            }
            let min = transform.translation.y - node.size().y * 0.5;
            let max = min + node.size().y;
            let delta = if top < min {
                top - min
            } else if top + height > max {
                (top + height - max).min(top - min)
            } else {
                0.0
            };
            if delta.abs() > 0.5 {
                scroll.0.y = (scroll.0.y + delta * node.inverse_scale_factor()).max(0.0);
                workspace.scroll[pane.0] = scroll.0.y;
                top -= delta;
            }
        }
    }
}
