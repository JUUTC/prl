use bevy::prelude::*;
use prl_core::{CoreSet, CoreState, Position};

use crate::actors::depth;
use crate::art::AtlasArt;
use crate::camera::{TILE_SIZE, tile_to_world};

#[derive(Component)]
struct Placeholder;

enum TerrainAppearance {
    Floor,
    Wall,
    Door { open: bool },
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        refresh
            .in_set(CoreSet::Presentation)
            .run_if(resource_exists::<CoreState>)
            .run_if(resource_changed::<CoreState>),
    );
}

fn tint(visible: bool) -> Color {
    if visible {
        Color::WHITE
    } else {
        Color::srgb(0.3, 0.3, 0.3)
    }
}

fn refresh(
    mut commands: Commands<'_, '_>,
    state: Res<'_, CoreState>,
    art: Option<Res<'_, AtlasArt>>,
    placeholders: Query<'_, '_, Entity, With<Placeholder>>,
) {
    for entity in &placeholders {
        commands.entity(entity).despawn();
    }
    spawn_terrain(&mut commands, &state, art.as_deref());
    spawn_loot(&mut commands, &state);
    if let Some(plan) = state.floor_plan() {
        for (position, label, marker) in [
            (plan.entrance, "<", "stairs_up"),
            (plan.exit, ">", "stairs_down"),
        ] {
            if marker == "stairs_up" && state.floor_number() == Some(1) {
                continue;
            }
            if !state.map().is_visible(position) && state.map().remembered_tile(position).is_none()
            {
                continue;
            }
            let centre = tile_to_world(position.x, position.y);
            if let Some((art, tile)) = art
                .as_deref()
                .and_then(|art| art.index.markers.get(marker).map(|tile| (art, tile)))
            {
                let mut sprite = art.sprite(tile.closed);
                sprite.color = tint(state.is_visible(position));
                commands.spawn((
                    Name::new(format!("Art {marker}")),
                    sprite,
                    Transform::from_translation(
                        (centre + tile.offset).extend(depth(position.y) + 0.0002),
                    ),
                    Placeholder,
                ));
                continue;
            }
            commands.spawn((
                Name::new("Placeholder stairs"),
                Text2d::new(label),
                TextFont {
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                TextColor(tint(state.is_visible(position))),
                Transform::from_xyz(centre.x, centre.y, 0.1),
                Placeholder,
            ));
        }
    }
}

fn spawn_loot(commands: &mut Commands<'_, '_>, state: &CoreState) {
    let Some(inventory) = state.inventory() else {
        return;
    };
    for position in state.map().visible_positions() {
        if !state
            .ground_items(position)
            .any(|item| inventory.ground_label(item).0)
        {
            continue;
        }
        let centre = tile_to_world(position.x, position.y);
        commands.spawn((
            Name::new("Placeholder ground loot"),
            Text2d::new("*"),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(centre.x + 16.0, centre.y - 16.0, 0.11),
            Placeholder,
        ));
    }
}

fn spawn_terrain(commands: &mut Commands<'_, '_>, state: &CoreState, art: Option<&AtlasArt>) {
    let map = state.map();
    for y in 0..map.height() {
        for x in 0..map.width() {
            let (x, y) = (i32::from(x), i32::from(y));
            let position = Position { x, y };
            // Memory carries the last observed door state, not unseen live terrain.
            let Some(memory) = map.remembered_tile(position) else {
                continue;
            };
            let Some(definition) = state.tile_definition(&memory.definition) else {
                continue;
            };
            let visible = map.is_visible(position);
            let centre = tile_to_world(x, y);
            let raised = definition.door || definition.blocks_movement;
            let z = if raised { depth(y) } else { 0.0 };
            let tile = art.and_then(|art| {
                art.index
                    .map_tiles
                    .get(map.definition())
                    .and_then(|tiles| tiles.get(&memory.definition))
                    .and_then(|marker| art.index.markers.get(marker))
                    .or_else(|| art.index.tiles.get(&memory.definition))
            });
            if let Some(tile) = tile {
                let index = if definition.door && memory.door_open {
                    tile.open.unwrap_or(tile.closed)
                } else {
                    tile.closed
                };
                let mut sprite = art.map_or_else(Sprite::default, |art| art.sprite(index));
                sprite.color = tint(visible);
                commands.spawn((
                    Name::new(format!("Placeholder art tile {x},{y}")),
                    sprite,
                    Transform::from_translation((centre + tile.offset).extend(z)),
                    Placeholder,
                ));
                continue;
            }
            let appearance = if definition.door {
                TerrainAppearance::Door {
                    open: memory.door_open,
                }
            } else if definition.blocks_movement {
                TerrainAppearance::Wall
            } else {
                TerrainAppearance::Floor
            };
            // Full-size cells: the movement grid stays hidden (SPEC 14, B-005).
            commands.spawn((
                Name::new(format!("P1 {:?} tile {x},{y}", map.id())),
                Sprite::from_color(terrain_color(appearance, visible), Vec2::splat(TILE_SIZE)),
                Transform::from_xyz(centre.x, centre.y, z),
                Placeholder,
            ));
            if definition.door {
                commands.spawn((
                    Name::new("P1 door shape marker"),
                    Sprite::from_color(
                        if visible {
                            Color::WHITE
                        } else {
                            Color::srgb(0.22, 0.22, 0.22)
                        },
                        if memory.door_open {
                            Vec2::new(8.0, 44.0)
                        } else {
                            Vec2::new(44.0, 8.0)
                        },
                    ),
                    Transform::from_xyz(centre.x, centre.y, z + 0.0001),
                    Placeholder,
                ));
            }
        }
    }
}

fn terrain_color(appearance: TerrainAppearance, visible: bool) -> Color {
    let [r, g, b] = match appearance {
        TerrainAppearance::Door { open: true } => [0.6, 0.5, 0.25],
        TerrainAppearance::Door { open: false } => [0.7, 0.4, 0.15],
        TerrainAppearance::Wall => [0.48, 0.48, 0.55],
        TerrainAppearance::Floor => [0.22, 0.26, 0.29],
    };
    let brightness = if visible { 1.0 } else { 0.3 };
    Color::srgb(r * brightness, g * brightness, b * brightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_is_dimmer_than_visible_terrain() {
        let visible = terrain_color(TerrainAppearance::Floor, true).to_srgba();
        let memory = terrain_color(TerrainAppearance::Floor, false).to_srgba();
        assert!(memory.red < visible.red);
        assert!(memory.green < visible.green);
        assert!(memory.blue < visible.blue);
    }

    #[test]
    fn presentation_does_not_advance_time_and_renders_only_visible_actors()
    -> Result<(), prl_core::CoreError> {
        let state = CoreState::demo()?;
        let expected_actors = state.visible_actors().count();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            prl_core::PrlCorePlugin,
            crate::PrlRenderPlugin,
        ))
        .insert_resource(state);
        for _ in 0..60 {
            app.update();
        }
        let world = app.world_mut();
        let actors = world
            .query::<&Name>()
            .iter(world)
            .filter(|name| name.as_str().starts_with("P2 actor "))
            .count();
        assert_eq!(actors, expected_actors);
        assert_eq!(world.resource::<CoreState>().clock().player_actions(), 0);
        assert_eq!(world.resource::<CoreState>().clock().ticks(), 0);
        Ok(())
    }
}
