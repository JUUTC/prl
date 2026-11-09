use bevy::prelude::*;
use prl_core::{CoreSet, CoreState};

use crate::input::Overlay;
use crate::{PlaceholderTheme, UiMode};

const CONTROLS: &str = "Move: arrows / numpad / vi (ykuhlbjn)\nWait: . / numpad 5 / Space\nDoor: O, then direction | Reset fixture: R\nHelp: F1 / ? | Pause/menu: Esc\nWorld zoom: +/- | UI scale: [ / ]";

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct LogText;

#[derive(Component)]
struct OverlayText;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup).add_systems(
        Update,
        refresh
            .in_set(CoreSet::Presentation)
            .run_if(resource_exists::<UiMode>),
    );
}

fn setup(mut commands: Commands<'_, '_>, theme: Res<'_, PlaceholderTheme>) {
    commands
        .spawn((
            Name::new("P1 placeholder HUD"),
            Node {
                position_type: PositionType::Absolute,
                top: px(8),
                left: px(8),
                max_width: percent(95),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(theme.panel),
        ))
        .with_child((
            Text::new("P1 PLACEHOLDER — awaiting core state"),
            theme.font(),
            TextColor(theme.text),
            HudText,
        ));
    commands
        .spawn((
            Name::new("P1 placeholder message log and controls"),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(8),
                left: px(8),
                max_width: percent(95),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(theme.panel),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    ..default()
                })
                .with_children(|column| {
                    column.spawn((Text::new(CONTROLS), theme.font(), TextColor(theme.text)));
                    column.spawn((
                        Text::new("Message log: no committed actions."),
                        theme.font(),
                        TextColor(theme.text),
                        LogText,
                    ));
                });
        });
    commands.spawn((
        Name::new("P1 placeholder pause/help overlay"),
        Text::default(),
        theme.font(),
        TextColor(theme.text),
        BackgroundColor(theme.panel),
        Node {
            position_type: PositionType::Absolute,
            top: percent(30),
            left: percent(15),
            max_width: percent(70),
            padding: UiRect::all(px(24)),
            display: Display::None,
            ..default()
        },
        ZIndex(10),
        OverlayText,
    ));
}

fn refresh(
    state: Option<Res<'_, CoreState>>,
    mode: Res<'_, UiMode>,
    mut hud: Query<'_, '_, &mut Text, (With<HudText>, Without<LogText>, Without<OverlayText>)>,
    mut log: Query<'_, '_, &mut Text, (With<LogText>, Without<HudText>, Without<OverlayText>)>,
    mut overlays: Query<
        '_,
        '_,
        (&mut Text, &mut Node),
        (With<OverlayText>, Without<HudText>, Without<LogText>),
    >,
) {
    if !mode.is_changed() && !state.as_ref().is_some_and(DetectChanges::is_changed) {
        return;
    }
    for mut text in &mut hud {
        **text = state.as_ref().map_or_else(
            || "P1 PLACEHOLDER — no playable core state.\nCheck startup logs/content errors.".to_owned(),
            |core| {
                let vitals = core.player().map_or_else(
                    || "Player unavailable".to_owned(),
                    |actor| format!("HP {}/{} | Stamina {}/{}", actor.hp(), actor.max_hp(), actor.stamina(), actor.max_stamina()),
                );
                format!(
                    "P1 PLACEHOLDER — not final art/font | Normal fixture\n{vitals} | Seed {}\nTick {} | Clock {}s | Actions {}{}",
                    core.seed(),
                    core.clock().ticks(),
                    core.clock().elapsed_seconds(),
                    core.clock().player_actions(),
                    if mode.door { "\nOpen door: choose direction (Esc cancels)" } else { "" },
                )
            },
        );
    }
    if let Some(core) = state.as_ref().filter(|core| core.is_changed()) {
        let messages: Vec<_> = core.messages().iter().rev().take(5).collect();
        let message_text = messages
            .into_iter()
            .rev()
            .map(|message| format!("#{} {}", message.sequence, message.text))
            .collect::<Vec<_>>()
            .join("\n");
        for mut text in &mut log {
            **text = format!("Message log\n{message_text}");
        }
    }
    for (mut text, mut node) in &mut overlays {
        node.display = if mode.is_paused() {
            Display::Flex
        } else {
            Display::None
        };
        **text = if mode.overlay == Overlay::Help {
            format!(
                "HELP — gameplay paused\n\n{CONTROLS}\n\nCyan square: player | Orange tall rectangle: other actor\nGrey: wall | Brown + bar: door | Dim: remembered terrain\nUnseen actors are hidden. Light/FOV are core-owned.\n\nOne key press, one intent; held keys do not repeat.\nO arms a directional door command; no time until direction.\nCombat, inventory, saves and final assets are unavailable.\n\nF1 / ? / Esc to close; no buffered movement."
            )
        } else if mode.overlay == Overlay::Menu {
            "PAUSED — P1 fixture menu\n\nEsc: resume | F1: controls/help\n[ / ]: independent UI scale | +/-: integer world zoom\n\nNo turns pass while browsing.\nClose this menu, then R to reset the fixture.\nFull options, saving and character creation are unavailable.".to_owned()
        } else {
            String::new()
        };
    }
}
