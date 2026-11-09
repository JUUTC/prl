use std::collections::VecDeque;

use bevy::prelude::*;
use prl_core::{ACTION_ENERGY, Actor, CoreSet, CoreState, OutcomeKind, RunStatus};

use crate::cursor::OutcomeCursor;
use crate::input::{CONFIRM_SECONDS, Overlay};
use crate::{PlaceholderTheme, SettingsStatus, UiMode};

/// Newest core messages kept visible; short so the log never hides the player.
const LOG_LINES: usize = 5;

const CONTROLS: &str = "Move / attack: arrows, numpad, vi (y k u h l b j n); bump an enemy to attack\nWait: . / numpad 5 / Space | Open door: O, then a direction\nNew run: R twice (or R then Y) while alive; R once after death\nMotion: T (Normal / Fast / Off) | Sound: M mute, 9 / 0 volume\nWorld zoom: + / - | UI scale: [ / ] | Pause/menu: Esc | Help: F1 / ?";

const LEGEND: &str = "Legend (placeholder shapes, not final art)\nCyan square: you (grey when dead)\nGreen wide block: slow enemy | Orange tall block: normal-speed enemy\nPink small diamond: fast enemy | Red bar under an actor: HP when hurt\nGrey: wall | Brown + bar: door | Dim: remembered terrain\nFloating numbers: damage dealt (CRIT = critical, glance = glancing blow)\nThe log explains each hit: raw damage and how much armor removed.";

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct SightText;

#[derive(Component)]
struct LogText;

#[derive(Component)]
struct OverlayText;

#[derive(Component)]
struct DeathBanner;

#[derive(Component)]
struct DeathText;

/// The UI log's own outcome consumer and its short message buffer.
#[derive(Resource, Default)]
struct MessageLog {
    cursor: OutcomeCursor,
    lines: VecDeque<String>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<MessageLog>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                consume_messages
                    .run_if(resource_exists::<CoreState>)
                    .run_if(resource_changed::<CoreState>),
                refresh
                    .after(consume_messages)
                    .run_if(resource_exists::<UiMode>)
                    .run_if(resource_exists::<SettingsStatus>),
            )
                .in_set(CoreSet::Presentation),
        );
}

fn spawn_panel(
    commands: &mut Commands<'_, '_>,
    theme: &PlaceholderTheme,
    name: &'static str,
    node: Node,
    initial: &str,
    marker: impl Component,
) {
    commands
        .spawn((
            Name::new(name),
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(px(8)),
                ..node
            },
            // Translucent so the map stays readable behind panels (B-003).
            BackgroundColor(theme.panel.with_alpha(0.72)),
        ))
        .with_child((
            Text::new(initial),
            theme.font(),
            TextColor(theme.text),
            marker,
        ));
}

fn setup(mut commands: Commands<'_, '_>, theme: Res<'_, PlaceholderTheme>) {
    spawn_panel(
        &mut commands,
        &theme,
        "P2 placeholder HUD",
        Node {
            top: px(8),
            left: px(8),
            max_width: percent(62),
            ..default()
        },
        "P2 PLACEHOLDER - awaiting core state",
        HudText,
    );
    spawn_panel(
        &mut commands,
        &theme,
        "P2 placeholder in-sight list",
        Node {
            top: px(8),
            right: px(8),
            max_width: percent(34),
            ..default()
        },
        "In sight: -",
        SightText,
    );
    spawn_panel(
        &mut commands,
        &theme,
        "P2 placeholder message log",
        Node {
            bottom: px(8),
            left: px(8),
            max_width: percent(70),
            ..default()
        },
        "No committed actions.",
        LogText,
    );
    commands
        .spawn((
            Name::new("P2 placeholder death banner"),
            Node {
                position_type: PositionType::Absolute,
                top: percent(18),
                width: percent(100),
                justify_content: JustifyContent::Center,
                display: Display::None,
                ..default()
            },
            ZIndex(5),
            DeathBanner,
        ))
        .with_child((
            Text::default(),
            theme.font(),
            TextColor(theme.text),
            TextLayout::justify(Justify::Center),
            BackgroundColor(theme.panel),
            Node {
                padding: UiRect::all(px(20)),
                ..default()
            },
            DeathText,
        ));
    commands.spawn((
        Name::new("P2 placeholder pause/help overlay"),
        Text::default(),
        theme.font(),
        TextColor(theme.text),
        BackgroundColor(theme.panel),
        Node {
            position_type: PositionType::Absolute,
            top: percent(10),
            left: percent(10),
            max_width: percent(80),
            padding: UiRect::all(px(24)),
            display: Display::None,
            ..default()
        },
        ZIndex(10),
        OverlayText,
    ));
}

/// Consumes new outcomes in order and keeps their correlated message text.
fn consume_messages(state: Res<'_, CoreState>, mut log: ResMut<'_, MessageLog>) {
    let log = &mut *log;
    let messages = state.messages();
    for outcome in log.cursor.take_new(state.outcomes(), "message log") {
        if matches!(outcome.kind, OutcomeKind::RunStarted { .. }) {
            log.lines.clear();
        }
        if let Ok(index) = messages.binary_search_by_key(&outcome.sequence, |m| m.sequence)
            && let Some(message) = messages.get(index)
        {
            log.lines.push_back(message.text.clone());
            if log.lines.len() > LOG_LINES {
                log.lines.pop_front();
            }
        }
    }
}

/// Threat hint from scheduler speed relative to one action per tick.
fn speed_hint(actor: &Actor) -> &'static str {
    match actor.speed().cmp(&ACTION_ENERGY) {
        std::cmp::Ordering::Less => "slow",
        std::cmp::Ordering::Equal => "normal",
        std::cmp::Ordering::Greater => "fast",
    }
}

fn header_text(core: &CoreState, mode: &UiMode, settings: &SettingsStatus) -> String {
    let vitals = core.player().map_or_else(
        || "Player unavailable".to_owned(),
        |actor| format!("HP {}/{}", actor.hp(), actor.max_hp()),
    );
    let prompt = if mode.confirm_new_run.is_some() {
        format!(
            "\nAbandon this run? Press R again (or Y) within {CONFIRM_SECONDS:.0} s; any other key cancels."
        )
    } else if mode.door {
        "\nOpen door: choose direction (Esc cancels)".to_owned()
    } else {
        String::new()
    };
    format!(
        "P2 PLACEHOLDER - not final art/font/audio | F1 help\nRun {} | Seed {} | {vitals}\nTick {} | Clock {}s | Actions {} | Motion {} (T) | Sound {} (M, 9/0){prompt}",
        core.run_number(),
        core.seed(),
        core.clock().ticks(),
        core.clock().elapsed_seconds(),
        core.clock().player_actions(),
        settings.motion,
        settings.sound,
    )
}

fn sight_text(core: &CoreState) -> String {
    let lines: Vec<String> = core
        .visible_actors()
        .filter(|actor| actor.is_hostile())
        .map(|actor| {
            format!(
                "{} HP {}/{} ({} speed)",
                actor.name(),
                actor.hp(),
                actor.max_hp(),
                speed_hint(actor)
            )
        })
        .collect();
    if lines.is_empty() {
        "In sight: no enemies".to_owned()
    } else {
        format!("In sight\n{}", lines.join("\n"))
    }
}

fn overlay_text(mode: &UiMode) -> String {
    match mode.overlay {
        Overlay::Help => format!(
            "HELP - gameplay paused\n\n{CONTROLS}\n\n{LEGEND}\n\nOne key press, one intent; held keys do not repeat.\nEffects never delay input. Inventory, saves and final assets are unavailable.\n\nF1 / ? / Esc to close; no buffered movement."
        ),
        Overlay::Menu => "PAUSED - P2 arena menu\n\nEsc: resume | F1: controls/help\n[ / ]: independent UI scale | +/-: integer world zoom\nT: motion | M: mute | 9 / 0: volume\n\nNo turns pass while browsing.\nClose this menu, then R to start a new run.\nFull options, saving and character creation are unavailable.".to_owned(),
        Overlay::None => String::new(),
    }
}

fn refresh(
    state: Option<Res<'_, CoreState>>,
    mode: Res<'_, UiMode>,
    settings: Res<'_, SettingsStatus>,
    log: Res<'_, MessageLog>,
    mut texts: ParamSet<
        '_,
        '_,
        (
            Query<'_, '_, &mut Text, With<HudText>>,
            Query<'_, '_, &mut Text, With<SightText>>,
            Query<'_, '_, &mut Text, With<LogText>>,
            Query<'_, '_, (&mut Text, &mut Node), With<OverlayText>>,
            Query<'_, '_, &mut Text, With<DeathText>>,
        ),
    >,
    mut banners: Query<'_, '_, &mut Node, (With<DeathBanner>, Without<OverlayText>)>,
) {
    if !mode.is_changed()
        && !settings.is_changed()
        && !state.as_ref().is_some_and(DetectChanges::is_changed)
    {
        return;
    }
    let header = state.as_ref().map_or_else(
        || {
            "P2 PLACEHOLDER - no playable core state.\nCheck startup logs/content errors."
                .to_owned()
        },
        |core| header_text(core, &mode, &settings),
    );
    for mut text in &mut texts.p0() {
        text.0.clone_from(&header);
    }
    if let Some(core) = state.as_ref() {
        let sight = sight_text(core);
        for mut text in &mut texts.p1() {
            text.0.clone_from(&sight);
        }
        let lines = if log.lines.is_empty() {
            "No committed actions.".to_owned()
        } else {
            log.lines.iter().cloned().collect::<Vec<_>>().join("\n")
        };
        for mut text in &mut texts.p2() {
            text.0.clone_from(&lines);
        }
    }
    for (mut text, mut node) in &mut texts.p3() {
        node.display = if mode.is_paused() {
            Display::Flex
        } else {
            Display::None
        };
        **text = overlay_text(&mode);
    }
    let dead = state
        .as_ref()
        .filter(|core| core.run_status() == RunStatus::Dead);
    for mut node in &mut banners {
        node.display = if dead.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
    if let Some(core) = dead {
        for mut text in &mut texts.p4() {
            **text = format!(
                "You died on run {}.\nPress R to start a new run.",
                core.run_number()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prl_core::{Command, Direction};

    fn hud_app(state: CoreState) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, prl_core::PrlCorePlugin))
            .init_resource::<PlaceholderTheme>()
            .init_resource::<UiMode>()
            .init_resource::<SettingsStatus>()
            .add_plugins(plugin)
            .insert_resource(state);
        app.update();
        app
    }

    fn text_of<T: Component>(app: &mut App) -> String {
        let world = app.world_mut();
        world
            .query_filtered::<&Text, With<T>>()
            .iter(world)
            .map(|text| text.0.clone())
            .collect()
    }

    #[test]
    fn header_shows_run_seed_hp_and_placeholder_notice() -> Result<(), prl_core::CoreError> {
        let mut app = hud_app(CoreState::demo()?);
        let header = text_of::<HudText>(&mut app);
        assert!(header.contains("P2 PLACEHOLDER - not final art/font/audio"));
        assert!(header.contains("Run 1 | Seed 1 | HP 40/40"));
        assert!(!header.contains("P1"));
        Ok(())
    }

    #[test]
    fn log_shows_message_text_and_resets_on_new_run() -> Result<(), prl_core::CoreError> {
        let mut app = hud_app(CoreState::demo()?);
        assert!(text_of::<LogText>(&mut app).contains("Run 1 begins"));
        app.world_mut()
            .resource_mut::<prl_core::CoreCommandQueue>()
            .push(Command::Move(Direction::North));
        app.update();
        app.world_mut()
            .resource_mut::<prl_core::CoreCommandQueue>()
            .push(Command::NewRun);
        app.update();
        let log = text_of::<LogText>(&mut app);
        assert!(log.contains("Run 2 begins"));
        assert!(!log.contains("Run 1 begins"), "per-run log resets: {log}");
        Ok(())
    }

    #[test]
    fn in_sight_lists_visible_hostiles_with_speed_hint() -> Result<(), prl_core::CoreError> {
        let mut state = CoreState::demo()?;
        state.issue(Command::Move(Direction::East));
        state.issue(Command::Move(Direction::East));
        state.issue(Command::OpenDoor(Direction::East));
        let mut app = hud_app(state);
        let sight = text_of::<SightText>(&mut app);
        assert!(sight.starts_with("In sight"));
        assert!(sight.contains("HP "), "{sight}");
        assert!(sight.contains("speed)"), "{sight}");
        Ok(())
    }

    #[test]
    fn death_banner_names_the_run_and_the_restart_key() -> Result<(), prl_core::CoreError> {
        let mut state = CoreState::demo()?;
        state.issue(Command::Move(Direction::East));
        state.issue(Command::Move(Direction::East));
        state.issue(Command::OpenDoor(Direction::East));
        for _ in 0..2000 {
            if state.is_player_dead() {
                break;
            }
            state.issue(Command::Wait);
        }
        let mut app = hud_app(state);
        assert_eq!(
            text_of::<DeathText>(&mut app),
            "You died on run 1.\nPress R to start a new run."
        );
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Node, With<DeathBanner>>()
                .iter(world)
                .all(|node| node.display == Display::Flex)
        );
        Ok(())
    }
}
