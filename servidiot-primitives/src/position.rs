//! Position primitives.
#![allow(clippy::module_name_repetitions)]
pub mod iteration;
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use thiserror::Error;

/// The position of some block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Display for BlockPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockPosition(x = {}, y = {}, z = {})", self.x, self.y, self.z)
    }
}
impl BlockPosition {
    /// Creates a new `BlockPosition` from a
    /// set of coordinates.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns a representation of this
    /// block position as an array.
    pub fn as_array(&self) -> [i32; 3] {
        [self.x, self.y, self.z]
    }

    /// Returns a representation of this
    /// block position as a tuple.
    pub fn as_tuple(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    /// Returns the chunk this block 
    /// position resides in.
    pub fn chunk(&self) -> ChunkPosition {
        ChunkPosition::new(self.x >> 4, self.z >> 4)
    }

    /// Offsets this position by (x, y, z).
    pub fn offset(&self, x: i32, y: i32, z: i32) -> Self {
        Self {
            x: self.x.wrapping_add(x),
            y: self.y.wrapping_add(y),
            z: self.z.wrapping_add(z)
        }
    }
}

/// Represents what world and dimension an object resides in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Location {
    /// The multiworld world ID the object resides in.
    pub world: u32,
    /// The dimension the object resides in.
    pub dimension: i32,
}

impl Location {
    pub const fn new(world: u32, dimension: i32) -> Self {
        Self {
            world,
            dimension
        }
    }
}


/// Represents the position of an entity.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32, on_ground: bool) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
            on_ground,
        }
    }

    /// This position as a block position.
    pub fn block(&self) -> BlockPosition {
        #[allow(clippy::cast_possible_truncation)]
        BlockPosition::new(self.x.round() as i32, self.y.round() as i32, self.z.round() as i32)
    }
    

    /// This position as a chunk position.
    pub fn chunk(&self) -> ChunkPosition {
        self.block().chunk()
    }
}
/// Represents the location of an entity.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct EntityLocation {
    pub position: Position,
    pub location: Location
}
impl Deref for EntityLocation {
    type Target = Position;

    fn deref(&self) -> &Self::Target {
        &self.position
    }
}

pub const MIN_BLOCK_X: i32 = -30_000_000;
pub const MAX_BLOCK_X: i32 = 30_000_000;
pub const MIN_BLOCK_Y: i32 = 0;
pub const MAX_BLOCK_Y: i32 = 255;
pub const MIN_BLOCK_Z: i32 = -30_000_000;
pub const MAX_BLOCK_Z: i32 = 30_000_000;

/// A block position verified to be within
/// the bounds of Minecraft's possibilities.
pub struct CheckedBlockPosition(BlockPosition);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BlockPositionError {
    #[error("x {0} too large, must be below {1}")]
    XTooLarge(i32, i32),
    #[error("y {0} too large, must be below {1}")]
    YTooLarge(i32, i32),
    #[error("z {0} too large, must be below {1}")]
    ZTooLarge(i32, i32),

    #[error("x {0} too small, must be above {1}")]
    XTooSmall(i32, i32),
    #[error("y {0} too small, must be above {1}")]
    YTooSmall(i32, i32),
    #[error("z {0} too small, must be above {1}")]
    ZTooSmall(i32, i32),
}

impl Debug for CheckedBlockPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckedBlockPosition")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish()
    }
}

impl CheckedBlockPosition {
    /// Converts a `BlockPosition` into a `CheckedBlockPosition`.
    ///
    /// # Errors
    /// Returns `Ok` if `position` is within possible bounds.
    /// Returns `Err` otherwise.
    pub fn convert(position: BlockPosition) -> std::result::Result<Self, BlockPositionError> {
        Self::new(position.x, position.y, position.z)
    }

    /// Creates a new `CheckedBlockPosition` from a
    /// set of coordinates.
    ///
    /// # Errors
    /// Returns `Ok` if the coordinates are within
    /// possible bounds. Returns `Err` otherwise.
    pub fn new(x: i32, y: i32, z: i32) -> std::result::Result<Self, BlockPositionError> {
        match (x, y, z) {
            (x, _, _) if x > MAX_BLOCK_X => Err(BlockPositionError::XTooLarge(x, MAX_BLOCK_X)),
            (_, y, _) if y > MAX_BLOCK_Y => Err(BlockPositionError::YTooLarge(y, MAX_BLOCK_Y)),
            (_, _, z) if z > MAX_BLOCK_Z => Err(BlockPositionError::ZTooLarge(z, MAX_BLOCK_Z)),
            (x, _, _) if x < MIN_BLOCK_X => Err(BlockPositionError::XTooSmall(x, MIN_BLOCK_X)),
            (_, y, _) if y < MIN_BLOCK_Y => Err(BlockPositionError::YTooSmall(y, MIN_BLOCK_Y)),
            (_, _, z) if z < MIN_BLOCK_Z => Err(BlockPositionError::ZTooSmall(z, MIN_BLOCK_Z)),
            _ => Ok(Self(BlockPosition::new(x, y, z))),
        }
    }
}
/// The location of a block, within some world.
pub struct BlockLocation {
    pub position: CheckedBlockPosition,
    pub location: Location,
}
impl BlockLocation {
    /// Creates a new `BlockLocation` from
    /// a position and location.
    pub fn new(position: CheckedBlockPosition, location: Location) -> Self {
        Self { position, location }
    }
}

impl Deref for CheckedBlockPosition {
    type Target = BlockPosition;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents the position of a chunk.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32
}
impl Display for ChunkPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkPosition(x = {}, z = {})", self.x, self.z)
    }
}
impl ChunkPosition {
    /// Creates a new `ChunkPosition` from
    /// a set of coordinates.
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z
        }
    }

    /// The region this chunk resides in.
    pub fn region(&self) -> RegionPosition {
        #[allow(clippy::cast_possible_truncation)]
        RegionPosition::new((self.x >> 5) as i16, (self.z >> 5) as i16)
    }


    /// Offset this position by (x, y).
    pub fn offset(&self, x: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            z: self.z + z
        }
    }
}

/// Represents the position of a chunk in multiple worlds/dimensions.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkLocation {
    pub position: ChunkPosition,
    pub location: Location
}
/// Represents the position of a region.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RegionPosition {
    pub x: i16,
    pub z: i16
}

impl RegionPosition {
    /// Creates a new `RegionPosition` from
    /// a set of coordinates.
    pub fn new(x: i16, z: i16) -> Self {
        Self {
            x,
            z
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::position::{
        BlockPositionError, MAX_BLOCK_X, MAX_BLOCK_Y, MAX_BLOCK_Z, MIN_BLOCK_X, MIN_BLOCK_Y,
        MIN_BLOCK_Z,
    };

    use super::CheckedBlockPosition;

    #[test]
    pub fn checked_block_position_test_err() {
        assert_eq!(
            CheckedBlockPosition::new(0, -1, 0).unwrap_err(),
            BlockPositionError::YTooSmall(-1, MIN_BLOCK_Y)
        );

        assert_eq!(
            CheckedBlockPosition::new(0, 256, 0).unwrap_err(),
            BlockPositionError::YTooLarge(256, MAX_BLOCK_Y)
        );

        assert_eq!(
            CheckedBlockPosition::new(-30_000_001, 0, 0).unwrap_err(),
            BlockPositionError::XTooSmall(-30_000_001, MIN_BLOCK_X)
        );

        assert_eq!(
            CheckedBlockPosition::new(30_000_001, 0, 0).unwrap_err(),
            BlockPositionError::XTooLarge(30_000_001, MAX_BLOCK_X)
        );

        assert_eq!(
            CheckedBlockPosition::new(0, 0, -30_000_001).unwrap_err(),
            BlockPositionError::ZTooSmall(-30_000_001, MIN_BLOCK_Z)
        );

        assert_eq!(
            CheckedBlockPosition::new(0, 0, 30_000_001).unwrap_err(),
            BlockPositionError::ZTooLarge(30_000_001, MAX_BLOCK_Z)
        );
    }

    #[test]
    pub fn checked_block_position_test_ok() {
        assert!(CheckedBlockPosition::new(0, 255, 0).is_ok());
        assert!(CheckedBlockPosition::new(30_000_000, 0, 0).is_ok());
        assert!(CheckedBlockPosition::new(0, 0, 30_000_000).is_ok());
        assert!(CheckedBlockPosition::new(-30_000_000, 0, 0).is_ok());
        assert!(CheckedBlockPosition::new(0, 0, -30_000_000).is_ok());
    }
}
