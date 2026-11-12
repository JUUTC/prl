use super::*;
use prl_core::CommandResult;

fn carried() -> CoreState {
    let mut core = crate::crypt_fixture();
    while let Some(command) = core.pickup_command() {
        assert!(matches!(
            core.issue(command),
            CommandResult::Accepted { .. }
        ));
    }
    core
}

fn find(core: &CoreState, predicate: impl Fn(&Item) -> bool) -> ItemId {
    core.inventory()
        .unwrap()
        .items()
        .find(|item| predicate(item))
        .unwrap()
        .id
}

fn app(core: CoreState) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        prl_core::PrlCorePlugin,
        crate::input::plugin,
        plugin,
    ))
    .insert_resource(core)
    .init_resource::<crate::PlaceholderTheme>()
    .init_resource::<crate::SettingsStatus>()
    .init_resource::<UiScale>()
    .init_resource::<ButtonInput<KeyCode>>();
    app.insert_resource(ItemIconImages(
        [
            (prl_content::ItemIcon::MainHand, 0.0),
            (prl_content::ItemIcon::Body, 32.0),
        ]
        .into_iter()
        .map(|(icon, offset)| {
            (
                icon,
                ImageNode {
                    rect: Some(Rect::from_corners(
                        Vec2::splat(offset),
                        Vec2::splat(offset + 32.0),
                    )),
                    ..default()
                },
            )
        })
        .collect(),
    ));
    app.world_mut().spawn((
        Window {
            focused: true,
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));
    app.update();
    app
}

fn act(app: &mut App, intent: Intent) {
    app.world_mut()
        .resource_scope(|world, mut workspace: Mut<'_, Workspace>| {
            world.resource_scope(|world, mut mode: Mut<'_, UiMode>| {
                world.resource_scope(|world, mut warnings: Mut<'_, InventoryWarningSettings>| {
                    world.resource_scope(|world, mut queue: Mut<'_, CoreCommandQueue>| {
                        activate(
                            intent,
                            world.resource::<CoreState>(),
                            &mut workspace,
                            &mut mode,
                            &mut warnings,
                            &mut queue,
                        );
                    });
                });
            });
        });
    app.update();
}

fn open(app: &mut App) {
    app.world_mut().resource_mut::<UiMode>().overlay = Overlay::Inventory;
    app.update();
}

#[test]
fn browsing_grid_list_and_unknown_source_views_are_read_only() {
    let core = carried();
    let before = core.snapshot();
    let mut app = app(core);
    open(&mut app);
    act(&mut app, Intent::List);
    let ids: Vec<_> = items(app.world().resource::<CoreState>())
        .iter()
        .map(|item| item.id)
        .collect();
    for id in ids {
        act(&mut app, Intent::Select(id));
        let world = app.world();
        let core = world.resource::<CoreState>();
        let workspace = world.resource::<Workspace>();
        actions(core, workspace);
        assert!(
            sources(core).is_empty(),
            "unidentified scrolls must never become sources"
        );
    }
    assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
}

#[test]
fn stack_actions_offer_one_and_half_splits_without_spending_game_time() {
    let mut config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
        "../../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
    ))
    .unwrap();
    config
        .items
        .items
        .iter_mut()
        .find(|definition| definition.id.name() == "healing_potion")
        .unwrap()
        .max_stack = 10;
    let mut core = config
        .build(&prl_content::ContentPack::placeholder().unwrap(), 1)
        .unwrap();
    while let Some(command) = core.pickup_command() {
        assert!(matches!(
            core.issue(command),
            CommandResult::Accepted { .. }
        ));
    }
    let potions: Vec<_> = core
        .inventory()
        .unwrap()
        .items()
        .filter(|item| item.definition.name() == "healing_potion")
        .map(|item| item.id)
        .collect();
    let scroll = find(&core, |item| item.definition.name() == "identify_scroll");
    core.issue(Command::Identify {
        scroll,
        target: potions[0],
    });
    core.issue(Command::Drop(potions[0]));
    core.issue(Command::Drop(potions[1]));
    core.issue(core.pickup_command().unwrap());
    let potion = find(&core, |item| {
        item.definition.name() == "healing_potion" && item.container == BACKPACK
    });
    assert_eq!(
        core.inventory()
            .unwrap()
            .items()
            .find(|item| item.id == potion)
            .unwrap()
            .quantity,
        3
    );
    let clock = core.clock();
    let mut app = app(core);
    open(&mut app);
    act(&mut app, Intent::Select(potion));
    let available: Vec<_> = actions(
        app.world().resource::<CoreState>(),
        app.world().resource::<Workspace>(),
    )
    .into_iter()
    .map(|action| action.verb)
    .collect();
    assert!(available.contains(&Verb::SplitOne));
    assert!(available.contains(&Verb::SplitHalf));
    act(&mut app, Intent::Act(Verb::SplitHalf));
    let core = app.world().resource::<CoreState>();
    assert_eq!(core.clock(), clock);
    let stacks: Vec<_> = core
        .inventory()
        .unwrap()
        .items()
        .filter(|item| item.definition.name() == "healing_potion" && item.container == BACKPACK)
        .map(|item| item.quantity)
        .collect();
    assert_eq!(stacks, [2, 1]);
    let remaining = app
        .world()
        .resource::<CoreState>()
        .inventory()
        .unwrap()
        .items()
        .find(|item| {
            item.definition.name() == "healing_potion"
                && item.container == BACKPACK
                && item.quantity == 2
        })
        .unwrap()
        .id;
    act(&mut app, Intent::Select(remaining));
    act(&mut app, Intent::Act(Verb::SplitCustom));
    act(&mut app, Intent::SplitDigit(1));
    act(&mut app, Intent::SplitSubmit);
    let core = app.world().resource::<CoreState>();
    let stacks: Vec<_> = core
        .inventory()
        .unwrap()
        .items()
        .filter(|item| item.definition.name() == "healing_potion" && item.container == BACKPACK)
        .map(|item| item.quantity)
        .collect();
    assert_eq!(stacks, [1, 1, 1]);
    assert_eq!(core.clock(), clock);
}

#[test]
fn rendered_unknown_gear_does_not_leak_hidden_affixes_or_action_hints() {
    let mut config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
        "../../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
    ))
    .unwrap();
    let id: prl_content::ContentId = "prl:keen_sword".parse().unwrap();
    config.items.supplies.insert(0, id.clone());
    let mut views = Vec::new();
    let mut icons = Vec::new();
    for secret in [2, 900] {
        let definition = config
            .items
            .items
            .iter_mut()
            .find(|item| item.id == id)
            .unwrap();
        definition.affix.damage = secret;
        definition.affix_name = format!("SECRET {secret}");
        let core = config
            .build(&prl_content::ContentPack::placeholder().unwrap(), 1)
            .unwrap();
        let mut app = app(core);
        open(&mut app);
        let world = app.world_mut();
        let text: Vec<_> = world
            .query::<&Text>()
            .iter(world)
            .map(|text| text.0.clone())
            .collect();
        assert!(!text.iter().any(|text| text.contains("SECRET")));
        views.push(text);
        icons.push(
            world
                .query::<&ImageNode>()
                .iter(world)
                .map(|image| (image.image.id(), image.rect, image.color))
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(views[0], views[1]);
    assert!(icons[0].len() >= 3);
    assert_eq!(icons[0], icons[1]);
}

#[test]
fn icon_children_preserve_buttons_selection_and_idle_pointer_lifecycle() {
    let mut app = app(carried());
    open(&mut app);
    let before = app.world().resource::<CoreState>().snapshot();
    let (button, icon) = {
        let world = app.world_mut();
        let button = world
            .query::<(Entity, &view::Control)>()
            .iter(world)
            .find(|(_, control)| matches!(control.0, Intent::Select(_)))
            .unwrap()
            .0;
        let icon = world
            .query::<(Entity, &ImageNode, &ChildOf)>()
            .iter(world)
            .find(|(_, _, parent)| parent.parent() == button)
            .unwrap()
            .0;
        (button, icon)
    };
    for _ in 0..5 {
        app.update();
        assert!(app.world().get::<view::Control>(button).is_some());
        assert!(app.world().get::<ImageNode>(icon).is_some());
        assert_eq!(
            *app.world().get::<Pickable>(icon).unwrap(),
            Pickable::IGNORE
        );
    }
    let intent = app.world().get::<view::Control>(button).unwrap().0;
    app.world_mut()
        .trigger(bevy::ui_widgets::Activate { entity: button });
    app.update();
    assert_eq!(app.world().resource::<Workspace>().focus, Some(intent));
    assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
    act(&mut app, Intent::List);
    let world = app.world_mut();
    assert!(world.query::<&ImageNode>().iter(world).count() >= 2);
}

#[test]
fn procedural_mode_keeps_text_controls_without_empty_icon_nodes() {
    let mut app = app(carried());
    app.world_mut().resource_mut::<ItemIconImages>().0.clear();
    open(&mut app);
    let world = app.world_mut();
    assert_eq!(world.query::<&ImageNode>().iter(world).count(), 0);
    assert!(
        world
            .query::<&view::Control>()
            .iter(world)
            .any(|control| matches!(control.0, Intent::Select(_)))
    );
}

#[test]
fn identification_chooses_and_consumes_only_an_explicit_known_source() {
    let mut core = carried();
    let scrolls: Vec<_> = core
        .inventory()
        .unwrap()
        .items()
        .filter(|item| {
            core.inventory().unwrap().definition(item).unwrap().kind == ItemKind::Identify
        })
        .map(|item| item.id)
        .collect();
    // Build known-source fixture through the existing explicit command, not hidden UI lookup.
    core.issue(Command::Identify {
        scroll: scrolls[0],
        target: scrolls[1],
    });
    let potion = find(&core, |item| {
        core.inventory()
            .unwrap()
            .definition(item)
            .is_some_and(|definition| matches!(definition.kind, ItemKind::Healing { .. }))
    });
    let mut app = app(core);
    open(&mut app);
    act(&mut app, Intent::Select(potion));
    let before = app.world().resource::<CoreState>().clock();
    act(&mut app, Intent::Act(Verb::Identify));
    assert!(app.world().resource::<Workspace>().stage == Stage::Sources);
    act(&mut app, Intent::Confirm);
    assert_eq!(app.world().resource::<CoreState>().clock(), before);
    act(&mut app, Intent::Source(scrolls[1]));
    assert!(matches!(
        app.world().resource::<Workspace>().stage,
        Stage::Review {
            warning: [false, false, false],
            ..
        }
    ));
    act(&mut app, Intent::Confirm);
    let core = app.world().resource::<CoreState>();
    assert!(
        !core
            .inventory()
            .unwrap()
            .items()
            .any(|item| item.id == scrolls[1])
    );
    assert!(
        core.inventory()
            .unwrap()
            .items()
            .find(|item| item.id == potion)
            .unwrap()
            .identified
    );
    assert_eq!(core.clock().player_actions(), before.player_actions() + 1);
}

#[test]
fn idle_frames_preserve_buttons_and_widget_activation_submits_once() {
    let mut app = app(crate::crypt_fixture());
    open(&mut app);
    app.update();
    let button = {
        let world = app.world_mut();
        world
            .query::<(Entity, &view::Control)>()
            .iter(world)
            .find(|(_, control)| control.0 == Intent::Act(Verb::Pickup))
            .unwrap()
            .0
    };
    let before = app.world().resource::<CoreState>().clock().player_actions();
    for _ in 0..5 {
        app.update();
        assert!(app.world().get::<view::Control>(button).is_some());
    }
    app.world_mut()
        .trigger(bevy::ui_widgets::Activate { entity: button });
    app.update();
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<CoreState>().clock().player_actions(),
        before + 1
    );
    assert_eq!(app.world().resource::<UiMode>().overlay, Overlay::Inventory);
    let feedback = &app.world().resource::<Workspace>().feedback;
    assert!(feedback.contains("50 energy"));
    assert!(!feedback.contains("CommandAccepted"));
}

#[test]
fn safe_equip_stays_open_retains_id_and_consumes_only_one_command() {
    let core = carried();
    assert!(!threatened(&core));
    let sword = find(&core, |item| {
        core.inventory()
            .unwrap()
            .definition(item)
            .is_some_and(|definition| definition.kind == ItemKind::Gear(EquipSlot::MainHand))
    });
    let before = core.clock().player_actions();
    let mut app = app(core);
    open(&mut app);
    act(&mut app, Intent::Select(sword));
    act(&mut app, Intent::Act(Verb::Equip));
    assert_eq!(app.world().resource::<UiMode>().overlay, Overlay::Inventory);
    assert_eq!(app.world().resource::<Workspace>().selected, Some(sword));
    assert_eq!(
        app.world().resource::<CoreState>().clock().player_actions(),
        before + 1
    );
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<CoreState>().clock().player_actions(),
        before + 1
    );
}

#[test]
fn rejected_unknown_healing_keeps_inventory_and_state_without_hidden_preview() {
    let core = carried();
    let potion = find(&core, |item| {
        core.inventory()
            .unwrap()
            .definition(item)
            .is_some_and(|definition| matches!(definition.kind, ItemKind::Healing { .. }))
    });
    let mut app = app(core);
    open(&mut app);
    act(&mut app, Intent::Select(potion));
    let before = app.world().resource::<CoreState>().snapshot();
    let before_clock = app.world().resource::<CoreState>().clock();
    assert!(
        actions(
            app.world().resource::<CoreState>(),
            app.world().resource::<Workspace>()
        )
        .iter()
        .find(|action| action.verb == Verb::Drink)
        .unwrap()
        .reason
        .is_none()
    );
    act(&mut app, Intent::Act(Verb::Drink));
    assert!(matches!(
        app.world().resource::<Workspace>().stage,
        Stage::Review {
            warning: [false, true, false],
            ..
        }
    ));
    act(&mut app, Intent::Confirm);
    let core = app.world().resource::<CoreState>();
    assert_eq!(core.clock(), before_clock);
    assert_eq!(
        core.inventory(),
        CoreState::restore(before).unwrap().inventory()
    );
    assert_eq!(app.world().resource::<UiMode>().overlay, Overlay::Inventory);
    assert!(
        app.world()
            .resource::<Workspace>()
            .feedback
            .contains("No effect")
    );
}

#[test]
fn cancellation_is_free_and_warning_checkbox_is_separate_from_game_state() {
    let core = carried();
    let mut app = app(core);
    open(&mut app);
    let before = app.world().resource::<CoreState>().snapshot();
    let before_actions = app.world().resource::<CoreState>().clock().player_actions();
    act(&mut app, Intent::Act(Verb::Drop));
    assert!(matches!(
        app.world().resource::<Workspace>().stage,
        Stage::Review {
            warning: [true, false, false],
            ..
        }
    ));
    act(&mut app, Intent::ToggleWarning(0));
    assert_eq!(
        app.world().resource::<InventoryWarningSettings>().enabled,
        [false, true, true]
    );
    act(&mut app, Intent::Cancel);
    assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
    act(&mut app, Intent::Act(Verb::Drop));
    assert!(app.world().resource::<Workspace>().flight.is_none());
    assert_eq!(
        app.world().resource::<CoreState>().clock().player_actions(),
        before_actions + 1
    );
}

#[test]
fn threat_after_action_closes_but_browsing_in_danger_is_free() {
    let core = threatened_with_body_equipped();
    let sword = find(&core, |item| {
        core.inventory()
            .unwrap()
            .definition(item)
            .is_some_and(|definition| definition.kind == ItemKind::Gear(EquipSlot::MainHand))
    });
    let mut app = app(core);
    open(&mut app);
    let before = app.world().resource::<CoreState>().snapshot();
    act(&mut app, Intent::List);
    assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
    act(&mut app, Intent::Select(sword));
    act(&mut app, Intent::Act(Verb::Equip));
    assert_eq!(app.world().resource::<UiMode>().overlay, Overlay::None);
}

#[test]
fn dropping_worn_body_keeps_danger_warning_when_drop_warning_is_disabled() {
    let core = threatened_with_body_equipped();
    let body = find(&core, |item| item.container == EQUIPMENT);
    let before = core.snapshot();
    let mut app = app(core);
    open(&mut app);
    act(&mut app, Intent::Select(body));
    act(&mut app, Intent::Act(Verb::Drop));
    assert!(matches!(
        app.world().resource::<Workspace>().stage,
        Stage::Review {
            warning: [true, false, true],
            cost: 300,
            ..
        }
    ));
    act(&mut app, Intent::ToggleWarning(0));
    act(&mut app, Intent::Cancel);
    act(&mut app, Intent::Act(Verb::Drop));
    assert!(matches!(
        app.world().resource::<Workspace>().stage,
        Stage::Review {
            warning: [false, false, true],
            cost: 300,
            ..
        }
    ));
    act(&mut app, Intent::Cancel);
    assert_eq!(app.world().resource::<CoreState>().snapshot(), before);
}

#[test]
fn organization_in_danger_is_free_and_keeps_inventory_open() {
    let core = threatened_with_body_equipped();
    let before = (
        core.clock(),
        core.rng_position(),
        core.actors().cloned().collect::<Vec<_>>(),
    );
    let mut app = app(core);
    open(&mut app);
    act(&mut app, Intent::Organize);
    let core = app.world().resource::<CoreState>();
    assert_eq!(
        (
            core.clock(),
            core.rng_position(),
            core.actors().cloned().collect()
        ),
        before
    );
    assert_eq!(app.world().resource::<UiMode>().overlay, Overlay::Inventory);
}

#[test]
fn placement_preview_cancel_invalid_and_drag_paths_preserve_time_and_ownership() {
    let core = carried();
    let sword = find(&core, |item| {
        core.inventory()
            .unwrap()
            .definition(item)
            .is_some_and(|definition| definition.kind == ItemKind::Gear(EquipSlot::MainHand))
    });
    let before = core.inventory().unwrap().clone();
    let clock = core.clock();
    let mut app = app(core);
    open(&mut app);
    act(&mut app, Intent::Select(sword));
    act(&mut app, Intent::List);
    act(&mut app, Intent::BeginPlacement);
    act(&mut app, Intent::Rotate);
    assert_eq!(
        app.world().resource::<CoreState>().inventory(),
        Some(&before)
    );
    act(
        &mut app,
        Intent::PlaceCell(prl_core::GridCell { x: 9, y: 5 }),
    );
    assert!(matches!(
        app.world().resource::<Workspace>().stage,
        Stage::Placement(_)
    ));
    assert_eq!(
        app.world().resource::<CoreState>().inventory(),
        Some(&before)
    );
    act(&mut app, Intent::Cancel);
    assert!(app.world().resource::<Workspace>().list);
    assert!(matches!(
        app.world().resource::<Workspace>().stage,
        Stage::Browse
    ));
    assert!(
        app.world()
            .resource::<Workspace>()
            .feedback
            .contains("Placement cancelled.")
    );
    assert_eq!(
        app.world().resource::<CoreState>().inventory(),
        Some(&before)
    );
    act(
        &mut app,
        Intent::DragPlace(prl_core::PlaceItem {
            item: sword,
            cell: prl_core::GridCell { x: 7, y: 5 },
            rotated: true,
        }),
    );
    let core = app.world().resource::<CoreState>();
    assert_eq!(core.clock(), clock);
    let bag = core.inventory().unwrap();
    assert_eq!(bag.items().count(), before.items().count());
    assert_eq!(bag.carried_weight(), before.carried_weight());
    assert_eq!(
        bag.items().find(|item| item.id == sword).unwrap().cell,
        Some(prl_core::GridCell { x: 7, y: 5 })
    );
}

fn threatened_with_body_equipped() -> CoreState {
    use prl_core::Direction;
    use std::collections::{BTreeMap, VecDeque};
    let config: prl_gen::CryptPlaytest = ron::from_str(include_str!(
        "../../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
    ))
    .unwrap();
    let mut core = config
        .build_legacy(&prl_content::ContentPack::placeholder().unwrap(), 1)
        .unwrap();
    while let Some(command) = core.pickup_command() {
        assert!(matches!(
            core.issue(command),
            CommandResult::Accepted { .. }
        ));
    }
    let body = find(&core, |item| {
        core.inventory()
            .unwrap()
            .definition(item)
            .is_some_and(|definition| definition.kind == ItemKind::Gear(EquipSlot::Body))
    });
    core.issue(Command::Transfer(TransferItems {
        item: body,
        destination: EQUIPMENT,
        cell: None,
    }));
    let start = core.player().unwrap().position();
    let target = core
        .actors()
        .find(|actor| actor.is_hostile())
        .unwrap()
        .position();
    let mut previous = BTreeMap::from([(start, None)]);
    let mut pending = VecDeque::from([start]);
    while let Some(position) = pending.pop_front() {
        if position == target {
            break;
        }
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let Some(next) = position.offset(direction) else {
                continue;
            };
            if previous.contains_key(&next) {
                continue;
            }
            if core
                .map()
                .tile(next)
                .and_then(|tile| core.tile_definition(&tile.definition))
                .is_some_and(|tile| !tile.blocks_movement || tile.door)
            {
                previous.insert(next, Some((position, direction)));
                pending.push_back(next);
            }
        }
    }
    let mut route = Vec::new();
    let mut position = target;
    while position != start {
        let (parent, direction) = previous[&position].unwrap();
        route.push(direction);
        position = parent;
    }
    route.reverse();
    for direction in route {
        if threatened(&core) {
            break;
        }
        let destination = core.player().unwrap().position().offset(direction).unwrap();
        if core.map().tile(destination).is_some_and(|tile| {
            !tile.door_open
                && core
                    .tile_definition(&tile.definition)
                    .is_some_and(|definition| definition.door)
        }) {
            core.issue(Command::OpenDoor(direction));
        }
        if !threatened(&core) {
            core.issue(Command::Move(direction));
        }
    }
    assert!(threatened(&core));
    core
}
