use crate::{nibble_vec::NibbleVec, block::BlockID};





/// Represents a Minecraft chunk section.
pub struct ChunkSection {
    /// This section's Y level.
    pub section_id: u8,
    /// This section's skylight.
    pub skylight: NibbleVec,
    /// This section's block light.
    pub block_light: NibbleVec,
    /// This section's block meta.
    pub block_meta: NibbleVec,
    /// This section's block types.
    pub block_types: Vec<u8>,
    /// The add section.
    pub block_types_add: Option<NibbleVec>,
}

impl ChunkSection {

    pub const HEIGHT: usize = 16;
    pub const WIDTH: usize = 16;
    pub const LENGTH: usize = 16;

    /// Converts a position within this section to an index.
    /// Returns `None` if the position is out of bounds.
    const fn position_to_index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x > Self::LENGTH || y > Self::HEIGHT || z > Self::WIDTH {
            return None;
        }
        Some((y * 16 + z) * 16 + x)
    }

    /// Gets the block at this index.
    pub fn block_type_at(&self, x: usize, y: usize, z: usize) -> Option<BlockID> {
        let index = Self::position_to_index(x, y, z)?;
        let ty = *self.block_types.get(index)?;
        let add = self.block_types_add.as_ref().map(|v| v.get(index));
        if let Some(add) = add {
            BlockID::new_with_add(ty, add)
        } else {
            BlockID::new(ty as u16)
        }
    }

    /// Sets a block ID within this section.
    pub fn set_block_type_at(&mut self, x: usize, y: usize, z: usize, ty: BlockID) -> Option<()> {
        let index = Self::position_to_index(x, y, z)?;
        if ty.needs_add() {
            todo!()
        }
        self.block_types[index] = ty.to_add_pair().0;
        Some(())
    }

    /// Gets the sky light value at some block.
    pub fn sky_light_at(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        let index = Self::position_to_index(x, y, z)?;
        Some(self.skylight.get(index))
    }

    /// Gets the block light value at some block.
    pub fn block_light_at(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        let index = Self::position_to_index(x, y, z)?;
        Some(self.block_light.get(index))
    }

    /// Sets the sky light value at some block.
    /// Returns the previous value.
    pub fn set_sky_light_at(&mut self, x: usize, y: usize, z: usize, value: u8) -> Option<u8> {
        let index = Self::position_to_index(x, y, z)?;
        let original = self.skylight.get(index);
        self.skylight.set(index, value)?;
        Some(original)
    }

    /// Sets the block light value at some block.
    /// Returns the previous value.
    pub fn set_block_light_at(&mut self, x: usize, y: usize, z: usize, value: u8) -> Option<u8> {
        let index = Self::position_to_index(x, y, z)?;
        let original = self.block_light.get(index);
        self.block_light.set(index, value)?;
        Some(original)
    }
}
