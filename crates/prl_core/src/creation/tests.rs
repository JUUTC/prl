use super::*;

fn id(name: &str) -> ContentId {
    ContentId::new("test", name).unwrap()
}

fn catalogue() -> BTreeSet<ContentId> {
    ["swords", "armor", "magic", "survival", "security"]
        .into_iter()
        .map(id)
        .collect()
}

fn skills(values: &[(&str, u16)]) -> StartingSkills {
    values
        .iter()
        .map(|(name, value)| (id(name), *value))
        .collect()
}

fn defaults() -> StartingSkills {
    skills(&[("swords", 500), ("armor", 500), ("magic", 500)])
}

fn random_skills() -> StartingSkills {
    skills(&[
        ("swords", 500),
        ("armor", 500),
        ("magic", 500),
        ("survival", 300),
    ])
}

fn base_stats() -> StartingStats {
    StartingStats {
        scores: [15, 15, 15, 8, 8, 8],
    }
}

fn draft() -> CreationDraft<Vec<&'static str>> {
    CreationDraft::new(
        id("warrior"),
        catalogue(),
        base_stats(),
        defaults(),
        vec!["sword"],
    )
    .unwrap()
}

#[test]
fn point_buy_table_and_mode_specific_score_ranges_are_exact() {
    let costs = [0, 1, 2, 3, 4, 5, 7, 9, 12, 15, 18];
    for (offset, expected) in costs.into_iter().enumerate() {
        let score = u8::try_from(offset).unwrap() + 8;
        let stats = StartingStats {
            scores: [score, 8, 8, 8, 8, 8],
        };
        assert_eq!(stat_cost(stats, CreationMode::Random), Ok(expected));
        if score <= 15 {
            assert_eq!(stat_cost(stats, CreationMode::Default), Ok(expected));
            assert_eq!(stat_cost(stats, CreationMode::Customize), Ok(expected));
        } else {
            assert!(matches!(stat_cost(stats, CreationMode::Customize),
                Err(CreationError::StatOutOfRange { score: value, maximum: 15, .. }) if value == score));
        }
    }
    for score in [0, 7, 19, 255] {
        assert!(
            matches!(stat_cost(StartingStats { scores: [8, 8, 8, 8, 8, score] }, CreationMode::Random),
            Err(CreationError::StatOutOfRange { index: 5, score: value, maximum: 18 }) if value == score)
        );
    }
}

#[test]
fn complete_stat_budgets_are_exact_or_inclusive_at_random_boundaries() {
    assert!(validate_stats(base_stats(), CreationMode::Default).is_ok());
    assert!(validate_stats(base_stats(), CreationMode::Customize).is_ok());
    for (scores, expected) in [
        ([18, 14, 8, 8, 8, 8], true),   // 25
        ([18, 13, 9, 8, 8, 8], true),   // 24
        ([18, 15, 11, 8, 8, 8], true),  // 30
        ([18, 15, 12, 8, 8, 8], false), // 31
        ([18, 12, 9, 8, 8, 8], false),  // 23
    ] {
        let valid = validate_stats(StartingStats { scores }, CreationMode::Random).is_ok();
        assert_eq!(valid, expected);
    }
    assert!(matches!(
        validate_stats(StartingStats { scores: [8; 6] }, CreationMode::Default),
        Err(CreationError::StatBudget {
            spent: 0,
            minimum: 27,
            maximum: 27
        })
    ));
    for (scores, spent) in [([15, 15, 14, 9, 8, 8], 26), ([15, 15, 15, 9, 8, 8], 28)] {
        assert!(
            matches!(validate_stats(StartingStats { scores }, CreationMode::Customize),
            Err(CreationError::StatBudget { spent: actual, minimum: 27, maximum: 27 }) if actual == spent)
        );
    }
}

#[test]
fn skills_use_fixed_point_catalogue_caps_and_mode_budgets() {
    let catalogue = catalogue();
    assert!(validate_skills(&defaults(), &catalogue, CreationMode::Default).is_ok());
    assert!(validate_skills(&defaults(), &catalogue, CreationMode::Customize).is_ok());
    assert!(validate_skills(&random_skills(), &catalogue, CreationMode::Random).is_ok());
    let fractional = skills(&[
        ("swords", 499),
        ("armor", 500),
        ("magic", 500),
        ("survival", 1),
    ]);
    assert!(validate_skills(&fractional, &catalogue, CreationMode::Customize).is_ok());
    assert!(matches!(
        validate_skills(&defaults(), &catalogue, CreationMode::Random),
        Err(CreationError::SkillBudget {
            spent_tenths: 1500,
            required_tenths: 1800
        })
    ));
    assert!(matches!(
        validate_skills(
            &skills(&[("swords", 501)]),
            &catalogue,
            CreationMode::Default
        ),
        Err(CreationError::SkillTooHigh { tenths: 501, .. })
    ));
    assert!(matches!(
        validate_skills(
            &skills(&[("unknown", 0)]),
            &catalogue,
            CreationMode::Default
        ),
        Err(CreationError::UnknownSkill(_))
    ));
}

#[test]
fn class_change_preserves_custom_and_random_sections_and_updates_defaults() {
    let mut draft = draft();
    draft.customize_stats();
    let custom = StartingStats {
        scores: [14, 14, 14, 9, 8, 8],
    };
    draft.edit_stats(custom).unwrap();
    draft.roll_gear(vec!["random gear"]);
    let replacement = skills(&[("security", 500), ("survival", 500), ("magic", 500)]);
    draft
        .change_class(
            id("rogue"),
            base_stats(),
            replacement.clone(),
            vec!["new gear"],
        )
        .unwrap();
    assert_eq!(draft.class(), &id("rogue"));
    assert_eq!(draft.stats().mode, CreationMode::Customize);
    assert_eq!(*draft.stats().value, custom);
    assert_eq!(draft.skills().mode, CreationMode::Default);
    assert_eq!(draft.skills().value, &replacement);
    assert_eq!(draft.gear().mode, CreationMode::Random);
    assert_eq!(draft.gear().value, &vec!["random gear"]);
}

#[test]
fn random_sections_reject_manual_edits_but_reroll_and_reset_are_independent() {
    let mut draft = draft();
    let rolled = StartingStats {
        scores: [18, 13, 9, 8, 8, 8],
    };
    draft.roll_stats(rolled).unwrap();
    draft.roll_skills(random_skills()).unwrap();
    draft.roll_gear(vec!["roll one"]);
    assert!(matches!(
        draft.edit_stats(base_stats()),
        Err(CreationError::WrongMode {
            actual: CreationMode::Random,
            ..
        })
    ));
    assert!(matches!(
        draft.edit_skills(defaults()),
        Err(CreationError::WrongMode {
            actual: CreationMode::Random,
            ..
        })
    ));
    assert!(matches!(
        draft.edit_gear(vec!["manual"]),
        Err(CreationError::WrongMode {
            actual: CreationMode::Random,
            ..
        })
    ));
    let again = StartingStats {
        scores: [18, 15, 11, 8, 8, 8],
    };
    draft.roll_stats(again).unwrap();
    draft.roll_gear(vec!["roll two"]);
    assert_eq!(*draft.stats().value, again);
    assert_eq!(draft.gear().value, &vec!["roll two"]);
    draft.reset_stats(base_stats()).unwrap();
    assert_eq!(draft.stats().mode, CreationMode::Default);
    assert_eq!(draft.skills().mode, CreationMode::Random);
    assert_eq!(draft.gear().mode, CreationMode::Random);
    draft.reset_skills(defaults()).unwrap();
    draft.reset_gear(vec!["default"]);
    assert!(draft.validate_allocations().is_ok());
}

#[test]
fn editing_allows_incomplete_drafts_but_final_validation_rejects_them() {
    let mut draft = draft();
    draft.customize_stats();
    draft.edit_stats(StartingStats { scores: [8; 6] }).unwrap();
    assert!(matches!(
        draft.validate_allocations(),
        Err(CreationError::StatBudget { .. })
    ));
    draft.edit_stats(base_stats()).unwrap();
    draft.customize_skills();
    draft.edit_skills(skills(&[("swords", 300)])).unwrap();
    assert!(matches!(
        draft.validate_allocations(),
        Err(CreationError::SkillBudget { .. })
    ));
    draft.edit_skills(defaults()).unwrap();
    assert!(draft.validate_allocations().is_ok());
}

#[test]
fn invalid_operations_leave_every_section_unchanged() {
    let mut draft = draft();
    let before = draft.clone();
    assert!(
        draft
            .change_class(
                id("mage"),
                base_stats(),
                skills(&[("unknown", 500)]),
                vec!["wand"]
            )
            .is_err()
    );
    assert_eq!(draft, before);
    assert!(draft.roll_stats(StartingStats { scores: [19; 6] }).is_err());
    assert_eq!(draft, before);
    assert!(draft.roll_skills(defaults()).is_err());
    assert_eq!(draft, before);
    draft.customize_stats();
    draft.customize_skills();
    let editing = draft.clone();
    assert!(draft.edit_stats(StartingStats { scores: [7; 6] }).is_err());
    assert!(draft.edit_skills(skills(&[("swords", 501)])).is_err());
    assert!(draft.reset_stats(StartingStats { scores: [8; 6] }).is_err());
    assert!(draft.reset_skills(random_skills()).is_err());
    assert_eq!(draft, editing);
}

#[test]
fn class_change_preserves_custom_skills_and_updates_default_gear() {
    let mut draft = draft();
    draft.customize_skills();
    let custom = skills(&[
        ("security", 400),
        ("survival", 500),
        ("magic", 500),
        ("armor", 100),
    ]);
    draft.edit_skills(custom.clone()).unwrap();
    let new_stats = StartingStats {
        scores: [8, 15, 8, 15, 8, 15],
    };
    draft
        .change_class(id("mage"), new_stats, defaults(), vec!["staff"])
        .unwrap();
    assert_eq!(draft.skills().value, &custom);
    assert_eq!(draft.skills().mode, CreationMode::Customize);
    assert_eq!(*draft.stats().value, new_stats);
    assert_eq!(draft.gear().value, &vec!["staff"]);
    assert!(draft.validate_allocations().is_ok());
}

#[test]
fn random_character_sections_survive_class_change_without_consuming_world_state() {
    let mut state = crate::CoreState::demo().unwrap();
    let before = state.snapshot();
    let mut draft = draft();
    draft
        .roll_stats(StartingStats {
            scores: [18, 13, 9, 8, 8, 8],
        })
        .unwrap();
    draft.roll_skills(random_skills()).unwrap();
    draft.roll_gear(vec!["random"]);
    let old = draft.clone();
    draft
        .change_class(id("mage"), base_stats(), defaults(), vec!["staff"])
        .unwrap();
    assert_eq!(draft.stats().value, old.stats().value);
    assert_eq!(draft.skills().value, old.skills().value);
    assert_eq!(draft.gear().value, old.gear().value);
    assert!(draft.validate_allocations().is_ok());
    assert_eq!(state.snapshot(), before);
    let mut reference = state.clone();
    assert_eq!(
        state.issue(crate::Command::Wait),
        reference.issue(crate::Command::Wait)
    );
    assert_eq!(state.snapshot(), reference.snapshot());
}

#[test]
fn switching_from_random_to_customize_requires_valid_edit_before_completion() {
    let mut draft = draft();
    draft
        .roll_stats(StartingStats {
            scores: [18, 13, 9, 8, 8, 8],
        })
        .unwrap();
    draft.customize_stats();
    assert_eq!(draft.stats().mode, CreationMode::Customize);
    assert!(matches!(
        draft.validate_allocations(),
        Err(CreationError::StatOutOfRange { score: 18, .. })
    ));
    draft.edit_stats(base_stats()).unwrap();
    assert!(draft.validate_allocations().is_ok());
}
