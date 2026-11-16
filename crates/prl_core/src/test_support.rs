//! Shared helpers for unit tests.

use crate::{ActorId, CoreState, OutcomeKind, Position};

pub(crate) fn arena() -> CoreState {
    CoreState::demo().unwrap()
}

pub(crate) fn count_waits(state: &CoreState, actor: ActorId) -> usize {
    state
        .outcomes()
        .iter()
        .filter(
            |outcome| matches!(outcome.kind, OutcomeKind::ActorWaited { actor: id } if id == actor),
        )
        .count()
}

pub(crate) fn place(state: &mut CoreState, actor: ActorId, position: Position) {
    state.actors.get_mut(&actor).unwrap().position = position;
    state.refresh_visibility();
}

pub(crate) fn set_speed(state: &mut CoreState, actor: ActorId, speed: u16) {
    state.actors.get_mut(&actor).unwrap().speed = speed;
}
/// Small two-floor state for persistence/item rule tests, independent of generation.
pub(crate) fn dungeon() -> CoreState {
    #[derive(serde::Deserialize)]
    struct CatalogSource {
        items: prl_content::ItemCatalog,
    }
    let mut pack = prl_content::ContentPack::placeholder().unwrap();
    let mut plans = Vec::new();
    for (name, enemy) in [("test_lower", "skeleton"), ("test_upper", "grave_rat")] {
        let mut map = pack.start_map().unwrap().clone();
        map.id = prl_content::ContentId::new("prl", name).unwrap();
        map.rows = vec![
            "#######".into(),
            "#.....#".into(),
            "#..+..#".into(),
            "#.....#".into(),
            "#.....#".into(),
            "#.....#".into(),
            "#######".into(),
        ];
        map.spawns = vec![
            prl_content::SpawnDefinition {
                actor: prl_content::ContentId::new("prl", "player").unwrap(),
                x: 1,
                y: 1,
            },
            prl_content::SpawnDefinition {
                actor: prl_content::ContentId::new("prl", enemy).unwrap(),
                x: 5,
                y: 5,
            },
        ];
        plans.insert(
            0,
            crate::FloorPlan {
                definition: map.id.clone(),
                entrance: Position::new(1, 1),
                exit: Position::new(5, 1),
            },
        );
        pack = pack.with_start_map(map).unwrap();
    }
    let mut state = CoreState::from_dungeon(
        pack,
        1,
        plans,
        &prl_content::ContentId::new("prl", "skeleton").unwrap(),
    )
    .unwrap();
    // Test item actions without an active attacker; inactive guardian is preserved.
    state.actors.retain(|_, actor| actor.is_player());
    let source: CatalogSource = ron::from_str(include_str!(
        "../../../assets/packs/prl/data/dungeons/crypt_playtest.ron"
    ))
    .unwrap();
    state.with_items(source.items).unwrap()
}
