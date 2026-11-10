//! Atomic grid/equipment transfers, known comparisons and consumable rules.

use std::collections::{BTreeMap, BTreeSet};

use prl_content::{
    EquipSlot, ItemBonus, ItemCatalog, ItemDefinition, ItemKind, ItemPredicate, Rarity,
};
use serde::{Deserialize, Serialize};

use crate::{Actor, CombatStats, CoreError, CoreState, MapId, Position, RejectReason};

/// Logical persistent item identity, scoped to a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemId(pub u64);

/// Logical container identity; not an ECS entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContainerId(pub u64);

/// One item's local grid placement, not a gameplay world position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridCell {
    /// Zero-based column.
    pub x: u8,
    /// Zero-based row.
    pub y: u8,
}

/// Container access/ownership for the slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Container {
    /// Carried backpack with a 10x6 grid.
    Backpack,
    /// Player's equipment slots (separate from bag grid cells).
    Equipment,
    /// Loose items at a map cell.
    Ground {
        /// Owning logical map.
        map: MapId,
        /// World cell.
        position: Position,
    },
}

/// A single, non-stacked slice item. General stack splitting is deferred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    /// Persistent item identity.
    pub id: ItemId,
    /// Definition supplying shape, weight and effects.
    pub definition: prl_content::ContentId,
    /// Whether hidden affixes are known.
    pub identified: bool,
    /// Exactly one owning container.
    pub container: ContainerId,
    /// Grid origin for a backpack item; absent in ground/equipment containers.
    pub cell: Option<GridCell>,
}

/// Explicit item transfer; validation completes before any mutation or time cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferItems {
    /// Item to move.
    pub item: ItemId,
    /// Destination container.
    pub destination: ContainerId,
    /// Requested backpack origin, or deterministic first-fit when absent.
    pub cell: Option<GridCell>,
}

/// Persisted item ownership, definitions and knowledge for the P3 slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub(crate) catalog: ItemCatalog,
    pub(crate) items: BTreeMap<ItemId, Item>,
    pub(crate) containers: BTreeMap<ContainerId, Container>,
    pub(crate) next_item: u64,
    pub(crate) next_container: u64,
    pub(crate) appearances: BTreeMap<prl_content::ContentId, String>,
}

/// Character backpack identity within the run's container allocator.
pub const BACKPACK: ContainerId = ContainerId(1);
/// Character equipment identity within the run's container allocator.
pub const EQUIPMENT: ContainerId = ContainerId(2);

/// Pure loadout evaluator used for actual combat and known-only comparisons.
///
/// The actor's definition-supplied values are P3's base inputs (Q-019);
/// only P4 replaces them with trained stats/skills. Unknown affixes apply in
/// actual combat but are omitted when `known_only` is true.
#[must_use]
pub fn derive_stats<'a>(
    actor: &Actor,
    loadout: impl IntoIterator<Item = (&'a ItemDefinition, bool)>,
    known_only: bool,
) -> CombatStats {
    let mut damage = i32::from(actor.combat.damage);
    let mut armor = i32::from(actor.combat.armor);
    let mut accuracy = i32::from(actor.combat.accuracy);
    let mut evasion = i32::from(actor.combat.evasion);
    for (definition, identified) in loadout {
        for bonus in [
            Some(definition.base),
            (!known_only || identified).then_some(definition.affix),
        ]
        .into_iter()
        .flatten()
        {
            damage = damage.saturating_add(i32::from(bonus.damage));
            armor = armor.saturating_add(i32::from(bonus.armor));
            accuracy = accuracy.saturating_add(i32::from(bonus.accuracy));
            evasion = evasion.saturating_add(i32::from(bonus.evasion));
        }
    }
    CombatStats {
        damage: u16::try_from(damage.clamp(1, i32::from(u16::MAX))).unwrap_or(1),
        armor: u16::try_from(armor.clamp(0, i32::from(u16::MAX))).unwrap_or(0),
        accuracy: i16::try_from(accuracy.clamp(i32::from(i16::MIN), i32::from(i16::MAX)))
            .unwrap_or(0),
        evasion: i16::try_from(evasion.clamp(i32::from(i16::MIN), i32::from(i16::MAX)))
            .unwrap_or(0),
        crit_multiplier_pct: actor.combat.crit_multiplier_pct,
    }
}

impl Inventory {
    /// Items in stable instance-ID order.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.items.values()
    }

    /// Read-only container access information.
    #[must_use]
    pub fn container(&self, id: ContainerId) -> Option<&Container> {
        self.containers.get(&id)
    }

    /// Looks up an item's base definition.
    #[must_use]
    pub fn definition(&self, item: &Item) -> Option<&ItemDefinition> {
        self.catalog
            .items
            .iter()
            .find(|definition| definition.id == item.definition)
    }

    /// Known name without leaking unidentified affixes.
    #[must_use]
    pub fn label(&self, item: &Item) -> String {
        let Some(definition) = self.definition(item) else {
            return "Invalid item definition".into();
        };
        let rarity = match definition.rarity {
            Rarity::Normal => "o",
            Rarity::Magic => "*",
        };
        if !item.identified {
            format!(
                "{rarity} {} (?)",
                self.appearances
                    .get(&item.definition)
                    .unwrap_or(&definition.name)
            )
        } else if definition.affix_name.is_empty() {
            format!("{rarity} {}", definition.name)
        } else {
            format!("{rarity} {} {}", definition.name, definition.affix_name)
        }
    }

    /// Filter label visibility/emphasis; inspection may bypass it.
    #[must_use]
    pub fn ground_label(&self, item: &Item) -> (bool, bool) {
        let Some(definition) = self.definition(item) else {
            return (true, false);
        };
        self.catalog
            .default_filter
            .iter()
            .find(|rule| rule.predicate.matches(definition, item.identified))
            .map_or((true, false), |rule| (rule.show, rule.emphasize))
    }

    /// Total carried/equipped weight in grams.
    #[must_use]
    pub fn carried_weight(&self) -> u64 {
        self.items
            .values()
            .filter(|item| matches!(item.container, BACKPACK | EQUIPMENT))
            .filter_map(|item| self.definition(item))
            .map(|definition| u64::from(definition.weight))
            .sum()
    }

    /// Provisional carry limit in grams.
    #[must_use]
    pub const fn carry_capacity(&self) -> u32 {
        self.catalog.carry_capacity
    }

    fn equipped(&self) -> impl Iterator<Item = (&ItemDefinition, bool)> {
        self.items
            .values()
            .filter(|item| item.container == EQUIPMENT)
            .filter_map(|item| {
                self.definition(item)
                    .map(|definition| (definition, item.identified))
            })
    }

    /// Known-only hypothetical comparison, without mutating state or time.
    #[must_use]
    pub fn compare(&self, actor: &Actor, candidate: ItemId) -> Option<(CombatStats, CombatStats)> {
        let item = self.items.get(&candidate)?;
        let definition = self.definition(item)?;
        let ItemKind::Gear(slot) = definition.kind else {
            return None;
        };
        let current = derive_stats(actor, self.equipped(), true);
        let others = self
            .equipped()
            .filter(|(definition, _)| definition.kind != ItemKind::Gear(slot));
        Some((
            current,
            derive_stats(actor, others.chain([(definition, item.identified)]), true),
        ))
    }

    fn fits(&self, id: ItemId, definition: &ItemDefinition, cell: GridCell) -> bool {
        if u16::from(cell.x) + u16::from(definition.width) > 10
            || u16::from(cell.y) + u16::from(definition.height) > 6
        {
            return false;
        }
        self.items
            .values()
            .filter(|item| item.id != id && item.container == BACKPACK)
            .all(|item| {
                let (Some(other), Some(origin)) = (self.definition(item), item.cell) else {
                    return false;
                };
                u16::from(cell.x) + u16::from(definition.width) <= u16::from(origin.x)
                    || u16::from(origin.x) + u16::from(other.width) <= u16::from(cell.x)
                    || u16::from(cell.y) + u16::from(definition.height) <= u16::from(origin.y)
                    || u16::from(origin.y) + u16::from(other.height) <= u16::from(cell.y)
            })
    }

    fn first_fit(&self, id: ItemId, definition: &ItemDefinition) -> Option<GridCell> {
        (0..6)
            .flat_map(|y| (0..10).map(move |x| GridCell { x, y }))
            .find(|cell| self.fits(id, definition, *cell))
    }

    pub(crate) fn transferred(
        &self,
        transfer: TransferItems,
        map: MapId,
        position: Position,
    ) -> Result<(Self, u16), RejectReason> {
        let item = self
            .items
            .get(&transfer.item)
            .ok_or(RejectReason::InvalidItem)?;
        let definition = self.definition(item).ok_or(RejectReason::InvalidItem)?;
        let source = self
            .containers
            .get(&item.container)
            .ok_or(RejectReason::InvalidItem)?;
        let destination = self
            .containers
            .get(&transfer.destination)
            .ok_or(RejectReason::InvalidItem)?;
        let accessible = |container: &Container| match container {
            Container::Ground {
                map: owner,
                position: cell,
            } => *owner == map && *cell == position,
            _ => true,
        };
        if !accessible(source) || !accessible(destination) || item.container == transfer.destination
        {
            return Err(RejectReason::ItemInaccessible);
        }
        if !matches!(destination, Container::Backpack) && transfer.cell.is_some() {
            return Err(RejectReason::InvalidItem);
        }
        let mut next = self.clone();
        let mut cost = if matches!(source, Container::Equipment)
            && definition.kind == ItemKind::Gear(EquipSlot::Body)
        {
            300
        } else {
            100
        };
        let cell = match destination {
            Container::Backpack => {
                if matches!(source, Container::Ground { .. }) {
                    cost = 50;
                }
                if matches!(source, Container::Ground { .. })
                    && self.carried_weight() + u64::from(definition.weight)
                        > u64::from(self.catalog.carry_capacity)
                {
                    return Err(RejectReason::TooHeavy);
                }
                let cell = transfer
                    .cell
                    .or_else(|| self.first_fit(item.id, definition))
                    .ok_or(RejectReason::BackpackFull)?;
                if !self.fits(item.id, definition, cell) {
                    return Err(RejectReason::BackpackFull);
                }
                Some(cell)
            }
            Container::Equipment => {
                if item.container != BACKPACK {
                    return Err(RejectReason::ItemInaccessible);
                }
                let ItemKind::Gear(slot) = definition.kind else {
                    return Err(RejectReason::InvalidItem);
                };
                if slot == EquipSlot::Body {
                    cost = 300;
                }
                // Remove the candidate before finding room for the displaced item.
                next.items.remove(&item.id);
                let displaced = self.items.values().find(|other| {
                    other.container == EQUIPMENT
                        && self
                            .definition(other)
                            .is_some_and(|definition| definition.kind == ItemKind::Gear(slot))
                });
                if let Some(displaced) = displaced {
                    let old_definition = self
                        .definition(displaced)
                        .ok_or(RejectReason::InvalidItem)?;
                    let cell = next
                        .first_fit(displaced.id, old_definition)
                        .ok_or(RejectReason::BackpackFull)?;
                    let mut old = displaced.clone();
                    old.container = BACKPACK;
                    old.cell = Some(cell);
                    next.items.insert(old.id, old);
                }
                None
            }
            Container::Ground { .. } => {
                if !matches!(source, Container::Backpack | Container::Equipment) {
                    return Err(RejectReason::ItemInaccessible);
                }
                None
            }
        };
        let mut moved = item.clone();
        moved.container = transfer.destination;
        moved.cell = cell;
        next.items.insert(moved.id, moved);
        Ok((next, cost))
    }

    pub(crate) fn validate(&self) -> Result<(), CoreError> {
        let invalid = |reason: &str| CoreError::InvalidRun(format!("items: {reason}"));
        if self.catalog.format != 1
            || self.catalog.items.is_empty()
            || self.catalog.items.len() > 128
            || self.catalog.carry_capacity == 0
            || self.containers.get(&BACKPACK) != Some(&Container::Backpack)
            || self.containers.get(&EQUIPMENT) != Some(&Container::Equipment)
            || self.items.len() > 4096
            || self.containers.len() > 4096
            || self.catalog.supplies.len() > 128
            || self.catalog.rewards.len() > 128
        {
            return Err(invalid("invalid format, capacity or containers"));
        }
        let mut definitions = BTreeSet::new();
        for definition in &self.catalog.items {
            if !definitions.insert(&definition.id)
                || definition.name.trim().is_empty()
                || !(1..=10).contains(&definition.width)
                || !(1..=6).contains(&definition.height)
                || (definition.rarity == Rarity::Normal && definition.affix != ItemBonus::default())
                || matches!(definition.kind, ItemKind::Healing { amount: 0 })
                || (!matches!(definition.kind, ItemKind::Gear(_))
                    && definition.appearances.is_empty())
            {
                return Err(invalid("duplicate or invalid item definition"));
            }
            if !matches!(definition.kind, ItemKind::Gear(_))
                && self
                    .appearances
                    .get(&definition.id)
                    .is_none_or(|appearance| !definition.appearances.contains(appearance))
            {
                return Err(invalid("unknown consumable appearance"));
            }
        }
        if !self
            .catalog
            .default_filter
            .last()
            .is_some_and(|rule| rule.predicate == ItemPredicate::Always)
        {
            return Err(invalid("Default filter must end with Always"));
        }
        for id in self.catalog.supplies.iter().chain(&self.catalog.rewards) {
            if !definitions.contains(id) {
                return Err(invalid("unknown supply or reward definition"));
            }
        }
        if !self.valid_filter(&definitions) {
            return Err(invalid(
                "Default filter is oversized or references unknown items",
            ));
        }
        let mut slots = BTreeSet::new();
        for (id, item) in &self.items {
            let definition = self
                .definition(item)
                .ok_or_else(|| invalid("unknown item definition"))?;
            if *id != item.id
                || id.0 == 0
                || id.0 >= self.next_item
                || (definition.rarity == Rarity::Normal
                    && matches!(definition.kind, ItemKind::Gear(_))
                    && !item.identified)
            {
                return Err(invalid("item identity or knowledge invalid"));
            }
            match self.containers.get(&item.container) {
                Some(Container::Backpack) => {
                    if !item
                        .cell
                        .is_some_and(|cell| self.fits(*id, definition, cell))
                    {
                        return Err(invalid("backpack bounds/overlap invalid"));
                    }
                }
                Some(Container::Equipment) => {
                    let ItemKind::Gear(slot) = definition.kind else {
                        return Err(invalid("non-gear equipped"));
                    };
                    if item.cell.is_some() || !slots.insert(slot) {
                        return Err(invalid("equipment occupancy invalid"));
                    }
                }
                Some(Container::Ground { .. }) if item.cell.is_none() => {}
                _ => return Err(invalid("item container/placement invalid")),
            }
        }
        if self.containers.iter().any(|(id, container)| {
            id.0 == 0
                || id.0 >= self.next_container
                || (*container == Container::Backpack && *id != BACKPACK)
                || (*container == Container::Equipment && *id != EQUIPMENT)
        }) || self.carried_weight() > u64::from(self.catalog.carry_capacity)
        {
            return Err(invalid("allocator or carry weight invalid"));
        }
        Ok(())
    }

    fn valid_filter(&self, definitions: &BTreeSet<&prl_content::ContentId>) -> bool {
        let mut pending: Vec<_> = self
            .catalog
            .default_filter
            .iter()
            .map(|rule| &rule.predicate)
            .collect();
        let mut count = 0;
        while let Some(predicate) = pending.pop() {
            count += 1;
            if count > 128 || pending.len() > 128 {
                return false;
            }
            match predicate {
                ItemPredicate::Definition(id) if !definitions.contains(id) => return false,
                ItemPredicate::All(children) | ItemPredicate::Any(children) => {
                    pending.extend(children);
                }
                _ => {}
            }
        }
        true
    }
}

impl CoreState {
    pub(crate) fn plan_drop(&self, id: ItemId) -> Result<crate::state::Action, RejectReason> {
        let inventory = self.inventory.as_ref().ok_or(RejectReason::InvalidItem)?;
        let player = self.player().ok_or(RejectReason::InvalidItem)?;
        let ground = Container::Ground {
            map: self.map.id(),
            position: player.position(),
        };
        let mut next = inventory.clone();
        let destination = if let Some((id, _)) = next
            .containers
            .iter()
            .find(|(_, container)| **container == ground)
        {
            *id
        } else {
            let id = ContainerId(next.next_container);
            next.next_container += 1;
            next.containers.insert(id, ground);
            id
        };
        let label = next
            .items
            .get(&id)
            .map(|item| next.label(item))
            .ok_or(RejectReason::InvalidItem)?;
        let (next, cost) = next.transferred(
            TransferItems {
                item: id,
                destination,
                cell: None,
            },
            self.map.id(),
            player.position(),
        )?;
        Ok(crate::state::Action::Items {
            inventory: Box::new(next),
            cost,
            healing: 0,
            message: format!("You drop {label}."),
        })
    }

    pub(crate) fn plan_quaff(&self, id: ItemId) -> Result<crate::state::Action, RejectReason> {
        let inventory = self.inventory.as_ref().ok_or(RejectReason::InvalidItem)?;
        let item = inventory
            .items
            .get(&id)
            .filter(|item| item.container == BACKPACK)
            .ok_or(RejectReason::ItemInaccessible)?;
        let definition = inventory
            .definition(item)
            .ok_or(RejectReason::InvalidItem)?;
        let ItemKind::Healing { amount } = definition.kind else {
            return Err(RejectReason::InvalidItem);
        };
        let player = self.player().ok_or(RejectReason::InvalidItem)?;
        if player.hp() == player.max_hp() {
            return Err(RejectReason::NoEffect);
        }
        let mut next = inventory.clone();
        next.items.remove(&id);
        for other in next
            .items
            .values_mut()
            .filter(|other| other.definition == item.definition)
        {
            other.identified = true;
        }
        Ok(crate::state::Action::Items {
            inventory: Box::new(next),
            cost: 100,
            healing: amount,
            message: format!(
                "You drink {} and recover up to {amount} HP.",
                definition.name
            ),
        })
    }

    pub(crate) fn plan_identify(
        &self,
        scroll: ItemId,
        target: ItemId,
    ) -> Result<crate::state::Action, RejectReason> {
        let inventory = self.inventory.as_ref().ok_or(RejectReason::InvalidItem)?;
        let scroll_item = inventory
            .items
            .get(&scroll)
            .filter(|item| item.container == BACKPACK)
            .ok_or(RejectReason::ItemInaccessible)?;
        if inventory
            .definition(scroll_item)
            .is_none_or(|definition| definition.kind != ItemKind::Identify)
            || scroll == target
        {
            return Err(RejectReason::InvalidItem);
        }
        let target_item = inventory
            .items
            .get(&target)
            .filter(|item| matches!(item.container, BACKPACK | EQUIPMENT))
            .ok_or(RejectReason::ItemInaccessible)?;
        if target_item.identified {
            return Err(RejectReason::NoEffect);
        }
        let mut next = inventory.clone();
        next.items.remove(&scroll);
        for item in next.items.values_mut() {
            if item.id == target
                || item.definition == scroll_item.definition
                || (item.definition == target_item.definition
                    && inventory
                        .definition(target_item)
                        .is_some_and(|definition| !matches!(definition.kind, ItemKind::Gear(_))))
            {
                item.identified = true;
            }
        }
        let label = next
            .items
            .get(&target)
            .map(|item| next.label(item))
            .ok_or(RejectReason::InvalidItem)?;
        Ok(crate::state::Action::Items {
            inventory: Box::new(next),
            cost: 100,
            healing: 0,
            message: format!("You identify {label}."),
        })
    }
}

impl CoreState {
    /// Validates an item action through the ordinary planner without committing.
    ///
    /// Presentation must only preview effects the player knows. In particular,
    /// calling this for an unknown potion can reveal its full-health rejection;
    /// submit an explicit attempt instead of exposing that hidden availability.
    ///
    /// # Errors
    ///
    /// Returns the ordinary rejection for invalid/inaccessible item actions,
    /// or [`RejectReason::InvalidItem`] for non-item commands.
    pub fn item_action_cost(&self, command: crate::Command) -> Result<u16, RejectReason> {
        if self.is_player_dead() {
            return Err(RejectReason::PlayerDead);
        }
        let action = match command {
            crate::Command::Transfer(transfer) => {
                let player = self.player().ok_or(RejectReason::InvalidItem)?;
                let inventory = self.inventory.as_ref().ok_or(RejectReason::InvalidItem)?;
                let (_, cost) =
                    inventory.transferred(transfer, self.map.id(), player.position())?;
                return Ok(cost);
            }
            crate::Command::Drop(item) => self.plan_drop(item)?,
            crate::Command::Quaff(item) => self.plan_quaff(item)?,
            crate::Command::Identify { scroll, target } => self.plan_identify(scroll, target)?,
            _ => return Err(RejectReason::InvalidItem),
        };
        let crate::state::Action::Items { cost, .. } = action else {
            return Err(RejectReason::InvalidItem);
        };
        Ok(cost)
    }

    /// Read-only item ownership; absent in the P2 arena fixture.
    #[must_use]
    pub const fn inventory(&self) -> Option<&Inventory> {
        self.inventory.as_ref()
    }

    /// Adds deterministic ground supplies/rewards to each floor's endpoints.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidRun`] for invalid catalog or non-dungeon runs.
    pub fn with_items(mut self, catalog: ItemCatalog) -> Result<Self, CoreError> {
        let dungeon = self
            .dungeon
            .as_ref()
            .ok_or_else(|| CoreError::InvalidRun("items require dungeon endpoints".into()))?;
        let mut inventory = Inventory {
            catalog,
            items: BTreeMap::new(),
            containers: BTreeMap::from([
                (BACKPACK, Container::Backpack),
                (EQUIPMENT, Container::Equipment),
            ]),
            next_item: 1,
            next_container: 3,
            appearances: BTreeMap::new(),
        };
        for definition in &inventory.catalog.items {
            if !definition.appearances.is_empty() {
                let mut hasher = blake3::Hasher::new_derive_key("PRL consumable appearance v1");
                hasher.update(&self.seed.to_le_bytes());
                hasher.update(definition.id.to_string().as_bytes());
                let bytes = hasher.finalize();
                let index = usize::from(bytes.as_bytes()[0]) % definition.appearances.len();
                inventory
                    .appearances
                    .insert(definition.id.clone(), definition.appearances[index].clone());
            }
        }
        inventory.validate()?;
        for (index, plan) in dungeon.plans.iter().enumerate() {
            let map = MapId::new(
                u64::try_from(index)
                    .map_err(|_| CoreError::InvalidRun("item floor overflow".into()))?
                    + 1,
            );
            for (position, definitions) in [
                (plan.entrance, &inventory.catalog.supplies),
                (plan.exit, &inventory.catalog.rewards),
            ] {
                let container = ContainerId(inventory.next_container);
                inventory.next_container += 1;
                inventory
                    .containers
                    .insert(container, Container::Ground { map, position });
                for id in definitions {
                    let definition = inventory
                        .catalog
                        .items
                        .iter()
                        .find(|definition| &definition.id == id)
                        .ok_or_else(|| CoreError::InvalidRun(format!("unknown drop `{id}`")))?;
                    let item_id = ItemId(inventory.next_item);
                    inventory.next_item += 1;
                    inventory.items.insert(
                        item_id,
                        Item {
                            id: item_id,
                            definition: id.clone(),
                            identified: definition.rarity == Rarity::Normal
                                && matches!(definition.kind, ItemKind::Gear(_)),
                            container,
                            cell: None,
                        },
                    );
                }
            }
        }
        inventory.validate()?;
        self.inventory = Some(inventory);
        Ok(self)
    }

    /// Items at a cell, including filtered items; caller must enforce visibility.
    pub fn ground_items(&self, position: Position) -> impl Iterator<Item = &Item> {
        self.inventory
            .iter()
            .flat_map(Inventory::items)
            .filter(move |item| {
                self.inventory
                    .as_ref()
                    .and_then(|inventory| inventory.container(item.container))
                    == Some(&Container::Ground {
                        map: self.map.id(),
                        position,
                    })
            })
    }

    /// First item on the player's cell in deterministic instance order.
    #[must_use]
    pub fn pickup_command(&self) -> Option<crate::Command> {
        let item = self.ground_items(self.player()?.position()).next()?;
        Some(crate::Command::Transfer(TransferItems {
            item: item.id,
            destination: BACKPACK,
            cell: None,
        }))
    }

    /// Actual combat stats (hidden equipment affixes still function).
    #[must_use]
    pub fn derived_stats(&self, actor: &Actor) -> CombatStats {
        if actor.is_player()
            && let Some(inventory) = &self.inventory
        {
            derive_stats(actor, inventory.equipped(), false)
        } else {
            derive_stats(actor, [], false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, CommandResult};

    #[test]
    fn item_previews_share_command_costs_and_never_mutate_state() {
        let mut state = crate::test_support::dungeon();
        let command = state.pickup_command().unwrap();
        let before = state.snapshot();
        assert_eq!(state.item_action_cost(command), Ok(50));
        assert_eq!(state.snapshot(), before);
        assert!(matches!(
            state.issue(command),
            CommandResult::Accepted { .. }
        ));
        let item = match command {
            Command::Transfer(transfer) => transfer.item,
            _ => unreachable!(),
        };
        let drop = Command::Drop(item);
        let before = state.snapshot();
        assert_eq!(state.item_action_cost(drop), Ok(100));
        assert_eq!(state.snapshot(), before);
        assert_eq!(
            state.item_action_cost(Command::Wait),
            Err(RejectReason::InvalidItem)
        );
    }

    fn pickup_all(state: &mut CoreState) {
        while let Some(command) = state.pickup_command() {
            assert!(matches!(
                state.issue(command),
                CommandResult::Accepted { .. }
            ));
        }
    }

    #[test]
    fn full_bag_cannot_silently_lose_displaced_gear_and_body_removal_costs_three_ticks() {
        let mut state = crate::test_support::dungeon();
        pickup_all(&mut state);
        let sword = find(&state, "iron_sword");
        let body = find(&state, "iron_mail");
        state.issue(Command::Transfer(TransferItems {
            item: body,
            destination: EQUIPMENT,
            cell: None,
        }));
        let before = state.clock().ticks();
        state.issue(Command::Transfer(TransferItems {
            item: body,
            destination: BACKPACK,
            cell: None,
        }));
        assert_eq!(state.clock().ticks() - before, 3);
        state.issue(Command::Transfer(TransferItems {
            item: sword,
            destination: EQUIPMENT,
            cell: None,
        }));
        let mut inventory = state.inventory().unwrap().clone();
        let mut scroll = inventory
            .items
            .values()
            .find(|item| inventory.definition(item).unwrap().kind == ItemKind::Identify)
            .unwrap()
            .clone();
        let mut candidate = inventory
            .items
            .values()
            .find(|item| item.definition.name() == "keen_sword")
            .unwrap()
            .clone();
        inventory
            .catalog
            .items
            .iter_mut()
            .find(|definition| definition.id.name() == "iron_sword")
            .unwrap()
            .width = 2;
        candidate.container = BACKPACK;
        candidate.cell = Some(GridCell { x: 0, y: 0 });
        inventory.items.retain(|id, _| *id == sword);
        inventory.items.insert(candidate.id, candidate.clone());
        scroll.container = BACKPACK;
        for y in 0..6 {
            for x in 0..10 {
                let cell = GridCell { x, y };
                if inventory.fits(
                    ItemId(inventory.next_item),
                    inventory.definition(&scroll).unwrap(),
                    cell,
                ) {
                    scroll.id = ItemId(inventory.next_item);
                    inventory.next_item += 1;
                    scroll.cell = Some(cell);
                    inventory.items.insert(scroll.id, scroll.clone());
                }
            }
        }
        inventory.validate().unwrap();
        state.inventory = Some(inventory.clone());
        let before = (state.clock(), state.rng_position());
        assert!(matches!(
            state.issue(Command::Transfer(TransferItems {
                item: candidate.id,
                destination: EQUIPMENT,
                cell: None
            })),
            CommandResult::Rejected {
                reason: RejectReason::BackpackFull,
                ..
            }
        ));
        assert_eq!(state.inventory(), Some(&inventory));
        assert_eq!((state.clock(), state.rng_position()), before);
    }

    fn find(state: &CoreState, name: &str) -> ItemId {
        state
            .inventory()
            .unwrap()
            .items()
            .find(|item| item.definition.name() == name && item.container == BACKPACK)
            .unwrap()
            .id
    }

    #[test]
    fn body_change_exposes_player_to_three_normal_enemy_actions() {
        let mut state = crate::test_support::dungeon();
        pickup_all(&mut state);
        let body = find(&state, "iron_mail");
        for _ in 0..4 {
            state.issue(Command::Move(crate::Direction::East));
        }
        state.issue(Command::Stairs(crate::StairDirection::Down));
        let guardian = state.dungeon.as_ref().unwrap().guardian;
        state.actors.get_mut(&guardian).unwrap().position = Position::new(2, 1);
        state.refresh_visibility();
        let attacks = |state: &CoreState| {
            state.outcomes().iter().filter(|outcome|
            matches!(outcome.kind, crate::OutcomeKind::Attacked { attacker, .. } if attacker == guardian)).count()
        };
        let before = attacks(&state);
        let tick = state.clock().ticks();
        state.issue(Command::Transfer(TransferItems {
            item: body,
            destination: EQUIPMENT,
            cell: None,
        }));
        assert_eq!(state.clock().ticks() - tick, 3);
        assert_eq!(attacks(&state) - before, 3);
        assert_eq!(
            CoreState::restore(state.snapshot()).unwrap().snapshot(),
            state.snapshot()
        );
    }

    #[test]
    fn pickup_equip_and_atomic_swap_conserve_items_and_charge_exact_costs() {
        let mut state = crate::test_support::dungeon();
        let initial_count = state.inventory().unwrap().items().count();
        let initial = state.clock();
        let pickup = state.pickup_command().unwrap();
        state.issue(pickup);
        assert_eq!(state.clock().ticks() - initial.ticks(), 1);
        let second = state.pickup_command().unwrap();
        state.issue(second);
        assert_eq!(
            state.clock().ticks() - initial.ticks(),
            1,
            "two 50-energy pickups fit a tick"
        );
        pickup_all(&mut state);
        let sword = find(&state, "iron_sword");
        let mail = find(&state, "iron_mail");
        let before = state.clock();
        state.issue(Command::Transfer(TransferItems {
            item: mail,
            destination: EQUIPMENT,
            cell: None,
        }));
        assert_eq!(
            state.clock().ticks() - before.ticks(),
            3,
            "body armor costs 300 energy"
        );
        state.issue(Command::Transfer(TransferItems {
            item: sword,
            destination: EQUIPMENT,
            cell: None,
        }));
        let actor = state.player().unwrap();
        let comparison = state.inventory().unwrap().compare(actor, sword).unwrap();
        assert_eq!(comparison.0, comparison.1);
        assert_eq!(state.derived_stats(actor), comparison.0);
        assert_eq!(state.inventory().unwrap().items().count(), initial_count);
        state.inventory().unwrap().validate().unwrap();
        let restored = CoreState::restore(state.snapshot()).unwrap();
        assert_eq!(restored.snapshot(), state.snapshot());
    }

    #[test]
    fn rejected_transfer_preserves_ownership_clock_and_rng_for_every_grid_origin() {
        let mut state = crate::test_support::dungeon();
        pickup_all(&mut state);
        let item = find(&state, "iron_sword");
        state.issue(Command::Transfer(TransferItems {
            item,
            destination: EQUIPMENT,
            cell: None,
        }));
        for x in 0..=u8::MAX {
            let before = state.inventory().unwrap().clone();
            let clock = state.clock();
            let rng = state.rng_position();
            let result = state.issue(Command::Transfer(TransferItems {
                item,
                destination: BACKPACK,
                cell: Some(GridCell { x, y: 5 }),
            }));
            assert!(matches!(
                result,
                CommandResult::Rejected {
                    reason: RejectReason::BackpackFull,
                    ..
                }
            ));
            assert_eq!(state.inventory().unwrap(), &before);
            assert_eq!(state.clock(), clock);
            assert_eq!(state.rng_position(), rng);
        }
        let mut heavy = crate::test_support::dungeon();
        heavy.inventory.as_mut().unwrap().catalog.carry_capacity = 1;
        let clock = heavy.clock();
        assert!(matches!(
            heavy.issue(heavy.pickup_command().unwrap()),
            CommandResult::Rejected {
                reason: RejectReason::TooHeavy,
                ..
            }
        ));
        assert_eq!(heavy.clock(), clock);
    }

    #[test]
    fn potions_and_scrolls_consume_only_on_effect_and_comparisons_hide_affixes() {
        let mut state = crate::test_support::dungeon();
        pickup_all(&mut state);
        let potion = find(&state, "healing_potion");
        let scroll = find(&state, "identify_scroll");
        let before = state.inventory().unwrap().clone();
        assert!(matches!(
            state.issue(Command::Quaff(potion)),
            CommandResult::Rejected {
                reason: RejectReason::NoEffect,
                ..
            }
        ));
        assert_eq!(state.inventory().unwrap(), &before);
        state.actors.get_mut(&state.player).unwrap().hp = 10;
        state.issue(Command::Quaff(potion));
        assert_eq!(state.player().unwrap().hp(), 34);
        assert!(!state.inventory().unwrap().items.contains_key(&potion));
        let mut inventory = state.inventory().unwrap().clone();
        let magic = inventory
            .items
            .values_mut()
            .find(|item| item.definition.name() == "keen_sword")
            .unwrap();
        magic.container = BACKPACK;
        magic.cell = Some(GridCell { x: 9, y: 0 });
        let magic = magic.id;
        state.inventory = Some(inventory);
        let known = state
            .inventory()
            .unwrap()
            .compare(state.player().unwrap(), magic)
            .unwrap()
            .1;
        assert!(
            !state
                .inventory()
                .unwrap()
                .label(&state.inventory().unwrap().items[&magic])
                .contains("keen edges")
        );
        state.issue(Command::Identify {
            scroll,
            target: magic,
        });
        let identified = state
            .inventory()
            .unwrap()
            .compare(state.player().unwrap(), magic)
            .unwrap()
            .1;
        assert!(identified.damage > known.damage);
        assert!(
            state
                .inventory()
                .unwrap()
                .label(&state.inventory().unwrap().items[&magic])
                .contains("keen edges")
        );
        assert!(!state.inventory().unwrap().items.contains_key(&scroll));
        state.inventory().unwrap().validate().unwrap();
    }

    #[test]
    fn filters_are_knowledge_based_and_never_mutate_gameplay() {
        let state = crate::test_support::dungeon();
        let before = state.snapshot();
        let inventory = state.inventory().unwrap();
        for item in inventory.items() {
            let definition = inventory.definition(item).unwrap();
            let (show, emphasize) = inventory.ground_label(item);
            assert!(show);
            assert_eq!(emphasize, definition.rarity == Rarity::Magic);
            let rule = ItemPredicate::All(vec![
                ItemPredicate::RarityAtLeast(Rarity::Magic),
                ItemPredicate::Identified(false),
            ]);
            assert_eq!(
                rule.matches(definition, item.identified),
                definition.rarity == Rarity::Magic && !item.identified
            );
        }
        assert_eq!(state.snapshot(), before);
    }
}
