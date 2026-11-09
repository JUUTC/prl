//! Grid coordinates and eight-way directions.

use serde::{Deserialize, Serialize};

/// Integer grid coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Horizontal map coordinate.
    pub x: i32,
    /// Vertical map coordinate.
    pub y: i32,
}

impl Position {
    /// Creates a coordinate.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Adjacent coordinate in a direction, or `None` on integer overflow.
    #[must_use]
    pub fn offset(self, direction: Direction) -> Option<Self> {
        let (dx, dy) = direction.delta();
        Some(Self {
            x: self.x.checked_add(dx)?,
            y: self.y.checked_add(dy)?,
        })
    }

    /// Chebyshev (king-move) distance, saturating on overflow.
    #[must_use]
    pub fn chebyshev_distance(self, other: Self) -> u32 {
        let dx = self.x.abs_diff(other.x);
        let dy = self.y.abs_diff(other.y);
        dx.max(dy)
    }

    /// Direction of an adjacent (Chebyshev distance 1) coordinate.
    #[must_use]
    pub fn direction_to(self, other: Self) -> Option<Direction> {
        Direction::ALL
            .into_iter()
            .find(|direction| self.offset(*direction) == Some(other))
    }
}

/// Eight-way movement direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Up on the logical map.
    North,
    /// Up and right.
    NorthEast,
    /// Right on the logical map.
    East,
    /// Down and right.
    SouthEast,
    /// Down on the logical map.
    South,
    /// Down and left.
    SouthWest,
    /// Left on the logical map.
    West,
    /// Up and left.
    NorthWest,
}

impl Direction {
    /// All directions in the fixed order used by deterministic searches.
    pub const ALL: [Self; 8] = [
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
        Self::NorthWest,
    ];

    /// Unit grid delta `(dx, dy)`; north is negative `y`.
    #[must_use]
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::NorthEast => (1, -1),
            Self::East => (1, 0),
            Self::SouthEast => (1, 1),
            Self::South => (0, 1),
            Self::SouthWest => (-1, 1),
            Self::West => (-1, 0),
            Self::NorthWest => (-1, -1),
        }
    }

    /// Whether the direction moves along both axes.
    #[must_use]
    pub const fn is_diagonal(self) -> bool {
        let (x, y) = self.delta();
        x != 0 && y != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_direction_and_chebyshev_distance_agree() {
        let origin = Position::new(3, 3);
        for direction in Direction::ALL {
            let next = origin.offset(direction).unwrap();
            assert_eq!(origin.chebyshev_distance(next), 1);
            assert_eq!(origin.direction_to(next), Some(direction));
        }
        assert_eq!(origin.direction_to(Position::new(5, 3)), None);
        assert_eq!(origin.chebyshev_distance(Position::new(0, 5)), 3);
    }
}
