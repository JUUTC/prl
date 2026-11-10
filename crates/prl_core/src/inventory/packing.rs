//! Free atomic organization; transfers and equipment retain their normal costs.

use super::{BACKPACK, GridCell, Inventory, ItemId};
use crate::{Command, CoreState, RejectReason, state::Action};
use serde::{Deserialize, Serialize};

/// Explicit placement of a carried item within the same backpack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceItem {
    /// Persistent item to reposition.
    pub item: ItemId,
    /// Requested top-left cell.
    pub cell: GridCell,
    /// Turn the definition's footprint 90 degrees.
    pub rotated: bool,
}

impl Inventory {
    /// Validates a proposed same-backpack placement without changing any state.
    ///
    /// # Errors
    ///
    /// Rejects missing/non-backpack items and out-of-bounds or overlapping shapes.
    pub fn check_placement(&self, placement: PlaceItem) -> Result<(), RejectReason> {
        let item = self
            .items
            .get(&placement.item)
            .ok_or(RejectReason::InvalidItem)?;
        if item.container != BACKPACK {
            return Err(RejectReason::ItemInaccessible);
        }
        let mut candidate = item.clone();
        candidate.rotated = placement.rotated;
        let shape = self
            .footprint(&candidate)
            .ok_or(RejectReason::InvalidItem)?;
        if u16::from(placement.cell.x) + u16::from(shape.0) > 10
            || u16::from(placement.cell.y) + u16::from(shape.1) > 6
        {
            return Err(RejectReason::OutOfBounds);
        }
        if !self.fits(item.id, shape, placement.cell) {
            return Err(RejectReason::BackpackFull);
        }
        Ok(())
    }

    fn placed(&self, placement: PlaceItem) -> Result<Self, RejectReason> {
        self.check_placement(placement)?;
        let mut next = self.clone();
        let item = next
            .items
            .get_mut(&placement.item)
            .ok_or(RejectReason::InvalidItem)?;
        item.cell = Some(placement.cell);
        item.rotated = placement.rotated;
        Ok(next)
    }

    fn organized(&self) -> Result<Self, RejectReason> {
        let mut order = self
            .items
            .values()
            .filter(|item| item.container == BACKPACK)
            .map(|item| {
                let (width, height) = self.footprint(item).ok_or(RejectReason::InvalidItem)?;
                Ok((
                    std::cmp::Reverse(u16::from(width) * u16::from(height)),
                    std::cmp::Reverse(width.max(height)),
                    item.id,
                ))
            })
            .collect::<Result<Vec<_>, RejectReason>>()?;
        order.sort_unstable();
        let mut next = self.clone();
        next.items.retain(|_, item| item.container != BACKPACK);
        for (_, _, id) in order {
            let mut item = self
                .items
                .get(&id)
                .cloned()
                .ok_or(RejectReason::InvalidItem)?;
            // Base orientation first; placement depends only on public shape and ID.
            let placement = [false, true]
                .into_iter()
                .find_map(|rotated| {
                    item.rotated = rotated;
                    let shape = next.footprint(&item)?;
                    next.first_fit(id, shape).map(|cell| (cell, rotated))
                })
                .ok_or(RejectReason::BackpackFull)?;
            item.cell = Some(placement.0);
            item.rotated = placement.1;
            next.items.insert(id, item);
        }
        Ok(next)
    }
}

impl CoreState {
    pub(crate) fn plan_organization(&self, command: Command) -> Result<Action, RejectReason> {
        let inventory = self.inventory.as_ref().ok_or(RejectReason::InvalidItem)?;
        let (next, message) = match command {
            Command::PlaceItem(placement) => (
                inventory.placed(placement)?,
                "Backpack placement updated. No time spent.",
            ),
            Command::Organize => (inventory.organized()?, "Backpack organized. No time spent."),
            _ => return Err(RejectReason::InvalidItem),
        };
        Ok(Action::Items {
            inventory: Box::new(next),
            cost: 0,
            healing: 0,
            message: message.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommandResult, EQUIPMENT, OutcomeKind, TransferItems};

    fn carried() -> CoreState {
        let mut core = crate::test_support::dungeon();
        while let Some(command) = core.pickup_command() {
            assert!(matches!(
                core.issue(command),
                CommandResult::Accepted { .. }
            ));
        }
        core
    }

    #[test]
    fn placement_is_free_atomic_and_rotated_shape_survives_transfer_and_restore() {
        let mut core = carried();
        let sword = core
            .inventory()
            .unwrap()
            .items()
            .find(|item| item.definition.name() == "iron_sword")
            .unwrap()
            .id;
        let before = (
            core.clock(),
            core.rng_position(),
            core.actors().cloned().collect::<Vec<_>>(),
        );
        let place = Command::PlaceItem(PlaceItem {
            item: sword,
            cell: GridCell { x: 7, y: 5 },
            rotated: true,
        });
        assert_eq!(core.item_action_cost(place), Ok(0));
        assert!(matches!(core.issue(place), CommandResult::Accepted { .. }));
        assert_eq!(
            (
                core.clock(),
                core.rng_position(),
                core.actors().cloned().collect()
            ),
            before
        );
        assert!(core.outcomes().iter().any(|outcome|
                matches!(outcome.kind, OutcomeKind::CommandAccepted { command, cost: 0 } if command == place)));
        let bag = core.inventory().unwrap();
        assert_eq!(
            bag.footprint(bag.items().find(|item| item.id == sword).unwrap()),
            Some((3, 1))
        );
        let inventory = bag.clone();
        assert!(matches!(
            core.issue(Command::PlaceItem(PlaceItem {
                item: sword,
                cell: GridCell { x: 8, y: 5 },
                rotated: true
            })),
            CommandResult::Rejected {
                reason: RejectReason::OutOfBounds,
                ..
            }
        ));
        assert_eq!(core.inventory(), Some(&inventory));
        assert_eq!(core.clock(), before.0);
        let mut restored = CoreState::restore(
            crate::CoreSnapshot::from_ron(&core.snapshot().to_ron().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(restored.snapshot(), core.snapshot());
        for destination in [EQUIPMENT, BACKPACK] {
            let command = Command::Transfer(TransferItems {
                item: sword,
                destination,
                cell: None,
            });
            assert_eq!(core.issue(command), restored.issue(command));
            assert_eq!(core.snapshot(), restored.snapshot());
            assert!(
                core.inventory()
                    .unwrap()
                    .items()
                    .find(|item| item.id == sword)
                    .unwrap()
                    .rotated
            );
        }
    }

    #[test]
    fn organize_is_deterministic_free_idempotent_and_preserves_every_item() {
        let mut core = carried();
        let bag = core.inventory().unwrap().clone();
        let before = (core.clock(), core.rng_position(), bag.carried_weight());
        let mut duplicate = CoreState::restore(core.snapshot()).unwrap();
        assert_eq!(
            core.issue(Command::Organize),
            duplicate.issue(Command::Organize)
        );
        assert_eq!(core.snapshot(), duplicate.snapshot());
        let after = core.inventory().unwrap().clone();
        for item in bag.items() {
            let organized = after.items().find(|other| other.id == item.id).unwrap();
            assert_eq!(organized.definition, item.definition);
            assert_eq!(organized.container, item.container);
            assert_eq!(organized.identified, item.identified);
        }
        assert_eq!(bag.items().count(), after.items().count());
        core.issue(Command::Organize);
        assert_eq!(core.inventory(), Some(&after));
        assert_eq!(
            (core.clock(), core.rng_position(), after.carried_weight()),
            before
        );
    }

    #[test]
    fn placement_cannot_make_ground_pickup_or_equipment_removal_free() {
        let mut core = crate::test_support::dungeon();
        let ground = core.inventory().unwrap().items().next().unwrap().id;
        let before = core.inventory().unwrap().clone();
        let command = Command::PlaceItem(PlaceItem {
            item: ground,
            cell: GridCell { x: 0, y: 0 },
            rotated: false,
        });
        assert!(matches!(
            core.issue(command),
            CommandResult::Rejected {
                reason: RejectReason::ItemInaccessible,
                ..
            }
        ));
        assert_eq!(core.inventory(), Some(&before));
        assert_eq!(core.clock().player_actions(), 0);
    }

    #[test]
    fn failed_best_effort_organize_keeps_an_existing_valid_layout_whole() {
        let mut core = with_shapes(&[
            (4, 1, 5, 5, false),
            (2, 0, 2, 2, false),
            (0, 2, 4, 4, false),
            (9, 3, 1, 3, false),
            (4, 0, 4, 1, false),
        ]);
        let before = (
            core.inventory().unwrap().clone(),
            core.clock(),
            core.rng_position(),
        );
        assert!(matches!(
            core.issue(Command::Organize),
            CommandResult::Rejected {
                reason: RejectReason::BackpackFull,
                ..
            }
        ));
        assert_eq!(
            (
                core.inventory().unwrap().clone(),
                core.clock(),
                core.rng_position()
            ),
            before
        );
    }

    #[test]
    fn organize_rotates_when_base_orientation_cannot_fit() {
        let mut core = with_shapes(&[(0, 0, 6, 4, false), (6, 0, 4, 3, false), (0, 4, 2, 5, true)]);
        let before = core.clock();
        assert!(matches!(
            core.issue(Command::Organize),
            CommandResult::Accepted { .. }
        ));
        core.inventory().unwrap().validate().unwrap();
        let item = core.inventory().unwrap().items().last().unwrap();
        assert!(item.rotated);
        assert_eq!(item.cell, Some(GridCell { x: 0, y: 4 }));
        assert_eq!(core.clock(), before);
    }

    fn with_shapes(layout: &[(u8, u8, u8, u8, bool)]) -> CoreState {
        let mut core = carried();
        let bag = core.inventory.as_mut().unwrap();
        let template = bag
            .items
            .values()
            .find(|item| item.definition.name() == "iron_sword")
            .unwrap()
            .clone();
        let definition = bag.definition(&template).unwrap().clone();
        bag.items.clear();
        for &(x, y, width, height, rotated) in layout {
            let mut item = template.clone();
            item.id = ItemId(bag.next_item);
            bag.next_item += 1;
            let mut shape = definition.clone();
            shape.id = format!("test:shape_{}", item.id.0).parse().unwrap();
            shape.width = width;
            shape.height = height;
            shape.weight = 1;
            item.definition = shape.id.clone();
            item.cell = Some(GridCell { x, y });
            item.rotated = rotated;
            bag.catalog.items.push(shape);
            bag.items.insert(item.id, item);
        }
        bag.validate().unwrap();
        core
    }
}
