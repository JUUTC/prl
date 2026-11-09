use std::collections::BTreeMap;

use bevy::prelude::*;
use prl_core::{ACTION_ENERGY, Actor, ActorBehavior, ActorId, CoreSet, CoreState, Position};

use crate::art::{Action, PlaceholderArt};
use crate::camera::{TILE_SIZE, tile_to_world};
use crate::facing::{ActorMotion, Motion};
use crate::motion::AnimationSpeed;

/// Persistent placeholder sprite for one visible actor, keyed by logical ID.
#[derive(Component, Debug)]
pub(crate) struct ActorView {
    pub(crate) id: ActorId,
    pub(crate) position: Position,
    pub(crate) base: Vec2,
    pub(crate) color: Color,
}

/// Draw order for raised things on row `y`: southern rows cover northern ones,
/// so a half-height wall hides only the lower part of an actor north of it.
pub(crate) fn depth(y: i32) -> f32 {
    1.0 - tile_to_world(0, y).y / TILE_SIZE * 0.001
}

/// Actors draw just above raised terrain on their own row.
pub(crate) fn actor_depth(y: i32) -> f32 {
    depth(y) + 0.0005
}

#[derive(Component)]
pub(crate) struct HpBar;

/// Scheduler speed relative to one action per tick; shared shape/threat hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpeedClass {
    Slow,
    Normal,
    Fast,
}

impl SpeedClass {
    pub(crate) fn of(actor: &Actor) -> Self {
        match actor.speed().cmp(&ACTION_ENERGY) {
            std::cmp::Ordering::Less => Self::Slow,
            std::cmp::Ordering::Equal => Self::Normal,
            std::cmp::Ordering::Greater => Self::Fast,
        }
    }
}

/// Disposable procedural placeholder look; not an art direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Appearance {
    pub(crate) color: Color,
    pub(crate) size: Vec2,
    /// Rotation in radians (a rotated square reads as a diamond).
    pub(crate) rotation: f32,
}

/// Chooses a shape from behaviour and speed class, never from content IDs.
pub(crate) fn appearance(actor: &Actor) -> Appearance {
    let shape = |color, size, rotation| Appearance {
        color,
        size,
        rotation,
    };
    match actor.behavior() {
        ActorBehavior::Player if actor.is_alive() => {
            shape(Color::srgb(0.2, 0.85, 1.0), Vec2::splat(34.0), 0.0)
        }
        ActorBehavior::Player => shape(Color::srgb(0.32, 0.36, 0.38), Vec2::splat(34.0), 0.0),
        ActorBehavior::Passive => shape(Color::srgb(0.7, 0.7, 0.7), Vec2::splat(26.0), 0.0),
        ActorBehavior::Hostile => match SpeedClass::of(actor) {
            SpeedClass::Slow => shape(Color::srgb(0.55, 0.75, 0.3), Vec2::new(46.0, 26.0), 0.0),
            SpeedClass::Normal => shape(Color::srgb(1.0, 0.55, 0.25), Vec2::new(24.0, 42.0), 0.0),
            SpeedClass::Fast => shape(
                Color::srgb(0.95, 0.35, 0.8),
                Vec2::splat(22.0),
                std::f32::consts::FRAC_PI_4,
            ),
        },
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_actor_views
                .run_if(resource_exists::<CoreState>)
                .run_if(resource_changed::<CoreState>),
            animate_frames
                .run_if(resource_exists::<CoreState>)
                .run_if(resource_exists::<PlaceholderArt>),
        )
            .chain()
            .in_set(CoreSet::Presentation),
    );
}

/// Atlas frame for an actor's current cosmetic motion; `None` when the
/// placeholder atlas has no entry for its definition (shape fallback).
fn atlas_frame(
    art: &PlaceholderArt,
    actor: &Actor,
    motion: Motion,
    now: std::time::Duration,
    speed: AnimationSpeed,
) -> Option<(usize, f32)> {
    let sprites = art.index.actors.get(actor.definition())?;
    // Motion Off holds the first idle frame of the current facing.
    let (action, since, now_ms) = if speed == AnimationSpeed::Off {
        (Action::Idle, 0, 0)
    } else {
        (
            motion.action,
            now.saturating_sub(motion.since).as_millis(),
            now.as_millis(),
        )
    };
    Some((
        sprites.frame(motion.facing, action, since, now_ms),
        sprites.offset_y,
    ))
}

/// The player's dead tint for the atlas; living actors keep their atlas colours.
fn atlas_tint(actor: &Actor) -> Color {
    if actor.is_player() && !actor.is_alive() {
        Color::srgb(0.4, 0.42, 0.44)
    } else {
        Color::WHITE
    }
}

/// Keeps one view per visible actor; positions come only from core.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_actor_views(
    mut commands: Commands<'_, '_>,
    state: Res<'_, CoreState>,
    art: Option<Res<'_, PlaceholderArt>>,
    motion: Option<Res<'_, ActorMotion>>,
    speed: Option<Res<'_, AnimationSpeed>>,
    time: Res<'_, Time>,
    mut views: Query<'_, '_, (Entity, &mut ActorView, &mut Transform, &mut Sprite)>,
    bars: Query<'_, '_, Entity, With<HpBar>>,
) {
    for entity in &bars {
        commands.entity(entity).despawn();
    }
    let mut existing: BTreeMap<ActorId, Entity> = BTreeMap::new();
    for (entity, view, ..) in &views {
        if existing.insert(view.id, entity).is_some() {
            commands.entity(entity).despawn();
        }
    }
    let now = time.elapsed();
    let speed = speed.as_deref().copied().unwrap_or_default();
    for actor in state.visible_actors() {
        let position = actor.position();
        let centre = tile_to_world(position.x, position.y);
        let id = actor.id();
        let current = motion
            .as_deref()
            .map(|motion| motion.get(id))
            .unwrap_or_default();
        let atlas = art.as_deref().and_then(|art| {
            atlas_frame(art, actor, current, now, speed)
                .map(|(index, offset_y)| (art.sprite(index), offset_y))
        });
        let (new_sprite, base, color, rotation) = if let Some((mut sprite, offset_y)) = atlas {
            let color = atlas_tint(actor);
            sprite.color = color;
            (sprite, centre + Vec2::new(0.0, offset_y), color, 0.0)
        } else {
            let look = appearance(actor);
            (
                Sprite::from_color(look.color, look.size),
                centre,
                look.color,
                look.rotation,
            )
        };
        let view = ActorView {
            id,
            position,
            base,
            color,
        };
        let transform = Transform::from_translation(base.extend(actor_depth(position.y)))
            .with_rotation(Quat::from_rotation_z(rotation));
        match existing
            .remove(&id)
            .and_then(|entity| views.get_mut(entity).ok())
        {
            Some((_, mut old_view, mut old_transform, mut old_sprite)) => {
                *old_view = view;
                *old_transform = transform;
                *old_sprite = new_sprite;
            }
            None => {
                commands.spawn((
                    Name::new(format!("P2 actor {id:?}")),
                    new_sprite,
                    transform,
                    view,
                ));
            }
        }
        if let Some(fraction) = damaged_fraction(actor) {
            spawn_hp_bar(&mut commands, centre, fraction);
        }
    }
    for entity in existing.into_values() {
        commands.entity(entity).despawn();
    }
}

/// Advances cosmetic atlas frames from wall-clock time; never touches core.
fn animate_frames(
    state: Res<'_, CoreState>,
    art: Res<'_, PlaceholderArt>,
    motion: Res<'_, ActorMotion>,
    speed: Res<'_, AnimationSpeed>,
    time: Res<'_, Time>,
    mut views: Query<'_, '_, (&ActorView, &mut Sprite)>,
) {
    let now = time.elapsed();
    for (view, mut sprite) in &mut views {
        let Some((index, _)) = state
            .actor(view.id)
            .and_then(|actor| atlas_frame(&art, actor, motion.get(view.id), now, *speed))
        else {
            continue;
        };
        if sprite
            .texture_atlas
            .as_ref()
            .is_some_and(|atlas| atlas.index != index)
            && let Some(atlas) = sprite.texture_atlas.as_mut()
        {
            atlas.index = index;
        }
    }
}

/// Remaining HP fraction for a damaged actor; `None` when unhurt.
pub(crate) fn damaged_fraction(actor: &Actor) -> Option<f32> {
    let max = actor.max_hp();
    (max > 0 && actor.hp() < max).then(|| f32::from(actor.hp()) / f32::from(max))
}

fn spawn_hp_bar(commands: &mut Commands<'_, '_>, base: Vec2, fraction: f32) {
    const WIDTH: f32 = 48.0;
    let y = base.y - 27.0;
    commands.spawn((
        Name::new("P2 HP bar background"),
        Sprite::from_color(Color::srgb(0.1, 0.02, 0.02), Vec2::new(WIDTH + 2.0, 7.0)),
        Transform::from_xyz(base.x, y, 1.5),
        HpBar,
    ));
    let fill = WIDTH * fraction.clamp(0.0, 1.0);
    commands.spawn((
        Name::new("P2 HP bar fill"),
        Sprite::from_color(Color::srgb(0.85, 0.15, 0.15), Vec2::new(fill, 5.0)),
        // Left-align the fill inside the background.
        Transform::from_xyz(base.x - (WIDTH - fill) / 2.0, y, 1.6),
        HpBar,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostiles_get_distinct_shapes_by_speed_class_and_player_stays_cyan()
    -> Result<(), prl_core::CoreError> {
        let state = CoreState::demo()?;
        let player = state.player().map(appearance);
        assert_eq!(
            player.map(|look| look.color),
            Some(Color::srgb(0.2, 0.85, 1.0))
        );
        let mut looks: Vec<(SpeedClass, Appearance)> = state
            .actors()
            .filter(|actor| actor.is_hostile())
            .map(|actor| (SpeedClass::of(actor), appearance(actor)))
            .collect();
        looks.sort_by_key(|(class, _)| *class);
        looks.dedup_by_key(|(class, _)| *class);
        assert_eq!(looks.len(), 3, "arena has slow, normal and fast hostiles");
        for (index, (_, left)) in looks.iter().enumerate() {
            for (_, right) in looks.iter().skip(index + 1) {
                assert_ne!(left.color, right.color);
                assert!(left.size != right.size || (left.rotation - right.rotation).abs() > 0.1);
            }
        }
        Ok(())
    }

    #[test]
    fn only_damaged_actors_report_an_hp_fraction() -> Result<(), prl_core::CoreError> {
        let state = CoreState::demo()?;
        assert!(
            state
                .actors()
                .all(|actor| damaged_fraction(actor).is_none())
        );
        Ok(())
    }

    #[test]
    fn placeholder_atlas_animates_visible_actors_without_touching_rules()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::art::{Facing, PlaceholderArtSource};
        use bevy::asset::AssetPlugin;
        use prl_core::{Command, CommandResult, Direction};

        let pack =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/prl");
        let state = CoreState::load_demo(pack.join("base.ron"))?;
        let expected_actors = state.visible_actors().count();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            prl_core::PrlCorePlugin,
            crate::PrlRenderPlugin,
        ))
        .init_asset::<Image>()
        .init_asset::<TextureAtlasLayout>()
        .insert_resource(PlaceholderArtSource(pack))
        .insert_resource(state);
        for _ in 0..10 {
            app.update();
        }
        let world = app.world_mut();
        assert!(world.contains_resource::<PlaceholderArt>());
        let atlas_actors = world
            .query::<(&ActorView, &Sprite)>()
            .iter(world)
            .filter(|(_, sprite)| sprite.texture_atlas.is_some())
            .count();
        assert_eq!(atlas_actors, expected_actors);
        assert_eq!(world.resource::<CoreState>().clock().ticks(), 0);

        let player = world.resource::<CoreState>().player_id();
        let stepped = [Direction::West, Direction::NorthWest, Direction::SouthWest]
            .into_iter()
            .any(|direction| {
                matches!(
                    world
                        .resource_mut::<CoreState>()
                        .issue(Command::Move(direction)),
                    CommandResult::Accepted { .. }
                )
            });
        assert!(stepped, "fixture should allow a westward step or attack");
        let ticks = world.resource::<CoreState>().clock().ticks();
        app.update();
        let world = app.world_mut();
        let motion = world.resource::<ActorMotion>().get(player);
        assert_eq!(motion.facing, Facing::Left);
        assert_ne!(motion.action, Action::Idle);
        // Presentation never advances the simulation on its own.
        assert_eq!(world.resource::<CoreState>().clock().ticks(), ticks);
        Ok(())
    }

    #[test]
    fn southern_rows_draw_over_northern_rows() {
        assert!(depth(5) > depth(4));
        assert!(actor_depth(4) > depth(4));
        assert!(actor_depth(4) < depth(5));
    }
}
