use std::collections::BTreeMap;

use bevy::prelude::*;
use prl_core::{ACTION_ENERGY, Actor, ActorBehavior, ActorId, CoreSet, CoreState, Position};

use crate::camera::tile_to_world;

/// Persistent placeholder sprite for one visible actor, keyed by logical ID.
#[derive(Component, Debug)]
pub(crate) struct ActorView {
    pub(crate) id: ActorId,
    pub(crate) position: Position,
    pub(crate) base: Vec2,
    pub(crate) color: Color,
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
        sync_actor_views
            .in_set(CoreSet::Presentation)
            .run_if(resource_exists::<CoreState>)
            .run_if(resource_changed::<CoreState>),
    );
}

/// Keeps one view per visible actor; positions come only from core.
pub(crate) fn sync_actor_views(
    mut commands: Commands<'_, '_>,
    state: Res<'_, CoreState>,
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
    for actor in state.visible_actors() {
        let position = actor.position();
        let base = tile_to_world(position.x, position.y);
        let look = appearance(actor);
        let id = actor.id();
        let updated = existing
            .remove(&id)
            .and_then(|entity| views.get_mut(entity).ok())
            .map(|(_, mut view, mut transform, mut sprite)| {
                *view = ActorView {
                    id,
                    position,
                    base,
                    color: look.color,
                };
                transform.translation = base.extend(1.0);
                transform.rotation = Quat::from_rotation_z(look.rotation);
                sprite.color = look.color;
                sprite.custom_size = Some(look.size);
            });
        if updated.is_none() {
            commands.spawn((
                Name::new(format!("P2 actor {id:?}")),
                Sprite::from_color(look.color, look.size),
                Transform::from_translation(base.extend(1.0))
                    .with_rotation(Quat::from_rotation_z(look.rotation)),
                ActorView {
                    id,
                    position,
                    base,
                    color: look.color,
                },
            ));
        }
        if let Some(fraction) = damaged_fraction(actor) {
            spawn_hp_bar(&mut commands, base, fraction);
        }
    }
    for entity in existing.into_values() {
        commands.entity(entity).despawn();
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
}
