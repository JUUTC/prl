use std::collections::BTreeMap;

use bevy::{prelude::*, text::FontSize};
use prl_core::{ActorId, CoreSet, CoreState, OutcomeKind, Position};

use crate::actors::{ActorView, actor_depth, sync_actor_views};
use crate::camera::{TILE_SIZE, tile_to_world};
use crate::cursor::OutcomeCursor;
use crate::motion::{AnimationSpeed, EffectTimings};

/// Fraction of a tile an attacker lunges toward its target.
const LUNGE_DISTANCE: f32 = TILE_SIZE * 0.3;
/// Upward travel of a moving damage number over its lifetime.
const NUMBER_RISE: f32 = 28.0;

/// Render's outcome consumer state plus in-flight cosmetic effects.
///
/// Effects are keyed by logical actor ID and never feed back into core.
#[derive(Resource, Default)]
pub(crate) struct CombatEffects {
    cursor: OutcomeCursor,
    lunges: Vec<Lunge>,
    flashes: Vec<Flash>,
}

struct Lunge {
    actor: ActorId,
    toward: Vec2,
    elapsed: f32,
    duration: f32,
}

struct Flash {
    actor: ActorId,
    elapsed: f32,
    duration: f32,
}

#[derive(Component)]
pub(crate) struct FloatingNumber {
    elapsed: f32,
    duration: f32,
    moves: bool,
    origin: Vec2,
    color: Color,
}

/// Atlas sprites are drawn white-tinted, so a white flash would not show.
const ATLAS_HIT_TINT: LinearRgba = LinearRgba::rgb(1.0, 0.25, 0.2);

/// What a dying actor last looked like, for its fade-out.
struct LastSeen {
    position: Position,
    base: Vec2,
    color: Color,
    sprite: Sprite,
}

#[derive(Component)]
struct Ghost {
    elapsed: f32,
    duration: f32,
    color: Color,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CombatEffects>().add_systems(
        Update,
        (
            consume_outcomes
                .before(sync_actor_views)
                .run_if(resource_exists::<CoreState>)
                .run_if(resource_changed::<CoreState>),
            animate
                .after(sync_actor_views)
                .after(consume_outcomes)
                .run_if(effects_active),
        )
            .in_set(CoreSet::Presentation),
    );
}

/// Player-facing damage label shared by the floating number.
pub(crate) fn damage_label(damage: u16, critical: bool, glancing: bool) -> String {
    if critical {
        format!("CRIT -{damage}")
    } else if glancing {
        format!("glance -{damage}")
    } else {
        format!("-{damage}")
    }
}

fn effects_active(
    effects: Res<'_, CombatEffects>,
    transient: Query<'_, '_, (), Or<(With<FloatingNumber>, With<Ghost>)>>,
) -> bool {
    !effects.lunges.is_empty() || !effects.flashes.is_empty() || !transient.is_empty()
}

fn consume_outcomes(
    mut commands: Commands<'_, '_>,
    state: Res<'_, CoreState>,
    speed: Res<'_, AnimationSpeed>,
    mut effects: ResMut<'_, CombatEffects>,
    views: Query<'_, '_, (&ActorView, &Sprite)>,
    transient: Query<'_, '_, Entity, Or<(With<FloatingNumber>, With<Ghost>)>>,
) {
    let effects = &mut *effects;
    let fresh = effects.cursor.take_new(state.outcomes(), "render effects");
    if fresh.is_empty() {
        return;
    }
    // New committed input snaps in-flight motion instead of queueing behind it.
    effects.lunges.clear();
    effects.flashes.clear();
    let timings = speed.timings();
    let last_seen = last_seen(&views);
    let position_of = |id: ActorId| {
        state
            .actor(id)
            .map(prl_core::Actor::position)
            .or_else(|| last_seen.get(&id).map(|seen| seen.position))
    };
    let shown = |id: ActorId| {
        id == state.player_id()
            || position_of(id).is_some_and(|position| state.is_visible(position))
    };
    let mut stacked: BTreeMap<ActorId, u8> = BTreeMap::new();
    for outcome in fresh {
        match &outcome.kind {
            OutcomeKind::RunStarted { .. } => {
                effects.lunges.clear();
                effects.flashes.clear();
                stacked.clear();
                for entity in &transient {
                    commands.entity(entity).despawn();
                }
            }
            OutcomeKind::Attacked {
                attacker,
                target,
                damage,
                critical,
                glancing,
                ..
            } => {
                if !(shown(*attacker) || shown(*target)) {
                    continue;
                }
                let target_position = position_of(*target);
                if let (Some(from), Some(to)) = (position_of(*attacker), target_position)
                    && timings.lunge > 0.0
                {
                    let from = tile_to_world(from.x, from.y);
                    let to = tile_to_world(to.x, to.y);
                    effects.lunges.push(Lunge {
                        actor: *attacker,
                        toward: (to - from).normalize_or_zero() * LUNGE_DISTANCE,
                        elapsed: 0.0,
                        duration: timings.lunge,
                    });
                }
                if timings.flash > 0.0 {
                    effects.flashes.push(Flash {
                        actor: *target,
                        elapsed: 0.0,
                        duration: timings.flash,
                    });
                }
                let Some(at) = target_position else {
                    continue;
                };
                let stack = stacked.entry(*target).or_default();
                let origin =
                    tile_to_world(at.x, at.y) + Vec2::new(0.0, 22.0 + 18.0 * f32::from(*stack));
                *stack = stack.saturating_add(1);
                let color = if *target == state.player_id() {
                    Color::srgb(1.0, 0.35, 0.3)
                } else if *critical {
                    Color::srgb(1.0, 0.9, 0.3)
                } else {
                    Color::WHITE
                };
                spawn_damage_number(
                    &mut commands,
                    damage_label(*damage, *critical, *glancing),
                    origin,
                    color,
                    timings,
                );
            }
            OutcomeKind::ActorDied { actor, .. } => {
                if *actor == state.player_id() || timings.ghost <= 0.0 || !shown(*actor) {
                    continue;
                }
                let Some(seen) = last_seen.get(actor) else {
                    continue;
                };
                spawn_ghost(&mut commands, seen, timings.ghost);
            }
            _ => {}
        }
    }
}

fn last_seen(views: &Query<'_, '_, (&ActorView, &Sprite)>) -> BTreeMap<ActorId, LastSeen> {
    views
        .iter()
        .map(|(view, sprite)| {
            let seen = LastSeen {
                position: view.position,
                base: view.base,
                color: view.color,
                sprite: sprite.clone(),
            };
            (view.id, seen)
        })
        .collect()
}

fn spawn_damage_number(
    commands: &mut Commands<'_, '_>,
    label: String,
    origin: Vec2,
    color: Color,
    timings: EffectTimings,
) {
    commands.spawn((
        Name::new("P2 damage number"),
        Text2d::new(label),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(origin.extend(3.0)),
        FloatingNumber {
            elapsed: 0.0,
            duration: timings.number,
            moves: timings.moves,
            origin,
            color,
        },
    ));
}

fn spawn_ghost(commands: &mut Commands<'_, '_>, seen: &LastSeen, duration: f32) {
    let (sprite, at) = if seen.sprite.texture_atlas.is_some() {
        (seen.sprite.clone(), seen.base)
    } else {
        let centre = tile_to_world(seen.position.x, seen.position.y);
        (Sprite::from_color(seen.color, Vec2::splat(30.0)), centre)
    };
    commands.spawn((
        Name::new("P2 death fade"),
        sprite,
        Transform::from_translation(at.extend(actor_depth(seen.position.y) - 0.0001)),
        Ghost {
            elapsed: 0.0,
            duration,
            color: seen.color,
        },
    ));
}

fn animate(
    mut commands: Commands<'_, '_>,
    time: Res<'_, Time>,
    mut effects: ResMut<'_, CombatEffects>,
    mut views: Query<'_, '_, (&ActorView, &mut Transform, &mut Sprite)>,
    mut numbers: Query<
        '_,
        '_,
        (Entity, &mut FloatingNumber, &mut Transform, &mut TextColor),
        Without<ActorView>,
    >,
    mut ghosts: Query<'_, '_, (Entity, &mut Ghost, &mut Sprite), Without<ActorView>>,
) {
    let delta = time.delta_secs();
    let effects = &mut *effects;
    for lunge in &mut effects.lunges {
        lunge.elapsed += delta;
    }
    for flash in &mut effects.flashes {
        flash.elapsed += delta;
    }
    effects
        .lunges
        .retain(|lunge| lunge.elapsed < lunge.duration);
    effects
        .flashes
        .retain(|flash| flash.elapsed < flash.duration);
    for (view, mut transform, mut sprite) in &mut views {
        let offset: Vec2 = effects
            .lunges
            .iter()
            .filter(|lunge| lunge.actor == view.id)
            .map(|lunge| {
                lunge.toward
                    * (std::f32::consts::PI * progress(lunge.elapsed, lunge.duration)).sin()
            })
            .sum();
        transform.translation = (view.base + offset).extend(transform.translation.z);
        let flash = effects
            .flashes
            .iter()
            .filter(|flash| flash.actor == view.id)
            .map(|flash| 1.0 - progress(flash.elapsed, flash.duration))
            .fold(0.0_f32, f32::max);
        let flash_to = if sprite.texture_atlas.is_some() {
            ATLAS_HIT_TINT
        } else {
            LinearRgba::WHITE
        };
        sprite.color = Color::LinearRgba(view.color.to_linear().mix(&flash_to, flash * 0.8));
    }
    for (entity, mut number, mut transform, mut color) in &mut numbers {
        number.elapsed += delta;
        let t = progress(number.elapsed, number.duration);
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if number.moves {
            transform.translation.y = number.origin.y + NUMBER_RISE * t;
            color.0 = number.color.with_alpha(1.0 - t * t);
        }
    }
    for (entity, mut ghost, mut sprite) in &mut ghosts {
        ghost.elapsed += delta;
        let t = progress(ghost.elapsed, ghost.duration);
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        sprite.color = ghost.color.with_alpha(0.7 * (1.0 - t));
    }
}

fn progress(elapsed: f32, duration: f32) -> f32 {
    if duration <= 0.0 {
        1.0
    } else {
        (elapsed / duration).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use prl_core::{Command, CoreCommandQueue, Direction};
    use std::time::Duration;

    #[test]
    fn damage_labels_name_crits_and_glances() {
        assert_eq!(damage_label(6, false, false), "-6");
        assert_eq!(damage_label(11, true, false), "CRIT -11");
        assert_eq!(damage_label(2, false, true), "glance -2");
    }

    /// Opens the antechamber door, holds it, then charges and starts a new run.
    fn scripted_commands() -> Vec<Command> {
        let mut commands = vec![
            Command::Move(Direction::East),
            Command::Move(Direction::East),
            Command::OpenDoor(Direction::East),
        ];
        commands.extend(std::iter::repeat_n(Command::Wait, 6));
        commands.extend(std::iter::repeat_n(Command::Move(Direction::East), 30));
        commands.extend(std::iter::repeat_n(Command::Wait, 40));
        commands.push(Command::NewRun);
        commands.extend([
            Command::Move(Direction::East),
            Command::Move(Direction::East),
            Command::OpenDoor(Direction::East),
            Command::Move(Direction::East),
        ]);
        commands
    }

    fn run_with(
        speed: AnimationSpeed,
    ) -> Result<(prl_core::CoreSnapshot, usize), prl_core::CoreError> {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            prl_core::PrlCorePlugin,
            crate::PrlRenderPlugin,
        ))
        .insert_resource(CoreState::demo()?)
        .insert_resource(speed)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            40,
        )));
        app.update();
        let mut numbers_seen = 0;
        for command in scripted_commands() {
            app.world_mut()
                .resource_mut::<CoreCommandQueue>()
                .push(command);
            // Effects are mid-flight on the following frames; input never waits.
            for _ in 0..3 {
                app.update();
                let world = app.world_mut();
                numbers_seen += world.query::<&FloatingNumber>().iter(world).count();
            }
        }
        // Every cosmetic effect expires on its own without further input.
        for _ in 0..30 {
            app.update();
        }
        let world = app.world_mut();
        assert_eq!(world.query::<&FloatingNumber>().iter(world).count(), 0);
        let world = app.world();
        assert!(world.resource::<CoreCommandQueue>().is_empty());
        let newest = world
            .resource::<CoreState>()
            .outcomes()
            .last()
            .map_or(0, |outcome| outcome.sequence);
        assert_eq!(world.resource::<CombatEffects>().cursor.last_seen(), newest);
        Ok((world.resource::<CoreState>().snapshot(), numbers_seen))
    }

    #[test]
    fn animation_speed_never_changes_core_outcomes() -> Result<(), prl_core::CoreError> {
        let (normal, normal_numbers) = run_with(AnimationSpeed::Normal)?;
        let (fast, _) = run_with(AnimationSpeed::Fast)?;
        let (off, off_numbers) = run_with(AnimationSpeed::Off)?;
        assert!(normal_numbers > 0, "script must exercise combat feedback");
        assert!(off_numbers > 0, "motion off still shows damage numbers");
        let state = CoreState::restore(normal.clone())?;
        assert!(
            state
                .outcomes()
                .iter()
                .any(|outcome| matches!(outcome.kind, OutcomeKind::Attacked { .. })),
        );
        assert_eq!(normal, fast);
        assert_eq!(normal, off);
        Ok(())
    }
}
