use bevy::{prelude::*, window::PrimaryWindow};

/// Presentation-only motion setting (Q-007 provisional).
///
/// It changes how long cosmetic combat effects last and whether they move. It
/// never changes ticks, RNG, damage, visibility or command outcomes, and effects
/// never delay input or the scheduler.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AnimationSpeed {
    /// Default timings (about 150 ms lunge/flash, 600 ms damage numbers).
    #[default]
    Normal,
    /// About 40% of the normal timings.
    Fast,
    /// No motion: no lunge or flash; damage numbers show briefly without moving.
    Off,
}

impl AnimationSpeed {
    /// Next setting in the `T` toggle cycle (Normal, Fast, Off).
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Normal => Self::Fast,
            Self::Fast => Self::Off,
            Self::Off => Self::Normal,
        }
    }

    /// Short player-facing label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Fast => "Fast",
            Self::Off => "Off",
        }
    }

    /// Effect durations in seconds for this setting.
    pub(crate) fn timings(self) -> EffectTimings {
        const NORMAL: EffectTimings = EffectTimings {
            lunge: 0.15,
            flash: 0.15,
            number: 0.6,
            ghost: 0.4,
            moves: true,
        };
        match self {
            Self::Normal => NORMAL,
            Self::Fast => EffectTimings {
                lunge: NORMAL.lunge * 0.4,
                flash: NORMAL.flash * 0.4,
                number: NORMAL.number * 0.4,
                ghost: NORMAL.ghost * 0.4,
                moves: true,
            },
            Self::Off => EffectTimings {
                lunge: 0.0,
                flash: 0.0,
                number: 0.8,
                ghost: 0.0,
                moves: false,
            },
        }
    }
}

/// Cosmetic effect durations in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EffectTimings {
    pub(crate) lunge: f32,
    pub(crate) flash: f32,
    pub(crate) number: f32,
    pub(crate) ghost: f32,
    /// Whether damage numbers rise and fade (false keeps them static).
    pub(crate) moves: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AnimationSpeed>().add_systems(
        Update,
        motion_key.run_if(resource_exists::<ButtonInput<KeyCode>>),
    );
}

fn motion_key(
    keys: Res<'_, ButtonInput<KeyCode>>,
    windows: Query<'_, '_, &Window, With<PrimaryWindow>>,
    mut speed: ResMut<'_, AnimationSpeed>,
) {
    if keys.just_pressed(KeyCode::KeyT) && windows.iter().any(|window| window.focused) {
        *speed = speed.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_cycles_normal_fast_off() {
        let speed = AnimationSpeed::default();
        assert_eq!(speed, AnimationSpeed::Normal);
        assert_eq!(speed.next(), AnimationSpeed::Fast);
        assert_eq!(speed.next().next(), AnimationSpeed::Off);
        assert_eq!(speed.next().next().next(), AnimationSpeed::Normal);
    }

    #[test]
    fn fast_is_shorter_than_normal_and_off_has_no_motion() {
        let normal = AnimationSpeed::Normal.timings();
        let fast = AnimationSpeed::Fast.timings();
        let off = AnimationSpeed::Off.timings();
        assert!((normal.lunge - 0.15).abs() < f32::EPSILON);
        assert!((normal.number - 0.6).abs() < f32::EPSILON);
        assert!((fast.lunge - 0.06).abs() < 1e-6);
        assert!((fast.number - 0.24).abs() < 1e-6);
        assert!(off.lunge <= 0.0 && off.flash <= 0.0 && !off.moves);
        assert!(off.number > 0.0);
    }
}
