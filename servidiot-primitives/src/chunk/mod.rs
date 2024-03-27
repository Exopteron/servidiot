use crate::{block::BlockID, position::ChunkPosition};

use self::section::ChunkSection;

pub mod section;
// pub mod store;

/// Represents a Minecraft chunk.
pub struct Chunk {
    biomes: [[u8; 16]; 16],
    heightmap: [[u8; 16]; 16],
    sections: [Option<ChunkSection>; ChunkSection::SECTIONS_PER_CHUNK],
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
            heightmap: [[0; 16]; 16],
            biomes: [[0; 16]; 16],
            sections: [(); 16].map(|_| None),
        }
    }

    /// Generates a bitmap for all present sections in this chunk.
    pub fn bitmap(&self) -> ChunkBitmap {
        let mut bitmap = ChunkBitmap::empty();

        for (idx, value) in self.sections.iter().enumerate() {
            if value.is_some() {
                bitmap.set(idx, true);
            }
        }

        bitmap
    }


    pub fn position(&self) -> ChunkPosition {
        self.position
    }

    /// Iterator over sections.
    pub fn sections(&self) -> impl Iterator<Item = &ChunkSection> {
        self.sections.iter().flatten()
    }

    /// Owned iterator over sections.
    pub fn into_sections(self) -> impl Iterator<Item = ChunkSection> {
        self.sections.into_iter().flatten()
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

    pub fn heightmap(&self) -> &[[u8; 16]; 16] {
        &self.heightmap
    }
    pub fn heightmap_mut(&mut self) -> &mut [[u8; 16]; 16] {
        &mut self.heightmap
    }

    /// Gets the block at this index.
    pub fn block_type_at(&self, x: usize, y: usize, z: usize) -> Option<BlockID> {
        let (x, y, z, section) = Self::position_to_index(x, y, z)?;
        self.sections[section].as_ref()?.block_type_at(x, y, z)
    }

    /// Sets a block ID within this chunk.
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


#[derive(Debug, Clone, Copy)]
pub struct ChunkBitmap(pub u16);
impl ChunkBitmap {
    /// Returns a full bitmap.
    pub const fn full() -> Self {
        Self(u16::MAX)
    }

    /// Returns an empty bitmap.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Sets a section within this bitmap.
    /// Returns `None` if `section` is out of bounds.
    pub fn set(&mut self, section: usize, value: bool) -> Option<()> {
        if section > 15 {
            return None;
        }
        if value {
            self.0 |= 1 << (section);
        } else {
            self.0 &= !(1 << (section));
        }
        Some(())
    }

    /// Test a section within this bitmap.
    /// Returns `None` if `section` is out of bounds.
    pub const fn get(&self, section: usize) -> Option<bool> {
        if section > 15 {
            None
        } else {
            Some((self.0 & (1 << ( section))) != 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChunkBitmap;

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn bitmap_test() {
        let mut bitmap = ChunkBitmap::empty();
        let test = [0, 1, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 0];
        for i in 0..15 {
            bitmap.set(i, test[i] == 1);
        }

        for i in 0..15 {
            assert_eq!(bitmap.get(i).unwrap(), test[i] == 1);
        }
    }
}