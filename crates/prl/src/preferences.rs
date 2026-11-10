//! Bridges UI warning preferences to the independent per-user settings file.

use bevy::prelude::*;
use prl_save::{InventoryWarnings, SnapshotStore};
use prl_ui::InventoryWarningSettings;

#[derive(Resource)]
pub(super) struct Preferences {
    store: SnapshotStore,
    previous: [bool; 3],
}

pub(super) fn install(app: &mut App, store: SnapshotStore) {
    let (settings, status) = match store.load_inventory_warnings() {
        Ok(settings) => (settings, "Warning preferences loaded".into()),
        Err(error) => {
            warn!("{error}");
            (
                InventoryWarnings::default(),
                format!(
                    "WARNING OPTIONS ERROR: {error}. Warnings enabled; toggle an option to retry saving."
                ),
            )
        }
    };
    let enabled = [
        settings.dropping,
        settings.unknown,
        settings.threatened_body,
    ];
    app.insert_resource(InventoryWarningSettings { enabled, status })
        .insert_resource(Preferences {
            store,
            previous: enabled,
        })
        .add_systems(
            Update,
            persist
                .after(prl_core::CoreSet::Input)
                .before(prl_core::CoreSet::Presentation),
        );
}

fn persist(
    mut preferences: ResMut<'_, Preferences>,
    mut settings: ResMut<'_, InventoryWarningSettings>,
) {
    if preferences.previous == settings.enabled {
        return;
    }
    preferences.previous = settings.enabled;
    let [dropping, unknown, threatened_body] = settings.enabled;
    match preferences
        .store
        .save_inventory_warnings(InventoryWarnings {
            dropping,
            unknown,
            threatened_body,
        }) {
        Ok(()) => settings.status = "Warning preferences saved".into(),
        Err(error) => {
            warn!("{error}");
            settings.status = format!("NOT SAVED: {error}. Toggle an option to retry.");
        }
    }
}
