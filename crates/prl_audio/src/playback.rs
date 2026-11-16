use std::collections::BTreeSet;

use bevy::{
    audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume},
    prelude::*,
};
use prl_core::{ActorId, Command, CoreOutcome, CoreSet, CoreState, OutcomeKind};

use crate::cursor::OutcomeCursor;
use crate::settings::AudioSettings;
use crate::synth::Sfx;

/// At most this many cues start per frame so bursts do not stack into noise.
const MAX_CUES_PER_FRAME: usize = 4;
/// Voices are removed after this long even if no device ever played them.
const VOICE_LIFETIME: f32 = 3.0;

/// Audio's own outcome consumer.
#[derive(Resource, Default)]
struct AudioConsumer {
    cursor: OutcomeCursor,
}

/// Generated placeholder sounds, present only when Bevy audio is available.
#[derive(Resource)]
struct SfxHandles(Vec<(Sfx, Handle<AudioSource>)>);

#[derive(Component)]
struct SfxVoice {
    age: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AudioConsumer>()
        .add_systems(Startup, generate_sounds)
        .add_systems(
            Update,
            (
                play_outcomes
                    .in_set(CoreSet::Presentation)
                    .run_if(resource_exists::<CoreState>)
                    .run_if(resource_changed::<CoreState>),
                expire_voices.run_if(any_with_component::<SfxVoice>),
            ),
        );
}

fn generate_sounds(
    mut commands: Commands<'_, '_>,
    sources: Option<ResMut<'_, Assets<AudioSource>>>,
) {
    let Some(mut sources) = sources else {
        info!("Bevy audio is not enabled; placeholder sound effects are disabled.");
        return;
    };
    let handles = Sfx::ALL
        .into_iter()
        .map(|sfx| {
            let source = AudioSource {
                bytes: sfx.wav().into(),
            };
            (sfx, sources.add(source))
        })
        .collect();
    commands.insert_resource(SfxHandles(handles));
}

/// Cues for newly committed outcomes the player can perceive, in order.
fn cues(state: &CoreState, outcomes: &[CoreOutcome]) -> Vec<Sfx> {
    let player = state.player_id();
    let perceivable = |id: ActorId| {
        id == player
            || state
                .actor(id)
                .is_some_and(|actor| state.is_visible(actor.position()))
    };
    // Slain non-players leave core, so remember which targets were heard being hit.
    let mut heard_targets = BTreeSet::new();
    let mut cues = Vec::new();
    for outcome in outcomes {
        let cue = match &outcome.kind {
            OutcomeKind::Attacked {
                attacker,
                target,
                critical,
                glancing,
                ..
            } if perceivable(*attacker) || perceivable(*target) => {
                heard_targets.insert(*target);
                Some(if *critical {
                    Sfx::Crit
                } else if *glancing {
                    Sfx::Glance
                } else {
                    Sfx::Hit
                })
            }
            OutcomeKind::ActorDied { actor, .. }
                if *actor == player || heard_targets.contains(actor) =>
            {
                Some(Sfx::Death)
            }
            OutcomeKind::CommandAccepted {
                command: Command::OpenDoor(_),
                ..
            }
            | OutcomeKind::RunStarted { .. } => Some(Sfx::Ui),
            _ => None,
        };
        cues.extend(cue);
    }
    cues
}

fn play_outcomes(
    mut commands: Commands<'_, '_>,
    state: Res<'_, CoreState>,
    settings: Res<'_, AudioSettings>,
    handles: Option<Res<'_, SfxHandles>>,
    mut consumer: ResMut<'_, AudioConsumer>,
) {
    let fresh = consumer.cursor.take_new(state.outcomes(), "audio");
    if fresh.is_empty() {
        return;
    }
    let gain = settings.gain();
    let Some(handles) = handles.filter(|_| gain > 0.0) else {
        return;
    };
    for cue in cues(&state, fresh).into_iter().take(MAX_CUES_PER_FRAME) {
        let Some((_, handle)) = handles.0.iter().find(|(sfx, _)| *sfx == cue) else {
            continue;
        };
        commands.spawn((
            Name::new(format!("P2 placeholder sfx {cue:?}")),
            AudioPlayer(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
            SfxVoice { age: 0.0 },
        ));
    }
}

fn expire_voices(
    mut commands: Commands<'_, '_>,
    time: Res<'_, Time>,
    mut voices: Query<'_, '_, (Entity, &mut SfxVoice)>,
) {
    for (entity, mut voice) in &mut voices {
        voice.age += time.delta_secs();
        if voice.age > VOICE_LIFETIME {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prl_core::Direction;

    #[test]
    fn combat_outcomes_map_to_hit_crit_glance_and_death_cues() -> Result<(), prl_core::CoreError> {
        let mut state = CoreState::demo()?;
        for command in [
            Command::Move(Direction::East),
            Command::Move(Direction::East),
            Command::OpenDoor(Direction::East),
        ] {
            state.issue(command);
        }
        for _ in 0..2000 {
            if state.is_player_dead() {
                break;
            }
            state.issue(Command::Wait);
        }
        let all = cues(&state, state.outcomes());
        assert!(
            all.contains(&Sfx::Death),
            "the player's death is always heard"
        );
        assert!(
            all.iter()
                .any(|cue| matches!(cue, Sfx::Hit | Sfx::Crit | Sfx::Glance))
        );
        let attacks = state
            .outcomes()
            .iter()
            .filter(|outcome| {
                matches!(outcome.kind, OutcomeKind::Attacked { target, .. } if target == state.player_id())
            })
            .count();
        let combat_cues = all
            .iter()
            .filter(|cue| matches!(cue, Sfx::Hit | Sfx::Crit | Sfx::Glance))
            .count();
        assert!(
            combat_cues >= attacks,
            "attacks on the player are always heard"
        );
        Ok(())
    }

    #[test]
    fn generated_wavs_decode_with_bevy_audio_wav_support() {
        use bevy::audio::{Decodable, Source};
        for sfx in Sfx::ALL {
            let bytes = sfx.wav();
            let expected = (bytes.len() - 44) / 2;
            let decoder = AudioSource {
                bytes: bytes.into(),
            }
            .decoder();
            assert_eq!(decoder.channels().get(), 1);
            assert_eq!(decoder.sample_rate().get(), crate::SAMPLE_RATE);
            assert_eq!(decoder.count(), expected, "{sfx:?}");
        }
    }

    #[test]
    fn headless_app_without_audio_device_consumes_outcomes_without_playing()
    -> Result<(), prl_core::CoreError> {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            prl_core::PrlCorePlugin,
            crate::PrlAudioPlugin,
        ))
        .insert_resource(CoreState::demo()?);
        app.update();
        app.world_mut()
            .resource_mut::<prl_core::CoreCommandQueue>()
            .push(Command::NewRun);
        app.update();
        let newest = app
            .world()
            .resource::<CoreState>()
            .outcomes()
            .last()
            .map_or(0, |outcome| outcome.sequence);
        assert_eq!(
            app.world().resource::<AudioConsumer>().cursor.last_seen(),
            newest
        );
        let world = app.world_mut();
        assert_eq!(world.query::<&SfxVoice>().iter(world).count(), 0);
        Ok(())
    }
}
