use bevy::{camera::ScalingMode, prelude::*, window::PrimaryWindow};

/// Logical tile edge in world units; placeholder shapes are not final assets.
pub const TILE_SIZE: f32 = 64.0;

/// Integer world magnification, deliberately independent of Bevy's UI scale.
#[derive(Resource, Debug)]
pub struct WorldZoom(u8);

impl Default for WorldZoom {
    fn default() -> Self {
        Self(1)
    }
}

impl WorldZoom {
    /// Current integer magnification.
    #[must_use]
    pub const fn get(&self) -> u8 {
        self.0
    }

    /// Set magnification, clamped to the supported placeholder range 1–4.
    pub fn set(&mut self, zoom: u8) {
        self.0 = zoom.clamp(1, 4);
    }
}

#[derive(Component)]
pub(super) struct WorldCamera;

/// Convert a logical cell centre to world coordinates (map north is screen up).
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "Pixel presentation may be lossy at extreme coordinates; core remains integer-only"
)]
pub fn tile_to_world(x: i32, y: i32) -> Vec2 {
    Vec2::new(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE)
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<WorldZoom>()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.025, 0.03)))
        .add_systems(Startup, spawn_camera)
        .add_systems(
            Update,
            zoom_keys.run_if(resource_exists::<ButtonInput<KeyCode>>),
        )
        .add_systems(
            Update,
            apply_zoom
                .after(zoom_keys)
                .run_if(resource_changed::<WorldZoom>),
        );
}

fn spawn_camera(mut commands: Commands<'_, '_>) {
    commands.spawn((
        Name::new("P1 placeholder world camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize,
            ..OrthographicProjection::default_2d()
        }),
        WorldCamera,
    ));
}

fn zoom_keys(
    keys: Res<'_, ButtonInput<KeyCode>>,
    windows: Query<'_, '_, &Window, With<PrimaryWindow>>,
    mut zoom: ResMut<'_, WorldZoom>,
) {
    if !windows.iter().any(|window| window.focused) {
        return;
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        let value = zoom.get().saturating_add(1);
        zoom.set(value);
    } else if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        let value = zoom.get().saturating_sub(1);
        zoom.set(value);
    }
}

fn apply_zoom(
    zoom: Res<'_, WorldZoom>,
    mut cameras: Query<'_, '_, &mut Projection, With<WorldCamera>>,
) {
    for mut projection in &mut cameras {
        if let Projection::Orthographic(orthographic) = &mut *projection {
            orthographic.scale = 1.0 / f32::from(zoom.get());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn north_is_up_and_tiles_are_sixty_four_units() {
        assert_eq!(tile_to_world(0, 0), Vec2::ZERO);
        assert_eq!(tile_to_world(2, 3), Vec2::new(128.0, -192.0));
        assert_eq!(tile_to_world(-1, -1), Vec2::new(-64.0, 64.0));
    }

    #[test]
    fn zoom_is_an_integer_in_supported_range() {
        let mut zoom = WorldZoom::default();
        zoom.set(0);
        assert_eq!(zoom.get(), 1);
        zoom.set(255);
        assert_eq!(zoom.get(), 4);
    }

    #[test]
    fn zoom_changes_camera_projection_without_input_or_rules() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, plugin));
        app.world_mut().resource_mut::<WorldZoom>().set(4);
        app.update();
        let world = app.world_mut();
        assert!(world.query::<&Projection>().iter(world).any(
            |projection| matches!(projection, Projection::Orthographic(orthographic)
                if (orthographic.scale - 0.25).abs() < f32::EPSILON)
        ));
    }
}
