use bevy::prelude::*;
use prl_core::{CoreSet, CoreState, Position};

use crate::camera::{TILE_SIZE, WorldCamera, tile_to_world};

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

fn refresh(
    mut commands: Commands<'_, '_>,
    state: Res<'_, CoreState>,
    placeholders: Query<'_, '_, Entity, With<Placeholder>>,
    mut cameras: Query<'_, '_, &mut Transform, With<WorldCamera>>,
) {
    for entity in &placeholders {
        commands.entity(entity).despawn();
    }
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
            let appearance = if definition.door {
                TerrainAppearance::Door {
                    open: memory.door_open,
                }
            } else if definition.blocks_movement {
                TerrainAppearance::Wall
            } else {
                TerrainAppearance::Floor
            };
            let color = terrain_color(appearance, visible);
            let centre = tile_to_world(x, y);
            commands.spawn((
                Name::new(format!("P1 {:?} tile {x},{y}", map.id())),
                Sprite::from_color(color, Vec2::splat(TILE_SIZE - 2.0)),
                Transform::from_xyz(centre.x, centre.y, 0.0),
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
                    Transform::from_xyz(centre.x, centre.y, 0.5),
                    Placeholder,
                ));
            }
        }
    }
    if let Some(player) = state.player() {
        let position = player.position();
        let centre = tile_to_world(position.x, position.y);
        for mut camera in &mut cameras {
            camera.translation.x = centre.x;
            camera.translation.y = centre.y;
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
