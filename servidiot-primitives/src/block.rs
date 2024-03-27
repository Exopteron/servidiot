use std::ops::Deref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
/// A block ID. Goes from
/// `0` to `4095`.
pub struct BlockID(u16);
impl Deref for BlockID {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BlockID {

    /// Creates a new `BlockID`.
    /// If `block_id` is greater than 4095, 
    /// returns `None`.
    pub const fn new(block_id: u16) -> Option<Self> {
        if block_id > 4095 {
            None
        } else {
            Some(Self(block_id))
        }
    }

    /// Creates a new `BlockID` from 
    /// the type byte and add byte.
    /// Returns `None` if `add` is greater than 15.
    pub const fn new_with_add(block_id: u8, add: u8) -> Option<Self> {
        if add > 15 {
            return None;
        }
        let mut id = (add as u16) << 8;
        id += block_id as u16;
        Some(Self(id))
    }

    /// Whether this block ID needs Add data
    /// to represent it.
    pub const fn needs_add(&self) -> bool {
        self.0 > 255
    }

    /// Extracts the ID and add pair out of this ID.
    pub const fn to_add_pair(&self) -> (u8, u8) {
        let id = (self.0 & 0xff) as u8;
        #[allow(clippy::cast_possible_truncation)]
        let add = (self.0 & 0xf00) as u8;
        (id, add)
    }

    /// # Safety
    /// Ensure `block_id` is no larger than 4096.
    pub const unsafe fn new_unchecked(block_id: u16) -> Self {
        Self(block_id)
    }
}


pub trait BlockType {
    const VALID_BLOCK_TYPES: &'static [BlockID];
}

#[cfg(test)]
mod tests {
    use super::BlockID;

    #[test]
    fn block_id_test() {
        assert_eq!(256, BlockID::new_with_add(0, 1).unwrap().0);
    }
}