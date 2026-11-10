//! P3 item definitions and shared knowledge-aware predicate data.

use serde::{Deserialize, Serialize};

use crate::ContentId;

/// Slice rarity subset; later rarities are not silently treated as Magic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rarity {
    /// No hidden affixes.
    Normal,
    /// One or two affixes, hidden until identified.
    Magic,
}

/// Initial equipment subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EquipSlot {
    /// Primary melee weapon.
    MainHand,
    /// Body armor (300-energy equip).
    Body,
}

/// Engine-supported slice effects, never matched by content ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    /// Adds stats while equipped.
    Gear(EquipSlot),
    /// Restores HP, limited by max HP.
    Healing {
        /// Fixed restored HP.
        amount: u16,
    },
    /// Identifies one carried item.
    Identify,
}

/// Integer additive modifiers reused by real and hypothetical loadouts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ItemBonus {
    /// Base melee damage change.
    pub damage: i16,
    /// Physical armor change.
    pub armor: i16,
    /// Accuracy change.
    pub accuracy: i16,
    /// Evasion change.
    pub evasion: i16,
}

/// A P3 item definition; ground instances have persistent IDs and knowledge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemDefinition {
    /// Namespaced identity.
    pub id: ContentId,
    /// Known base name; never includes hidden affix text.
    pub name: String,
    /// Item rarity.
    pub rarity: Rarity,
    /// Grid width in cells.
    pub width: u8,
    /// Grid height in cells.
    pub height: u8,
    /// Per-instance weight in grams.
    pub weight: u32,
    /// Effect or equipment slot.
    pub kind: ItemKind,
    /// Visible base-type modifiers.
    pub base: ItemBonus,
    /// Hidden Magic modifiers (a fixed small affix subset for this slice).
    pub affix: ItemBonus,
    /// Text shown only once the affix is identified.
    pub affix_name: String,
    /// Shared filter/optimizer/storage classification tags.
    pub tags: Vec<String>,
    /// Seed-selected anonymous descriptions for unknown consumables.
    #[serde(default)]
    pub appearances: Vec<String>,
}

/// Knowledge-aware shared filter expression, also reusable for storage rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemPredicate {
    /// Match any item.
    Always,
    /// Match by known base definition.
    Definition(ContentId),
    /// Match by visible rarity.
    RarityAtLeast(Rarity),
    /// Match a content tag.
    Tag(String),
    /// Match identification state.
    Identified(bool),
    /// Match all nested rules.
    All(Vec<Self>),
    /// Match any nested rule.
    Any(Vec<Self>),
}

impl ItemPredicate {
    /// Evaluates only public item knowledge; never reads hidden affix values.
    #[must_use]
    pub fn matches(&self, definition: &ItemDefinition, identified: bool) -> bool {
        match self {
            Self::Always => true,
            Self::Definition(id) => &definition.id == id,
            Self::RarityAtLeast(rarity) => definition.rarity >= *rarity,
            Self::Tag(tag) => definition.tags.contains(tag),
            Self::Identified(known) => identified == *known,
            Self::All(rules) => rules
                .iter()
                .all(|rule| rule.matches(definition, identified)),
            Self::Any(rules) => rules
                .iter()
                .any(|rule| rule.matches(definition, identified)),
        }
    }
}

/// First-match ground-label rule; no auto-pickup in P3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootRule {
    /// Knowledge-based selector.
    pub predicate: ItemPredicate,
    /// Whether a matching label is visible.
    pub show: bool,
    /// Whether it has an emphasis marker.
    pub emphasize: bool,
}

/// Versioned slice item catalog and built-in Default ground-label preset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemCatalog {
    /// Schema format (1).
    pub format: u32,
    /// Item definitions used by the slice.
    pub items: Vec<ItemDefinition>,
    /// Default label rules, in priority order, ending in an Always rule.
    pub default_filter: Vec<LootRule>,
    /// Definitions placed at each floor's entrance (one instance each).
    pub supplies: Vec<ContentId>,
    /// Definitions placed at each floor's exit (one instance each).
    pub rewards: Vec<ContentId>,
    /// Provisional Human carry capacity until P4 stats exist (grams).
    pub carry_capacity: u32,
}
