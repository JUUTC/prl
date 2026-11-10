use std::collections::VecDeque;

use bevy::prelude::*;
use prl_core::{ACTION_ENERGY, Actor, CoreSet, CoreState, OutcomeKind, RunStatus};

use crate::cursor::OutcomeCursor;
use crate::input::{CONFIRM_SECONDS, Overlay};
use crate::{PlaceholderTheme, RepeatSettings, SettingsStatus, UiMode};

/// Newest core messages kept visible; short so the log never hides the player.
const LOG_LINES: usize = 5;

const CONTROLS: &str = "Move / attack: arrows, numpad, vi (y k u h l b j n); bump an enemy to attack\nWait: . / numpad 5 / Space | Open door: O, then a direction\nHeld input: F2 on/off | F3 delay | F4 interval | F5 remap movement/wait\nNew run: R twice (or R then Y) while alive; R once after death\nMotion: T (Normal / Fast / Off) | Sound: M mute, 9 / 0 volume\nWorld zoom: + / - | UI scale: [ / ] | Pause/menu: Esc | Help: F1 / ?";

const LEGEND: &str = "Legend (placeholder art, not final)\nYou: the figure with shield and coif (greys out when dead)\nEnemies: shambler (slow), skeleton (normal), grave rat (fast); red flash = hit\nRed bar under an actor: HP when hurt\nRaised blocks: walls | Planks: door | Dim: remembered terrain\nIf art fails to load: cyan square = you; green wide = slow, orange tall = normal,\npink diamond = fast; grey = wall; brown + bar = door\nFloating numbers: damage dealt (CRIT = critical, glance = glancing blow)\nThe log explains each hit: raw damage and how much armor removed.";

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct SightText;

#[derive(Component)]
struct LogText;

#[derive(Component)]
struct OverlayText;

#[derive(Component)]
struct OverlayPanel;

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
        .init_resource::<RepeatSettings>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            scroll_overlay
                .in_set(CoreSet::Presentation)
                .run_if(resource_exists::<ButtonInput<KeyCode>>),
        )
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
    font: TextFont,
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
        .with_child((Text::new(initial), font, TextColor(theme.text), marker));
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
        theme.font(),
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
        theme.label(),
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
        theme.font(),
        LogText,
    );
    commands
        .spawn((
            Name::new("P2 placeholder death banner"),
            Node {
                position_type: PositionType::Absolute,
                // Above the five-line message log and clear of the map's top rows (B-008).
                bottom: px(160),
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
            BackgroundColor(theme.panel.with_alpha(0.72)),
            Node {
                padding: UiRect::all(px(20)),
                ..default()
            },
            DeathText,
        ));
    spawn_overlay(&mut commands, &theme);
}

fn spawn_overlay(commands: &mut Commands<'_, '_>, theme: &PlaceholderTheme) {
    commands
        .spawn((
            Name::new("Placeholder modal panel"),
            BackgroundColor(theme.panel),
            Node {
                position_type: PositionType::Absolute,
                top: percent(10),
                left: percent(10),
                max_width: percent(80),
                max_height: percent(80),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                padding: UiRect::all(px(24)),
                display: Display::None,
                ..default()
            },
            ZIndex(10),
            ScrollPosition::default(),
            OverlayPanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Name::new("Placeholder modal text"),
                Text::default(),
                theme.font(),
                TextColor(theme.text),
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                OverlayText,
            ));
        });
}

fn scroll_overlay(
    keys: Res<'_, ButtonInput<KeyCode>>,
    mode: Res<'_, UiMode>,
    mut previous: Local<'_, Overlay>,
    mut panels: Query<'_, '_, &mut ScrollPosition, With<OverlayPanel>>,
) {
    for mut position in &mut panels {
        if *previous != mode.overlay {
            position.0.y = 0.0;
        }
        if mode.is_paused() {
            if keys.just_pressed(KeyCode::PageDown) {
                position.0.y += 120.0;
            }
            if keys.just_pressed(KeyCode::PageUp) {
                position.0.y = (position.0.y - 120.0).max(0.0);
            }
        }
    }
    *previous = mode.overlay;
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
    let dungeon = core.floor_number().map_or_else(String::new, |floor| {
        let weight = core.inventory().map_or_else(String::new, |inventory|
            format!(" | Carry {}/{} g", inventory.carried_weight(), inventory.carry_capacity()));
        format!("\nCrypt floor {floor}/2{weight} | G pickup | I inventory | X examine | Enter down | , up{}",
            if core.slice_completed() { "\nCRYPT CLEARED - playtest endpoint. R: new run." } else { "" })
    });
    let stage = if core.floor_number().is_some() {
        "P3"
    } else {
        "P2"
    };
    format!(
        "{stage} PLACEHOLDER - not final art/font/audio | F1 help\nRun {} | Seed {} | {vitals}\nTick {} | Clock {}s | Actions {} | Motion {} (T) | Sound {} (M, 9/0) | Save {}{prompt}{dungeon}",
        core.run_number(),
        core.seed(),
        core.clock().ticks(),
        core.clock().elapsed_seconds(),
        core.clock().player_actions(),
        settings.motion,
        settings.sound,
        settings.persistence,
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

fn examine_text(core: &CoreState, position: Option<prl_core::Position>) -> String {
    let Some(position) = position else {
        return "No selected cell.".into();
    };
    let mut lines = vec![format!(
        "EXAMINE ({}, {}) - no time passes\nArrows/vi: move selection | Esc: close\n",
        position.x, position.y
    )];
    if let Some(memory) = core.map().remembered_tile(position) {
        lines.push(format!(
            "\nTerrain: {}{}",
            memory.definition,
            if core.is_visible(position) {
                ""
            } else {
                " (remembered; no live contents shown)"
            }
        ));
    } else {
        lines.push("\nUnexplored.".into());
    }
    if core.is_visible(position) {
        for actor in core
            .visible_actors()
            .filter(|actor| actor.position() == position)
        {
            lines.push(format!(
                "\n{}: HP {}/{}",
                actor.name(),
                actor.hp(),
                actor.max_hp()
            ));
        }
        if let Some(inventory) = core.inventory() {
            for item in core.ground_items(position) {
                lines.push(format!(
                    "\n{} ({} g)",
                    inventory.label(item),
                    inventory
                        .definition(item)
                        .map_or(0, |definition| definition.weight)
                ));
            }
        }
        if let Some(plan) = core.floor_plan() {
            if position == plan.entrance {
                lines.push("\n< Arrival / stairs up (comma; floor 1 has no outside map).".into());
            }
            if position == plan.exit {
                lines.push("\n> Stairs down / guardian endpoint (Enter).".into());
            }
        }
    }
    lines.concat()
}

fn loot_text(core: &CoreState, show_all: bool) -> String {
    let Some(inventory) = core.inventory() else {
        return String::new();
    };
    let mut labels = Vec::new();
    for position in core.map().visible_positions() {
        for item in core.ground_items(position) {
            let (show, emphasize) = inventory.ground_label(item);
            if show || show_all {
                labels.push(format!(
                    "{}{},{}: {}",
                    if emphasize { "! " } else { "" },
                    position.x,
                    position.y,
                    inventory.label(item)
                ));
            }
        }
    }
    if labels.is_empty() {
        String::new()
    } else {
        format!(
            "\nGround loot (V: show all = {show_all})\n{}",
            labels.into_iter().take(6).collect::<Vec<_>>().join("\n")
        )
    }
}

fn overlay_text(mode: &UiMode, core: Option<&CoreState>) -> String {
    let text = match mode.overlay {
        Overlay::Help => format!(
            "HELP - gameplay paused\n\n{CONTROLS}\nG: pick up one item | I: inventory/equipment | X: examine with movement keys\nEnter: descend/finish at > | Comma: ascend at <\n\n{LEGEND}\n\nHeld movement/wait stops at danger, low HP, loot, doors and stairs.\nAttacks never repeat. Release and press again after an interruption.\nInventory: click or arrows select; Tab/Enter focus/activate; Esc cancels.\nE equip, Q drink, D drop, C choose known Identify source, G pick up.\nBrowsing is free; safe actions stay open; danger/damage closes inventory.\nInventory Options persist separate drop/unknown/slow-Body warnings.\nAutomatic Continue/checkpoints and text morgues are available.\nPacking/rotation, unknown-scroll Read, final assets and full options remain unavailable.\n\nF1 / ? / Esc to close; no buffered movement."
        ),
        Overlay::Menu => "PAUSED - placeholder playtest\n\nEsc: resume | F1: controls/help\n[ / ]: independent UI scale | +/-: integer world zoom\nT: motion | M: mute | 9 / 0: volume\nF2: held input | F3: delay | F4: interval | F5: remap\n\nNo turns pass while browsing.\nClose this menu, then R to start a new run.\nAutomatic saving is active; full options and character creation are unavailable.".to_owned(),
        Overlay::None | Overlay::Inventory => String::new(),
        Overlay::Look => core.map_or_else(|| "No map available.".into(), |core| examine_text(core, mode.look)),
    };
    if mode.is_paused() {
        format!("PgUp/PgDn: scroll this view\n{text}")
    } else {
        text
    }
}

fn refresh(
    state: Option<Res<'_, CoreState>>,
    mode: Res<'_, UiMode>,
    settings: Res<'_, SettingsStatus>,
    repeat: Res<'_, RepeatSettings>,
    log: Res<'_, MessageLog>,
    mut texts: ParamSet<
        '_,
        '_,
        (
            Query<'_, '_, &mut Text, With<HudText>>,
            Query<'_, '_, &mut Text, With<SightText>>,
            Query<'_, '_, &mut Text, With<LogText>>,
            Query<'_, '_, &mut Text, With<OverlayText>>,
            Query<'_, '_, &mut Text, With<DeathText>>,
            Query<'_, '_, &mut Node, With<OverlayPanel>>,
        ),
    >,
    mut banners: Query<'_, '_, &mut Node, (With<DeathBanner>, Without<OverlayPanel>)>,
) {
    if !mode.is_changed()
        && !settings.is_changed()
        && !repeat.is_changed()
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
        let sight = format!(
            "{}{}",
            sight_text(core),
            loot_text(core, mode.show_all_loot)
        );
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
    for mut text in &mut texts.p3() {
        **text = format!(
            "{}\n\nHeld input: {} | delay {} ms | interval {} ms\n{}",
            overlay_text(&mode, state.as_deref()),
            if repeat.enabled { "On" } else { "Off" },
            repeat.delay.as_millis(),
            repeat.interval.as_millis(),
            mode.binding_prompt(),
        );
    }
    for mut node in &mut texts.p5() {
        node.display = if mode.is_paused() && mode.overlay != Overlay::Inventory {
            Display::Flex
        } else {
            Display::None
        };
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

    #[test]
    fn crypt_views_label_subset_and_do_not_disclose_unseen_live_contents() {
        let core = crate::crypt_fixture();
        let header = header_text(&core, &UiMode::default(), &SettingsStatus::default());
        assert!(header.contains("P3 PLACEHOLDER"));
        assert!(header.contains("Crypt floor 1/2"));
        let entrance = core.floor_plan().unwrap().entrance;
        let exit = core.floor_plan().unwrap().exit;
        assert!(!core.is_visible(exit));
        let unseen = examine_text(&core, Some(exit));
        assert!(unseen.contains("Unexplored"));
        assert!(!unseen.contains("Steel"));
        assert!(!unseen.contains("guardian endpoint"));
        let visible = examine_text(&core, Some(entrance));
        assert!(visible.contains("Iron sword"));
        assert!(visible.contains("Arrival"));
        assert!(loot_text(&core, false).contains("Iron sword"));
        assert!(!loot_text(&core, true).contains("Steel"));
    }

    #[test]
    fn modal_scrolling_never_changes_rules_and_resets_on_view_change() {
        let mut app = hud_app(crate::crypt_fixture());
        app.insert_resource(ButtonInput::<KeyCode>::default());
        let before = app.world().resource::<CoreState>().snapshot();
        app.world_mut().resource_mut::<UiMode>().overlay = Overlay::Inventory;
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::PageDown);
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<&ScrollPosition, With<OverlayPanel>>()
                .single(world)
                .unwrap()
                .0
                .y,
            120.0
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut().resource_mut::<UiMode>().overlay = Overlay::Help;
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<&ScrollPosition, With<OverlayPanel>>()
                .single(world)
                .unwrap()
                .0
                .y,
            0.0
        );
        assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
    }

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
