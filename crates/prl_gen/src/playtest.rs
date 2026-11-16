//! Composes the bounded outdoor approach and crypt without rule writes from rendering.

#[cfg(test)]
#[path = "playtest_tests.rs"]
mod tests;

use prl_content::{ActorBehavior, ContentId, ContentPack, MapDefinition, SpawnDefinition};
use prl_core::{CoreError, CoreState, FloorPlan, Position, RunFactory};
use serde::{Deserialize, Serialize};

use crate::{CryptSettings, generate_crypt_floor};

/// Versioned crypt encounter blueprint; numbers remain playtest defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CryptPlaytest {
    /// Schema format (1).
    pub format: u32,
    /// Optional bounded outdoor approach preceding the crypt.
    #[serde(default)]
    pub approach: Option<CryptApproach>,
    /// Exactly two explicitly identified layout configurations.
    pub floors: Vec<CryptSettings>,
    /// Enemies placed on each floor, not including the final guardian.
    pub enemies: Vec<ContentId>,
    /// One final-floor enemy marks the endpoint objective.
    pub guardian: ContentId,
    /// Validated slice item definitions, supplies and label preset.
    pub items: prl_content::ItemCatalog,
    /// Closed-door terrain definition placed on corridor choke points.
    pub door: ContentId,
}

/// Static, validated outdoor playtest map and its travel endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CryptApproach {
    /// Content map, including a player spawn.
    pub map: MapDefinition,
    /// Initial player location.
    pub entrance: Position,
    /// Entrance to the upper crypt.
    pub exit: Position,
}

impl CryptPlaytest {
    /// Builds the approach, both floors, encounters and persistent core state.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when settings, content, placement or generation fail.
    pub fn build(&self, base: &ContentPack, seed: u64) -> Result<CoreState, CoreError> {
        self.build_with_approach(base, seed, true)
    }

    /// Rebuilds the prior two-floor content for validating existing saves.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] if the prior crypt cannot be rebuilt.
    pub fn build_legacy(&self, base: &ContentPack, seed: u64) -> Result<CoreState, CoreError> {
        self.build_with_approach(base, seed, false)
    }

    fn build_with_approach(
        &self,
        base: &ContentPack,
        seed: u64,
        include_approach: bool,
    ) -> Result<CoreState, CoreError> {
        self.validate(base)?;
        let mut content = base.clone();
        let mut plans = Vec::new();
        for (number, settings) in self.floors.iter().enumerate() {
            let floor = generate_crypt_floor(
                &content,
                settings,
                seed,
                u32::try_from(number)
                    .map_err(|_| CoreError::InvalidRun("floor overflow".into()))?,
            )
            .map_err(|error| CoreError::InvalidRun(format!("{}: {error}", settings.map)))?;
            let mut definition = floor
                .content
                .start_map()
                .cloned()
                .ok_or_else(|| CoreError::InvalidRun("generated map missing".into()))?;
            let cells = spawn_cells(&definition, settings, floor.entrance, floor.exit);
            let mut placed = Vec::new();
            let enemies = self
                .enemies
                .iter()
                .chain((number == 1).then_some(&self.guardian));
            for enemy in enemies {
                let position = cells
                    .iter()
                    .copied()
                    .find(|cell| {
                        placed
                            .iter()
                            .all(|other| cell.chebyshev_distance(*other) >= 3)
                    })
                    .ok_or_else(|| {
                        CoreError::InvalidRun(format!(
                            "{}: no spaced enemy spawn available",
                            settings.map
                        ))
                    })?;
                placed.push(position);
                definition.spawns.push(SpawnDefinition {
                    actor: enemy.clone(),
                    x: u16::try_from(position.x)
                        .map_err(|_| CoreError::InvalidRun("spawn overflow".into()))?,
                    y: u16::try_from(position.y)
                        .map_err(|_| CoreError::InvalidRun("spawn overflow".into()))?,
                });
            }
            add_door(
                &mut definition,
                &cells,
                &placed,
                settings,
                &self.door,
                floor.exit,
            )?;
            content = floor.content.with_start_map(definition)?;
            plans.push(FloorPlan {
                definition: settings.map.clone(),
                entrance: floor.entrance,
                exit: floor.exit,
            });
        }

        if include_approach && let Some(approach) = &self.approach {
            if plans.iter().any(|plan| plan.definition == approach.map.id) {
                return Err(CoreError::InvalidRun("approach map must be unique".into()));
            }
            content = content.with_start_map(approach.map.clone())?;
            let preview = CoreState::from_content_with_seed(content.clone(), seed)?;
            if !crate::crypt::connected(&preview, approach.entrance, approach.exit) {
                return Err(CoreError::InvalidRun(
                    "approach must be connected and its exit reachable".into(),
                ));
            }
            plans.insert(
                0,
                FloorPlan {
                    definition: approach.map.id.clone(),
                    entrance: approach.entrance,
                    exit: approach.exit,
                },
            );
        }
        let first = content
            .map(&plans[0].definition)
            .cloned()
            .ok_or_else(|| CoreError::MissingDefinition(plans[0].definition.clone()))?;
        content = content.with_start_map(first)?;
        let state = CoreState::from_dungeon(content, seed, plans, &self.guardian)?;
        if include_approach && self.approach.is_some() {
            state.with_items_without_first_reward(self.items.clone())
        } else {
            state.with_items(self.items.clone())
        }
    }

    fn validate(&self, base: &ContentPack) -> Result<(), CoreError> {
        if self.format != 1
            || self.floors.len() != 2
            || self.enemies.len() > 8
            || self.enemies.contains(&self.guardian)
        {
            return Err(CoreError::InvalidRun("playtest requires format 1, two floors and at most eight non-guardian enemies per floor".into()));
        }
        for id in self.enemies.iter().chain([&self.guardian]) {
            if base
                .actor(id)
                .is_none_or(|actor| actor.behavior != ActorBehavior::Hostile)
            {
                return Err(CoreError::InvalidRun(format!(
                    "crypt enemy `{id}` must be hostile"
                )));
            }
        }
        if base.tile(&self.door).is_none_or(|tile| !tile.door) {
            return Err(CoreError::InvalidRun(format!(
                "`{}` must be a door",
                self.door
            )));
        }
        Ok(())
    }

    /// Creates the runtime dispatcher used to regenerate layouts on retry.
    #[must_use]
    pub fn factory(self, content: ContentPack) -> RunFactory {
        RunFactory::new(move |seed| self.build(&content, seed))
    }
}

fn spawn_cells(
    definition: &MapDefinition,
    settings: &CryptSettings,
    entrance: Position,
    exit: Position,
) -> Vec<Position> {
    let mut cells: Vec<_> = (0..definition.height())
        .flat_map(|y| {
            (0..definition.width()).filter_map(move |x| {
                (definition.tile_at(x, y) == Some(&settings.floor))
                    .then_some(Position::new(i32::from(x), i32::from(y)))
            })
        })
        .filter(|cell| *cell != entrance && *cell != exit && cell.chebyshev_distance(entrance) >= 6)
        .collect();
    cells.sort_by_key(|cell| (std::cmp::Reverse(cell.chebyshev_distance(entrance)), *cell));
    cells
}

fn add_door(
    definition: &mut MapDefinition,
    cells: &[Position],
    placed: &[Position],
    settings: &CryptSettings,
    door: &ContentId,
    exit: Position,
) -> Result<(), CoreError> {
    let candidate = cells.iter().copied().find(|cell| {
        if placed.contains(cell) || cell.chebyshev_distance(exit) < 3 {
            return false;
        }
        let at = |dx, dy| {
            u16::try_from(cell.x + dx)
                .ok()
                .zip(u16::try_from(cell.y + dy).ok())
                .and_then(|(x, y)| definition.tile_at(x, y))
        };
        (at(-1, 0) == Some(&settings.floor)
            && at(1, 0) == Some(&settings.floor)
            && at(0, -1) == Some(&settings.wall)
            && at(0, 1) == Some(&settings.wall))
            || (at(0, -1) == Some(&settings.floor)
                && at(0, 1) == Some(&settings.floor)
                && at(-1, 0) == Some(&settings.wall)
                && at(1, 0) == Some(&settings.wall))
    });
    if let Some(cell) = candidate {
        let y = usize::try_from(cell.y)
            .map_err(|_| CoreError::InvalidRun("door row overflow".into()))?;
        let x = usize::try_from(cell.x)
            .map_err(|_| CoreError::InvalidRun("door column overflow".into()))?;
        let mut row: Vec<_> = definition.rows[y].chars().collect();
        row[x] = '+';
        definition.rows[y] = row.into_iter().collect();
        definition.legend.push(('+', door.clone()));
    }
    Ok(())
}
