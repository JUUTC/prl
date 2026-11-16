//! Namespaced content identifiers (`namespace:name`).

use std::fmt;
use std::str::FromStr;

use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Stable identifier for any content definition (monster, item, skill, ability…).
///
/// Format: `namespace:name`, e.g. `prl:goblin_archer` or `my_mod:frost_wyrm`.
/// The namespace is the owning content pack (`prl` is the base game). Both parts
/// are lowercase ASCII `snake_case`: `[a-z][a-z0-9_]*`.
///
/// IDs are what saves, mods and data files reference, so they must never change
/// once released. Rename via an alias table instead.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentId {
    namespace: String,
    name: String,
}

impl ContentId {
    /// Namespace of the base game content pack.
    pub const BASE_NAMESPACE: &'static str = "prl";

    /// Builds an ID from its parts, validating both.
    ///
    /// # Errors
    ///
    /// Returns [`ContentIdError`] if either part is not `[a-z][a-z0-9_]*`.
    pub fn new(namespace: &str, name: &str) -> Result<Self, ContentIdError> {
        if !is_valid_part(namespace) {
            return Err(ContentIdError::InvalidNamespace(namespace.to_owned()));
        }
        if !is_valid_part(name) {
            return Err(ContentIdError::InvalidName(name.to_owned()));
        }
        Ok(Self {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
        })
    }

    /// The owning content pack's namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// The name within the namespace.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.name)
    }
}

impl Serialize for ContentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ContentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ContentIdVisitor;

        impl Visitor<'_> for ContentIdVisitor {
            type Value = ContentId;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a namespaced content ID such as `prl:stone_floor`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(ContentIdVisitor)
    }
}

impl FromStr for ContentId {
    type Err = ContentIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (namespace, name) = s
            .split_once(':')
            .ok_or_else(|| ContentIdError::MissingSeparator(s.to_owned()))?;
        Self::new(namespace, name)
    }
}

/// Error returned when parsing or building a [`ContentId`] fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentIdError {
    /// The string has no `:` separator.
    MissingSeparator(String),
    /// The namespace part is empty or contains invalid characters.
    InvalidNamespace(String),
    /// The name part is empty or contains invalid characters.
    InvalidName(String),
}

impl fmt::Display for ContentIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator(s) => {
                write!(f, "content id `{s}` must be `namespace:name`")
            }
            Self::InvalidNamespace(s) => {
                write!(
                    f,
                    "invalid content id namespace `{s}` (expected [a-z][a-z0-9_]*)"
                )
            }
            Self::InvalidName(s) => {
                write!(
                    f,
                    "invalid content id name `{s}` (expected [a-z][a-z0-9_]*)"
                )
            }
        }
    }
}

impl std::error::Error for ContentIdError {}

fn is_valid_part(part: &str) -> bool {
    let mut chars = part.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_displays_round_trip() {
        let id: ContentId = "prl:goblin_archer".parse().unwrap();
        assert_eq!(id.namespace(), "prl");
        assert_eq!(id.name(), "goblin_archer");
        assert_eq!(id.to_string(), "prl:goblin_archer");
    }

    #[test]
    fn rejects_missing_separator() {
        assert_eq!(
            "goblin".parse::<ContentId>(),
            Err(ContentIdError::MissingSeparator("goblin".into()))
        );
    }

    #[test]
    fn rejects_invalid_parts() {
        assert!(matches!(
            "Prl:goblin".parse::<ContentId>(),
            Err(ContentIdError::InvalidNamespace(_))
        ));
        assert!(matches!(
            "prl:".parse::<ContentId>(),
            Err(ContentIdError::InvalidName(_))
        ));
        assert!(matches!(
            "prl:9lives".parse::<ContentId>(),
            Err(ContentIdError::InvalidName(_))
        ));
        assert!(matches!(
            "prl:frost-wyrm".parse::<ContentId>(),
            Err(ContentIdError::InvalidName(_))
        ));
    }
}
