//! Placement input shares the headless validator; dragging never mutates items.

use super::{Intent, PointerIntent, Stage, Workspace, selected, submit};
use crate::input::{Overlay, UiMode};
use bevy::{
    picking::{
        events::{PointerDragDrop, PointerDragEnd, PointerDragStart},
        pointer::PointerButton,
    },
    prelude::*,
};
use prl_core::{BACKPACK, Command, CoreCommandQueue, CoreState, GridCell, PlaceItem};

#[derive(Component)]
pub(super) struct Grid;

#[derive(Component)]
struct DragPreview;

#[derive(Component)]
struct DragLabel;

#[derive(Resource, Default)]
pub(super) struct Drag {
    placement: Option<PlaceItem>,
    source: Option<Entity>,
    suppress: bool,
}

impl Drag {
    pub(super) fn suppress_activation(&self) -> bool {
        self.placement.is_some() || self.suppress
    }

    pub(super) fn cancel(&mut self) {
        self.source = None;
        if self.placement.take().is_some() {
            self.suppress = true;
        }
    }

    pub(super) fn keyboard(&mut self, keys: &ButtonInput<KeyCode>) -> bool {
        self.suppress = false;
        if self.placement.is_none() {
            return false;
        }
        if keys.just_pressed(KeyCode::Escape) {
            self.cancel();
        } else if keys.just_pressed(KeyCode::KeyR)
            && let Some(placement) = self.placement.as_mut()
        {
            placement.rotated = !placement.rotated;
        }
        true
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Drag>()
        .add_observer(start)
        .add_observer(drop_on_grid)
        .add_observer(end)
        .add_systems(
            Update,
            cancel_interrupted
                .in_set(prl_core::CoreSet::Input)
                .after(crate::input::handle_input)
                .before(super::input),
        )
        .add_systems(
            Update,
            update_preview
                .in_set(prl_core::CoreSet::Presentation)
                .after(super::input)
                .run_if(resource_exists::<CoreState>),
        );
}

fn start(
    event: On<'_, '_, PointerDragStart>,
    controls: Query<'_, '_, &super::view::Control>,
    core: Res<'_, CoreState>,
    workspace: Res<'_, Workspace>,
    mode: Res<'_, UiMode>,
    mut drag: ResMut<'_, Drag>,
) {
    if event.button != PointerButton::Primary
        || mode.overlay != Overlay::Inventory
        || workspace.stage != Stage::Browse
    {
        return;
    }
    if let Ok(super::view::Control(Intent::Select(id))) = controls.get(event.entity)
        && let Some(item) = core
            .inventory()
            .and_then(|bag| bag.items().find(|item| item.id == *id))
        && item.container == BACKPACK
        && let Some(cell) = item.cell
    {
        drag.placement = Some(PlaceItem {
            item: *id,
            cell,
            rotated: item.rotated,
        });
        drag.source = Some(event.entity);
    }
}

fn drop_on_grid(
    event: On<'_, '_, PointerDragDrop>,
    mut drag: ResMut<'_, Drag>,
    mut pointer: ResMut<'_, PointerIntent>,
    grids: Query<'_, '_, (&ComputedNode, &UiGlobalTransform), With<Grid>>,
    windows: Query<'_, '_, &Window, With<bevy::window::PrimaryWindow>>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok((node, transform)) = grids.get(event.entity) else {
        return;
    };
    let Some(mut placement) = drag.placement.take() else {
        return;
    };
    drag.source = None;
    drag.suppress = true;
    let scale = windows.iter().next().map_or(1.0, Window::scale_factor);
    let cell = node
        .normalize_point(*transform, event.pointer.position * scale)
        .and_then(|point| grid_cell(point + Vec2::splat(0.5)));
    pointer.0 = Some(if let Some(cell) = cell {
        placement.cell = cell;
        Intent::DragPlace(placement)
    } else {
        Intent::CancelDrag
    });
}

fn end(
    event: On<'_, '_, PointerDragEnd>,
    mut drag: ResMut<'_, Drag>,
    mut pointer: ResMut<'_, PointerIntent>,
) {
    if event.button == PointerButton::Primary && drag.placement.is_some() {
        drag.cancel();
        pointer.0 = Some(Intent::CancelDrag);
    }
}

fn cancel_interrupted(
    mode: Res<'_, UiMode>,
    mouse: Option<Res<'_, ButtonInput<MouseButton>>>,
    controls: Query<'_, '_, (), With<super::view::Control>>,
    mut drag: ResMut<'_, Drag>,
    mut pointer: ResMut<'_, PointerIntent>,
) {
    if drag.placement.is_some()
        && (mode.overlay != Overlay::Inventory
            || !mode.window_focused()
            || drag.source.is_none_or(|source| !controls.contains(source))
            || mouse.is_some_and(|mouse| mouse.just_released(MouseButton::Left)))
    {
        drag.cancel();
        pointer.0 = Some(Intent::CancelDrag);
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Finite normalized coordinates are bounded before conversion to the 10x6 grid"
)]
fn grid_cell(point: Vec2) -> Option<GridCell> {
    (point.is_finite() && point.x >= 0.0 && point.y >= 0.0 && point.x < 1.0 && point.y < 1.0).then(
        || GridCell {
            x: (point.x * 10.0).floor() as u8,
            y: (point.y * 6.0).floor() as u8,
        },
    )
}

pub(super) fn keyboard(keys: &ButtonInput<KeyCode>, workspace: &mut Workspace) -> Option<Intent> {
    if keys.just_pressed(KeyCode::KeyR) {
        return Some(Intent::Rotate);
    }
    let Stage::Placement(mut placement) = workspace.stage else {
        return None;
    };
    if keys.just_pressed(KeyCode::ArrowLeft) {
        placement.cell.x = placement.cell.x.saturating_sub(1);
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        placement.cell.x = (placement.cell.x + 1).min(9);
    } else if keys.just_pressed(KeyCode::ArrowUp) {
        placement.cell.y = placement.cell.y.saturating_sub(1);
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        placement.cell.y = (placement.cell.y + 1).min(5);
    } else {
        return None;
    }
    workspace.stage = Stage::Placement(placement);
    workspace.focus = Some(Intent::PlaceCell(placement.cell));
    None
}

pub(super) fn activate(
    intent: Intent,
    core: &CoreState,
    workspace: &mut Workspace,
    queue: &mut CoreCommandQueue,
) -> bool {
    match intent {
        Intent::BeginPlacement if workspace.stage == Stage::Browse => {
            let Some(item) = selected(core, workspace).filter(|item| item.container == BACKPACK)
            else {
                workspace.feedback = "Select a carried backpack item to move.".into();
                return true;
            };
            if let Some(cell) = item.cell {
                workspace.stage = Stage::Placement(PlaceItem {
                    item: item.id,
                    cell,
                    rotated: item.rotated,
                });
                workspace.focus = Some(Intent::PlaceCell(cell));
                workspace.feedback = "Placement is free. Click a cell to place; arrows preview, R rotates, Enter places, Esc cancels.".into();
            }
        }
        Intent::Rotate => {
            if let Stage::Placement(ref mut placement) = workspace.stage {
                placement.rotated = !placement.rotated;
            }
        }
        Intent::PlaceCell(cell) => {
            if let Stage::Placement(mut placement) = workspace.stage {
                placement.cell = cell;
                workspace.stage = Stage::Placement(placement);
                commit(placement, core, workspace, queue);
            }
        }
        Intent::Place => {
            if let Stage::Placement(placement) = workspace.stage {
                commit(placement, core, workspace, queue);
            }
        }
        Intent::DragPlace(placement) if workspace.stage == Stage::Browse => {
            workspace.selected = Some(placement.item);
            commit(placement, core, workspace, queue);
        }
        Intent::Cancel if matches!(workspace.stage, Stage::Placement(_)) => {
            workspace.stage = Stage::Browse;
            workspace.focus = workspace.selected.map(Intent::Select);
            workspace.feedback = "Placement cancelled. Original layout kept; no time spent.".into();
        }
        Intent::CancelDrag => {
            workspace.feedback = "Placement cancelled. Original layout kept; no time spent.".into();
        }
        Intent::Organize if workspace.stage == Stage::Browse => {
            match core.item_action_cost(Command::Organize) {
                Ok(_) => {
                    if workspace.selected.is_none() {
                        workspace.selected = super::items(core).first().map(|item| item.id);
                    }
                    if workspace.selected.is_some() {
                        submit(Command::Organize, core, workspace, queue);
                    } else {
                        workspace.feedback = "Backpack empty. Nothing to organize.".into();
                    }
                }
                Err(reason) => {
                    workspace.feedback = format!(
                        "Organize could not fit every item ({reason:?}). Entire layout kept; no time spent."
                    );
                }
            }
        }
        _ => return false,
    }
    true
}

fn commit(
    placement: PlaceItem,
    core: &CoreState,
    workspace: &mut Workspace,
    queue: &mut CoreCommandQueue,
) {
    match core.item_action_cost(Command::PlaceItem(placement)) {
        Ok(_) => submit(Command::PlaceItem(placement), core, workspace, queue),
        Err(reason) => {
            let reason = match reason {
                prl_core::RejectReason::OutOfBounds => "Shape extends outside the 10x6 backpack",
                prl_core::RejectReason::BackpackFull => "Shape overlaps another backpack item",
                prl_core::RejectReason::ItemInaccessible => "Item is no longer in the backpack",
                prl_core::RejectReason::PlayerDead => "The run has ended",
                _ => "Item is not valid for placement",
            };
            workspace.feedback = format!("{reason}. Original layout kept; no time spent.");
        }
    }
}

fn preview_style(core: &CoreState, placement: PlaceItem) -> Option<(Node, Color)> {
    let bag = core.inventory()?;
    let mut item = bag.items().find(|item| item.id == placement.item)?.clone();
    item.rotated = placement.rotated;
    let (width, height) = bag.footprint(&item)?;
    let valid = bag.check_placement(placement).is_ok();
    Some((
        Node {
            position_type: PositionType::Absolute,
            left: percent(f32::from(placement.cell.x) * 10.0),
            top: percent(f32::from(placement.cell.y) * 100.0 / 6.0),
            width: percent(f32::from(width) * 10.0),
            height: percent(f32::from(height) * 100.0 / 6.0),
            border: UiRect::all(px(3)),
            ..default()
        },
        if valid {
            Color::srgba(0.1, 0.6, 0.3, 0.7)
        } else {
            Color::srgba(0.8, 0.1, 0.1, 0.7)
        },
    ))
}

pub(super) fn preview(
    parent: &mut ChildSpawnerCommands<'_>,
    core: &CoreState,
    placement: PlaceItem,
    theme: &crate::PlaceholderTheme,
) {
    if let Some((node, color)) = preview_style(core, placement) {
        let valid = core
            .inventory()
            .is_some_and(|bag| bag.check_placement(placement).is_ok());
        parent
            .spawn((
                node,
                BackgroundColor(color),
                BorderColor::all(Color::WHITE),
                Pickable::IGNORE,
                ZIndex(2),
            ))
            .with_child((
                Text::new(if valid { "OK" } else { "X" }),
                theme.font(),
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
    }
}

pub(super) fn drag_preview_node(
    parent: &mut ChildSpawnerCommands<'_>,
    theme: &crate::PlaceholderTheme,
) {
    parent
        .spawn((
            DragPreview,
            Node {
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::WHITE),
            Pickable::IGNORE,
            ZIndex(2),
        ))
        .with_child((
            DragLabel,
            Text::new(""),
            theme.font(),
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        ));
}

fn update_preview(
    drag: Res<'_, Drag>,
    core: Res<'_, CoreState>,
    grids: Query<'_, '_, (&ComputedNode, &UiGlobalTransform, Option<&CalculatedClip>), With<Grid>>,
    windows: Query<'_, '_, &Window, With<bevy::window::PrimaryWindow>>,
    mut previews: Query<'_, '_, (&mut Node, &mut BackgroundColor), With<DragPreview>>,
    mut labels: Query<'_, '_, &mut Text, With<DragLabel>>,
) {
    let placement = drag.placement.and_then(|mut placement| {
        let window = windows.iter().next()?;
        let position = window.cursor_position()? * window.scale_factor();
        let (node, transform, clip) = grids.iter().next()?;
        if clip.is_some_and(|clip| !clip.contains_point(position)) {
            return None;
        }

        placement.cell = grid_cell(node.normalize_point(*transform, position)? + Vec2::splat(0.5))?;
        Some(placement)
    });
    for (mut node, mut color) in &mut previews {
        if let Some((style, background)) =
            placement.and_then(|placement| preview_style(&core, placement))
        {
            *node = style;
            color.0 = background;
        } else if node.display != Display::None {
            node.display = Display::None;
        }
        if let Some(placement) = placement {
            let label = if core
                .inventory()
                .is_some_and(|bag| bag.check_placement(placement).is_ok())
            {
                "OK"
            } else {
                "X"
            };
            for mut text in &mut labels {
                if text.0 != label {
                    text.0 = label.into();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        camera::NormalizedRenderTarget,
        picking::{
            backend::HitData,
            events::Pointer,
            pointer::{Location, PointerId},
        },
        window::WindowRef,
    };
    use prl_core::ItemId;

    #[test]
    fn drop_requires_a_picked_grid_target_not_just_overlapping_coordinates() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Drag>()
            .init_resource::<PointerIntent>()
            .add_observer(drop_on_grid)
            .add_observer(end);
        let window = app.world_mut().spawn(Window::default()).id();
        let source = app.world_mut().spawn_empty().id();
        let grid = app
            .world_mut()
            .spawn((
                Grid,
                ComputedNode {
                    size: Vec2::new(100.0, 60.0),
                    ..default()
                },
                UiGlobalTransform::default(),
            ))
            .id();
        let cell = app.world_mut().spawn(ChildOf(grid)).id();
        let footer = app.world_mut().spawn_empty().id();
        let pointer = Pointer::new(
            PointerId::Mouse,
            Location {
                target: NormalizedRenderTarget::Window(
                    WindowRef::Entity(window).normalize(None).expect("window"),
                ),
                position: Vec2::ZERO,
            },
        );
        let placement = PlaceItem {
            item: ItemId(1),
            cell: GridCell { x: 0, y: 0 },
            rotated: true,
        };
        for target in [cell, footer] {
            app.world_mut().insert_resource(Drag {
                placement: Some(placement),
                source: Some(source),
                suppress: false,
            });
            app.world_mut().resource_mut::<PointerIntent>().0 = None;
            app.world_mut().trigger(PointerDragDrop {
                entity: target,
                pointer: pointer.clone(),
                button: PointerButton::Primary,
                dropped: source,
                hit: HitData::new(window, 0.0, None, None),
            });
            app.world_mut().trigger(PointerDragEnd {
                entity: source,
                pointer: pointer.clone(),
                button: PointerButton::Primary,
                distance: Vec2::ZERO,
            });
            let expected = if target == cell {
                Intent::DragPlace(PlaceItem {
                    cell: GridCell { x: 5, y: 3 },
                    ..placement
                })
            } else {
                Intent::CancelDrag
            };
            assert_eq!(app.world().resource::<PointerIntent>().0, Some(expected));
            let drag = app.world().resource::<Drag>();
            assert!(drag.placement.is_none());
            assert!(drag.source.is_none());
            assert!(drag.suppress_activation());
        }
    }

    #[test]
    fn preview_coordinates_reject_nonfinite_and_outside_grid_points() {
        for point in [
            Vec2::new(f32::NAN, 0.0),
            Vec2::new(0.0, f32::INFINITY),
            Vec2::new(-0.001, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
        ] {
            assert_eq!(grid_cell(point), None);
        }
        assert_eq!(grid_cell(Vec2::ZERO), Some(GridCell { x: 0, y: 0 }));
        assert_eq!(grid_cell(Vec2::splat(0.999)), Some(GridCell { x: 9, y: 5 }));
    }
}
