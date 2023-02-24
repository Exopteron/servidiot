use crate::{block::BlockID, position::ChunkPosition};

use self::section::ChunkSection;

pub mod section;

/// Represents a Minecraft chunk.
pub struct Chunk {
    biomes: [[u8; 16]; 16],
    sections: [Option<ChunkSection>; 16],
    position: ChunkPosition,
}

impl Chunk {
    pub const HEIGHT: usize = 256;
    pub const WIDTH: usize = 16;
    pub const LENGTH: usize = 16;

    /// Converts a position to a section index and section position.
    const fn position_to_index(
        x: usize,
        y: usize,
        z: usize,
    ) -> Option<(usize, usize, usize, usize)> {
        if x > Self::LENGTH || y > Self::HEIGHT || z > Self::WIDTH {
            return None;
        }
        let section = y / 16;
        let y = y & 15;
        Some((x, y, z, section))
    }

    /// Create a new, empty chunk.
    pub fn new(position: ChunkPosition) -> Self {
        Self {
            position,
            biomes: [[0; 16]; 16],
            sections: [(); 16].map(|_| None),
        }
    }

    pub fn position(&self) -> ChunkPosition {
        self.position
    }

    /// Iterator over sections.
    pub fn sections(&self) -> impl Iterator<Item = &ChunkSection> {
        self.sections.iter().flatten()
    }

    /// Set a chunk section.
    pub fn set_section(&mut self, index: u8, section: ChunkSection) {
        self.sections[index as usize] = Some(section);
    }

    /// Set a chunk section.
    pub fn get_section(&self, index: u8) -> Option<&ChunkSection> {
        self.sections[index as usize].as_ref()
    }

    pub fn biomes(&self) -> &[[u8; 16]; 16] {
        &self.biomes
    }
    pub fn biomes_mut(&mut self) -> &mut [[u8; 16]; 16] {
        &mut self.biomes
    }

    /// Gets the block at this index.
    pub fn block_type_at(&self, x: usize, y: usize, z: usize) -> Option<BlockID> {
        let (x, y, z, section) = Self::position_to_index(x, y, z)?;
        self.sections[section].as_ref()?.block_type_at(x, y, z)
    }

    /// Sets a block ID within this section.
    pub fn set_block_type_at(&mut self, x: usize, y: usize, z: usize, ty: BlockID) -> Option<()> {
        let (x, y, z, section) = Self::position_to_index(x, y, z)?;
        self.sections[section]
            .as_mut()?
            .set_block_type_at(x, y, z, ty)
    }

    /// Gets the sky light value at some block.
    pub fn sky_light_at(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        let (x, y, z, section) = Self::position_to_index(x, y, z)?;
        self.sections[section].as_ref()?.sky_light_at(x, y, z)
    }

    /// Gets the block light value at some block.
    pub fn block_light_at(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        let (x, y, z, section) = Self::position_to_index(x, y, z)?;
        self.sections[section].as_ref()?.block_light_at(x, y, z)
    }

    /// Sets the sky light value at some block.
    /// Returns the previous value.
    pub fn set_sky_light_at(&mut self, x: usize, y: usize, z: usize, value: u8) -> Option<u8> {
        let (x, y, z, section) = Self::position_to_index(x, y, z)?;
        self.sections[section]
            .as_mut()?
            .set_sky_light_at(x, y, z, value)
    }

    /// Sets the block light value at some block.
    /// Returns the previous value.
    pub fn set_block_light_at(&mut self, x: usize, y: usize, z: usize, value: u8) -> Option<u8> {
        let (x, y, z, section) = Self::position_to_index(x, y, z)?;
        self.sections[section]
            .as_mut()?
            .set_block_light_at(x, y, z, value)
    }
}
